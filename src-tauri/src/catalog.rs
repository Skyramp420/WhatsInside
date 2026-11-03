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
    App {
        name: "Microsoft Office",
        patterns: &["officeclicktorun", "clicktorunsvc", "\\microsoft office\\"],
        verdict: Verdict::Needed,
        about: "Aktualizácie a spúšťanie Office. Bez tejto služby sa Word a Excel nespustia.",
    },
    App {
        name: "LookAtThis",
        patterns: &["lookatthis"],
        verdict: Verdict::Needed,
        about: "Z rodiny WheresTheApp. Klipy z hier, beží nenápadne na pozadí.",
    },
    App {
        name: "WorkingOnIt",
        patterns: &["workingonit"],
        verdict: Verdict::Needed,
        about: "Z rodiny WheresTheApp. Prehľad práce na projektoch.",
    },
    App {
        name: "Riot Vanguard",
        patterns: &["\\riot vanguard\\", "vgc.exe", "vgtray", "vgk.sys"],
        verdict: Verdict::Needed,
        about: "Anti-cheat pre Valorant a League of Legends. Štartuje už s Windowsom a bez neho sa tieto hry nespustia. Ak ich nehráš, odinštaluj ho cez Aplikácie.",
    },
    App {
        name: "Riot Client",
        patterns: &["riotclientservices", "riot client"],
        verdict: Verdict::Optional,
        about: "Spúšťač hier od Riotu. Nemusí bežať, kým nehráš.",
    },
    App {
        name: "EA app",
        patterns: &["eadesktop", "ea desktop", "eabackgroundservice", "\\electronic arts\\ea desktop"],
        verdict: Verdict::Optional,
        about: "Spúšťač hier od EA. Nemusí bežať, kým nehráš.",
    },
    App {
        name: "Ubisoft Connect",
        patterns: &["upc.exe", "ubisoftconnect", "ubisoft connect", "uplay"],
        verdict: Verdict::Optional,
        about: "Spúšťač hier od Ubisoftu. Nemusí bežať, kým nehráš.",
    },
    App {
        name: "Battle.net",
        patterns: &["battle.net"],
        verdict: Verdict::Optional,
        about: "Spúšťač hier od Blizzardu. Nemusí bežať, kým nehráš.",
    },
    App {
        name: "GOG Galaxy",
        patterns: &["galaxyclient", "gog galaxy", "galaxycommunication"],
        verdict: Verdict::Optional,
        about: "Spúšťač hier od GOG. Nemusí bežať, kým nehráš.",
    },
    App {
        name: "Rockstar Games Launcher",
        patterns: &["rockstar games launcher", "rockstarservice", "socialclubhelper"],
        verdict: Verdict::Optional,
        about: "Spúšťač hier od Rockstaru. Nemusí bežať, kým nehráš.",
    },
    App {
        name: "Overwolf",
        patterns: &["overwolf"],
        verdict: Verdict::Optional,
        about: "Overlaye a doplnky do hier (CurseForge, štatistiky). Beží v niekoľkých procesoch a pri hraní ukrajuje výkon.",
    },
    App {
        name: "MSI Afterburner / RivaTuner",
        patterns: &["msiafterburner", "rtss.exe", "rivatuner"],
        verdict: Verdict::Optional,
        about: "Pretaktovanie a FPS overlay. Ak máš nastavenú krivku alebo undervolt grafiky, musí sa spúšťať s Windowsom, inak sa nastavenie neaplikuje.",
    },
    App {
        name: "Wallpaper Engine",
        patterns: &["wallpaper32", "wallpaper64", "wallpaperservice", "wallpaper_engine"],
        verdict: Verdict::Optional,
        about: "Živá tapeta. Pri hre sa sama pozastaví, ale stále drží pamäť.",
    },
    App {
        name: "Google aktualizácie",
        patterns: &["googleupdate", "googleupdater", "\\google\\update\\", "gupdate", "googlecrashhandler"],
        verdict: Verdict::Optional,
        about: "Aktualizuje Chrome a iné appky od Googlu. Ak ho vypneš z automatického spúšťania, Chrome sa aktualizuje sám pri spustení.",
    },
    App {
        name: "Microsoft Edge aktualizácie",
        patterns: &["microsoftedgeupdate", "edgeupdate", "\\microsoft\\edgeupdate\\"],
        verdict: Verdict::Optional,
        about: "Aktualizuje Edge a WebView2. Môže štartovať ručne, keď ho Edge potrebuje. Úplne vypnúť ho neodporúčam, WebView2 používa veľa appiek.",
    },
    App {
        name: "Adobe Acrobat aktualizácie",
        patterns: &["adobearm", "armsvc", "\\adobe\\arm\\"],
        verdict: Verdict::Optional,
        about: "Aktualizácie Acrobat Readeru. Stačí, keď štartuje ručne.",
    },
    App {
        name: "Brave aktualizácie",
        patterns: &["braveupdate", "brave update"],
        verdict: Verdict::Optional,
        about: "Aktualizácie prehliadača Brave. Prehliadač sa aktualizuje aj pri spustení.",
    },
    App {
        name: "Opera",
        patterns: &["opera_autoupdate", "opera browser assistant", "\\opera gx\\", "\\programs\\opera\\"],
        verdict: Verdict::Optional,
        about: "Aktualizácie a asistent Opery. Spúšťa sa s Windowsom, ale nemusí.",
    },
    App {
        name: "Java aktualizácie",
        patterns: &["jusched", "jucheck", "\\java update\\"],
        verdict: Verdict::Optional,
        about: "Pravidelne kontroluje aktualizácie Javy. Nemusí bežať stále.",
    },
    App {
        name: "Dropbox",
        patterns: &["dropbox"],
        verdict: Verdict::Optional,
        about: "Synchronizácia súborov. Ak Dropbox nepoužívaš, nemusí sa spúšťať.",
    },
    App {
        name: "CCleaner",
        patterns: &["ccleaner"],
        verdict: Verdict::Optional,
        about: "Beží na pozadí kvôli „monitorovaniu“ a reklamám na Pro verziu. Nič, čo by Windows nevedel sám.",
    },
    App {
        name: "Intel Driver & Support Assistant",
        patterns: &["dsatray", "dsaservice", "dsaupdateservice", "driver & support assistant"],
        verdict: Verdict::Optional,
        about: "Kontroluje ovládače Intel. Stačí ho spustiť raz za čas ručne.",
    },
    App {
        name: "Apple aktualizácie",
        patterns: &["softwareupdate.exe", "apple software update"],
        verdict: Verdict::Optional,
        about: "Aktualizácie appiek od Apple. Nemusí bežať stále.",
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

/// What we suggest for a built-in Windows service that starts automatically.
#[derive(Serialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Advice {
    /// Safe to switch to manual start for most people.
    Off,
    /// Depends on what you use; explained in `about`.
    Maybe,
    /// Leave it, explained because people often get told to disable it.
    Keep,
}

pub struct WinService {
    pub key: &'static str,
    pub title: &'static str,
    pub advice: Advice,
    pub about: &'static str,
}

/// Built-in services people ask about. Honest: switching these off saves a
/// few MB and a little background work, it won't add FPS.
pub const WIN_SERVICES: &[WinService] = &[
    WinService { key: "diagtrack", title: "Telemetria Windowsu", advice: Advice::Off,
        about: "Posiela Microsoftu diagnostické údaje. Windows funguje rovnako aj bez nej." },
    WinService { key: "dmwappushservice", title: "Smerovanie push správ (telemetria)", advice: Advice::Off,
        about: "Pomocník telemetrie. Bez nej nič nestratíš." },
    WinService { key: "mapsbroker", title: "Správca stiahnutých máp", advice: Advice::Off,
        about: "Offline mapy pre appku Mapy. Ak ju nepoužívaš, nemusí štartovať." },
    WinService { key: "retaildemo", title: "Režim predajne", advice: Advice::Off,
        about: "Ukážkový režim pre PC vystavené v obchode." },
    WinService { key: "remoteregistry", title: "Vzdialený register", advice: Advice::Off,
        about: "Umožňuje meniť register cez sieť. Doma ho nepotrebuješ a vypnutý je bezpečnejší." },
    WinService { key: "fax", title: "Fax", advice: Advice::Off, about: "Fax." },
    WinService { key: "trkwks", title: "Sledovanie distribuovaných prepojení", advice: Advice::Off,
        about: "Opravuje odkazy na súbory presunuté v sieti firmy. Doma zbytočné." },
    WinService { key: "pcasvc", title: "Asistent kompatibility programov", advice: Advice::Maybe,
        about: "Pri starých programoch ponúka režim kompatibility. Ak hráš staré hry, nechaj ho." },
    WinService { key: "spooler", title: "Tlač (Print Spooler)", advice: Advice::Maybe,
        about: "Potrebná na tlač aj „Tlačiť do PDF“. Ak netlačíš, môže štartovať ručne." },
    WinService { key: "wsearch", title: "Indexovanie vyhľadávania", advice: Advice::Maybe,
        about: "Zrýchľuje hľadanie v Štarte a Prieskumníkovi. Bez neho hľadanie funguje, len pomalšie." },
    WinService { key: "sysmain", title: "SysMain (Superfetch)", advice: Advice::Keep,
        about: "Pred-načítava appky, ktoré často používaš. Na rýchlom SSD je efekt malý, ale ani neškodí. Často sa radí ho vypnúť, výkon to však nepridá." },
    WinService { key: "iphlpsvc", title: "Pomocník IP", advice: Advice::Maybe,
        about: "Prechodové technológie IPv6. Väčšinou nie je potrebná, ale niektoré hry a VPN ju využívajú." },
    WinService { key: "xblauthmanager", title: "Xbox Live prihlásenie", advice: Advice::Keep,
        about: "Potrebné pre hry z Game Passu a appku Xbox." },
    WinService { key: "xboxgipsvc", title: "Xbox príslušenstvo", advice: Advice::Keep,
        about: "Potrebné pre Xbox ovládač." },
    WinService { key: "wersvc", title: "Hlásenie chýb Windowsu", advice: Advice::Maybe,
        about: "Pri páde appky pošle hlásenie Microsoftu. Bez neho nič nestratíš, len neuvidíš okno s hlásením." },
];

pub fn win_service(key: &str) -> Option<&'static WinService> {
    let k = key.to_lowercase();
    WIN_SERVICES.iter().find(|w| w.key == k)
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
    fn knows_updaters_and_windows_services() {
        assert_eq!(find(&[r"C:\Program Files (x86)\Google\GoogleUpdater\updater.exe"]).unwrap().name, "Google aktualizácie");
        assert_eq!(find(&["vgc.exe"]).unwrap().verdict, Verdict::Needed);
        assert_eq!(win_service("DiagTrack").unwrap().advice, Advice::Off);
        assert_eq!(win_service("SysMain").unwrap().advice, Advice::Keep);
        assert!(win_service("AudioSrv").is_none());
    }

    #[test]
    fn detects_system() {
        assert!(is_system_name("svchost.exe"));
        assert!(is_windows_path(r"C:\Windows\System32\dwm.exe"));
        assert!(!is_windows_path(r"C:\Program Files\App\app.exe"));
    }
}
