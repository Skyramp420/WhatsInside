//! The few things WhatsInside changes. Every change is reversible and logged
//! in `%LOCALAPPDATA%\WhatsInside\changes.json`, so it can be undone later.
//!
//! - Run keys / Startup folder: the same StartupApproved switch Task Manager
//!   flips, nothing gets deleted.
//! - Scheduled tasks: disabled, not deleted.
//! - Services: switched to manual start, not disabled, so anything that
//!   really needs one can still start it.
//! - Tweaks: registry values or the power plan, each with a Windows-default
//!   "off" state (see `tweaks`).
//! - Processes: ended, never for Windows itself. Logged, but can't be undone.
//!
//! Things that need admin rights are collected and run in one elevated copy
//! of ourselves (`--apply in.json out.json`), so the user sees one UAC prompt
//! instead of one per item.

use crate::autostart::Toggle;
use crate::tweaks::{self, Cmd};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Action {
    Startup {
        id: String,
        name: String,
        toggle: Toggle,
        enable: bool,
    },
    Kill {
        pid: u32,
        exe: String,
    },
    /// Start an app again after a game (only an existing .exe, no arguments).
    Launch {
        path: String,
    },
    /// Remove a preinstalled Store app from the known list (`appx`).
    Uninstall {
        full: String,
        name: String,
    },
    /// Restart File Explorer so taskbar and menu changes show up.
    RestartExplorer,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Outcome {
    pub ok: bool,
    pub error: Option<String>,
    /// Done through the UAC prompt.
    pub elevated: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Change {
    /// Unix seconds.
    pub at: u64,
    pub id: String,
    pub name: String,
    /// `None` for things that can't be undone (an ended process).
    #[serde(default)]
    pub toggle: Option<Toggle>,
    /// State after the change.
    pub enabled: bool,
    #[serde(default)]
    pub undone: bool,
}

/// The exact commands that perform a change. Kept separate so they can be
/// tested without Windows.
pub fn commands_for(toggle: &Toggle, enable: bool) -> Vec<Cmd> {
    let one = |exe: &'static str, args: Vec<String>| vec![Cmd { exe, args, ignore_err: false }];
    match toggle {
        Toggle::Approved { hive, key, name } => one(
            "reg",
            vec![
                "add".into(),
                format!(r"{hive}\{key}"),
                "/v".into(),
                name.clone(),
                "/t".into(),
                "REG_BINARY".into(),
                "/d".into(),
                format!("{}0000000000000000000000", if enable { "02" } else { "03" }),
                "/f".into(),
            ],
        ),
        Toggle::Task { path } => one(
            "schtasks",
            vec![
                "/Change".into(),
                "/TN".into(),
                path.clone(),
                if enable { "/ENABLE" } else { "/DISABLE" }.into(),
            ],
        ),
        Toggle::Service { name, on } => one(
            "sc",
            vec![
                "config".into(),
                name.clone(),
                "start=".into(),
                if enable { on.clone() } else { "demand".into() },
            ],
        ),
        Toggle::Tweak { key } => tweaks::find(key)
            .map(|t| tweaks::commands(t, enable))
            .unwrap_or_default(),
    }
}

/// Services always need admin; others are tried as the user first.
fn always_admin(a: &Action) -> bool {
    match a {
        Action::Startup { toggle: Toggle::Service { .. }, .. } => true,
        Action::Startup { toggle: Toggle::Tweak { key }, .. } => tweaks::find(key).is_some_and(|t| t.admin),
        _ => false,
    }
}

fn data_dir() -> Option<PathBuf> {
    std::env::var("LOCALAPPDATA")
        .ok()
        .map(|d| PathBuf::from(d).join("WhatsInside"))
}

fn log_path() -> Option<PathBuf> {
    data_dir().map(|d| d.join("changes.json"))
}

pub fn changes() -> Vec<Change> {
    log_path()
        .and_then(|p| std::fs::read(p).ok())
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}

fn save_changes(c: &[Change]) {
    let Some(p) = log_path() else { return };
    if let Some(d) = p.parent() {
        let _ = std::fs::create_dir_all(d);
    }
    if let Ok(json) = serde_json::to_vec_pretty(c) {
        let _ = std::fs::write(p, json);
    }
}

/// Services the user switched here, so they stay listed after going manual.
pub fn changed_services() -> HashSet<String> {
    changes()
        .into_iter()
        .filter_map(|c| match c.toggle {
            Some(Toggle::Service { name, .. }) => Some(name.to_lowercase()),
            _ => None,
        })
        .collect()
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Applies actions: as the user where possible, the rest in one UAC prompt.
pub fn apply(actions: Vec<Action>) -> Vec<Outcome> {
    let elevated_now = platform::is_admin();
    let mut out = vec![Outcome::default(); actions.len()];
    let mut later = Vec::new();
    for (i, a) in actions.iter().enumerate() {
        if !elevated_now && always_admin(a) {
            later.push(i);
            continue;
        }
        match platform::apply_one(a) {
            Ok(()) => out[i].ok = true,
            Err(e) => {
                // Only startup changes are worth a UAC retry.
                if !elevated_now && matches!(a, Action::Startup { .. }) {
                    later.push(i);
                } else {
                    out[i].error = Some(e);
                }
            }
        }
    }
    if !later.is_empty() {
        let batch: Vec<Action> = later.iter().map(|&i| actions[i].clone()).collect();
        match platform::run_elevated(&batch) {
            Ok(res) => {
                for (k, &i) in later.iter().enumerate() {
                    let r = res.get(k).cloned().unwrap_or(Outcome {
                        ok: false,
                        error: Some("Bez odpovede.".into()),
                        elevated: true,
                    });
                    out[i] = Outcome { elevated: true, ..r };
                }
            }
            Err(e) => {
                for &i in &later {
                    out[i].error = Some(e.clone());
                }
            }
        }
    }
    // Log what worked.
    let mut log = changes();
    for (a, o) in actions.iter().zip(&out) {
        if !o.ok {
            continue;
        }
        match a {
            Action::Startup { id, name, toggle, enable } => log.push(Change {
                at: now(),
                id: id.clone(),
                name: name.clone(),
                toggle: Some(toggle.clone()),
                enabled: *enable,
                undone: false,
            }),
            Action::Kill { pid, exe } => log.push(Change {
                at: now(),
                id: format!("kill:{pid}"),
                name: exe.clone(),
                toggle: None,
                enabled: false,
                undone: false,
            }),
            Action::Uninstall { full, name } => log.push(Change {
                at: now(),
                id: format!("appx:{}", full.split('_').next().unwrap_or(full)),
                name: name.clone(),
                toggle: None,
                enabled: false,
                undone: false,
            }),
            Action::Launch { .. } | Action::RestartExplorer => {}
        }
    }
    if log.len() > 300 {
        let cut = log.len() - 300;
        log.drain(..cut);
    }
    save_changes(&log);
    out
}

/// Reverts one logged change.
pub fn undo(index: usize) -> Result<(), String> {
    let log = changes();
    let c = log.get(index).cloned().ok_or("Táto zmena už neexistuje.")?;
    if c.undone {
        return Err("Táto zmena už bola vrátená.".into());
    }
    let toggle = c.toggle.clone().ok_or("Ukončený proces sa vrátiť nedá.")?;
    let res = apply(vec![Action::Startup {
        id: c.id.clone(),
        name: c.name.clone(),
        toggle,
        enable: !c.enabled,
    }]);
    let r = res.into_iter().next().unwrap_or_default();
    if !r.ok {
        return Err(r.error.unwrap_or_else(|| "Nepodarilo sa.".into()));
    }
    // apply() logged the reverse change; the original marked "undone" says it all.
    let mut log = changes();
    if log.last().is_some_and(|l| l.id == c.id && l.enabled != c.enabled && !l.undone) {
        log.pop();
    }
    if let Some(x) = log.get_mut(index) {
        x.undone = true;
    }
    save_changes(&log);
    Ok(())
}

/// Entry point of the elevated copy: `whatsinside.exe --apply in.json out.json`.
pub fn run_batch(input: &str, output: &str) {
    let actions: Vec<Action> = std::fs::read(input)
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();
    let res: Vec<Outcome> = actions
        .iter()
        .map(|a| match platform::apply_one(a) {
            Ok(()) => Outcome { ok: true, error: None, elevated: true },
            Err(e) => Outcome { ok: false, error: Some(e), elevated: true },
        })
        .collect();
    let _ = std::fs::write(output, serde_json::to_vec(&res).unwrap_or_default());
}

#[cfg(windows)]
mod platform {
    use super::{commands_for, Action, Outcome};
    use crate::catalog;
    use std::os::windows::process::CommandExt;
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, TerminateProcess, WaitForSingleObject,
        INFINITE, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_TERMINATE,
    };
    use windows::Win32::UI::Shell::{
        IsUserAnAdmin, ShellExecuteExW, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW,
    };

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    pub fn is_admin() -> bool {
        unsafe { IsUserAnAdmin().as_bool() }
    }

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    pub fn apply_one(a: &Action) -> Result<(), String> {
        match a {
            Action::Startup { toggle, enable, .. } => {
                let cmds = commands_for(toggle, *enable);
                if cmds.is_empty() {
                    return Err("Neznáma zmena.".into());
                }
                for c in cmds {
                    let o = std::process::Command::new(c.exe)
                        .args(&c.args)
                        .creation_flags(CREATE_NO_WINDOW)
                        .output()
                        .map_err(|e| e.to_string())?;
                    if !o.status.success() && !c.ignore_err {
                        let msg = crate::autostart::decode(&o.stderr);
                        let msg = if msg.trim().is_empty() {
                            crate::autostart::decode(&o.stdout)
                        } else {
                            msg
                        };
                        return Err(msg.trim().lines().last().unwrap_or("Nepodarilo sa.").to_string());
                    }
                }
                Ok(())
            }
            Action::Kill { pid, exe } => kill(*pid, exe),
            Action::Launch { path } => launch(path),
            Action::Uninstall { full, .. } => crate::appx::uninstall(full),
            Action::RestartExplorer => {
                let _ = std::process::Command::new("taskkill")
                    .args(["/F", "/IM", "explorer.exe"])
                    .creation_flags(CREATE_NO_WINDOW)
                    .output();
                std::thread::sleep(std::time::Duration::from_millis(600));
                let windir = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into());
                std::process::Command::new(std::path::Path::new(&windir).join("explorer.exe"))
                    .spawn()
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            }
        }
    }

    fn launch(path: &str) -> Result<(), String> {
        let p = std::path::Path::new(path);
        if !p.is_file() || !path.to_lowercase().ends_with(".exe") || catalog::is_windows_path(path) {
            return Err("Tento program sa nedá spustiť.".into());
        }
        std::process::Command::new(p)
            .current_dir(p.parent().unwrap_or(p))
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    fn kill(pid: u32, exe: &str) -> Result<(), String> {
        if pid <= 4 || pid == std::process::id() || catalog::is_system_name(exe) {
            return Err("Toto je súčasť Windowsu, tú neukončím.".into());
        }
        unsafe {
            // Make sure the PID still belongs to the same exe.
            let q = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)
                .map_err(|_| "Proces už nebeží alebo k nemu nemám prístup.".to_string())?;
            let mut buf = [0u16; 1024];
            let mut len = buf.len() as u32;
            let same = QueryFullProcessImageNameW(
                q,
                PROCESS_NAME_WIN32,
                windows::core::PWSTR(buf.as_mut_ptr()),
                &mut len,
            )
            .is_ok()
                && String::from_utf16_lossy(&buf[..len as usize])
                    .to_lowercase()
                    .ends_with(&exe.to_lowercase());
            let _ = CloseHandle(q);
            if !same {
                return Err("Proces sa medzitým zmenil, obnov zoznam.".into());
            }
            if catalog::is_windows_path(&String::from_utf16_lossy(&buf[..len as usize])) {
                return Err("Toto je súčasť Windowsu, tú neukončím.".into());
            }
            let h = OpenProcess(PROCESS_TERMINATE, false, pid)
                .map_err(|_| "Nemám oprávnenie ho ukončiť (beží ako správca).".to_string())?;
            let r = TerminateProcess(h, 1);
            let _ = CloseHandle(h);
            r.map_err(|e| e.message().to_string())
        }
    }

    pub fn run_elevated(batch: &[Action]) -> Result<Vec<Outcome>, String> {
        let dir = std::env::temp_dir();
        let tag = format!("{}-{}", std::process::id(), super::now());
        let input = dir.join(format!("whatsinside-apply-{tag}.json"));
        let output = dir.join(format!("whatsinside-result-{tag}.json"));
        std::fs::write(&input, serde_json::to_vec(batch).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let params = format!(
            "--apply \"{}\" \"{}\"",
            input.to_string_lossy(),
            output.to_string_lossy()
        );
        let (verb, file, params) = (
            wide("runas"),
            wide(&exe.to_string_lossy()),
            wide(&params),
        );
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
                let _ = std::fs::remove_file(&input);
                return Err("Potrebujem povolenie správcu. Okno UAC bolo zrušené.".into());
            }
            if !info.hProcess.is_invalid() {
                WaitForSingleObject(info.hProcess, INFINITE);
                let _ = CloseHandle(info.hProcess);
            }
        }
        let res = std::fs::read(&output)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .ok_or_else(|| "Zmena s právami správcu nevrátila výsledok.".to_string());
        let _ = std::fs::remove_file(&input);
        let _ = std::fs::remove_file(&output);
        res
    }
}

#[cfg(not(windows))]
mod platform {
    use super::{Action, Outcome};
    pub fn is_admin() -> bool {
        false
    }
    pub fn apply_one(_: &Action) -> Result<(), String> {
        Err("Len na Windows.".into())
    }
    pub fn run_elevated(_: &[Action]) -> Result<Vec<Outcome>, String> {
        Err("Len na Windows.".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_the_same_commands_task_manager_would() {
        let c = commands_for(
            &Toggle::Approved {
                hive: "HKCU".into(),
                key: r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run".into(),
                name: "Discord".into(),
            },
            false,
        );
        assert_eq!(c[0].exe, "reg");
        assert_eq!(c[0].args[3], "Discord");
        assert_eq!(c[0].args[7], "030000000000000000000000");

        let c = commands_for(&Toggle::Service { name: "edgeupdate".into(), on: "delayed-auto".into() }, false);
        assert_eq!((c[0].exe, c[0].args.join(" ").as_str()), ("sc", "config edgeupdate start= demand"));
        let c = commands_for(&Toggle::Service { name: "edgeupdate".into(), on: "delayed-auto".into() }, true);
        assert_eq!(c[0].args[3], "delayed-auto");

        let c = commands_for(&Toggle::Task { path: r"\MSI\LEDKeeper".into() }, true);
        assert_eq!((c[0].exe, c[0].args.join(" ").as_str()), ("schtasks", r"/Change /TN \MSI\LEDKeeper /ENABLE"));

        let c = commands_for(&Toggle::Tweak { key: "game_dvr".into() }, true);
        assert_eq!(c.len(), 2);
        assert!(commands_for(&Toggle::Tweak { key: "nope".into() }, true).is_empty());
        assert!(always_admin(&Action::Startup { id: String::new(), name: String::new(), toggle: Toggle::Tweak { key: "hags".into() }, enable: true }));
    }

    #[test]
    fn actions_round_trip_through_json() {
        let json = r#"[{"op":"startup","id":"task:\\msi\\x","name":"X","toggle":{"t":"task","path":"\\MSI\\X"},"enable":false},{"op":"kill","pid":42,"exe":"steam.exe"},{"op":"startup","id":"tweak:tips","name":"Tipy","toggle":{"t":"tweak","key":"tips"},"enable":true}]"#;
        let a: Vec<Action> = serde_json::from_str(json).unwrap();
        assert_eq!(a.len(), 3);
        // Old log entries (before kills were logged) still load.
        let old: Change = serde_json::from_str(r#"{"at":1,"id":"x","name":"X","toggle":{"t":"task","path":"\\X"},"enabled":false}"#).unwrap();
        assert!(old.toggle.is_some() && !old.undone);
        assert!(always_admin(&Action::Startup { id: String::new(), name: String::new(), toggle: Toggle::Service { name: "x".into(), on: "auto".into() }, enable: true }));
        assert!(!always_admin(&a[0]));
    }
}
