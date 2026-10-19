//! What starts together with Windows. Read-only in v0.2.
//!
//! Sources: Run keys (with the enabled/disabled state Task Manager uses),
//! Startup folders and third-party services set to start automatically.
//! Vendor apps like MSI Center or GIGABYTE Control Center usually start
//! as services, which is why the Run keys alone aren't enough.

use crate::catalog::{self, Verdict};
use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct Item {
    /// Where it comes from, shown to the user.
    pub source: String,
    pub name: String,
    pub command: String,
    pub app: Option<&'static str>,
    pub verdict: Verdict,
    pub about: Option<&'static str>,
    /// `false` when it is registered but switched off (e.g. in Task Manager).
    pub enabled: bool,
    /// For services: currently running.
    pub running: Option<bool>,
}

fn classify(name: &str, command: &str) -> (Option<&'static str>, Verdict, Option<&'static str>) {
    match catalog::find(&[name, command]) {
        Some(a) => (Some(a.name), a.verdict, Some(a.about)),
        None => (None, Verdict::Unknown, None),
    }
}

pub fn list() -> Vec<Item> {
    let mut items = platform::run_keys();
    items.extend(platform::startup_folders());
    items.extend(platform::services());
    // Interesting things first: vendor/optional apps, then unknown, then needed.
    let rank = |v: Verdict| match v {
        Verdict::Rgb => 0,
        Verdict::Optional => 1,
        Verdict::Peripheral => 2,
        Verdict::Unknown => 3,
        Verdict::Needed => 4,
        Verdict::System => 5,
    };
    items.sort_by(|a, b| {
        rank(a.verdict)
            .cmp(&rank(b.verdict))
            .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    items
}

/// Parses `reg query` output lines: `    Name    REG_SZ    value`.
pub fn parse_reg_query(text: &str) -> Vec<(String, String)> {
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

/// StartupApproved values are binary; the first byte is even (02, 06) when
/// the entry is enabled and odd (03, 07…) when switched off.
pub fn approved_enabled(hex: &str) -> bool {
    let first = hex
        .get(0..2)
        .and_then(|b| u8::from_str_radix(b, 16).ok())
        .unwrap_or(2);
    first & 1 == 0
}

#[cfg(windows)]
mod platform {
    use super::{approved_enabled, classify, parse_reg_query, Item};
    use std::collections::HashMap;
    use std::os::windows::process::CommandExt;
    use std::process::Command;
    use windows::core::{PCWSTR, PWSTR};
    use windows::Win32::System::Services::{
        CloseServiceHandle, EnumServicesStatusExW, OpenSCManagerW, OpenServiceW,
        QueryServiceConfigW, ENUM_SERVICE_STATUS_PROCESSW, QUERY_SERVICE_CONFIGW,
        SC_ENUM_PROCESS_INFO, SC_MANAGER_CONNECT, SC_MANAGER_ENUMERATE_SERVICE, SERVICE_AUTO_START,
        SERVICE_QUERY_CONFIG, SERVICE_RUNNING, SERVICE_STATE_ALL, SERVICE_WIN32,
    };

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    fn reg_query(key: &str) -> String {
        Command::new("reg")
            .args(["query", key])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
            .unwrap_or_default()
    }

    pub fn run_keys() -> Vec<Item> {
        const RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
        const APPROVED: &str =
            r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";
        const APPROVED32: &str =
            r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run32";
        let sources = [
            (
                "HKCU",
                RUN.to_string(),
                APPROVED,
                "Pri prihlásení (používateľ)",
            ),
            ("HKLM", RUN.to_string(), APPROVED, "Pri prihlásení (všetci)"),
            (
                "HKLM",
                RUN.replace("Software\\", "Software\\WOW6432Node\\"),
                APPROVED32,
                "Pri prihlásení (všetci, 32-bit)",
            ),
        ];
        let mut out = Vec::new();
        for (hive, key, approved_key, label) in sources {
            let approved: HashMap<String, String> =
                parse_reg_query(&reg_query(&format!(r"{hive}\{approved_key}")))
                    .into_iter()
                    .collect();
            for (name, command) in parse_reg_query(&reg_query(&format!(r"{hive}\{key}"))) {
                let enabled = approved
                    .get(&name)
                    .map(|h| approved_enabled(h))
                    .unwrap_or(true);
                let (app, verdict, about) = classify(&name, &command);
                out.push(Item {
                    source: label.into(),
                    name,
                    command,
                    app,
                    verdict,
                    about,
                    enabled,
                    running: None,
                });
            }
        }
        out
    }

    pub fn startup_folders() -> Vec<Item> {
        let mut out = Vec::new();
        let dirs = [
            (
                std::env::var("APPDATA").ok(),
                r"Microsoft\Windows\Start Menu\Programs\Startup",
                "Priečinok Po spustení (používateľ)",
            ),
            (
                std::env::var("ProgramData").ok(),
                r"Microsoft\Windows\Start Menu\Programs\StartUp",
                "Priečinok Po spustení (všetci)",
            ),
        ];
        for (base, rel, label) in dirs {
            let Some(base) = base else { continue };
            let dir = std::path::Path::new(&base).join(rel);
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
            for e in entries.flatten() {
                let file = e.file_name().to_string_lossy().into_owned();
                if file.eq_ignore_ascii_case("desktop.ini") {
                    continue;
                }
                let name = file.trim_end_matches(".lnk").to_string();
                let command = e.path().to_string_lossy().into_owned();
                let (app, verdict, about) = classify(&name, &command);
                out.push(Item {
                    source: label.into(),
                    name,
                    command,
                    app,
                    verdict,
                    about,
                    enabled: true,
                    running: None,
                });
            }
        }
        out
    }

    fn pwstr(p: PWSTR) -> String {
        if p.is_null() {
            String::new()
        } else {
            unsafe { p.to_string().unwrap_or_default() }
        }
    }

    /// Third-party Win32 services that start automatically.
    pub fn services() -> Vec<Item> {
        let mut out = Vec::new();
        unsafe {
            let Ok(scm) = OpenSCManagerW(
                PCWSTR::null(),
                PCWSTR::null(),
                SC_MANAGER_CONNECT | SC_MANAGER_ENUMERATE_SERVICE,
            ) else {
                return out;
            };
            let mut needed = 0u32;
            let mut count = 0u32;
            let mut resume = 0u32;
            // First call sizes the buffer.
            let _ = EnumServicesStatusExW(
                scm,
                SC_ENUM_PROCESS_INFO,
                SERVICE_WIN32,
                SERVICE_STATE_ALL,
                None,
                &mut needed,
                &mut count,
                Some(&mut resume),
                PCWSTR::null(),
            );
            let mut buf = vec![0u8; needed as usize + 4096];
            resume = 0;
            if EnumServicesStatusExW(
                scm,
                SC_ENUM_PROCESS_INFO,
                SERVICE_WIN32,
                SERVICE_STATE_ALL,
                Some(&mut buf),
                &mut needed,
                &mut count,
                Some(&mut resume),
                PCWSTR::null(),
            )
            .is_ok()
            {
                let list = std::slice::from_raw_parts(
                    buf.as_ptr() as *const ENUM_SERVICE_STATUS_PROCESSW,
                    count as usize,
                );
                for s in list {
                    let svc_name = pwstr(s.lpServiceName);
                    let display = pwstr(s.lpDisplayName);
                    let running = s.ServiceStatusProcess.dwCurrentState == SERVICE_RUNNING;
                    let Ok(h) = OpenServiceW(
                        scm,
                        PCWSTR(s.lpServiceName.0 as *const u16),
                        SERVICE_QUERY_CONFIG,
                    ) else {
                        continue;
                    };
                    let mut need = 0u32;
                    let _ = QueryServiceConfigW(h, None, 0, &mut need);
                    let mut cfg_buf = vec![0u8; need.max(8) as usize];
                    let cfg_ptr = cfg_buf.as_mut_ptr() as *mut QUERY_SERVICE_CONFIGW;
                    if QueryServiceConfigW(h, Some(cfg_ptr), cfg_buf.len() as u32, &mut need)
                        .is_ok()
                    {
                        let cfg = &*cfg_ptr;
                        let binary = pwstr(cfg.lpBinaryPathName);
                        let lower = binary.to_lowercase();
                        let is_windows = lower.contains(r"\windows\")
                            || lower.contains("svchost.exe")
                            || lower.starts_with(r"\systemroot");
                        if cfg.dwStartType == SERVICE_AUTO_START && !is_windows {
                            let (app, verdict, about) =
                                classify(&format!("{svc_name} {display}"), &binary);
                            out.push(Item {
                                source: "Služba".into(),
                                name: if display.is_empty() {
                                    svc_name.clone()
                                } else {
                                    display.clone()
                                },
                                command: binary,
                                app,
                                verdict,
                                about,
                                enabled: true,
                                running: Some(running),
                            });
                        }
                    }
                    let _ = CloseServiceHandle(h);
                }
            }
            let _ = CloseServiceHandle(scm);
        }
        out
    }
}

#[cfg(not(windows))]
mod platform {
    use super::Item;
    pub fn run_keys() -> Vec<Item> {
        Vec::new()
    }
    pub fn startup_folders() -> Vec<Item> {
        Vec::new()
    }
    pub fn services() -> Vec<Item> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_reg_output() {
        let out = "\r\nHKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Run\r\n    LGHUB    REG_SZ    \"C:\\Program Files\\LGHUB\\system_tray\\lghub_system_tray.exe\" --minimized\r\n    Discord    REG_SZ    C:\\Discord\\Update.exe --processStart Discord.exe\r\n";
        let v = parse_reg_query(out);
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].0, "LGHUB");
        assert!(v[0].1.ends_with("--minimized"));
        assert_eq!(classify(&v[1].0, &v[1].1).0, Some("Discord"));
    }

    #[test]
    fn reads_task_manager_state() {
        assert!(approved_enabled("020000000000000000000000"));
        assert!(approved_enabled("060000000000000000000000"));
        assert!(!approved_enabled("03000000A1B2C3D4E5F60708"));
    }
}
