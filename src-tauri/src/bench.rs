//! FPS measurement with Intel PresentMon (MIT licensed), bundled in the exe
//! and extracted on first use.
//!
//! PresentMon reads frame timing from Windows' own event tracing, so it works
//! with any game without injecting anything. That needs admin rights, so each
//! measurement shows one UAC prompt. While it records we also sample the
//! graphics card (temperature, clocks, throttling).
//!
//! Besides FPS, PresentMon tells how long the CPU and the GPU worked on each
//! frame, so we can say what holds the game back: a GPU-bound game wants
//! lower settings or DLSS/FSR, a CPU-bound one wants faster RAM (XMP) or
//! fewer things in the background.

use crate::gpu;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Run {
    /// Milliseconds since epoch, also the id.
    pub id: u64,
    pub exe: String,
    pub label: String,
    pub seconds: u32,
    pub frames: u32,
    pub avg_fps: f64,
    /// FPS of the slowest 1 % / 0.1 % of frames (99th / 99.9th percentile frame time).
    pub low1: f64,
    pub low01: f64,
    pub frametime: f64,
    pub cpu_busy: Option<f64>,
    pub gpu_busy: Option<f64>,
    /// "gpu", "cpu", "limit" (FPS cap / V-Sync) or "mixed"; empty when unknown.
    pub bound: String,
    pub gpu: Option<gpu::Summary>,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct Status {
    /// "idle", "starting" (UAC), "running", "done", "error"
    pub phase: String,
    pub exe: String,
    pub delay: u32,
    pub seconds: u32,
    /// When recording effectively began (after the UAC prompt), ms since epoch.
    pub started_ms: u64,
    pub result: Option<Run>,
    pub error: Option<String>,
    pub live: Option<gpu::Sample>,
}

/// Frame data from a PresentMon CSV, already narrowed to the main swap chain.
#[derive(Debug, Default, PartialEq)]
pub struct Frames {
    pub frametimes: Vec<f64>,
    pub cpu_busy: Vec<f64>,
    pub gpu_busy: Vec<f64>,
}

/// Reads a PresentMon CSV (1.x or 2.x columns). Overlays like Discord or the
/// Steam overlay can add their own swap chains, so only the one with the
/// most frames is kept.
pub fn parse_csv(text: &str) -> Frames {
    let mut lines = text.lines().filter(|l| !l.trim().is_empty());
    let Some(header) = lines.next() else { return Frames::default() };
    let cols: Vec<String> = header.split(',').map(|c| c.trim().to_lowercase()).collect();
    let find = |names: &[&str]| names.iter().find_map(|n| cols.iter().position(|c| c == n));
    let Some(ft) = find(&["msbetweenpresents", "frametime", "msbetweenappstart"]) else {
        return Frames::default();
    };
    let cpu = find(&["mscpubusy", "cpubusy"]);
    let gpu = find(&["msgpubusy", "gpubusy"]);
    let chain = find(&["swapchainaddress"]);

    let mut by_chain: std::collections::HashMap<String, Frames> = std::collections::HashMap::new();
    for l in lines {
        let f: Vec<&str> = l.split(',').collect();
        let get = |i: Option<usize>| i.and_then(|i| f.get(i)).and_then(|v| v.trim().parse::<f64>().ok());
        let Some(t) = get(Some(ft)) else { continue };
        if !(t > 0.0 && t < 5000.0) {
            continue;
        }
        let key = chain.and_then(|c| f.get(c)).map(|s| s.to_string()).unwrap_or_default();
        let e = by_chain.entry(key).or_default();
        e.frametimes.push(t);
        if let Some(c) = get(cpu) {
            e.cpu_busy.push(c);
        }
        if let Some(g) = get(gpu) {
            e.gpu_busy.push(g);
        }
    }
    by_chain
        .into_values()
        .max_by_key(|f| f.frametimes.len())
        .unwrap_or_default()
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn mean(v: &[f64]) -> Option<f64> {
    (!v.is_empty()).then(|| v.iter().sum::<f64>() / v.len() as f64)
}

/// avg FPS, 1 % low, 0.1 % low, avg frame time, CPU busy, GPU busy, bottleneck.
type Stats = (f64, f64, f64, f64, Option<f64>, Option<f64>, String);

/// Turns frames into the numbers people compare.
pub fn stats(f: &Frames) -> Option<Stats> {
    if f.frametimes.len() < 30 {
        return None;
    }
    let total: f64 = f.frametimes.iter().sum();
    let avg_fps = f.frametimes.len() as f64 * 1000.0 / total;
    let mut sorted = f.frametimes.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let low1 = 1000.0 / percentile(&sorted, 0.99);
    let low01 = 1000.0 / percentile(&sorted, 0.999);
    let ft = total / f.frametimes.len() as f64;
    let cpu = mean(&f.cpu_busy);
    let gpu = mean(&f.gpu_busy);
    let bound = match (cpu, gpu) {
        (Some(c), Some(g)) => {
            let (c, g) = (c / ft, g / ft);
            if g >= 0.88 {
                "gpu"
            } else if c >= 0.8 {
                "cpu"
            } else if c < 0.7 && g < 0.7 {
                "limit"
            } else {
                "mixed"
            }
        }
        _ => "",
    };
    Some((avg_fps, low1, low01, ft, cpu, gpu, bound.to_string()))
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn data_dir() -> Option<PathBuf> {
    std::env::var("LOCALAPPDATA").ok().map(|d| PathBuf::from(d).join("WhatsInside"))
}

fn store() -> Option<PathBuf> {
    data_dir().map(|d| d.join("benchmarks.json"))
}

pub fn list() -> Vec<Run> {
    store()
        .and_then(|p| std::fs::read(p).ok())
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}

fn save(runs: &[Run]) {
    let Some(p) = store() else { return };
    if let Some(d) = p.parent() {
        let _ = std::fs::create_dir_all(d);
    }
    if let Ok(j) = serde_json::to_vec_pretty(runs) {
        let _ = std::fs::write(p, j);
    }
}

pub fn delete(id: u64) {
    let mut r = list();
    r.retain(|x| x.id != id);
    save(&r);
}

/// Exe names only, e.g. "cs2.exe".
pub fn valid_exe(exe: &str) -> bool {
    let e = exe.to_lowercase();
    e.ends_with(".exe")
        && e.len() < 120
        && !e.contains(['\\', '/', '"', ':', '*', '?', '<', '>', '|'])
}

#[derive(Default)]
pub struct Bench {
    st: Arc<Mutex<Status>>,
}

impl Bench {
    pub fn status(&self) -> Status {
        let mut s = self.st.lock().map(|s| s.clone()).unwrap_or_default();
        if s.phase.is_empty() {
            s.phase = "idle".into();
        }
        s
    }

    pub fn start(&self, exe: String, label: String, seconds: u32, delay: u32) -> Result<(), String> {
        if !valid_exe(&exe) {
            return Err("Neplatný názov hry.".into());
        }
        let seconds = seconds.clamp(10, 300);
        let delay = delay.min(60);
        {
            let mut s = self.st.lock().map_err(|e| e.to_string())?;
            if s.phase == "starting" || s.phase == "running" {
                return Err("Jedno meranie už beží.".into());
            }
            *s = Status {
                phase: "starting".into(),
                exe: exe.clone(),
                delay,
                seconds,
                ..Default::default()
            };
        }
        let st = self.st.clone();
        std::thread::spawn(move || {
            let res = run(&st, &exe, &label, seconds, delay);
            if let Ok(mut s) = st.lock() {
                match res {
                    Ok(r) => {
                        s.phase = "done".into();
                        s.result = Some(r);
                    }
                    Err(e) => {
                        s.phase = "error".into();
                        s.error = Some(e);
                    }
                }
                s.live = None;
            }
        });
        Ok(())
    }

    pub fn cancel(&self) -> Result<(), String> {
        platform::stop_session()
    }
}

fn run(st: &Arc<Mutex<Status>>, exe: &str, label: &str, seconds: u32, delay: u32) -> Result<Run, String> {
    let pm = platform::presentmon()?;
    let dir = std::env::temp_dir();
    let csv = dir.join(format!("whatsinside-fps-{}.csv", now_ms()));
    let args = vec![
        "--process_name".to_string(),
        exe.to_string(),
        "--output_file".into(),
        csv.to_string_lossy().into_owned(),
        "--delay".into(),
        delay.to_string(),
        "--timed".into(),
        seconds.to_string(),
        "--terminate_after_timed".into(),
        "--no_console_stats".into(),
        "--stop_existing_session".into(),
        "--session_name".into(),
        "WhatsInside".into(),
    ];

    // Sample the graphics card while recording.
    let stop = Arc::new(AtomicBool::new(false));
    let samples = Arc::new(Mutex::new(Vec::<gpu::Sample>::new()));
    let started = Arc::new(Mutex::new(None::<u64>));
    let sampler = {
        let (stop, samples, started, st) = (stop.clone(), samples.clone(), started.clone(), st.clone());
        std::thread::spawn(move || {
            while !stop.load(Ordering::Relaxed) {
                let begun = *started.lock().unwrap_or_else(|e| e.into_inner());
                if let Some(t0) = begun {
                    if let Some(s) = gpu::sample() {
                        if now_ms() >= t0 + delay as u64 * 1000 {
                            samples.lock().unwrap_or_else(|e| e.into_inner()).push(s.clone());
                        }
                        if let Ok(mut x) = st.lock() {
                            x.live = Some(s);
                        }
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(1500));
            }
        })
    };

    let on_started = || {
        let t = now_ms();
        *started.lock().unwrap_or_else(|e| e.into_inner()) = Some(t);
        if let Ok(mut s) = st.lock() {
            s.phase = "running".into();
            s.started_ms = t;
        }
    };
    let res = platform::run_presentmon(&pm, &args, on_started);
    stop.store(true, Ordering::Relaxed);
    let _ = sampler.join();
    res?;

    let text = std::fs::read_to_string(&csv).unwrap_or_default();
    let _ = std::fs::remove_file(&csv);
    let frames = parse_csv(&text);
    let (avg_fps, low1, low01, frametime, cpu_busy, gpu_busy, bound) = stats(&frames).ok_or(
        "Nezachytil som dosť snímkov. Bola hra spustená a mala si ju počas merania v popredí?",
    )?;
    let samples = samples.lock().map(|s| s.clone()).unwrap_or_default();
    let r = Run {
        id: now_ms(),
        exe: exe.to_string(),
        label: label.chars().take(60).collect(),
        seconds,
        frames: frames.frametimes.len() as u32,
        avg_fps,
        low1,
        low01,
        frametime,
        cpu_busy,
        gpu_busy,
        bound,
        gpu: gpu::summarize(&samples),
    };
    let mut all = list();
    all.push(r.clone());
    if all.len() > 200 {
        let cut = all.len() - 200;
        all.drain(..cut);
    }
    save(&all);
    Ok(r)
}

#[cfg(all(windows, has_presentmon))]
const PRESENTMON: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/bin/PresentMon.exe"));

#[cfg(windows)]
mod platform {
    use std::path::{Path, PathBuf};
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{GetExitCodeProcess, WaitForSingleObject, INFINITE};
    use windows::Win32::UI::Shell::{ShellExecuteExW, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW};

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    #[cfg(has_presentmon)]
    pub fn presentmon() -> Result<PathBuf, String> {
        let bytes = super::PRESENTMON;
        let hash = crate::update::sha256_hex(bytes);
        let dir = super::data_dir().ok_or("Chýba LOCALAPPDATA.")?.join("tools");
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let path = dir.join(format!("PresentMon-{}.exe", &hash[..12]));
        let ok = std::fs::read(&path).map(|b| crate::update::sha256_hex(&b) == hash).unwrap_or(false);
        if !ok {
            std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
        }
        Ok(path)
    }

    #[cfg(not(has_presentmon))]
    pub fn presentmon() -> Result<PathBuf, String> {
        Err("Táto zostava nemá pribalený PresentMon.".into())
    }

    /// Checks the extracted copy right before it runs as admin.
    fn verify(path: &Path) -> Result<(), String> {
        #[cfg(has_presentmon)]
        {
            let b = std::fs::read(path).map_err(|e| e.to_string())?;
            if crate::update::sha256_hex(&b) != crate::update::sha256_hex(super::PRESENTMON) {
                return Err("PresentMon na disku nesedí s pribalenou verziou.".into());
            }
        }
        let _ = path;
        Ok(())
    }

    fn quote(a: &str) -> String {
        if a.contains([' ', '\t']) { format!("\"{a}\"") } else { a.to_string() }
    }

    /// Runs PresentMon elevated (one UAC prompt) and waits for it.
    pub fn run_presentmon(pm: &Path, args: &[String], on_started: impl FnOnce()) -> Result<(), String> {
        verify(pm)?;
        let params = args.iter().map(|a| quote(a)).collect::<Vec<_>>().join(" ");
        let (verb, file, params) = (wide("runas"), wide(&pm.to_string_lossy()), wide(&params));
        unsafe {
            let mut info = SHELLEXECUTEINFOW {
                cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
                fMask: SEE_MASK_NOCLOSEPROCESS,
                lpVerb: PCWSTR(verb.as_ptr()),
                lpFile: PCWSTR(file.as_ptr()),
                lpParameters: PCWSTR(params.as_ptr()),
                nShow: 0,
                ..Default::default()
            };
            if ShellExecuteExW(&mut info).is_err() {
                return Err("Meranie potrebuje povolenie správcu. Okno UAC bolo zrušené.".into());
            }
            on_started();
            if !info.hProcess.is_invalid() {
                WaitForSingleObject(info.hProcess, INFINITE);
                let mut code = 0u32;
                let _ = GetExitCodeProcess(info.hProcess, &mut code);
                let _ = CloseHandle(info.hProcess);
                if code != 0 {
                    return Err(format!("PresentMon skončil s chybou {code}."));
                }
            }
        }
        Ok(())
    }

    pub fn stop_session() -> Result<(), String> {
        let pm = presentmon()?;
        let args = ["--terminate_existing_session".to_string(), "--session_name".into(), "WhatsInside".into()];
        run_presentmon(&pm, &args, || {})
    }
}

#[cfg(not(windows))]
mod platform {
    use std::path::{Path, PathBuf};
    pub fn presentmon() -> Result<PathBuf, String> {
        Err("Len na Windows.".into())
    }
    pub fn run_presentmon(_: &Path, _: &[String], _: impl FnOnce()) -> Result<(), String> {
        Err("Len na Windows.".into())
    }
    pub fn stop_session() -> Result<(), String> {
        Err("Len na Windows.".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn csv(rows: &[(&str, f64, f64, f64)]) -> String {
        let mut s = String::from("Application,ProcessID,SwapChainAddress,PresentRuntime,MsCPUBusy,MsGPUBusy,MsBetweenPresents\n");
        for (chain, ft, c, g) in rows {
            s.push_str(&format!("cs2.exe,100,{chain},DXGI,{c},{g},{ft}\n"));
        }
        s
    }

    #[test]
    fn keeps_the_main_swap_chain() {
        let mut rows = vec![("0x1", 10.0, 9.0, 4.0); 200];
        rows.extend(vec![("0x2", 33.0, 1.0, 1.0); 20]); // overlay
        let f = parse_csv(&csv(&rows));
        assert_eq!(f.frametimes.len(), 200);
        let (fps, low1, _, ft, _, _, bound) = stats(&f).unwrap();
        assert!((fps - 100.0).abs() < 0.01);
        assert!((low1 - 100.0).abs() < 0.01);
        assert!((ft - 10.0).abs() < 0.01);
        assert_eq!(bound, "cpu");
    }

    #[test]
    fn finds_lows_and_bottleneck() {
        // 985 frames at 10 ms, 15 hitches at 50 ms, GPU busy almost all the time.
        let mut rows = vec![("0x1", 10.0, 4.0, 9.5); 985];
        rows.extend(vec![("0x1", 50.0, 4.0, 9.5); 15]);
        let (fps, low1, low01, _, _, _, bound) = stats(&parse_csv(&csv(&rows))).unwrap();
        assert!(fps > 90.0 && fps < 100.0);
        assert!(low1 < 25.0, "1% low should catch the hitches: {low1}");
        assert!((low01 - 20.0).abs() < 0.01);
        assert_eq!(bound, "gpu");
    }

    #[test]
    fn handles_v1_columns_and_caps() {
        let text = "Application,ProcessID,SwapChainAddress,MsBetweenPresents\ngame.exe,1,0x1,16.6\n".to_string();
        let mut t = text.clone();
        for _ in 0..50 {
            t.push_str("game.exe,1,0x1,16.6\n");
        }
        let f = parse_csv(&t);
        let (fps, .., bound) = stats(&f).unwrap();
        assert!((fps - 60.24).abs() < 0.1);
        assert_eq!(bound, "");
        // FPS cap: both CPU and GPU mostly idle.
        let rows = vec![("0x1", 16.6, 5.0, 6.0); 100];
        assert_eq!(stats(&parse_csv(&csv(&rows))).unwrap().6, "limit");
        assert!(stats(&parse_csv("nonsense")).is_none());
    }

    #[test]
    fn only_plain_exe_names() {
        assert!(valid_exe("cs2.exe"));
        assert!(valid_exe("Cyberpunk2077.exe"));
        assert!(!valid_exe("..\\evil.exe"));
        assert!(!valid_exe("a.exe\" --etl_file x"));
        assert!(!valid_exe("cmd"));
    }
}
