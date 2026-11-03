//! Installed games, found without a disk scan: Steam and Epic manifests,
//! plus well-known folders of Riot, Battle.net and Minecraft. The UI matches
//! them against its list of per-game tips.
//!
//! Also sizes the shader caches, because a full NVIDIA cache means
//! recompiling shaders and stutter when a game loads new areas.

use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Game {
    pub name: String,
    /// "Steam", "Epic Games", "Riot", "Battle.net", "Minecraft"
    pub source: &'static str,
    pub dir: String,
    pub size: u64,
    pub appid: Option<String>,
}

/// `"path"  "D:\\SteamLibrary"` lines from libraryfolders.vdf.
pub fn parse_library_paths(vdf: &str) -> Vec<String> {
    vdf.lines()
        .filter_map(|l| {
            let l = l.trim();
            let rest = l.strip_prefix("\"path\"")?.trim();
            let v = rest.trim_matches('"');
            Some(v.replace("\\\\", "\\"))
        })
        .collect()
}

/// Value of a top-level key in an appmanifest .acf.
pub fn acf_value(acf: &str, key: &str) -> Option<String> {
    let k = format!("\"{key}\"");
    acf.lines().find_map(|l| {
        let l = l.trim();
        let rest = l.strip_prefix(&k)?.trim();
        Some(rest.trim_matches('"').replace("\\\\", "\\"))
    })
}

/// Tools and redistributables Steam installs like games.
fn steam_noise(name: &str) -> bool {
    let n = name.to_lowercase();
    n.contains("redistributable")
        || n.contains("steamworks")
        || n.contains("proton")
        || n.contains("steam linux runtime")
        || n.contains("dedicated server")
        || n.contains("soundtrack")
        || n == "steamvr"
}

pub fn steam_games(steam_root: &Path) -> Vec<Game> {
    let mut libs = vec![steam_root.to_path_buf()];
    if let Ok(v) = std::fs::read_to_string(steam_root.join("steamapps").join("libraryfolders.vdf")) {
        for p in parse_library_paths(&v) {
            let p = PathBuf::from(p);
            if !libs.iter().any(|l| l.to_string_lossy().eq_ignore_ascii_case(&p.to_string_lossy())) {
                libs.push(p);
            }
        }
    }
    let mut out = Vec::new();
    for lib in libs {
        let apps = lib.join("steamapps");
        let Ok(rd) = std::fs::read_dir(&apps) else { continue };
        for e in rd.flatten() {
            let f = e.file_name().to_string_lossy().to_lowercase();
            if !(f.starts_with("appmanifest_") && f.ends_with(".acf")) {
                continue;
            }
            let Ok(acf) = std::fs::read_to_string(e.path()) else { continue };
            let (Some(name), Some(dir)) = (acf_value(&acf, "name"), acf_value(&acf, "installdir")) else { continue };
            if steam_noise(&name) {
                continue;
            }
            out.push(Game {
                name,
                source: "Steam",
                dir: apps.join("common").join(dir).to_string_lossy().into_owned(),
                size: acf_value(&acf, "SizeOnDisk").and_then(|s| s.parse().ok()).unwrap_or(0),
                appid: acf_value(&acf, "appid"),
            });
        }
    }
    out
}

/// Epic writes one JSON `.item` per installed app.
pub fn epic_item(json: &str) -> Option<Game> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    let cats = v.get("AppCategories")?.as_array()?;
    if !cats.iter().any(|c| c.as_str() == Some("games")) {
        return None;
    }
    Some(Game {
        name: v.get("DisplayName")?.as_str()?.to_string(),
        source: "Epic Games",
        dir: v.get("InstallLocation")?.as_str()?.to_string(),
        size: v.get("InstallSize").and_then(|s| s.as_u64()).unwrap_or(0),
        appid: v.get("AppName").and_then(|s| s.as_str()).map(String::from),
    })
}

fn dir_size(p: &Path, depth: u32) -> u64 {
    let Ok(rd) = std::fs::read_dir(p) else { return 0 };
    rd.flatten()
        .map(|e| match e.file_type() {
            Ok(t) if t.is_dir() && depth < 12 => dir_size(&e.path(), depth + 1),
            Ok(t) if t.is_file() => e.metadata().map(|m| m.len()).unwrap_or(0),
            _ => 0,
        })
        .sum()
}

fn fixed_drives() -> Vec<String> {
    ('C'..='Z')
        .map(|c| format!("{c}:\\"))
        .filter(|d| Path::new(d).is_dir())
        .collect()
}

pub fn list() -> Vec<Game> {
    let mut out = Vec::new();
    if let Some(root) = platform::steam_root() {
        out.extend(steam_games(&root));
    }
    if let Ok(pd) = std::env::var("ProgramData") {
        let dir = Path::new(&pd).join(r"Epic\EpicGamesLauncher\Data\Manifests");
        if let Ok(rd) = std::fs::read_dir(dir) {
            for e in rd.flatten() {
                if e.path().extension().is_some_and(|x| x.eq_ignore_ascii_case("item")) {
                    if let Some(g) = std::fs::read_to_string(e.path()).ok().and_then(|j| epic_item(&j)) {
                        out.push(g);
                    }
                }
            }
        }
    }
    // Well-known folders on any drive.
    const KNOWN: &[(&str, &str, &str)] = &[
        (r"Riot Games\League of Legends", "League of Legends", "Riot"),
        (r"Riot Games\VALORANT", "VALORANT", "Riot"),
        (r"Program Files (x86)\World of Warcraft", "World of Warcraft", "Battle.net"),
        (r"Program Files (x86)\Overwatch", "Overwatch 2", "Battle.net"),
        (r"Program Files (x86)\Diablo IV", "Diablo IV", "Battle.net"),
        (r"Program Files (x86)\Call of Duty", "Call of Duty", "Battle.net"),
    ];
    for d in fixed_drives() {
        for (rel, name, source) in KNOWN {
            let p = Path::new(&d).join(rel);
            if p.is_dir() && !out.iter().any(|g| g.name.eq_ignore_ascii_case(name)) {
                out.push(Game {
                    name: name.to_string(),
                    source,
                    dir: p.to_string_lossy().into_owned(),
                    size: 0,
                    appid: None,
                });
            }
        }
    }
    if let Ok(ad) = std::env::var("APPDATA") {
        let mc = Path::new(&ad).join(".minecraft");
        if mc.join("versions").is_dir() {
            out.push(Game {
                name: "Minecraft: Java Edition".into(),
                source: "Minecraft",
                dir: mc.to_string_lossy().into_owned(),
                size: 0,
                appid: None,
            });
        }
    }
    out.sort_by(|a, b| b.size.cmp(&a.size).then(a.name.cmp(&b.name)));
    out
}

#[derive(Serialize, Clone, Debug)]
pub struct ShaderCache {
    pub label: &'static str,
    pub path: String,
    pub size: u64,
}

pub fn shader_caches() -> Vec<ShaderCache> {
    let Ok(la) = std::env::var("LOCALAPPDATA") else { return Vec::new() };
    let base = PathBuf::from(la);
    [
        ("NVIDIA DirectX", base.join(r"NVIDIA\DXCache")),
        ("NVIDIA OpenGL/Vulkan", base.join(r"NVIDIA\GLCache")),
        ("AMD", base.join(r"AMD\DxCache")),
        ("Windows (DirectX)", base.join("D3DSCache")),
    ]
    .into_iter()
    .filter(|(_, p)| p.is_dir())
    .map(|(label, p)| ShaderCache { label, size: dir_size(&p, 0), path: p.to_string_lossy().into_owned() })
    .collect()
}

#[cfg(windows)]
mod platform {
    use std::os::windows::process::CommandExt;
    use std::path::PathBuf;
    pub fn steam_root() -> Option<PathBuf> {
        let o = std::process::Command::new("reg")
            .args(["query", r"HKCU\Software\Valve\Steam", "/v", "SteamPath"])
            .creation_flags(0x0800_0000)
            .output()
            .ok()?;
        let (_, v) = crate::autostart::parse_reg_query(&String::from_utf8_lossy(&o.stdout))
            .into_iter()
            .next()?;
        let p = PathBuf::from(v.replace('/', "\\"));
        p.is_dir().then_some(p)
    }
}

#[cfg(not(windows))]
mod platform {
    pub fn steam_root() -> Option<std::path::PathBuf> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_steam_libraries() {
        let vdf = "\"libraryfolders\"\n{\n\t\"0\"\n\t{\n\t\t\"path\"\t\t\"C:\\\\Program Files (x86)\\\\Steam\"\n\t}\n\t\"1\"\n\t{\n\t\t\"path\"\t\t\"D:\\\\SteamLibrary\"\n\t}\n}";
        assert_eq!(parse_library_paths(vdf), vec![r"C:\Program Files (x86)\Steam", r"D:\SteamLibrary"]);
        let acf = "\"AppState\"\n{\n\t\"appid\"\t\t\"730\"\n\t\"name\"\t\t\"Counter-Strike 2\"\n\t\"installdir\"\t\t\"Counter-Strike Global Offensive\"\n\t\"SizeOnDisk\"\t\t\"35000000000\"\n}";
        assert_eq!(acf_value(acf, "appid").as_deref(), Some("730"));
        assert_eq!(acf_value(acf, "name").as_deref(), Some("Counter-Strike 2"));
        assert_eq!(acf_value(acf, "SizeOnDisk").as_deref(), Some("35000000000"));
        assert!(steam_noise("Steamworks Common Redistributables"));
        assert!(!steam_noise("Cyberpunk 2077"));
    }

    #[test]
    fn steam_library_on_disk() {
        let base = std::env::temp_dir().join(format!("wi-steam-{}", std::process::id()));
        let apps = base.join("steamapps");
        std::fs::create_dir_all(&apps).unwrap();
        std::fs::write(apps.join("appmanifest_730.acf"), "\"AppState\"\n{\n\"appid\" \"730\"\n\"name\" \"Counter-Strike 2\"\n\"installdir\" \"cs\"\n\"SizeOnDisk\" \"100\"\n}").unwrap();
        std::fs::write(apps.join("appmanifest_228980.acf"), "\"AppState\"\n{\n\"appid\" \"228980\"\n\"name\" \"Steamworks Common Redistributables\"\n\"installdir\" \"x\"\n}").unwrap();
        let g = steam_games(&base);
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].name, "Counter-Strike 2");
        assert_eq!(g[0].size, 100);
        assert!(g[0].dir.ends_with("cs"));
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn reads_epic_manifest() {
        let j = r#"{"DisplayName":"Fortnite","InstallLocation":"C:\\Program Files\\Epic Games\\Fortnite","InstallSize":50000,"AppName":"Fortnite","AppCategories":["public","games","applications"]}"#;
        let g = epic_item(j).unwrap();
        assert_eq!(g.name, "Fortnite");
        assert_eq!(g.source, "Epic Games");
        let launcher = r#"{"DisplayName":"Unreal Engine","InstallLocation":"x","AppCategories":["engines"]}"#;
        assert!(epic_item(launcher).is_none());
    }
}
