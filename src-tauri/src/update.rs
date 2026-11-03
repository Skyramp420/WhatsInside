//! Updates from GitHub Releases.
//!
//! Every release carries `latest.json` (version, installer URL, SHA-256),
//! written by CI. We download with the `curl.exe` that ships with Windows 10
//! and 11, check the hash, and run the installer. Nothing runs in the
//! background: we only check when the app is open.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const REPO: &str = "Skyramp420/WhatsInside";

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Latest {
    pub version: String,
    pub url: String,
    pub sha256: String,
    #[serde(default)]
    pub notes: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct Check {
    pub current: String,
    pub latest: Option<Latest>,
    pub newer: bool,
    /// Running from the installed copy (otherwise a portable exe).
    pub installed: bool,
    pub error: Option<String>,
}

pub fn current() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn parts(v: &str) -> Vec<u64> {
    v.trim()
        .trim_start_matches('v')
        .split(['.', '-', '+'])
        .take(3)
        .map(|p| p.parse().unwrap_or(0))
        .collect()
}

/// True when `latest` is a higher version than `current`.
pub fn is_newer(latest: &str, current: &str) -> bool {
    let (a, b) = (parts(latest), parts(current));
    for i in 0..3 {
        let (x, y) = (a.get(i).copied().unwrap_or(0), b.get(i).copied().unwrap_or(0));
        if x != y {
            return x > y;
        }
    }
    false
}

/// Only installers from our own releases are accepted.
pub fn trusted_url(url: &str) -> bool {
    let prefix = format!("https://github.com/{REPO}/releases/download/");
    url.starts_with(&prefix)
        && url.to_lowercase().ends_with(".exe")
        && !url[prefix.len()..].contains("..")
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

pub fn check() -> Check {
    let installed = platform::installed_dir().is_some();
    match platform::fetch_latest() {
        Ok(l) => Check {
            current: current().into(),
            newer: is_newer(&l.version, current()),
            latest: Some(l),
            installed,
            error: None,
        },
        Err(e) => Check {
            current: current().into(),
            latest: None,
            newer: false,
            installed,
            error: Some(e),
        },
    }
}

/// Downloads and starts the installer. On success the caller quits the app.
pub fn install(l: &Latest) -> Result<(), String> {
    if !trusted_url(&l.url) {
        return Err("Neznámy zdroj aktualizácie.".into());
    }
    platform::install(l)
}

#[cfg(windows)]
mod platform {
    use super::{sha256_hex, Latest, REPO};
    use std::os::windows::process::CommandExt;
    use std::path::PathBuf;
    use std::process::Command;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    const DETACHED_PROCESS: u32 = 0x0000_0008;

    fn curl() -> PathBuf {
        let root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into());
        PathBuf::from(root).join("System32").join("curl.exe")
    }

    pub fn fetch_latest() -> Result<Latest, String> {
        let url = format!("https://github.com/{REPO}/releases/latest/download/latest.json");
        let o = Command::new(curl())
            .args(["-fsSL", "--max-time", "15", &url])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|_| "Nenašiel som curl.exe (potrebný Windows 10 alebo 11).".to_string())?;
        if !o.status.success() {
            return Err("Nepodarilo sa spojiť s GitHubom.".into());
        }
        serde_json::from_slice(&o.stdout).map_err(|_| "Neplatná odpoveď z GitHubu.".into())
    }

    /// The install folder when running from the installed copy.
    pub fn installed_dir() -> Option<PathBuf> {
        let exe = std::env::current_exe().ok()?;
        let dir = exe.parent()?.to_path_buf();
        dir.join("uninstall.exe").exists().then_some(dir)
    }

    pub fn install(l: &Latest) -> Result<(), String> {
        let dir = std::env::temp_dir().join("WhatsInside-update");
        let _ = std::fs::create_dir_all(&dir);
        let setup = dir.join(format!("WhatsInside-{}-setup.exe", l.version.replace(['\\', '/', '"'], "")));
        let o = Command::new(curl())
            .args(["-fsSL", "--max-time", "300", "-o"])
            .arg(&setup)
            .arg(&l.url)
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| e.to_string())?;
        if !o.status.success() {
            return Err("Sťahovanie zlyhalo.".into());
        }
        let bytes = std::fs::read(&setup).map_err(|e| e.to_string())?;
        if sha256_hex(&bytes) != l.sha256.to_lowercase() {
            let _ = std::fs::remove_file(&setup);
            return Err("Stiahnutý súbor nesedí s kontrolným súčtom, nespustím ho.".into());
        }
        match installed_dir() {
            Some(_) => {
                // Wait for us to quit, install silently, start the new version.
                let exe = std::env::current_exe().map_err(|e| e.to_string())?;
                let script = dir.join("update.cmd");
                let body = format!(
                    "@echo off\r\nping -n 3 127.0.0.1 >nul\r\n\"{}\" /S\r\nstart \"\" \"{}\"\r\n",
                    setup.display(),
                    exe.display()
                );
                std::fs::write(&script, body).map_err(|e| e.to_string())?;
                Command::new("cmd")
                    .arg("/c")
                    .arg(&script)
                    .creation_flags(CREATE_NO_WINDOW | DETACHED_PROCESS)
                    .spawn()
                    .map_err(|e| e.to_string())?;
            }
            None => {
                // Portable copy: show the normal installer.
                Command::new(&setup).spawn().map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }
}

#[cfg(not(windows))]
mod platform {
    use super::Latest;
    pub fn fetch_latest() -> Result<Latest, String> {
        Err("Len na Windows.".into())
    }
    pub fn installed_dir() -> Option<std::path::PathBuf> {
        None
    }
    pub fn install(_: &Latest) -> Result<(), String> {
        Err("Len na Windows.".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_versions() {
        assert!(is_newer("0.4.1", "0.4.0"));
        assert!(is_newer("v1.0.0", "0.9.9"));
        assert!(is_newer("0.10.0", "0.9.0"));
        assert!(!is_newer("0.4.0", "0.4.0"));
        assert!(!is_newer("0.3.9", "0.4.0"));
    }

    #[test]
    fn only_trusts_our_releases() {
        assert!(trusted_url("https://github.com/Skyramp420/WhatsInside/releases/download/v0.4.0/WhatsInside_0.4.0_x64-setup.exe"));
        assert!(!trusted_url("https://github.com/someone/else/releases/download/v1/x.exe"));
        assert!(!trusted_url("https://github.com/Skyramp420/WhatsInside/releases/download/../../x.exe"));
        assert!(!trusted_url("http://github.com/Skyramp420/WhatsInside/releases/download/v1/x.exe"));
    }

    #[test]
    fn hashes() {
        assert_eq!(sha256_hex(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
        let l: Latest = serde_json::from_str(r#"{"version":"0.4.0","url":"u","sha256":"x"}"#).unwrap();
        assert_eq!(l.notes, "");
    }
}
