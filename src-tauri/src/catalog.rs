//! Catalog of programs we can explain to the user.
//!
//! Plain data so it can be tested on any OS. Descriptions are written for
//! a normal person, in Slovak, and are honest: we only say "you can turn
//! this off" when that is really true.

use serde::Serialize;

#[derive(Serialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    /// Part of Windows. Leave it alone.
    System,
    /// Needed for something to work (drivers, runtimes).
    Needed,
    /// Vendor RGB software. Will be replaceable by WhatsInside later.
    Rgb,
    /// Mouse/keyboard/headset software. Fine to keep if you use it.
    Peripheral,
    /// Doesn't need to run all the time; usually safe to remove from autostart.
    Optional,
    /// Not in the catalog.
    Unknown,
}

pub struct App {
    pub name: &'static str,
    /// Lower-case substrings matched against exe name, full path, service
    /// name or autostart command. Keep them specific: "msi" alone would
    /// match msiexec.exe (Windows Installer).
    pub patterns: &'static [&'static str],
    pub verdict: Verdict,
    pub about: &'static str,
}

pub const APPS: &[App] = &[
    App {
        name: "WhatsInside",
        patterns: &["whatsinside"],
        verdict: Verdict::Needed,
        about: "To som ja. Keď ma zatvoríš, nebeží nič.",
    },
    App {
        name: "MSI Center / Mystic Light",
        patterns: &["msi.centralserver", "msi_central", "\\msi\\msi center", "msi center", "ledkeeper", "mysticlight", "mystic light", "msi.terminalserver", "\\msi\\one dragon"],
        verdict: Verdict::Rgb,
        about: "Ovláda svetlá dosky a všetkého, čo je na nej zapojené (vetráky, AIO). Beží stále, aj keď ju práve nepoužívaš.",
    },
    App {
        name: "GIGABYTE Control Center",
        patterns: &["\\gigabyte\\control center", "gigabyte control center", "gcc.exe", "rgbfusion", "rgb fusion"],
        verdict: Verdict::Rgb,
        about: "Ovláda svetlá a displej grafiky. Má niekoľko pomocných procesov (meranie FPS, PresentMon), ktoré bežia stále.",
    },
    App {
        name: "SignalRGB",
        patterns: &["signalrgb"],
        verdict: Verdict::Rgb,
        about: "Ovláda RGB naprieč výrobcami. Kreslí efekty v reálnom čase, takže musí bežať stále.",
    },
    App {
        name: "OpenRGB",
        patterns: &["openrgb"],
        verdict: Verdict::Rgb,
        about: "Open-source ovládanie RGB.",
    },
    App {
        name: "ASUS Armoury Crate",
        patterns: &["armourycrate", "armoury crate", "lightingservice", "asus\\aura"],
        verdict: Verdict::Rgb,
        about: "Svetlá a nastavenia ASUS hardvéru. Známy tým, že beží v mnohých procesoch naraz.",
    },
    App {
        name: "Corsair iCUE",
        patterns: &["icue"],
        verdict: Verdict::Rgb,
        about: "Svetlá a vetráky Corsair.",
    },
    App {
        name: "NZXT CAM",
        patterns: &["nzxt cam"],
        verdict: Verdict::Rgb,
        about: "Svetlá, vetráky a monitoring NZXT.",
    },
    App {
        name: "Logitech G HUB",
        patterns: &["lghub"],
        verdict: Verdict::Peripheral,
        about: "Nastavenia myši a klávesnice Logitech. Ak má myš profil uložený v sebe, G HUB nemusí bežať stále.",
    },
    App {
        name: "SteelSeries GG / Sonar",
        patterns: &["steelseriesgg", "steelseries gg", "steelseriesengine", "steelseries sonar", "steelseriessonar", "\\steelseries\\"],
        verdict: Verdict::Peripheral,
        about: "Nastavenia slúchadiel SteelSeries. Ak používaš Sonar (virtuálne zvukové zariadenia), musí bežať.",
    },
    App {
        name: "Razer Synapse",
        patterns: &["razer synapse", "razersynapse", "rzsynapse", "razer central"],
        verdict: Verdict::Peripheral,
        about: "Nastavenia periférií Razer.",
    },
    App {
        name: "NVIDIA ovládač",
        patterns: &["nvdisplay.container"],
        verdict: Verdict::Needed,
        about: "Súčasť ovládača grafiky. Nechaj tak.",
    },
    App {
        name: "NVIDIA app / Overlay",
        patterns: &["nvidia app", "nvidia overlay", "nvcontainer", "nvsphelper", "nvidia share", "nvoawrapper"],
        verdict: Verdict::Optional,
        about: "Overlay, nahrávanie (ShadowPlay) a aktualizácie ovládača. Ak ich nepoužívaš, nemusí bežať stále.",
    },
    App {
        name: "Microsoft Edge WebView2",
        patterns: &["msedgewebview2"],
        verdict: Verdict::Needed,
        about: "Kreslí okná mnohých appiek (aj tejto). Patrí k appke, ktorá ho spustila, nezabíjaj ho samostatne.",
    },
    App {
        name: "Microsoft Edge",
        patterns: &["msedge.exe"],
        verdict: Verdict::Optional,
        about: "Edge vie bežať na pozadí aj po zatvorení. Vypína sa v nastaveniach Edge → Systém → Pokračovať v spúšťaní rozšírení a aplikácií na pozadí.",
    },
    App {
        name: "Microsoft OneDrive",
        patterns: &["onedrive.exe", "onedrive.sync", "\\microsoft\\onedrive\\", "filecoauth"],
        verdict: Verdict::Optional,
        about: "Synchronizácia súborov s cloudom. Ak OneDrive nepoužívaš, dá sa vypnúť z automatického spúšťania.",
    },
    App {
        name: "Microsoft Teams",
        patterns: &["ms-teams", "\\teams\\"],
        verdict: Verdict::Optional,
        about: "Spúšťa sa s Windowsom, aj keď ho nepotrebuješ. Dá sa vypnúť v nastaveniach Teams.",
    },
    App {
        name: "Miniaplikácie (Widgets)",
        patterns: &["widgets.exe", "widgetservice", "widgetboard"],
        verdict: Verdict::Optional,
        about: "Panel s počasím a správami. Ak ho nepoužívaš, vypni ho v nastaveniach panela úloh.",
    },
    App {
        name: "Prepojenie s telefónom",
        patterns: &["phoneexperiencehost", "crossdevice"],
        verdict: Verdict::Optional,
        about: "Prepojenie s mobilom. Ak ho nepoužívaš, dá sa vypnúť.",
    },
    App {
        name: "Xbox Game Bar",
        patterns: &["gamebar.exe", "gamebarftserver", "gamebarpresencewriter"],
        verdict: Verdict::Optional,
        about: "Herný overlay Windowsu (Win+G). Ak ho nepoužívaš, nemusí bežať.",
    },
    App {
        name: "Discord",
        patterns: &["discord"],
        verdict: Verdict::Optional,
        about: "Spúšťa sa s Windowsom. Dá sa vypnúť v Discorde → Nastavenia → Nastavenia systému Windows.",
    },
    App {
        name: "Steam",
        patterns: &["steam.exe", "steamwebhelper", "steamservice"],
        verdict: Verdict::Optional,
        about: "Spúšťa sa s Windowsom a drží v pamäti webové okná (steamwebhelper). Nemusí bežať, kým nehráš.",
    },
    App {
        name: "Epic Games Launcher",
        patterns: &["epicgameslauncher", "epicwebhelper"],
        verdict: Verdict::Optional,
        about: "Nemusí bežať, kým nehráš.",
    },
    App {
        name: "Spotify",
        patterns: &["spotify"],
        verdict: Verdict::Optional,
        about: "Spúšťa sa s Windowsom. Dá sa vypnúť v nastaveniach Spotify.",
    },
    App {
        name: "Adobe Creative Cloud",
        patterns: &["creative cloud", "ccxprocess", "adobeipcbroker", "coresync", "adobe desktop service", "acrotray", "adobecollabsync", "adobe genuine", "agsservice", "adobeupdateservice"],
        verdict: Verdict::Optional,
        about: "Synchronizácia, fonty a aktualizácie Adobe. Fonty z Adobe Fonts potrebujú bežiaci Creative Cloud, inak nemusí bežať stále.",
    },
    App {
        name: "Parsec",
        patterns: &["parsecd", "\\parsec\\"],
        verdict: Verdict::Optional,
        about: "Vzdialený prístup k PC. Ak sa naň nepripájaš na diaľku, nemusí bežať.",
    },
    App {
        name: "Meta Quest Link",
        patterns: &["oculus", "ovrservice", "ovrserver", "\\meta\\", "meta quest"],
        verdict: Verdict::Optional,
        about: "Potrebné len pri hraní VR cez Quest Link.",
    },
];

pub fn find(haystacks: &[&str]) -> Option<&'static App> {
    let lowered: Vec<String> = haystacks.iter().map(|h| h.to_lowercase()).collect();
    APPS.iter().find(|app| {
        app.patterns
            .iter()
            .any(|p| lowered.iter().any(|h| h.contains(p)))
    })
}

/// True when a path points inside the Windows directory itself.
pub fn is_windows_path(path: &str) -> bool {
    let p = path.to_lowercase();
    let windir = std::env::var("SystemRoot")
        .unwrap_or_else(|_| r"C:\Windows".into())
        .to_lowercase();
    p.starts_with(&(windir + "\\")) || p.starts_with(r"\systemroot\")
}

/// Core Windows processes that we can't open as a normal user (no path),
/// but which are obviously system.
pub fn is_system_name(exe: &str) -> bool {
    const NAMES: &[&str] = &[
        "system",
        "system idle process",
        "[system process]",
        "registry",
        "smss.exe",
        "csrss.exe",
        "wininit.exe",
        "services.exe",
        "lsass.exe",
        "svchost.exe",
        "winlogon.exe",
        "fontdrvhost.exe",
        "dwm.exe",
        "memory compression",
        "lsaiso.exe",
        "secure system",
        "spoolsv.exe",
        "wudfhost.exe",
        "msmpeng.exe",
        "nissrv.exe",
        "securityhealthservice.exe",
        "searchindexer.exe",
        "audiodg.exe",
        "dashost.exe",
        "conhost.exe",
        "sgrmbroker.exe",
        "mpdefendercoreservice.exe",
        "vmmem",
        "vmmemws",
    ];
    NAMES.contains(&exe.to_lowercase().as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_vendor_helpers() {
        assert_eq!(
            find(&[
                "GvFPS.exe",
                r"C:\Program Files\GIGABYTE\Control Center\Lib\GBT_VGA\Service\Dll\GvFPS.exe"
            ])
            .unwrap()
            .name,
            "GIGABYTE Control Center"
        );
        assert_eq!(find(&["LEDKeeper2.exe"]).unwrap().verdict, Verdict::Rgb);
        assert_eq!(
            find(&["lghub_agent.exe"]).unwrap().verdict,
            Verdict::Peripheral
        );
    }

    #[test]
    fn does_not_confuse_windows_installer_or_webview() {
        assert!(find(&["msiexec.exe", r"C:\Windows\System32\msiexec.exe"]).is_none());
        assert_eq!(
            find(&["msedgewebview2.exe"]).unwrap().name,
            "Microsoft Edge WebView2"
        );
        assert_eq!(find(&["msedge.exe"]).unwrap().name, "Microsoft Edge");
    }

    #[test]
    fn detects_system() {
        assert!(is_system_name("svchost.exe"));
        assert!(is_windows_path(r"C:\Windows\System32\dwm.exe"));
        assert!(!is_windows_path(r"C:\Program Files\App\app.exe"));
    }
}
