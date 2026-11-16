//! First setup of a fresh Windows: the apps people install first, through
//! winget (the package manager built into Windows 10 and 11), and the
//! Microsoft apps people remove (Edge where Windows allows it, OneDrive).
//!
//! Only packages from this list can be installed or removed. Installers
//! come from winget's own source, the same files the vendors publish.

use serde::Serialize;

pub struct Pkg {
    /// winget package id.
    pub id: &'static str,
    pub name: &'static str,
    pub about: &'static str,
    /// "browser", "chat", "games", "media", "tools"
    pub cat: &'static str,
    /// Installs into the user's profile and must not run as admin (it would
    /// then start as admin after installing, or refuses to install at all).
    pub user: bool,
}

const fn p(id: &'static str, name: &'static str, cat: &'static str, user: bool, about: &'static str) -> Pkg {
    Pkg { id, name, about, cat, user }
}

pub const PKGS: &[Pkg] = &[
    p("Google.Chrome", "Google Chrome", "browser", false, "Najpoužívanejší prehliadač, synchronizácia s Google účtom."),
    p("Mozilla.Firefox", "Firefox", "browser", false, "Nezávislý prehliadač, dobré súkromie a doplnky."),
    p("Brave.Brave", "Brave", "browser", false, "Chrome bez sledovania, blokuje reklamy sám."),
    p("Opera.OperaGX", "Opera GX", "browser", false, "Prehliadač pre hráčov s limitmi na RAM a procesor."),
    p("Discord.Discord", "Discord", "chat", true, "Hlasový chat a servery."),
    p("Telegram.TelegramDesktop", "Telegram", "chat", true, "Správy, kanály a skupiny."),
    p("Valve.Steam", "Steam", "games", false, "Najväčší obchod s hrami na PC."),
    p("EpicGames.EpicGamesLauncher", "Epic Games", "games", false, "Fortnite a hry zadarmo každý týždeň."),
    p("ElectronicArts.EADesktop", "EA app", "games", false, "Hry od EA (FC, Battlefield, The Sims)."),
    p("Ubisoft.Connect", "Ubisoft Connect", "games", false, "Hry od Ubisoftu."),
    p("GOG.Galaxy", "GOG Galaxy", "games", false, "Hry bez DRM a knižnica všetkých obchodov na jednom mieste."),
    p("Spotify.Spotify", "Spotify", "media", true, "Hudba a podcasty."),
    p("VideoLAN.VLC", "VLC", "media", false, "Prehrá každé video a zvuk."),
    p("OBSProject.OBSStudio", "OBS Studio", "media", false, "Nahrávanie obrazovky a streamovanie."),
    p("7zip.7zip", "7-Zip", "tools", false, "Rozbalí zip, rar, 7z a ďalšie archívy."),
    p("Notepad++.Notepad++", "Notepad++", "tools", false, "Poznámkový blok pre pokročilých."),
    p("Microsoft.PowerToys", "PowerToys", "tools", false, "Užitočné doplnky Windowsu od Microsoftu (rozloženie okien, výber farby, hromadné premenovanie…)."),
    p("voidtools.Everything", "Everything", "tools", false, "Nájde súbor na disku okamžite, kým píšeš."),
    p("ShareX.ShareX", "ShareX", "tools", false, "Snímky obrazovky, nahrávanie a úpravy."),
    p("Bitwarden.Bitwarden", "Bitwarden", "tools", false, "Správca hesiel zadarmo."),
    p("Microsoft.VisualStudioCode", "Visual Studio Code", "tools", false, "Editor kódu."),
];

/// Things to remove, not install.
pub const REMOVABLE: &[Pkg] = &[
    p("Microsoft.Edge", "Microsoft Edge", "browser", false, "Odinštalovať sa dá len v Európskom hospodárskom priestore. WebView2, na ktorom bežia iné appky (aj WhatsInside), ostane."),
    p("Microsoft.OneDrive", "OneDrive", "tools", true, "Synchronizácia s cloudom Microsoftu. Súbory, ktoré sú už v OneDrive, ostanú v cloude aj v priečinku na disku."),
];

pub fn find(id: &str) -> Option<&'static Pkg> {
    PKGS.iter().chain(REMOVABLE).find(|p| p.id == id)
}

/// Countries where Microsoft lets people remove Edge (EEA, Digital Markets Act).
const EEA: &[&str] = &[
    "AT", "BE", "BG", "HR", "CY", "CZ", "DK", "EE", "FI", "FR", "DE", "GR", "HU", "IE", "IT", "LV", "LT", "LU", "MT", "NL", "PL",
    "PT", "RO", "SK", "SI", "ES", "SE", "IS", "LI", "NO",
];

pub fn in_eea(region: &str) -> bool {
    EEA.iter().any(|c| c.eq_ignore_ascii_case(region.trim()))
}

#[derive(Serialize, Clone, Debug)]
pub struct Item {
    pub id: &'static str,
    pub name: &'static str,
    pub about: &'static str,
    pub cat: &'static str,
    pub user: bool,
    pub installed: bool,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct Status {
    /// winget is there and answers.
    pub winget: bool,
    pub apps: Vec<Item>,
    pub remove: Vec<Item>,
    /// Two-letter region from Windows settings ("SK").
    pub region: String,
    pub edge_removable: bool,
}

/// Package ids from `winget export` JSON.
pub fn parse_export(json: &str) -> Vec<String> {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(json.trim_start_matches('\u{feff}')) else { return Vec::new() };
    v.get("Sources")
        .and_then(|s| s.as_array())
        .into_iter()
        .flatten()
        .filter_map(|s| s.get("Packages")?.as_array())
        .flatten()
        .filter_map(|p| p.get("PackageIdentifier")?.as_str().map(String::from))
        .collect()
}

pub fn status_from(winget: bool, installed: &[String], region: String, edge: bool, onedrive: bool) -> Status {
    let has = |id: &str| installed.iter().any(|i| i.eq_ignore_ascii_case(id));
    let item = |p: &'static Pkg, installed: bool| Item { id: p.id, name: p.name, about: p.about, cat: p.cat, user: p.user, installed };
    Status {
        winget,
        apps: PKGS.iter().map(|p| item(p, has(p.id))).collect(),
        remove: REMOVABLE
            .iter()
            .map(|p| item(p, match p.id { "Microsoft.Edge" => edge || has(p.id), "Microsoft.OneDrive" => onedrive || has(p.id), _ => has(p.id) }))
            .collect(),
        edge_removable: in_eea(&region),
        region,
    }
}

/// Exit codes winget uses for "nothing to do" (already installed, no newer
/// version) and "done, restart later". Both count as success.
pub fn ok_code(code: i32) -> bool {
    matches!(code as u32, 0 | 0x8A15_002B | 0x8A15_0061 | 0x8A15_0109 | 0x8A15_010A | 0x8A15_0101)
}

/// A readable reason for a failed winget run.
pub fn explain(id: &str, install: bool, code: i32, out: &str) -> String {
    let c = code as u32;
    if !install && id == "Microsoft.Edge" {
        return "Windows Edge odinštalovať nedovolil. Ide to len v krajinách EÚ (a Nórsku, Islande, Lichtenštajnsku) s aktuálnym Windowsom.".into();
    }
    match c {
        0x8A15_0014 | 0x8A15_000F => return "Winget sa nevedel spojiť so zdrojom balíkov. Skontroluj internet.".into(),
        0x8A15_0017 => return "Balík sa nenašiel.".into(),
        0x8A15_002C | 0x8A15_0029 => return "Appka nie je nainštalovaná.".into(),
        0x8A15_0108 | 0x8A15_0104 => return "Inštalátor nemohol pokračovať, appka je otvorená. Zavri ju a skús znova.".into(),
        0x8A15_0068 => return "Inštaláciu zrušilo okno UAC.".into(),
        _ => {}
    }
    let line = out
        .lines()
        .flat_map(|l| l.split('\r'))
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.chars().all(|ch| "-\\|/█▒ ".contains(ch)) && !l.contains("KB /") && !l.contains("MB /"))
        .next_back()
        .unwrap_or("");
    if line.is_empty() {
        format!("Nepodarilo sa (kód 0x{c:08X}).")
    } else {
        line.chars().take(200).collect()
    }
}

pub fn valid_id(id: &str) -> bool {
    find(id).is_some()
}

#[cfg(windows)]
mod platform {
    use std::os::windows::process::CommandExt;
    const NO_WINDOW: u32 = 0x0800_0000;

    pub fn winget(args: &[&str]) -> std::io::Result<std::process::Output> {
        std::process::Command::new("winget").args(args).creation_flags(NO_WINDOW).output()
    }

    pub fn run(id: &str, install: bool) -> Result<(), String> {
        let mut args = vec![if install { "install" } else { "uninstall" }, "--id", id, "-e", "--silent", "--accept-source-agreements", "--disable-interactivity"];
        if install {
            args.extend(["--source", "winget", "--accept-package-agreements"]);
        }
        let o = winget(&args).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "Chýba winget (Inštalátor aplikácií z Microsoft Store).".to_string()
            } else {
                e.to_string()
            }
        })?;
        let code = o.status.code().unwrap_or(-1);
        if super::ok_code(code) {
            Ok(())
        } else {
            Err(super::explain(id, install, code, &crate::autostart::decode(&o.stdout)))
        }
    }

    pub fn status() -> super::Status {
        let tmp = std::env::temp_dir().join(format!("whatsinside-winget-{}.json", std::process::id()));
        let path = tmp.to_string_lossy().to_string();
        let ran = winget(&["export", "-o", &path, "--source", "winget", "--accept-source-agreements", "--disable-interactivity"]);
        let winget = ran.is_ok();
        let installed = std::fs::read_to_string(&tmp).map(|s| super::parse_export(&s)).unwrap_or_default();
        let _ = std::fs::remove_file(&tmp);
        let region = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER)
            .open_subkey(r"Control Panel\International\Geo")
            .and_then(|k| k.get_value::<String, _>("Name"))
            .unwrap_or_default();
        let pf86 = std::env::var("ProgramFiles(x86)").unwrap_or_else(|_| r"C:\Program Files (x86)".into());
        let pf = std::env::var("ProgramFiles").unwrap_or_else(|_| r"C:\Program Files".into());
        let la = std::env::var("LOCALAPPDATA").unwrap_or_default();
        let exists = |p: String| std::path::Path::new(&p).exists();
        let edge = exists(format!(r"{pf86}\Microsoft\Edge\Application\msedge.exe")) || exists(format!(r"{pf}\Microsoft\Edge\Application\msedge.exe"));
        let onedrive = exists(format!(r"{la}\Microsoft\OneDrive\OneDrive.exe")) || exists(format!(r"{pf}\Microsoft OneDrive\OneDrive.exe"));
        super::status_from(winget, &installed, region, edge, onedrive)
    }
}

#[cfg(not(windows))]
mod platform {
    pub fn run(_: &str, _: bool) -> Result<(), String> {
        Err("Len na Windows.".into())
    }
    pub fn status() -> super::Status {
        super::status_from(false, &[], String::new(), false, false)
    }
}

#[cfg_attr(not(windows), allow(unused_imports))]
pub use platform::{run, status};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_winget_export() {
        let j = "\u{feff}{\"Sources\":[{\"Packages\":[{\"PackageIdentifier\":\"Google.Chrome\"},{\"PackageIdentifier\":\"Valve.Steam\"}],\"SourceDetails\":{}}]}";
        assert_eq!(parse_export(j), ["Google.Chrome", "Valve.Steam"]);
        assert!(parse_export("nonsense").is_empty());
        let s = status_from(true, &parse_export(j), "SK".into(), true, false);
        assert!(s.apps.iter().find(|a| a.id == "Google.Chrome").unwrap().installed);
        assert!(!s.apps.iter().find(|a| a.id == "Mozilla.Firefox").unwrap().installed);
        assert!(s.remove.iter().find(|a| a.id == "Microsoft.Edge").unwrap().installed);
        assert!(s.edge_removable);
        assert!(!status_from(true, &[], "US".into(), true, false).edge_removable);
    }

    #[test]
    fn only_listed_packages() {
        assert!(valid_id("Google.Chrome"));
        assert!(valid_id("Microsoft.Edge"));
        assert!(!valid_id("Evil.Package"));
        assert!(!valid_id("Google.Chrome --override x"));
        let mut ids: Vec<_> = PKGS.iter().chain(REMOVABLE).map(|p| p.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), PKGS.len() + REMOVABLE.len());
    }

    #[test]
    fn winget_results() {
        assert!(ok_code(0));
        assert!(ok_code(0x8A15_002Bu32 as i32));
        assert!(!ok_code(1));
        assert!(explain("Microsoft.Edge", false, 1, "").contains("EÚ"));
        assert_eq!(explain("Google.Chrome", true, 1, "Found Google Chrome\r\n  ██████  12 MB / 100 MB\r\nInstaller failed with exit code: 1603\r\n"), "Installer failed with exit code: 1603");
        assert!(explain("Google.Chrome", true, 0x8A15_0017u32 as i32, "").contains("nenašiel"));
    }
}
