//! System tweaks that do something real, each with a plain "off" state to
//! go back to (the Windows default).
//!
//! Deliberately missing: the usual "gaming tweak" folklore (Nagle,
//! SystemResponsiveness, Win32PrioritySeparation, HPET/bcdedit, RAM
//! cleaners, turning off SysMain or the page file). They either do nothing
//! measurable on Windows 10/11 or make things worse. The UI says so.

use serde::Serialize;

pub struct RegOp {
    pub key: &'static str,
    pub name: &'static str,
    /// REG_DWORD or REG_SZ.
    pub ty: &'static str,
    /// Value when the tweak is on; `None` = value absent.
    pub on: Option<&'static str>,
    /// Value when off (Windows default); `None` = delete the value.
    pub off: Option<&'static str>,
    /// What Windows assumes when the value doesn't exist.
    pub absent: Option<&'static str>,
    /// When turning off: delete this whole key instead of the value.
    pub delete_key: Option<&'static str>,
}

pub enum Op {
    Reg(RegOp),
    /// Active power plan: High performance when on, Balanced when off.
    PowerPlan,
}

pub struct Tweak {
    pub key: &'static str,
    pub title: &'static str,
    pub about: &'static str,
    /// "big" = you notice it, "small" = small but real.
    pub effect: &'static str,
    /// "responsive", "games", "background", "looks"
    pub category: &'static str,
    /// Icon name in the UI.
    pub icon: &'static str,
    pub admin: bool,
    /// "", "explorer" (restart File Explorer), "logoff" or "reboot"
    pub restart: &'static str,
    /// Only offered when the first value already exists (e.g. HAGS needs a
    /// GPU and driver that support it; Windows creates the value then).
    pub only_if_exists: bool,
    pub ops: &'static [Op],
}

const fn reg(
    key: &'static str,
    name: &'static str,
    ty: &'static str,
    on: Option<&'static str>,
    off: Option<&'static str>,
) -> Op {
    Op::Reg(RegOp { key, name, ty, on, off, absent: None, delete_key: None })
}

/// An empty default value whose mere existence changes behaviour (like the
/// classic context menu); off deletes `parent` entirely.
const fn reg_marker(key: &'static str, parent: &'static str) -> Op {
    Op::Reg(RegOp { key, name: "", ty: "REG_SZ", on: Some(""), off: None, absent: None, delete_key: Some(parent) })
}

/// Like `reg`, for values whose absence Windows reads as "on".
const fn reg_default_on(
    key: &'static str,
    name: &'static str,
    on: &'static str,
    off: &'static str,
) -> Op {
    Op::Reg(RegOp { key, name, ty: "REG_DWORD", on: Some(on), off: Some(off), absent: Some(on), delete_key: None })
}

pub const TWEAKS: &[Tweak] = &[
    Tweak {
        key: "startup_delay",
        icon: "rocket",
        title: "Spúšťať appky po prihlásení hneď",
        about: "Windows po prihlásení čaká niekoľko sekúnd, kým spustí appky zo spúšťania. Na SSD je to čakanie zbytočné, plocha je skôr použiteľná.",
        effect: "big",
        category: "responsive",
        admin: false,
        restart: "",
        only_if_exists: false,
        ops: &[reg(r"HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Serialize", "StartupDelayInMSec", "REG_DWORD", Some("0"), None)],
    },
    Tweak {
        key: "menu_delay",
        icon: "menu",
        title: "Rýchlejšie podponuky",
        about: "Windows čaká 0,4 s, kým otvorí podponuku. Takto stačí 0,1 s, menu pôsobia svižnejšie.",
        effect: "small",
        category: "responsive",
        admin: false,
        restart: "logoff",
        only_if_exists: false,
        ops: &[reg(r"HKCU\Control Panel\Desktop", "MenuShowDelay", "REG_SZ", Some("100"), Some("400"))],
    },
    Tweak {
        key: "power_plan",
        icon: "bolt",
        title: "Plán napájania Vysoký výkon",
        about: "Procesor menej zaspáva a rýchlejšie reaguje. Na stolnom PC je rozdiel malý, ale merateľný v odozve. Na notebooku viac vybíja batériu.",
        effect: "small",
        category: "responsive",
        admin: false,
        restart: "",
        only_if_exists: false,
        ops: &[Op::PowerPlan],
    },
    Tweak {
        key: "game_dvr",
        icon: "video",
        title: "Vypnúť nahrávanie hier na pozadí",
        about: "Xbox Game Bar vie stále nahrávať posledné minúty hry, čo zaberá grafiku. Ak klipy robíš inou appkou alebo vôbec, netreba to.",
        effect: "big",
        category: "games",
        admin: false,
        restart: "",
        only_if_exists: false,
        ops: &[
            reg(r"HKCU\System\GameConfigStore", "GameDVR_Enabled", "REG_DWORD", Some("0"), Some("1")),
            reg(r"HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR", "AppCaptureEnabled", "REG_DWORD", Some("0"), Some("1")),
        ],
    },
    Tweak {
        key: "game_mode",
        icon: "game",
        title: "Herný režim",
        about: "Počas hry Windows odkladá aktualizácie a dáva hre prednosť. Na dnešných PC neškodí, odporúčam mať zapnutý.",
        effect: "small",
        category: "games",
        admin: false,
        restart: "",
        only_if_exists: false,
        ops: &[reg_default_on(r"HKCU\Software\Microsoft\GameBar", "AutoGameModeEnabled", "1", "0")],
    },
    Tweak {
        key: "hags",
        icon: "gpu",
        title: "Hardvérové plánovanie GPU (HAGS)",
        about: "Grafika si sama plánuje prácu, čo trochu znižuje oneskorenie. Na NVIDIA RTX 40 a novších ho potrebuje Frame Generation (DLSS 3 a 4).",
        effect: "small",
        category: "games",
        admin: true,
        restart: "reboot",
        only_if_exists: true,
        ops: &[reg(r"HKLM\SYSTEM\CurrentControlSet\Control\GraphicsDrivers", "HwSchMode", "REG_DWORD", Some("2"), Some("1"))],
    },
    Tweak {
        key: "hvci_off",
        icon: "shield",
        title: "Vypnúť integritu pamäte (HVCI)",
        about: "Bezpečnostná funkcia Windowsu, ktorá v hrách berie zhruba 5 % výkonu, na starších procesoroch (Ryzen 1000/2000, Intel 6. generácie a starší) aj viac. Vypnutím sa PC stáva o niečo zraniteľnejším voči škodlivým ovládačom. Niektoré anti-cheaty a firemné PC ju vyžadujú.",
        effect: "big",
        category: "games",
        admin: true,
        restart: "reboot",
        only_if_exists: true,
        ops: &[reg(r"HKLM\SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\HypervisorEnforcedCodeIntegrity", "Enabled", "REG_DWORD", Some("0"), Some("1"))],
    },
    Tweak {
        key: "mouse_accel",
        icon: "mouse",
        title: "Vypnúť zrýchlenie myši",
        about: "Kurzor sa posunie o rovnaký kus podľa toho, kam myšou pohneš, nie podľa toho, ako rýchlo. Väčšina hráčov to chce, je to ale vec zvyku.",
        effect: "small",
        category: "games",
        admin: false,
        restart: "logoff",
        only_if_exists: false,
        ops: &[
            reg(r"HKCU\Control Panel\Mouse", "MouseSpeed", "REG_SZ", Some("0"), Some("1")),
            reg(r"HKCU\Control Panel\Mouse", "MouseThreshold1", "REG_SZ", Some("0"), Some("6")),
            reg(r"HKCU\Control Panel\Mouse", "MouseThreshold2", "REG_SZ", Some("0"), Some("10")),
        ],
    },
    Tweak {
        key: "background_apps",
        icon: "app",
        title: "Zakázať appkám zo Storu beh na pozadí",
        about: "Appky z Microsoft Store (Počasie, Prepojenie s telefónom a podobne) nebudú bežať na pozadí a ich upozornenia môžu chodiť neskôr. Klasických programov sa to netýka.",
        effect: "small",
        category: "background",
        admin: false,
        restart: "",
        only_if_exists: false,
        ops: &[reg(r"HKCU\Software\Microsoft\Windows\CurrentVersion\BackgroundAccessApplications", "GlobalUserDisabled", "REG_DWORD", Some("1"), None)],
    },
    Tweak {
        key: "edge_boost",
        icon: "globe",
        title: "Edge sa neprednačítava a po zavretí nebeží",
        about: "Edge sa spúšťa už so štartom Windowsu a beží aj po zavretí okna. Po zapnutí uvidíš v Edge hlásenie „Spravované vašou organizáciou“. To len znamená, že platí toto nastavenie.",
        effect: "small",
        category: "background",
        admin: true,
        restart: "",
        only_if_exists: false,
        ops: &[
            reg(r"HKLM\SOFTWARE\Policies\Microsoft\Edge", "StartupBoostEnabled", "REG_DWORD", Some("0"), None),
            reg(r"HKLM\SOFTWARE\Policies\Microsoft\Edge", "BackgroundModeEnabled", "REG_DWORD", Some("0"), None),
        ],
    },
    Tweak {
        key: "tips",
        icon: "bell",
        title: "Vypnúť tipy a návrhy Windowsu",
        about: "Žiadne „Tipy na používanie Windowsu“, návrhy v Štarte ani v Nastaveniach. Menej rušenia a menej vecí na pozadí.",
        effect: "small",
        category: "background",
        admin: false,
        restart: "",
        only_if_exists: false,
        ops: &[
            reg(r"HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager", "SubscribedContent-338389Enabled", "REG_DWORD", Some("0"), Some("1")),
            reg(r"HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager", "SystemPaneSuggestionsEnabled", "REG_DWORD", Some("0"), Some("1")),
            reg(r"HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager", "SoftLandingEnabled", "REG_DWORD", Some("0"), Some("1")),
        ],
    },
    Tweak {
        key: "transparency",
        icon: "layers",
        title: "Vypnúť priehľadnosť",
        about: "Pomôže na slabších PC a notebookoch s integrovanou grafikou. Na hernom PC rozdiel nespoznáš.",
        effect: "small",
        category: "looks",
        admin: false,
        restart: "",
        only_if_exists: false,
        ops: &[reg(r"HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize", "EnableTransparency", "REG_DWORD", Some("0"), Some("1"))],
    },
    Tweak {
        key: "min_animate",
        icon: "window",
        title: "Okná bez animácie pri minimalizovaní",
        about: "Okná sa minimalizujú a obnovujú hneď. Na slabšom PC pôsobí systém rýchlejšie.",
        effect: "small",
        category: "looks",
        admin: false,
        restart: "logoff",
        only_if_exists: false,
        ops: &[reg(r"HKCU\Control Panel\Desktop\WindowMetrics", "MinAnimate", "REG_SZ", Some("0"), Some("1"))],
    },
    // ---- Windows 11: less noise, clearer Explorer, safer updates
    Tweak {
        key: "classic_menu",
        icon: "menu",
        title: "Klasické kontextové menu",
        about: "Pravý klik hneď ukáže všetky možnosti, bez „Zobraziť ďalšie možnosti“. Programy ako 7-Zip alebo WinRAR sú zase hneď po ruke.",
        effect: "big",
        category: "w_explorer",
        admin: false,
        restart: "explorer",
        only_if_exists: false,
        ops: &[reg_marker(r"HKCU\Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}\InprocServer32", r"HKCU\Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}")],
    },
    Tweak {
        key: "file_ext",
        icon: "file",
        title: "Zobrazovať prípony súborov",
        about: "Uvidíš, či je súbor .jpg, .png alebo .exe. Aj bezpečnejšie: „faktura.pdf.exe“ hneď spoznáš.",
        effect: "big",
        category: "w_explorer",
        admin: false,
        restart: "explorer",
        only_if_exists: false,
        ops: &[reg(r"HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced", "HideFileExt", "REG_DWORD", Some("0"), Some("1"))],
    },
    Tweak {
        key: "explorer_thispc",
        icon: "pc",
        title: "Prieskumník otvárať na Tento počítač",
        about: "Namiesto stránky Domov s nedávnymi súbormi uvidíš hneď disky a priečinky.",
        effect: "small",
        category: "w_explorer",
        admin: false,
        restart: "",
        only_if_exists: false,
        ops: &[reg(r"HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced", "LaunchTo", "REG_DWORD", Some("1"), None)],
    },
    Tweak {
        key: "taskbar_left",
        icon: "taskbar",
        title: "Panel úloh zarovnaný doľava",
        about: "Štart a appky vľavo ako vo Windowse 10. Vec vkusu, na širokom monitore je to pohodlnejšie.",
        effect: "small",
        category: "w_explorer",
        admin: false,
        restart: "",
        only_if_exists: false,
        ops: &[reg(r"HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced", "TaskbarAl", "REG_DWORD", Some("0"), Some("1"))],
    },
    Tweak {
        key: "taskview_off",
        icon: "taskbar",
        title: "Skryť tlačidlo Zobrazenie úloh",
        about: "Menej ikon na paneli úloh. Zobrazenie úloh ostáva na Win + Tab.",
        effect: "small",
        category: "w_explorer",
        admin: false,
        restart: "",
        only_if_exists: false,
        ops: &[reg(r"HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced", "ShowTaskViewButton", "REG_DWORD", Some("0"), Some("1"))],
    },
    Tweak {
        key: "end_task",
        icon: "x",
        title: "„Ukončiť úlohu“ v paneli úloh",
        about: "Pravý klik na appku v paneli úloh ju ukončí, aj keď zamrzla. Bez Správcu úloh.",
        effect: "big",
        category: "w_explorer",
        admin: false,
        restart: "",
        only_if_exists: false,
        ops: &[reg(r"HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\TaskbarDeveloperSettings", "TaskbarEndTask", "REG_DWORD", Some("1"), Some("0"))],
    },
    Tweak {
        key: "start_web",
        icon: "search",
        title: "Štart hľadá len v PC, nie na Bingu",
        about: "Vyhľadávanie v Štarte je rýchlejšie a ukáže tvoje súbory a appky, nie webové výsledky. Ide o politiku, v Nastaveniach preto môžeš vidieť „spravované organizáciou“.",
        effect: "big",
        category: "w_clean",
        admin: false,
        restart: "explorer",
        only_if_exists: false,
        ops: &[reg(r"HKCU\Software\Policies\Microsoft\Windows\Explorer", "DisableSearchBoxSuggestions", "REG_DWORD", Some("1"), None)],
    },
    Tweak {
        key: "start_ads",
        icon: "ad",
        title: "Bez odporúčaní a reklám v Štarte",
        about: "Štart prestane ponúkať „odporúčané“ appky a upozornenia na účet Microsoft.",
        effect: "small",
        category: "w_clean",
        admin: false,
        restart: "",
        only_if_exists: false,
        ops: &[
            reg(r"HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced", "Start_IrisRecommendations", "REG_DWORD", Some("0"), Some("1")),
            reg(r"HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced", "Start_AccountNotifications", "REG_DWORD", Some("0"), Some("1")),
        ],
    },
    Tweak {
        key: "lock_ads",
        icon: "lock",
        title: "Bez tipov a zaujímavostí na zamknutej obrazovke",
        about: "Na zamknutej obrazovke ostane len tvoj obrázok a čas.",
        effect: "small",
        category: "w_clean",
        admin: false,
        restart: "",
        only_if_exists: false,
        ops: &[
            reg(r"HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager", "RotatingLockScreenOverlayEnabled", "REG_DWORD", Some("0"), Some("1")),
            reg(r"HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager", "SubscribedContent-338387Enabled", "REG_DWORD", Some("0"), Some("1")),
        ],
    },
    Tweak {
        key: "setup_nags",
        icon: "bell",
        title: "Bez „Dokončite nastavenie zariadenia“",
        about: "Po aktualizácii ťa Windows nebude ťahať cez obrazovky s ponukou OneDrive, Microsoft 365 a Game Passu.",
        effect: "small",
        category: "w_clean",
        admin: false,
        restart: "",
        only_if_exists: false,
        ops: &[
            reg(r"HKCU\Software\Microsoft\Windows\CurrentVersion\UserProfileEngagement", "ScoobeSystemSettingEnabled", "REG_DWORD", Some("0"), Some("1")),
            reg(r"HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager", "SubscribedContent-310093Enabled", "REG_DWORD", Some("0"), Some("1")),
        ],
    },
    Tweak {
        key: "explorer_ads",
        icon: "ad",
        title: "Bez reklám v Prieskumníkovi",
        about: "Prieskumník prestane ukazovať ponuky OneDrive a Microsoft 365.",
        effect: "small",
        category: "w_clean",
        admin: false,
        restart: "explorer",
        only_if_exists: false,
        ops: &[reg(r"HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced", "ShowSyncProviderNotifications", "REG_DWORD", Some("0"), Some("1"))],
    },
    Tweak {
        key: "widgets_off",
        icon: "layers",
        title: "Vypnúť Miniaplikácie (Widgets)",
        about: "Zmizne panel so správami a počasím aj jeho procesy na pozadí. Ide o politiku, platí pre celý počítač.",
        effect: "small",
        category: "w_clean",
        admin: true,
        restart: "explorer",
        only_if_exists: false,
        ops: &[reg(r"HKLM\SOFTWARE\Policies\Microsoft\Dsh", "AllowNewsAndInterests", "REG_DWORD", Some("0"), None)],
    },
    Tweak {
        key: "ad_id",
        icon: "eye",
        title: "Vypnúť reklamné ID a „prispôsobené zážitky“",
        about: "Microsoft podľa nich cieli reklamy a tipy. Vypnutím nič nestratíš.",
        effect: "small",
        category: "w_privacy",
        admin: false,
        restart: "",
        only_if_exists: false,
        ops: &[
            reg(r"HKCU\Software\Microsoft\Windows\CurrentVersion\AdvertisingInfo", "Enabled", "REG_DWORD", Some("0"), Some("1")),
            reg(r"HKCU\Software\Microsoft\Windows\CurrentVersion\Privacy", "TailoredExperiencesWithDiagnosticDataEnabled", "REG_DWORD", Some("0"), Some("1")),
        ],
    },
    Tweak {
        key: "recall_off",
        icon: "eye",
        title: "Vypnúť Recall",
        about: "Recall na Copilot+ PC priebežne fotí obrazovku a ukladá históriu všetkého, čo si robil. Na ostatných PC táto úprava nič nemení.",
        effect: "small",
        category: "w_privacy",
        admin: false,
        restart: "logoff",
        only_if_exists: false,
        ops: &[reg(r"HKCU\Software\Policies\Microsoft\Windows\WindowsAI", "DisableAIDataAnalysis", "REG_DWORD", Some("1"), None)],
    },
    Tweak {
        key: "no_driver_updates",
        icon: "gpu",
        title: "Windows Update nesmie meniť ovládače",
        about: "Windows rád nainštaluje starší ovládač grafiky cez ten tvoj. Po zapnutí si ovládače (grafika, čipová sada, Wi-Fi) aktualizuješ sám od výrobcu.",
        effect: "big",
        category: "w_update",
        admin: true,
        restart: "",
        only_if_exists: false,
        ops: &[reg(r"HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate", "ExcludeWUDriversInQualityUpdate", "REG_DWORD", Some("1"), None)],
    },
    Tweak {
        key: "no_auto_restart",
        icon: "restart",
        title: "Nereštartovať sám, keď som prihlásený",
        about: "Po aktualizácii ti Windows dá vedieť, ale nereštartuje počítač, kým si prihlásený. Žiadna stratená práca ani prerušený render cez noc.",
        effect: "big",
        category: "w_update",
        admin: true,
        restart: "",
        only_if_exists: false,
        ops: &[reg(r"HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU", "NoAutoRebootWithLoggedOnUsers", "REG_DWORD", Some("1"), None)],
    },
    Tweak {
        key: "fast_startup_off",
        icon: "power",
        title: "Vypnúť Rýchle spustenie",
        about: "Vypnutie PC potom znamená naozajstné vypnutie. Menej záhadných problémov s ovládačmi a aktualizáciami, funguje aj dual-boot. Na SSD štartuje PC skoro rovnako rýchlo.",
        effect: "small",
        category: "w_update",
        admin: true,
        restart: "",
        only_if_exists: true,
        ops: &[reg(r"HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Power", "HiberbootEnabled", "REG_DWORD", Some("0"), Some("1"))],
    },
    Tweak {
        key: "clipboard_history",
        icon: "clipboard",
        title: "História schránky (Win + V)",
        about: "Win + V ukáže posledné veci, ktoré si skopíroval, aj obrázky. Dajú sa pripnúť.",
        effect: "big",
        category: "w_features",
        admin: false,
        restart: "",
        only_if_exists: false,
        ops: &[reg(r"HKCU\Software\Microsoft\Clipboard", "EnableClipboardHistory", "REG_DWORD", Some("1"), Some("0"))],
    },
    Tweak {
        key: "storage_sense",
        icon: "broom",
        title: "Senzor úložiska",
        about: "Windows sám maže dočasné súbory a obsah Koša starší ako 30 dní, keď dochádza miesto.",
        effect: "small",
        category: "w_features",
        admin: false,
        restart: "",
        only_if_exists: false,
        ops: &[reg(r"HKCU\Software\Microsoft\Windows\CurrentVersion\StorageSense\Parameters\StoragePolicy", "01", "REG_DWORD", Some("1"), Some("0"))],
    },
];

pub fn find(key: &str) -> Option<&'static Tweak> {
    TWEAKS.iter().find(|t| t.key == key)
}

/// One command to run. `ignore_err` for deleting values that may not exist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cmd {
    pub exe: &'static str,
    pub args: Vec<String>,
    pub ignore_err: bool,
}

pub fn commands(t: &Tweak, enable: bool) -> Vec<Cmd> {
    t.ops
        .iter()
        .map(|op| match op {
            Op::Reg(r) => match if enable { r.on } else { r.off } {
                Some(v) => {
                    // An empty name means the key's default value.
                    let mut args: Vec<String> = vec!["add".into(), r.key.into()];
                    if r.name.is_empty() {
                        args.push("/ve".into());
                    } else {
                        args.extend(["/v".into(), r.name.into()]);
                    }
                    args.extend(["/t".into(), r.ty.into(), "/d".into(), v.into(), "/f".into()]);
                    Cmd { exe: "reg", args, ignore_err: false }
                }
                None => match r.delete_key {
                    Some(k) => Cmd { exe: "reg", args: vec!["delete".into(), k.into(), "/f".into()], ignore_err: true },
                    None => Cmd {
                        exe: "reg",
                        args: vec!["delete".into(), r.key.into(), "/v".into(), r.name.into(), "/f".into()],
                        ignore_err: true,
                    },
                },
            },
            Op::PowerPlan => Cmd {
                exe: "powercfg",
                args: vec![
                    "/setactive".into(),
                    if enable { crate::system::HIGH_PERF } else { crate::system::BALANCED }.into(),
                ],
                ignore_err: false,
            },
        })
        .collect()
}

/// Compares a value from `reg query` ("0x1", "100") with ours ("1", "100").
pub fn same_value(ty: &str, current: &str, ours: &str) -> bool {
    if ty == "REG_DWORD" {
        crate::system::reg_number(current) == ours.parse::<u64>().ok()
    } else {
        current.trim() == ours
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct State {
    pub key: &'static str,
    pub icon: &'static str,
    pub title: &'static str,
    pub about: &'static str,
    pub effect: &'static str,
    pub category: &'static str,
    pub admin: bool,
    pub restart: &'static str,
    pub on: bool,
    pub available: bool,
}

/// `read(key, name)` returns the current value or None; `power` the active plan GUID.
pub fn states(read: impl Fn(&str, &str) -> Option<String>, power: Option<String>) -> Vec<State> {
    TWEAKS
        .iter()
        .map(|t| {
            let mut on = true;
            let mut available = true;
            for (i, op) in t.ops.iter().enumerate() {
                match op {
                    Op::Reg(r) => {
                        let raw = read(r.key, r.name);
                        if i == 0 && t.only_if_exists && raw.is_none() {
                            available = false;
                        }
                        let cur = raw.or(r.absent.map(String::from));
                        let matches = match (&cur, r.on) {
                            (Some(c), Some(v)) => same_value(r.ty, c, v),
                            (None, None) => true,
                            _ => false,
                        };
                        on &= matches;
                    }
                    Op::PowerPlan => {
                        let p = power.clone().unwrap_or_default();
                        on &= p == crate::system::HIGH_PERF || p == crate::system::ULTIMATE;
                    }
                }
            }
            State {
                key: t.key,
                icon: t.icon,
                title: t.title,
                about: t.about,
                effect: t.effect,
                category: t.category,
                admin: t.admin,
                restart: t.restart,
                on,
                available,
            }
        })
        .collect()
}

#[cfg(windows)]
pub fn list() -> Vec<State> {
    use std::os::windows::process::CommandExt;
    let read = |key: &str, name: &str| -> Option<String> {
        let mut cmd = std::process::Command::new("reg");
        cmd.args(["query", key]);
        if name.is_empty() {
            cmd.arg("/ve");
        } else {
            cmd.args(["/v", name]);
        }
        let o = cmd.creation_flags(0x0800_0000).output().ok()?;
        if !o.status.success() {
            return None;
        }
        let vals = crate::autostart::parse_reg_query(&String::from_utf8_lossy(&o.stdout));
        if name.is_empty() {
            // "(Default)" is localized, e.g. "(Predvolené)".
            return vals.into_iter().next().map(|(_, v)| v);
        }
        vals.into_iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|(_, v)| v)
    };
    states(read, crate::system::facts().power_guid)
}

#[cfg(not(windows))]
pub fn list() -> Vec<State> {
    states(|_, _| None, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn every_tweak_can_go_back() {
        for t in TWEAKS {
            assert!(!commands(t, true).is_empty(), "{}", t.key);
            assert!(!commands(t, false).is_empty(), "{}", t.key);
            for op in t.ops {
                if let Op::Reg(r) = op {
                    assert!(r.on.is_some() || r.off.is_some(), "{} has no state", t.key);
                    assert!(r.key.starts_with("HKCU\\") || (r.key.starts_with("HKLM\\") && t.admin), "{} needs admin flag", t.key);
                }
            }
        }
    }

    #[test]
    fn classic_menu_marker() {
        let t = find("classic_menu").unwrap();
        let on = commands(t, true);
        assert!(on[0].args.contains(&"/ve".to_string()));
        assert_eq!(on[0].args.last().unwrap(), "/f");
        let off = commands(t, false);
        assert_eq!(off[0].args, vec!["delete", r"HKCU\Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}", "/f"]);
        let s = states(|k, n| (k.ends_with("InprocServer32") && n.is_empty()).then(String::new), None);
        assert!(s.iter().find(|x| x.key == "classic_menu").unwrap().on);
        assert!(TWEAKS.iter().all(|t| !t.icon.is_empty()));
        let keys: std::collections::HashSet<_> = TWEAKS.iter().map(|t| t.key).collect();
        assert_eq!(keys.len(), TWEAKS.len(), "duplicate tweak keys");
    }

    #[test]
    fn builds_reg_commands() {
        let t = find("startup_delay").unwrap();
        let on = commands(t, true);
        assert_eq!(on[0].args.join(" "), r"add HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Serialize /v StartupDelayInMSec /t REG_DWORD /d 0 /f");
        let off = commands(t, false);
        assert!(off[0].ignore_err && off[0].args[0] == "delete");
        let p = commands(find("power_plan").unwrap(), false);
        assert_eq!(p[0].args[1], crate::system::BALANCED);
    }

    #[test]
    fn reads_state() {
        let mut vals: HashMap<(&str, &str), &str> = HashMap::new();
        vals.insert((r"HKCU\System\GameConfigStore", "GameDVR_Enabled"), "0x0");
        vals.insert((r"HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR", "AppCaptureEnabled"), "0x0");
        vals.insert((r"HKCU\Control Panel\Desktop", "MenuShowDelay"), "400");
        let read = |k: &str, n: &str| vals.iter().find(|((a, b), _)| *a == k && *b == n).map(|(_, v)| v.to_string());
        let s = states(read, Some(crate::system::HIGH_PERF.into()));
        let get = |k: &str| s.iter().find(|x| x.key == k).unwrap();
        assert!(get("game_dvr").on);
        assert!(!get("menu_delay").on);
        assert!(get("power_plan").on);
        assert!(!get("startup_delay").on);
        assert!(!get("hags").available);
        // Absent value: Windows treats game mode as on.
        assert!(get("game_mode").on);
    }
}
