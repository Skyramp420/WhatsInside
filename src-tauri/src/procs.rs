//! Live process overview, grouped the way a person thinks about it:
//! "GIGABYTE Control Center – 85 MB", not four unrelated exe names.
//!
//! Read-only. We never kill or suspend anything.

use crate::catalog::{self, Verdict};
use serde::Serialize;
use std::collections::HashMap;
use std::time::Instant;

/// One process as read from the OS.
#[derive(Clone, Debug)]
pub struct RawProc {
    pub pid: u32,
    pub exe: String,
    pub path: Option<String>,
    pub ram: Option<u64>,
    /// Kernel + user CPU time in 100 ns units.
    pub cpu_time: Option<u64>,
    /// Bytes read + written since process start.
    pub io_bytes: Option<u64>,
}

#[derive(Serialize, Clone, Debug)]
pub struct Proc {
    pub pid: u32,
    pub exe: String,
    pub path: Option<String>,
    pub ram: Option<u64>,
    pub cpu: f32,
    pub io_bps: f64,
}

#[derive(Serialize, Clone, Debug)]
pub struct Group {
    pub key: String,
    pub name: String,
    pub verdict: Verdict,
    pub about: Option<&'static str>,
    pub cpu: f32,
    pub ram: u64,
    pub io_bps: f64,
    pub procs: Vec<Proc>,
}

#[derive(Serialize, Default)]
pub struct Snapshot {
    pub cpu_count: usize,
    pub cpu_total: f32,
    pub ram_total: u64,
    pub ram_used: u64,
    pub groups: Vec<Group>,
}

/// Remembers the previous sample so CPU and disk can be shown as rates.
#[derive(Default)]
pub struct Sampler {
    last: HashMap<u32, (u64, u64)>,
    last_at: Option<Instant>,
}

impl Sampler {
    pub fn snapshot(&mut self) -> Snapshot {
        let raws = platform::processes();
        let (ram_total, ram_used) = platform::memory();
        let cpu_count = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);

        let now = Instant::now();
        let elapsed = self
            .last_at
            .map(|t| now.duration_since(t).as_secs_f64())
            .unwrap_or(0.0);
        let mut next = HashMap::with_capacity(raws.len());
        let mut procs = Vec::with_capacity(raws.len());
        for r in raws {
            let (mut cpu, mut io_bps) = (0.0f32, 0.0f64);
            if let (Some(t), Some(io)) = (r.cpu_time, r.io_bytes) {
                if let (Some(&(pt, pio)), true) = (self.last.get(&r.pid), elapsed > 0.05) {
                    let dt = t.saturating_sub(pt) as f64 / 10_000_000.0; // 100 ns → s
                    cpu = ((dt / elapsed / cpu_count as f64) * 100.0).min(100.0) as f32;
                    io_bps = io.saturating_sub(pio) as f64 / elapsed;
                }
                next.insert(r.pid, (t, io));
            }
            procs.push((r, cpu, io_bps));
        }
        self.last = next;
        self.last_at = Some(now);

        let groups = group(procs);
        let cpu_total = groups.iter().map(|g| g.cpu).sum::<f32>().min(100.0);
        Snapshot {
            cpu_count,
            cpu_total,
            ram_total,
            ram_used,
            groups,
        }
    }
}

/// Groups processes by catalog app, Windows itself, or exe name.
pub fn group(procs: Vec<(RawProc, f32, f64)>) -> Vec<Group> {
    let mut map: HashMap<String, Group> = HashMap::new();
    for (r, cpu, io_bps) in procs {
        let path = r.path.as_deref().unwrap_or("");
        let (key, name, verdict, about) = if let Some(app) = catalog::find(&[&r.exe, path]) {
            (
                app.name.to_string(),
                app.name.to_string(),
                app.verdict,
                Some(app.about),
            )
        } else if catalog::is_system_name(&r.exe) || catalog::is_windows_path(path) || r.pid <= 4 {
            (
                "windows".to_string(),
                "Windows".to_string(),
                Verdict::System,
                Some("Súčasti Windowsu. Nechaj ich tak."),
            )
        } else {
            let stem = r
                .exe
                .strip_suffix(".exe")
                .or_else(|| r.exe.strip_suffix(".EXE"))
                .unwrap_or(&r.exe);
            (
                format!("exe:{}", r.exe.to_lowercase()),
                stem.to_string(),
                Verdict::Unknown,
                None,
            )
        };
        let g = map.entry(key.clone()).or_insert_with(|| Group {
            key,
            name,
            verdict,
            about,
            cpu: 0.0,
            ram: 0,
            io_bps: 0.0,
            procs: Vec::new(),
        });
        g.cpu += cpu;
        g.ram += r.ram.unwrap_or(0);
        g.io_bps += io_bps;
        g.procs.push(Proc {
            pid: r.pid,
            exe: r.exe,
            path: r.path,
            ram: r.ram,
            cpu,
            io_bps,
        });
    }
    let mut groups: Vec<Group> = map.into_values().collect();
    for g in &mut groups {
        g.procs
            .sort_by(|a, b| b.ram.unwrap_or(0).cmp(&a.ram.unwrap_or(0)));
    }
    groups.sort_by(|a, b| b.ram.cmp(&a.ram));
    groups
}

#[cfg(windows)]
mod platform {
    use super::RawProc;
    use windows::core::PWSTR;
    use windows::Win32::Foundation::{CloseHandle, FILETIME};
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };
    use windows::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
    use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    use windows::Win32::System::Threading::{
        GetProcessIoCounters, GetProcessTimes, OpenProcess, QueryFullProcessImageNameW,
        IO_COUNTERS, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
    };

    fn wide(buf: &[u16]) -> String {
        let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        String::from_utf16_lossy(&buf[..end])
    }

    fn ft(f: FILETIME) -> u64 {
        ((f.dwHighDateTime as u64) << 32) | f.dwLowDateTime as u64
    }

    pub fn processes() -> Vec<RawProc> {
        let mut out = Vec::new();
        unsafe {
            let Ok(snap) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else {
                return out;
            };
            let mut e = PROCESSENTRY32W {
                dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
                ..Default::default()
            };
            let mut ok = Process32FirstW(snap, &mut e).is_ok();
            while ok {
                let pid = e.th32ProcessID;
                let mut p = RawProc {
                    pid,
                    exe: wide(&e.szExeFile),
                    path: None,
                    ram: None,
                    cpu_time: None,
                    io_bytes: None,
                };
                if pid != 0 {
                    if let Ok(h) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
                        let mut buf = [0u16; 1024];
                        let mut len = buf.len() as u32;
                        if QueryFullProcessImageNameW(
                            h,
                            PROCESS_NAME_WIN32,
                            PWSTR(buf.as_mut_ptr()),
                            &mut len,
                        )
                        .is_ok()
                        {
                            p.path = Some(String::from_utf16_lossy(&buf[..len as usize]));
                        }
                        let cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
                        let mut mem = PROCESS_MEMORY_COUNTERS {
                            cb,
                            ..Default::default()
                        };
                        if GetProcessMemoryInfo(h, &mut mem, cb).is_ok() {
                            p.ram = Some(mem.WorkingSetSize as u64);
                        }
                        let (mut c, mut x, mut k, mut u) = (
                            FILETIME::default(),
                            FILETIME::default(),
                            FILETIME::default(),
                            FILETIME::default(),
                        );
                        if GetProcessTimes(h, &mut c, &mut x, &mut k, &mut u).is_ok() {
                            p.cpu_time = Some(ft(k) + ft(u));
                        }
                        let mut io = IO_COUNTERS::default();
                        if GetProcessIoCounters(h, &mut io).is_ok() {
                            p.io_bytes = Some(io.ReadTransferCount + io.WriteTransferCount);
                        }
                        let _ = CloseHandle(h);
                    }
                }
                out.push(p);
                ok = Process32NextW(snap, &mut e).is_ok();
            }
            let _ = CloseHandle(snap);
        }
        out
    }

    pub fn memory() -> (u64, u64) {
        unsafe {
            let mut m = MEMORYSTATUSEX {
                dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
                ..Default::default()
            };
            if GlobalMemoryStatusEx(&mut m).is_ok() {
                (
                    m.ullTotalPhys,
                    m.ullTotalPhys.saturating_sub(m.ullAvailPhys),
                )
            } else {
                (0, 0)
            }
        }
    }
}

#[cfg(not(windows))]
mod platform {
    pub fn processes() -> Vec<super::RawProc> {
        Vec::new()
    }
    pub fn memory() -> (u64, u64) {
        (0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw(pid: u32, exe: &str, path: Option<&str>, ram_mb: u64) -> (RawProc, f32, f64) {
        (
            RawProc {
                pid,
                exe: exe.into(),
                path: path.map(Into::into),
                ram: Some(ram_mb << 20),
                cpu_time: None,
                io_bytes: None,
            },
            1.0,
            0.0,
        )
    }

    #[test]
    fn groups_like_a_person_would() {
        let g = group(vec![
            raw(
                10,
                "GCC.exe",
                Some(r"C:\Program Files\GIGABYTE\Control Center\GCC.exe"),
                21,
            ),
            raw(
                11,
                "GvFPS.exe",
                Some(r"C:\Program Files\GIGABYTE\Control Center\Lib\GBT_VGA\Service\Dll\GvFPS.exe"),
                13,
            ),
            raw(12, "svchost.exe", None, 5),
            raw(13, "dwm.exe", Some(r"C:\Windows\System32\dwm.exe"), 100),
            raw(
                14,
                "chrome.exe",
                Some(r"C:\Program Files\Google\Chrome\Application\chrome.exe"),
                300,
            ),
            raw(
                15,
                "chrome.exe",
                Some(r"C:\Program Files\Google\Chrome\Application\chrome.exe"),
                200,
            ),
        ]);
        let names: Vec<_> = g
            .iter()
            .map(|g| (g.name.as_str(), g.procs.len(), g.ram >> 20))
            .collect();
        assert_eq!(
            names,
            vec![
                ("chrome", 2, 500),
                ("Windows", 2, 105),
                ("GIGABYTE Control Center", 2, 34)
            ]
        );
        assert_eq!(g[2].verdict, Verdict::Rgb);
        assert!((g[0].cpu - 2.0).abs() < f32::EPSILON);
    }
}
