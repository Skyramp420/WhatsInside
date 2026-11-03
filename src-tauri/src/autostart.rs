//! What starts together with Windows, including the things Task Manager's
//! Startup tab doesn't show.
//!
//! Sources:
//! - Run keys and Startup folders, with the same on/off state Task Manager
//!   uses (StartupApproved), so switching here and there is the same thing.
//! - Scheduled tasks that run at logon or at boot, and third-party tasks that
//!   run on a timer (updaters love those).
//! - Services that start automatically: third-party ones, and Windows ones
//!   (with honest advice for the few people usually ask about).
//! - RunOnce and a hijacked Winlogon Shell/Userinit, which are rare but are
//!   exactly where unwanted things hide.

use crate::catalog::{self, Advice, Verdict};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// How an entry is switched on or off. Sent back by the UI as-is.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "t", rename_all = "snake_case")]
pub enum Toggle {
    /// Run key or Startup folder: the StartupApproved value Task Manager uses.
    Approved {
        hive: String,
        key: String,
        name: String,
    },
    /// Scheduled task, by its full path (`\Vendor\Task`).
    Task { path: String },
    /// Service: off = manual start, on = `on` ("auto" or "delayed-auto").
    Service { name: String, on: String },
}

#[derive(Serialize, Clone, Debug)]
pub struct Item {
    pub id: String,
    /// "run", "folder", "task", "service", "runonce", "winlogon"
    pub kind: &'static str,
    /// When it starts, shown to the user.
    pub when: String,
    /// Where it comes from, shown to the user.
    pub source: String,
    pub name: String,
    pub command: String,
    /// Lower-case exe file name, so the UI can match it with running processes.
    pub exe: Option<String>,
    pub app: Option<&'static str>,
    pub verdict: Verdict,
    pub about: Option<&'static str>,
    /// Only for built-in Windows services we have an opinion about.
    pub advice: Option<Advice>,
    /// `false` when it is registered but switched off.
    pub enabled: bool,
    /// For services: currently running.
    pub running: Option<bool>,
    /// Not shown in Task Manager's Startup tab.
    pub hidden: bool,
    /// Switching it will most likely ask for admin rights (UAC).
    pub needs_admin: bool,
    pub toggle: Option<Toggle>,
}

pub(crate) fn classify(name: &str, command: &str) -> (Option<&'static str>, Verdict, Option<&'static str>) {
    match catalog::find(&[name, command]) {
        Some(a) => (Some(a.name), a.verdict, Some(a.about)),
        None => {
            if is_ms_component(command) {
                (None, Verdict::System, Some("Súčasť Windowsu. Nechaj tak."))
            } else {
                (None, Verdict::Unknown, None)
            }
        }
    }
}

/// Windows parts that live outside C:\Windows (Defender, Edge's WebView runtime…).
pub fn is_ms_component(path: &str) -> bool {
    let p = path.to_lowercase();
    catalog::is_windows_path(path.trim_matches('"'))
        || p.contains(r"\windows\system32\")
        || p.contains(r"\windows defender")
        || p.contains(r"\microsoft\windows defender")
        || p.contains("%systemroot%")
        || p.contains("%windir%")
        || p.starts_with("system32\\")
}

/// `names` holds services the user changed with WhatsInside, so they stay in
/// the list (switched off) and can be switched back on.
pub fn list(changed_services: &HashSet<String>) -> Vec<Item> {
    let mut items = platform::run_keys();
    items.extend(platform::startup_folders());
    items.extend(platform::run_once());
    items.extend(platform::winlogon());
    items.extend(platform::tasks());
    items.extend(platform::services(changed_services));
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

/// `reg query HKLM\...\Services /s /v DelayedAutostart`: names of services
/// with the value set to 1.
pub fn parse_delayed(text: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    let mut current: Option<String> = None;
    for l in text.lines() {
        if l.starts_with("HKEY_") {
            current = l.rsplit('\\').next().map(|s| s.trim().to_lowercase());
        } else if let Some((name, value)) = parse_reg_query(l).into_iter().next() {
            if name.eq_ignore_ascii_case("DelayedAutostart") && value.trim().ends_with('1') {
                if let Some(c) = &current {
                    out.insert(c.clone());
                }
            }
        }
    }
    out
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

/// The exe file name from a command line, lower-case.
pub fn exe_of(command: &str) -> Option<String> {
    let c = command.trim();
    let path = if let Some(rest) = c.strip_prefix('"') {
        rest.split('"').next()?.to_string()
    } else {
        let lower = c.to_lowercase();
        match lower.find(".exe") {
            Some(i) => c[..i + 4].to_string(),
            None => c.split_whitespace().next()?.to_string(),
        }
    };
    let name = path.rsplit(['\\', '/']).next()?.trim().to_lowercase();
    (!name.is_empty()).then_some(name)
}

/// One scheduled task, as much as we need of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskInfo {
    pub path: String,
    pub enabled: bool,
    pub logon: bool,
    pub boot: bool,
    pub timer: bool,
    pub command: String,
    pub author: String,
}

fn xml_unescape(s: &str) -> String {
    s.replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

fn tag<'a>(xml: &'a str, name: &str) -> Option<&'a str> {
    let open = format!("<{name}>");
    let close = format!("</{name}>");
    let s = xml.find(&open)? + open.len();
    let e = xml[s..].find(&close)? + s;
    Some(&xml[s..e])
}

/// A trigger block like `<LogonTrigger>…</LogonTrigger>` that isn't disabled.
fn has_trigger(triggers: &str, name: &str) -> bool {
    let open = format!("<{name}");
    let close = format!("</{name}>");
    let mut rest = triggers;
    while let Some(i) = rest.find(&open) {
        let after = &rest[i + open.len()..];
        // `<LogonTrigger/>` or `<LogonTrigger>…</LogonTrigger>`
        if after.starts_with("/>") || after.starts_with(" />") {
            return true;
        }
        let body_end = after.find(&close).unwrap_or(after.len());
        let body = &after[..body_end];
        if !body.contains("<Enabled>false</Enabled>") {
            return true;
        }
        rest = &after[body_end..];
    }
    false
}

/// Parses `schtasks /query /xml` output: tasks come as
/// `<!-- \Path\Name -->` followed by `<Task …>…</Task>`.
pub fn parse_tasks_xml(text: &str) -> Vec<TaskInfo> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(i) = rest.find("<!--") {
        let after = &rest[i + 4..];
        let Some(j) = after.find("-->") else { break };
        let path = after[..j].trim().to_string();
        let body_start = &after[j + 3..];
        let end = body_start.find("</Task>").map(|e| e + 7).unwrap_or(body_start.len());
        let xml = &body_start[..end];
        rest = &body_start[end..];
        if !path.starts_with('\\') {
            continue;
        }
        let settings = tag(xml, "Settings").unwrap_or("");
        // Settings has no other <Enabled> children, triggers keep their own.
        let enabled = !settings.contains("<Enabled>false</Enabled>");
        let triggers = tag(xml, "Triggers").unwrap_or("");
        let command = match tag(xml, "Exec") {
            Some(exec) => {
                let cmd = xml_unescape(tag(exec, "Command").unwrap_or("").trim());
                let args = xml_unescape(tag(exec, "Arguments").unwrap_or("").trim());
                let cmd = if cmd.contains(' ') && !cmd.starts_with('"') {
                    format!("\"{cmd}\"")
                } else {
                    cmd
                };
                if args.is_empty() { cmd } else { format!("{cmd} {args}") }
            }
            None => String::new(),
        };
        out.push(TaskInfo {
            path,
            enabled,
            logon: has_trigger(triggers, "LogonTrigger"),
            boot: has_trigger(triggers, "BootTrigger"),
            timer: has_trigger(triggers, "TimeTrigger")
                || has_trigger(triggers, "CalendarTrigger"),
            command,
            author: xml_unescape(tag(xml, "Author").unwrap_or("").trim()),
        });
    }
    out
}

/// Turns a scheduled task into an item, or None if it isn't about startup.
pub fn task_item(t: &TaskInfo) -> Option<Item> {
    let microsoft = t.path.to_lowercase().starts_with("\\microsoft\\");
    // Built-in tasks are only interesting when they start with Windows.
    if !(t.logon || t.boot || (!microsoft && t.timer)) {
        return None;
    }
    if t.command.is_empty() && microsoft {
        return None; // COM handler inside Windows
    }
    let name = t.path.rsplit('\\').next().unwrap_or(&t.path).to_string();
    let (app, mut verdict, mut about) = classify(&format!("{} {}", t.path, t.author), &t.command);
    if microsoft && app.is_none() {
        verdict = Verdict::System;
        about = Some("Úloha Windowsu. Nechaj tak.");
    }
    let when = if t.boot {
        "Pri štarte Windowsu"
    } else if t.logon {
        "Pri prihlásení"
    } else {
        "Pravidelne na pozadí"
    };
    Some(Item {
        id: format!("task:{}", t.path.to_lowercase()),
        kind: "task",
        when: when.into(),
        source: format!("Naplánovaná úloha {}", t.path),
        name,
        command: if t.command.is_empty() {
            "(vnútorná súčasť, nespúšťa program)".into()
        } else {
            t.command.clone()
        },
        exe: exe_of(&t.command),
        app,
        verdict,
        about,
        advice: None,
        enabled: t.enabled,
        running: None,
        hidden: true,
        needs_admin: microsoft || t.boot,
        toggle: Some(Toggle::Task {
            path: t.path.clone(),
        }),
    })
}

/// Turns a service into an item, or None if it isn't interesting.
#[allow(clippy::too_many_arguments)]
pub fn service_item(
    key: &str,
    display: &str,
    binary: &str,
    auto: bool,
    delayed: bool,
    running: bool,
    changed: bool,
) -> Option<Item> {
    if !auto && !changed {
        return None;
    }
    let win = catalog::win_service(key);
    let windows = is_ms_component(binary) || binary.to_lowercase().contains("svchost.exe");
    let name = if display.is_empty() { key.to_string() } else { display.to_string() };
    let (app, verdict, about, advice) = if let Some(w) = win {
        let v = match w.advice {
            Advice::Keep => Verdict::System,
            _ => Verdict::Optional,
        };
        (Some(w.title), v, Some(w.about), Some(w.advice))
    } else if windows {
        (None, Verdict::System, Some("Služba Windowsu. Nechaj tak."), None)
    } else {
        let (a, v, ab) = classify(&format!("{key} {display}"), binary);
        (a, v, ab, None)
    };
    Some(Item {
        id: format!("service:{}", key.to_lowercase()),
        kind: "service",
        when: if delayed { "Po štarte Windowsu (oneskorene)".into() } else { "Pri štarte Windowsu".into() },
        source: format!("Služba {key}"),
        name,
        command: binary.to_string(),
        exe: exe_of(binary),
        app,
        verdict,
        about,
        advice,
        enabled: auto,
        running: Some(running),
        hidden: true,
        needs_admin: true,
        toggle: Some(Toggle::Service {
            name: key.to_string(),
            on: if delayed { "delayed-auto".into() } else { "auto".into() },
        }),
    })
}

#[cfg(windows)]
mod platform {
    use super::{
        approved_enabled, classify, exe_of, parse_delayed, parse_reg_query, parse_tasks_xml,
        service_item, task_item, Item, Toggle,
    };
    use crate::catalog::Verdict;
    use std::collections::{HashMap, HashSet};
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

    pub(crate) fn decode(bytes: &[u8]) -> String {
        if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
            let w: Vec<u16> = bytes[2..]
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();
            return String::from_utf16_lossy(&w);
        }
        String::from_utf8_lossy(bytes).into_owned()
    }

    fn run(cmd: &str, args: &[&str]) -> String {
        Command::new(cmd)
            .args(args)
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map(|o| decode(&o.stdout))
            .unwrap_or_default()
    }

    fn reg_query(key: &str) -> String {
        run("reg", &["query", key])
    }

    pub fn run_keys() -> Vec<Item> {
        const RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
        const APPROVED: &str =
            r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";
        const APPROVED32: &str =
            r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run32";
        let sources = [
            ("HKCU", RUN.to_string(), APPROVED, "Run (používateľ)"),
            ("HKLM", RUN.to_string(), APPROVED, "Run (všetci používatelia)"),
            (
                "HKLM",
                RUN.replace("Software\\", "Software\\WOW6432Node\\"),
                APPROVED32,
                "Run (všetci, 32-bit)",
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
                    id: format!("run:{hive}:{approved_key}:{name}").to_lowercase(),
                    kind: "run",
                    when: "Pri prihlásení".into(),
                    source: label.into(),
                    exe: exe_of(&command),
                    name: name.clone(),
                    command,
                    app,
                    verdict,
                    about,
                    advice: None,
                    enabled,
                    running: None,
                    hidden: false,
                    needs_admin: hive == "HKLM",
                    toggle: Some(Toggle::Approved {
                        hive: hive.into(),
                        key: approved_key.into(),
                        name,
                    }),
                });
            }
        }
        out
    }

    pub fn startup_folders() -> Vec<Item> {
        const APPROVED: &str =
            r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\StartupFolder";
        let mut out = Vec::new();
        let dirs = [
            (
                std::env::var("APPDATA").ok(),
                r"Microsoft\Windows\Start Menu\Programs\Startup",
                "Priečinok Po spustení (používateľ)",
                "HKCU",
            ),
            (
                std::env::var("ProgramData").ok(),
                r"Microsoft\Windows\Start Menu\Programs\StartUp",
                "Priečinok Po spustení (všetci)",
                "HKLM",
            ),
        ];
        for (base, rel, label, hive) in dirs {
            let Some(base) = base else { continue };
            let dir = std::path::Path::new(&base).join(rel);
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
            let approved: HashMap<String, String> =
                parse_reg_query(&reg_query(&format!(r"{hive}\{APPROVED}")))
                    .into_iter()
                    .collect();
            for e in entries.flatten() {
                let file = e.file_name().to_string_lossy().into_owned();
                if file.eq_ignore_ascii_case("desktop.ini") {
                    continue;
                }
                let name = file.trim_end_matches(".lnk").to_string();
                let command = e.path().to_string_lossy().into_owned();
                let (app, verdict, about) = classify(&name, &command);
                out.push(Item {
                    id: format!("folder:{hive}:{file}").to_lowercase(),
                    kind: "folder",
                    when: "Pri prihlásení".into(),
                    source: label.into(),
                    exe: Some(format!("{}.exe", name.to_lowercase())),
                    name,
                    command,
                    app,
                    verdict,
                    about,
                    advice: None,
                    enabled: approved.get(&file).map(|h| approved_enabled(h)).unwrap_or(true),
                    running: None,
                    hidden: false,
                    needs_admin: hive == "HKLM",
                    toggle: Some(Toggle::Approved {
                        hive: hive.into(),
                        key: APPROVED.into(),
                        name: file,
                    }),
                });
            }
        }
        out
    }

    pub fn run_once() -> Vec<Item> {
        let mut out = Vec::new();
        for hive in ["HKCU", "HKLM"] {
            let key = format!(r"{hive}\Software\Microsoft\Windows\CurrentVersion\RunOnce");
            for (name, command) in parse_reg_query(&reg_query(&key)) {
                let (app, verdict, about) = classify(&name, &command);
                out.push(Item {
                    id: format!("runonce:{hive}:{name}").to_lowercase(),
                    kind: "runonce",
                    when: "Raz, pri ďalšom prihlásení".into(),
                    source: format!("RunOnce ({hive})"),
                    exe: exe_of(&command),
                    name,
                    command,
                    app,
                    verdict,
                    about: about.or(Some("Spustí sa raz a sám sa zmaže. Väčšinou dokončenie inštalácie.")),
                    advice: None,
                    enabled: true,
                    running: None,
                    hidden: true,
                    needs_admin: hive == "HKLM",
                    toggle: None,
                });
            }
        }
        out
    }

    /// Only reported when someone changed the defaults.
    pub fn winlogon() -> Vec<Item> {
        let mut out = Vec::new();
        let vals = parse_reg_query(&reg_query(
            r"HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon",
        ));
        for (name, value) in vals {
            let v = value.trim().trim_end_matches(',').to_lowercase();
            let unusual = match name.as_str() {
                "Shell" => v != "explorer.exe",
                "Userinit" => !v.ends_with(r"\system32\userinit.exe") && v != "userinit.exe",
                _ => false,
            };
            if unusual {
                out.push(Item {
                    id: format!("winlogon:{name}").to_lowercase(),
                    kind: "winlogon",
                    when: "Pri prihlásení, pred plochou".into(),
                    source: format!("Winlogon {name}"),
                    exe: exe_of(&value),
                    name: format!("Winlogon {name}"),
                    command: value,
                    app: None,
                    verdict: Verdict::Unknown,
                    about: Some("Toto sa mení len zriedka a rado sa sem skrýva škodlivý softvér. Ak nevieš, čo to je, preskenuj PC Defenderom."),
                    advice: None,
                    enabled: true,
                    running: None,
                    hidden: true,
                    needs_admin: true,
                    toggle: None,
                });
            }
        }
        out
    }

    pub fn tasks() -> Vec<Item> {
        let xml = run("schtasks", &["/query", "/xml", "ONE"]);
        parse_tasks_xml(&xml).iter().filter_map(task_item).collect()
    }

    fn pwstr(p: PWSTR) -> String {
        if p.is_null() {
            String::new()
        } else {
            unsafe { p.to_string().unwrap_or_default() }
        }
    }

    pub fn services(changed: &HashSet<String>) -> Vec<Item> {
        let delayed = parse_delayed(&run(
            "reg",
            &[
                "query",
                r"HKLM\SYSTEM\CurrentControlSet\Services",
                "/s",
                "/v",
                "DelayedAutostart",
            ],
        ));
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
                    let key = pwstr(s.lpServiceName);
                    let display = pwstr(s.lpDisplayName);
                    let running = s.ServiceStatusProcess.dwCurrentState == SERVICE_RUNNING;
                    // Per-user services (name_1a2b3) can't be configured on their own.
                    if key.contains('_') && key.rsplit('_').next().is_some_and(|t| t.len() >= 4 && t.chars().all(|c| c.is_ascii_hexdigit())) {
                        continue;
                    }
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
                        let lk = key.to_lowercase();
                        if let Some(item) = service_item(
                            &key,
                            &display,
                            &binary,
                            cfg.dwStartType == SERVICE_AUTO_START,
                            delayed.contains(&lk),
                            running,
                            changed.contains(&lk),
                        ) {
                            out.push(item);
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
    use std::collections::HashSet;
    pub fn run_keys() -> Vec<Item> {
        Vec::new()
    }
    pub fn startup_folders() -> Vec<Item> {
        Vec::new()
    }
    pub fn run_once() -> Vec<Item> {
        Vec::new()
    }
    pub fn winlogon() -> Vec<Item> {
        Vec::new()
    }
    pub fn tasks() -> Vec<Item> {
        Vec::new()
    }
    pub fn services(_: &HashSet<String>) -> Vec<Item> {
        Vec::new()
    }
}

#[cfg(windows)]
pub(crate) use platform::decode;

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
        assert_eq!(exe_of(&v[0].1).as_deref(), Some("lghub_system_tray.exe"));
        assert_eq!(exe_of(&v[1].1).as_deref(), Some("update.exe"));
        assert_eq!(
            exe_of(r"C:\Program Files\MSI\MSI Center\MSI.CentralServer.exe -k").as_deref(),
            Some("msi.centralserver.exe")
        );
    }

    #[test]
    fn reads_task_manager_state() {
        assert!(approved_enabled("020000000000000000000000"));
        assert!(approved_enabled("060000000000000000000000"));
        assert!(!approved_enabled("03000000A1B2C3D4E5F60708"));
    }

    #[test]
    fn reads_delayed_services() {
        let out = "\r\nHKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services\\edgeupdate\r\n    DelayedAutostart    REG_DWORD    0x1\r\n\r\nHKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services\\Foo\r\n    DelayedAutostart    REG_DWORD    0x0\r\n";
        let d = parse_delayed(out);
        assert!(d.contains("edgeupdate"));
        assert!(!d.contains("foo"));
    }

    const TASKS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<Tasks>
  <!-- \GoogleUpdaterTaskUser{ABC} -->
  <Task version="1.3" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
    <RegistrationInfo>
      <Author>Google</Author>
    </RegistrationInfo>
    <Triggers>
      <LogonTrigger>
        <Enabled>true</Enabled>
      </LogonTrigger>
      <TimeTrigger>
        <StartBoundary>2026-01-01T10:00:00</StartBoundary>
      </TimeTrigger>
    </Triggers>
    <Settings>
      <IdleSettings>
        <StopOnIdleEnd>true</StopOnIdleEnd>
      </IdleSettings>
      <Enabled>true</Enabled>
    </Settings>
    <Actions Context="Author">
      <Exec>
        <Command>C:\Users\Alex\AppData\Local\Google\GoogleUpdater\updater.exe</Command>
        <Arguments>--wake --system</Arguments>
      </Exec>
    </Actions>
  </Task>
  <!-- \MSI\MSI Task Host - LEDKeeper -->
  <Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
    <Triggers>
      <LogonTrigger />
    </Triggers>
    <Settings>
      <Enabled>false</Enabled>
    </Settings>
    <Actions Context="Author">
      <Exec>
        <Command>"C:\Program Files (x86)\MSI\One Dragon Center\Mystic_Light\LEDKeeper2.exe"</Command>
      </Exec>
    </Actions>
  </Task>
  <!-- \Microsoft\Windows\Defrag\ScheduledDefrag -->
  <Task version="1.6" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
    <Triggers>
      <CalendarTrigger><Enabled>true</Enabled></CalendarTrigger>
    </Triggers>
    <Settings><Enabled>true</Enabled></Settings>
    <Actions><Exec><Command>%windir%\system32\defrag.exe</Command></Exec></Actions>
  </Task>
  <!-- \Vendor\Disabled logon trigger -->
  <Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
    <Triggers>
      <LogonTrigger><Enabled>false</Enabled></LogonTrigger>
    </Triggers>
    <Actions><Exec><Command>C:\x.exe</Command></Exec></Actions>
  </Task>
</Tasks>"#;

    #[test]
    fn reads_scheduled_tasks() {
        let t = parse_tasks_xml(TASKS);
        assert_eq!(t.len(), 4);
        assert_eq!(t[0].path, r"\GoogleUpdaterTaskUser{ABC}");
        assert!(t[0].logon && t[0].timer && t[0].enabled);
        assert!(t[0].command.ends_with("updater.exe --wake --system"));
        assert!(t[1].logon && !t[1].enabled);
        assert!(!t[2].logon && t[2].timer);
        assert!(!t[3].logon);

        let items: Vec<Item> = t.iter().filter_map(task_item).collect();
        // Defrag (Microsoft, timer only) and the disabled trigger are not startup things.
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].app, Some("Google aktualizácie"));
        assert_eq!(items[0].exe.as_deref(), Some("updater.exe"));
        assert_eq!(items[1].verdict, Verdict::Rgb);
        assert!(!items[1].enabled && items[1].hidden);
        assert_eq!(
            items[1].toggle,
            Some(Toggle::Task {
                path: r"\MSI\MSI Task Host - LEDKeeper".into()
            })
        );
    }

    #[test]
    fn services_get_honest_advice() {
        let s = service_item("DiagTrack", "Telemetria", r"C:\WINDOWS\System32\svchost.exe -k utcsvc -p", true, false, true, false).unwrap();
        assert_eq!(s.verdict, Verdict::Optional);
        assert_eq!(s.advice, Some(Advice::Off));
        let s = service_item("AudioSrv", "Zvuk", r"C:\WINDOWS\System32\svchost.exe -k LocalServiceNetworkRestricted -p", true, false, true, false).unwrap();
        assert_eq!(s.verdict, Verdict::System);
        let s = service_item("MSI_Central_Service", "MSI Central Service", r#""C:\Program Files (x86)\MSI\MSI Center\MSI.CentralServer.exe""#, true, true, true, false).unwrap();
        assert_eq!(s.verdict, Verdict::Rgb);
        assert_eq!(s.toggle, Some(Toggle::Service { name: "MSI_Central_Service".into(), on: "delayed-auto".into() }));
        // Manual services only show up after the user changed them here.
        assert!(service_item("Foo", "Foo", r"C:\foo.exe", false, false, false, false).is_none());
        let s = service_item("Foo", "Foo", r"C:\foo.exe", false, false, false, true).unwrap();
        assert!(!s.enabled);
    }
}
