//! Apps Windows 11 comes with that many people never use. Only a known list
//! is offered, never system parts (Store, Photos, Calculator, Xbox for Game
//! Pass…). Everything here can be installed again from Microsoft Store.

use serde::{Deserialize, Serialize};

pub struct Known {
    /// Package Name or its prefix (ends with '.').
    pub id: &'static str,
    pub title: &'static str,
    pub about: &'static str,
    /// Pre-checked in the list: almost nobody misses it.
    pub junk: bool,
}

pub const KNOWN: &[Known] = &[
    Known { id: "Clipchamp.Clipchamp", title: "Clipchamp", about: "Jednoduchý strihač videa od Microsoftu.", junk: true },
    Known { id: "Microsoft.BingNews", title: "Správy", about: "Microsoft Start, správy s reklamami.", junk: true },
    Known { id: "Microsoft.BingSearch", title: "Vyhľadávanie Bing", about: "Doplnok webového hľadania.", junk: true },
    Known { id: "Microsoft.BingWeather", title: "Počasie", about: "Ak počasie pozeráš v mobile, netreba.", junk: false },
    Known { id: "Microsoft.MicrosoftSolitaireCollection", title: "Solitaire", about: "Kartové hry s reklamami a predplatným.", junk: true },
    Known { id: "Microsoft.GetHelp", title: "Získať pomoc", about: "Chat s podporou Microsoftu.", junk: true },
    Known { id: "Microsoft.Getstarted", title: "Tipy", about: "Návody na Windows.", junk: true },
    Known { id: "Microsoft.WindowsFeedbackHub", title: "Centrum pripomienok", about: "Posielanie spätnej väzby Microsoftu.", junk: true },
    Known { id: "Microsoft.MicrosoftOfficeHub", title: "Microsoft 365 (Office)", about: "Len rozcestník a ponuka predplatného. Nainštalovaný Word ani Excel neodinštaluje.", junk: true },
    Known { id: "Microsoft.PowerAutomateDesktop", title: "Power Automate", about: "Automatizácia pre firmy.", junk: true },
    Known { id: "Microsoft.WindowsMaps", title: "Mapy", about: "Offline mapy Windowsu.", junk: true },
    Known { id: "Microsoft.ZuneVideo", title: "Filmy a TV", about: "Požičovňa filmov od Microsoftu.", junk: true },
    Known { id: "Microsoft.People", title: "Ľudia", about: "Kontakty, staršia appka.", junk: true },
    Known { id: "Microsoft.549981C3F5F10", title: "Cortana", about: "Asistentka, ktorú Microsoft už ukončil.", junk: true },
    Known { id: "Microsoft.Windows.DevHome", title: "Dev Home", about: "Nástroje pre vývojárov, Microsoft ich ukončuje.", junk: true },
    Known { id: "Microsoft.MixedReality.Portal", title: "Portál zmiešanej reality", about: "Pre okuliare Windows Mixed Reality, ktoré už nie sú podporované.", junk: true },
    Known { id: "Microsoft.SkypeApp", title: "Skype", about: "Skype bol ukončený.", junk: true },
    Known { id: "Microsoft.Todos", title: "Microsoft To Do", about: "Úlohy. Ak ho nepoužívaš, netreba.", junk: false },
    Known { id: "Microsoft.OutlookForWindows", title: "Nový Outlook", about: "Pošta od Microsoftu. Ak používaš inú, netreba.", junk: false },
    Known { id: "MSTeams", title: "Microsoft Teams", about: "Ak ho nepoužívaš na prácu ani školu, netreba.", junk: false },
    Known { id: "MicrosoftTeams", title: "Microsoft Teams (osobný)", about: "Ak ho nepoužívaš, netreba.", junk: false },
    Known { id: "Microsoft.YourPhone", title: "Prepojenie s telefónom", about: "Správy a fotky z mobilu na PC. Ak ho nepoužívaš, netreba.", junk: false },
    Known { id: "Microsoft.Copilot", title: "Copilot", about: "AI asistent od Microsoftu.", junk: false },
    Known { id: "BytedancePte.Ltd.TikTok", title: "TikTok", about: "Predinštalovaná ponuka.", junk: true },
    Known { id: "Facebook.", title: "Facebook / Instagram", about: "Predinštalovaná ponuka.", junk: true },
    Known { id: "Disney.", title: "Disney+", about: "Predinštalovaná ponuka.", junk: true },
    Known { id: "king.com.", title: "Candy Crush a ďalšie hry od King", about: "Predinštalovaná ponuka.", junk: true },
    Known { id: "AmazonVideo.PrimeVideo", title: "Prime Video", about: "Predinštalovaná ponuka.", junk: false },
    Known { id: "SpotifyAB.SpotifyMusic", title: "Spotify (zo Storu)", about: "Ak Spotify nepoužívaš, netreba.", junk: false },
];

pub fn known(name: &str) -> Option<&'static Known> {
    KNOWN.iter().find(|k| {
        if k.id.ends_with('.') {
            name.starts_with(k.id)
        } else {
            name.eq_ignore_ascii_case(k.id)
        }
    })
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct App {
    pub name: String,
    pub full: String,
    pub title: &'static str,
    pub about: &'static str,
    pub junk: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Raw {
    name: String,
    package_full_name: String,
    #[serde(default)]
    non_removable: Option<bool>,
    #[serde(default)]
    is_framework: Option<bool>,
}

/// Parses `Get-AppxPackage | Select … | ConvertTo-Json` (an array, or a
/// single object when there's only one package).
pub fn parse(json: &str) -> Vec<App> {
    let raws: Vec<Raw> = serde_json::from_str::<Vec<Raw>>(json)
        .or_else(|_| serde_json::from_str::<Raw>(json).map(|r| vec![r]))
        .unwrap_or_default();
    let mut out: Vec<App> = raws
        .into_iter()
        .filter(|r| !r.non_removable.unwrap_or(false) && !r.is_framework.unwrap_or(false))
        .filter_map(|r| {
            let k = known(&r.name)?;
            Some(App { name: r.name, full: r.package_full_name, title: k.title, about: k.about, junk: k.junk })
        })
        .collect();
    out.sort_by(|a, b| b.junk.cmp(&a.junk).then(a.title.cmp(b.title)));
    out.dedup_by(|a, b| a.full == b.full);
    out
}

/// Package full names only contain these; anything else is refused before
/// it reaches PowerShell.
pub fn valid_full_name(s: &str) -> bool {
    !s.is_empty() && s.len() < 200 && s.chars().all(|c| c.is_ascii_alphanumeric() || "._-~".contains(c))
}

#[cfg(windows)]
pub fn list() -> Vec<App> {
    use std::os::windows::process::CommandExt;
    let o = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "[Console]::OutputEncoding=[Text.Encoding]::UTF8; Get-AppxPackage | Select-Object Name,PackageFullName,NonRemovable,IsFramework | ConvertTo-Json -Compress",
        ])
        .creation_flags(0x0800_0000)
        .output();
    match o {
        Ok(o) => parse(String::from_utf8_lossy(&o.stdout).trim()),
        Err(_) => Vec::new(),
    }
}

#[cfg(not(windows))]
pub fn list() -> Vec<App> {
    Vec::new()
}

#[cfg(windows)]
pub fn uninstall(full: &str) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    if !valid_full_name(full) {
        return Err("Neplatný názov balíka.".into());
    }
    let name = full.split('_').next().unwrap_or("");
    if known(name).is_none() {
        return Err("Túto appku neodinštalujem.".into());
    }
    let o = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &format!("Remove-AppxPackage -Package '{full}'")])
        .creation_flags(0x0800_0000)
        .output()
        .map_err(|e| e.to_string())?;
    if o.status.success() && o.stderr.is_empty() {
        Ok(())
    } else {
        let e = String::from_utf8_lossy(&o.stderr);
        Err(e.lines().find(|l| !l.trim().is_empty()).unwrap_or("Odinštalovanie zlyhalo.").trim().to_string())
    }
}

#[cfg(not(windows))]
pub fn uninstall(_: &str) -> Result<(), String> {
    Err("Len na Windows.".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_only_known_removable_apps() {
        let j = r#"[{"Name":"Clipchamp.Clipchamp","PackageFullName":"Clipchamp.Clipchamp_3.1.0.0_neutral__yxz26nhyzhsrt","NonRemovable":false,"IsFramework":false},
            {"Name":"Microsoft.WindowsStore","PackageFullName":"Microsoft.WindowsStore_22.0.0.0_x64__8wekyb3d8bbwe","NonRemovable":false,"IsFramework":false},
            {"Name":"Microsoft.BingWeather","PackageFullName":"Microsoft.BingWeather_4.53.0.0_x64__8wekyb3d8bbwe","NonRemovable":false,"IsFramework":false},
            {"Name":"Microsoft.Getstarted","PackageFullName":"Microsoft.Getstarted_1.0.0.0_x64__8wekyb3d8bbwe","NonRemovable":true,"IsFramework":false},
            {"Name":"king.com.CandyCrushSaga","PackageFullName":"king.com.CandyCrushSaga_1.0.0.0_x86__kgqvnymyfvs32","NonRemovable":false,"IsFramework":false}]"#;
        let a = parse(j);
        let names: Vec<_> = a.iter().map(|x| x.title).collect();
        assert_eq!(names, vec!["Candy Crush a ďalšie hry od King", "Clipchamp", "Počasie"]);
        assert!(a[0].junk && !a[2].junk);
        let one = parse(r#"{"Name":"Microsoft.SkypeApp","PackageFullName":"Microsoft.SkypeApp_1_x64__kzf8qxf38zg5c"}"#);
        assert_eq!(one.len(), 1);
    }

    #[test]
    fn refuses_odd_package_names() {
        assert!(valid_full_name("Clipchamp.Clipchamp_3.1.0.0_neutral__yxz26nhyzhsrt"));
        assert!(!valid_full_name("x'; Remove-Item C:\\ -Recurse; '"));
        assert!(!valid_full_name(""));
    }
}
