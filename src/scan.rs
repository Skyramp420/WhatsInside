//! Read-only system scan. Nothing in here opens a device for writing,
//! sends a report, or changes any setting.

use crate::report::Report;

#[cfg(windows)]
pub fn run() -> Report {
    let mut r = Report {
        version: env!("CARGO_PKG_VERSION"),
        ..Default::default()
    };

    match win::hid_devices() {
        Ok(v) => r.hid = v,
        Err(e) => r.errors.push(format!("USB HID zariadenia: {e}")),
    }
    match win::display_adapters() {
        Ok(v) => r.gpus = v,
        Err(e) => r.errors.push(format!("Grafické karty: {e}")),
    }
    r.nvapi_present = win::nvapi_present();
    match win::conflicting_processes() {
        Ok(v) => r.processes = v,
        Err(e) => r.errors.push(format!("Bežiace procesy: {e}")),
    }
    r.autostart = win::autostart_entries(&mut r.errors);
    r
}

#[cfg(not(windows))]
pub fn run() -> Report {
    Report {
        version: env!("CARGO_PKG_VERSION"),
        errors: vec!["Táto appka funguje len na Windows.".into()],
        ..Default::default()
    }
}

#[cfg(windows)]
mod win {
    use super::parse_reg_query;
    use crate::known;
    use crate::report::{AutostartEntry, ConflictProcess, Gpu, HidDevice};
    use std::process::Command;
    use windows::core::{PCWSTR, PWSTR};
    use windows::Win32::Devices::DeviceAndDriverInstallation::{
        SetupDiDestroyDeviceInfoList, SetupDiEnumDeviceInfo, SetupDiGetClassDevsW,
        SetupDiGetDeviceInstanceIdW, DIGCF_PRESENT, GUID_DEVCLASS_DISPLAY, SP_DEVINFO_DATA,
    };
    use windows::Win32::Foundation::{CloseHandle, HANDLE};
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };
    use windows::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };

    fn wide_to_string(buf: &[u16]) -> String {
        let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        String::from_utf16_lossy(&buf[..end])
    }

    /// Lists HID interfaces. Enumeration only reads descriptors the OS
    /// already has; no device is opened for I/O.
    pub fn hid_devices() -> Result<Vec<HidDevice>, String> {
        let api = hidapi::HidApi::new().map_err(|e| e.to_string())?;
        Ok(api
            .device_list()
            .map(|d| HidDevice {
                vid: d.vendor_id(),
                pid: d.product_id(),
                usage_page: d.usage_page(),
                usage: d.usage(),
                interface: d.interface_number(),
                manufacturer: d.manufacturer_string().map(str::to_owned),
                product: d.product_string().map(str::to_owned),
            })
            .collect())
    }

    /// Instance IDs of present display adapters (PCI\VEN_..&SUBSYS_..).
    pub fn display_adapters() -> Result<Vec<Gpu>, String> {
        let mut out = Vec::new();
        unsafe {
            let set = SetupDiGetClassDevsW(
                Some(&GUID_DEVCLASS_DISPLAY as *const _),
                PCWSTR::null(),
                None,
                DIGCF_PRESENT,
            )
            .map_err(|e| e.to_string())?;
            let mut i = 0u32;
            loop {
                let mut info = SP_DEVINFO_DATA {
                    cbSize: std::mem::size_of::<SP_DEVINFO_DATA>() as u32,
                    ..Default::default()
                };
                if SetupDiEnumDeviceInfo(set, i, &mut info).is_err() {
                    break;
                }
                let mut buf = [0u16; 512];
                if SetupDiGetDeviceInstanceIdW(set, &info, Some(&mut buf), None).is_ok() {
                    out.push(Gpu {
                        instance_id: wide_to_string(&buf),
                    });
                }
                i += 1;
            }
            let _ = SetupDiDestroyDeviceInfoList(set);
        }
        Ok(out)
    }

    /// NVAPI ships with the NVIDIA driver. We only check the DLL exists;
    /// we do not load it in v0.1.
    pub fn nvapi_present() -> bool {
        let root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into());
        std::path::Path::new(&root)
            .join(r"System32\nvapi64.dll")
            .exists()
    }

    fn process_details(pid: u32) -> (Option<String>, Option<u64>) {
        unsafe {
            let Ok(h) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else {
                return (None, None);
            };
            let mut buf = [0u16; 1024];
            let mut len = buf.len() as u32;
            let path = QueryFullProcessImageNameW(
                h,
                PROCESS_NAME_WIN32,
                PWSTR(buf.as_mut_ptr()),
                &mut len,
            )
            .ok()
            .map(|_| String::from_utf16_lossy(&buf[..len as usize]));
            let cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
            let mut mem = PROCESS_MEMORY_COUNTERS {
                cb,
                ..Default::default()
            };
            let ram = GetProcessMemoryInfo(h, &mut mem, cb)
                .ok()
                .map(|_| mem.WorkingSetSize as u64);
            let _ = CloseHandle(h);
            (path, ram)
        }
    }

    pub fn conflicting_processes() -> Result<Vec<ConflictProcess>, String> {
        let mut out = Vec::new();
        unsafe {
            let snap: HANDLE =
                CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).map_err(|e| e.to_string())?;
            let mut e = PROCESSENTRY32W {
                dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
                ..Default::default()
            };
            let mut ok = Process32FirstW(snap, &mut e).is_ok();
            while ok {
                let exe = wide_to_string(&e.szExeFile);
                let pid = e.th32ProcessID;
                // Path is needed too: some vendor tools have generic exe names
                // and are only recognisable by their install folder.
                let (path, ram) = process_details(pid);
                if let Some(app) = known::match_conflict(&exe, path.as_deref()) {
                    out.push(ConflictProcess {
                        app: app.name,
                        peripheral: app.peripheral,
                        controls: app.controls,
                        exe,
                        pid,
                        path,
                        ram_bytes: ram,
                    });
                }
                ok = Process32NextW(snap, &mut e).is_ok();
            }
            let _ = CloseHandle(snap);
        }
        Ok(out)
    }

    /// Autostart entries from the Run keys. Uses `reg query` so we stay
    /// strictly read-only and don't need registry API bindings.
    pub fn autostart_entries(errors: &mut Vec<String>) -> Vec<AutostartEntry> {
        const KEYS: &[&str] = &[
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
            r"HKLM\Software\Microsoft\Windows\CurrentVersion\Run",
            r"HKLM\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Run",
        ];
        let mut out = Vec::new();
        for key in KEYS {
            let output = match Command::new("reg").args(["query", key]).output() {
                Ok(o) => o,
                Err(e) => {
                    errors.push(format!("Autostart ({key}): {e}"));
                    continue;
                }
            };
            let text = String::from_utf8_lossy(&output.stdout);
            for (name, command) in parse_reg_query(&text) {
                if let Some(app) = known::match_conflict(&name, Some(&command)) {
                    out.push(AutostartEntry {
                        app: app.name,
                        peripheral: app.peripheral,
                        location: key.to_string(),
                        name,
                        command,
                    });
                }
            }
        }
        out
    }
}

/// Parses lines like `    Name    REG_SZ    "C:\path\app.exe" -flag`.
#[cfg_attr(not(windows), allow(dead_code))]
fn parse_reg_query(text: &str) -> Vec<(String, String)> {
    text.lines()
        .filter_map(|l| {
            let l = l.trim_start();
            let idx = l.find("    REG_")?;
            let name = l[..idx].trim().to_string();
            let rest = l[idx..].trim_start();
            let value = rest
                .split_once(char::is_whitespace)
                .map(|x| x.1)
                .unwrap_or("")
                .trim()
                .to_string();
            Some((name, value))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn parses_reg_output() {
        let out = "\r\nHKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Run\r\n    SignalRGB    REG_SZ    \"C:\\Users\\Alex\\AppData\\Local\\VortxEngine\\SignalRgbLauncher.exe\" --silent\r\n    Discord    REG_SZ    C:\\Discord\\Update.exe\r\n";
        let v = super::parse_reg_query(out);
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].0, "SignalRGB");
        assert!(v[0].1.ends_with("--silent"));
    }
}
