//! Tables of hardware and software we know about.
//!
//! Everything here is plain data so it can be unit-tested on any OS.
//! Device IDs for supported hardware come from the OpenRGB project
//! (GPL-2.0), which has verified them on real boards and cards.

/// A USB HID device we recognise.
pub struct KnownHid {
    pub vid: u16,
    pub pid: u16,
    /// HID usage page of the interface that carries the RGB protocol.
    /// `None` = any interface.
    pub usage_page: Option<u16>,
    pub name: &'static str,
    pub role: HidRole,
    pub note: &'static str,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HidRole {
    /// RGB controller we plan to drive.
    Rgb,
    /// Shares a vendor ID with RGB gear but has nothing to do with lighting.
    Ignore,
}

pub const KNOWN_HID: &[KnownHid] = &[
    KnownHid {
        vid: 0x1462,
        pid: 0x7D91,
        usage_page: Some(0x0001),
        name: "MSI MAG Z790 TOMAHAWK WIFI – Mystic Light",
        role: HidRole::Rgb,
        note: "185-bajtový protokol, per-LED direct mód. Ovláda aj ARGB headery (AIO, vetráky cez hub).",
    },
    KnownHid {
        vid: 0x0DB0,
        pid: 0xA74B,
        usage_page: None,
        name: "Realtek USB Audio (zvuk na doske)",
        role: HidRole::Ignore,
        note: "Nie je RGB, ignorujeme.",
    },
];

/// USB vendor IDs that commonly belong to RGB controllers. Used to flag
/// unknown devices worth a closer look.
pub const RGB_VENDORS: &[(u16, &str)] = &[
    (0x1462, "MSI"),
    (0x0DB0, "MSI"),
    (0x048D, "ITE (napr. Gigabyte RGB Fusion 2)"),
    (0x1044, "Gigabyte"),
    (0x0B05, "ASUS"),
    (0x1B1C, "Corsair"),
    (0x1532, "Razer"),
    (0x046D, "Logitech"),
    (0x1E71, "NZXT"),
    (0x2516, "Cooler Master"),
    (0x0CF2, "Lian Li / ENE"),
    (0x26CE, "ASRock"),
    (0x264A, "Thermaltake"),
];

pub fn find_hid(vid: u16, pid: u16, usage_page: u16) -> Option<&'static KnownHid> {
    KNOWN_HID
        .iter()
        .find(|k| k.vid == vid && k.pid == pid && k.usage_page.is_none_or(|p| p == usage_page))
}

pub fn rgb_vendor(vid: u16) -> Option<&'static str> {
    RGB_VENDORS.iter().find(|(v, _)| *v == vid).map(|(_, n)| *n)
}

/// A graphics card we recognise, keyed by PCI IDs.
pub struct KnownGpu {
    pub ven: u16,
    pub dev: u16,
    pub sub_ven: u16,
    pub sub_dev: u16,
    pub name: &'static str,
    /// I2C address of the RGB controller (reached through NVAPI).
    pub rgb_i2c_addr: u8,
    pub note: &'static str,
}

pub const KNOWN_GPU: &[KnownGpu] = &[
    KnownGpu {
        ven: 0x10DE,
        dev: 0x2C02,
        sub_ven: 0x1458,
        sub_dev: 0x418C,
        name: "GIGABYTE AORUS GeForce RTX 5080 MASTER ICE",
        rgb_i2c_addr: 0x71,
        note:
            "RGB Fusion 2 cez I2C. Svetlá áno, LCD displej zatiaľ nie (protokol nie je zmapovaný).",
    },
    KnownGpu {
        ven: 0x10DE,
        dev: 0x2C02,
        sub_ven: 0x1458,
        sub_dev: 0x4178,
        name: "GIGABYTE AORUS GeForce RTX 5080 MASTER",
        rgb_i2c_addr: 0x71,
        note: "RGB Fusion 2 cez I2C. Svetlá áno, LCD displej zatiaľ nie.",
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PciId {
    pub ven: u16,
    pub dev: u16,
    pub sub_ven: u16,
    pub sub_dev: u16,
}

/// Parses a Windows PCI instance ID such as
/// `PCI\VEN_10DE&DEV_2C02&SUBSYS_418C1458&REV_A1\...`.
/// Note the SUBSYS field is device first, vendor second.
pub fn parse_pci(instance_id: &str) -> Option<PciId> {
    let upper = instance_id.to_ascii_uppercase();
    if !upper.starts_with("PCI\\") {
        return None;
    }
    let field = |key: &str, len: usize| -> Option<&str> {
        let start = upper.find(key)? + key.len();
        upper.get(start..start + len)
    };
    let hex = |s: &str| u16::from_str_radix(s, 16).ok();
    let ven = hex(field("VEN_", 4)?)?;
    let dev = hex(field("DEV_", 4)?)?;
    let subsys = field("SUBSYS_", 8)?;
    Some(PciId {
        ven,
        dev,
        sub_dev: hex(&subsys[0..4])?,
        sub_ven: hex(&subsys[4..8])?,
    })
}

pub fn find_gpu(id: &PciId) -> Option<&'static KnownGpu> {
    KNOWN_GPU.iter().find(|g| {
        g.ven == id.ven && g.dev == id.dev && g.sub_ven == id.sub_ven && g.sub_dev == id.sub_dev
    })
}

/// Vendor software that talks to the same RGB controllers. Two programs
/// writing to one controller fight each other (flicker, resets), so we
/// need to know about them before we ever write anything.
pub struct ConflictApp {
    pub name: &'static str,
    pub controls: &'static str,
    /// Lower-case substrings matched against the process exe name and full
    /// path. Kept specific on purpose: "msi" alone would match msiexec.exe.
    pub patterns: &'static [&'static str],
}

pub const CONFLICT_APPS: &[ConflictApp] = &[
    ConflictApp {
        name: "MSI Center / Mystic Light",
        controls: "RGB dosky a ARGB headerov",
        patterns: &[
            "msi.centralserver",
            "msi_central",
            "msi center",
            "ledkeeper",
            "mysticlight",
            "mystic light",
            "\\msi\\one dragon",
            "msi.terminalserver",
        ],
    },
    ConflictApp {
        name: "GIGABYTE Control Center",
        controls: "RGB grafiky (a displej)",
        patterns: &[
            "gigabyte control center",
            "\\gigabyte\\control center",
            "gcc.exe",
            "rgbfusion",
            "rgb fusion",
        ],
    },
    ConflictApp {
        name: "SignalRGB",
        controls: "RGB všetkého, čo podporuje",
        patterns: &["signalrgb"],
    },
    ConflictApp {
        name: "OpenRGB",
        controls: "RGB všetkého, čo podporuje",
        patterns: &["openrgb"],
    },
    ConflictApp {
        name: "ASUS Armoury Crate / Aura",
        controls: "RGB ASUS zariadení",
        patterns: &[
            "armourycrate",
            "armoury crate",
            "lightingservice",
            "aura service",
        ],
    },
    ConflictApp {
        name: "Corsair iCUE",
        controls: "RGB Corsair zariadení",
        patterns: &["icue"],
    },
    ConflictApp {
        name: "Razer Synapse",
        controls: "RGB Razer zariadení",
        patterns: &["razer synapse", "razersynapse", "rzsynapse"],
    },
    ConflictApp {
        name: "NZXT CAM",
        controls: "RGB a vetráky NZXT",
        patterns: &["nzxt cam"],
    },
    ConflictApp {
        name: "Logitech G HUB",
        controls: "RGB Logitech periférií",
        patterns: &["lghub"],
    },
];

/// Matches a process (exe name and optional full path) against the list.
pub fn match_conflict(exe: &str, path: Option<&str>) -> Option<&'static ConflictApp> {
    let exe = exe.to_lowercase();
    let path = path.map(|p| p.to_lowercase());
    CONFLICT_APPS.iter().find(|app| {
        app.patterns
            .iter()
            .any(|p| exe.contains(p) || path.as_deref().is_some_and(|path| path.contains(p)))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_alex_gpu() {
        let id =
            parse_pci(r"PCI\VEN_10DE&DEV_2C02&SUBSYS_418C1458&REV_A1\48C4C379202DB04800").unwrap();
        assert_eq!(
            id,
            PciId {
                ven: 0x10DE,
                dev: 0x2C02,
                sub_ven: 0x1458,
                sub_dev: 0x418C
            }
        );
        assert_eq!(
            find_gpu(&id).unwrap().name,
            "GIGABYTE AORUS GeForce RTX 5080 MASTER ICE"
        );
    }

    #[test]
    fn ignores_virtual_displays() {
        assert!(parse_pci(r"ROOT\DISPLAY\0000").is_none());
    }

    #[test]
    fn finds_board_only_on_rgb_interface() {
        assert!(find_hid(0x1462, 0x7D91, 0x0001).is_some());
        assert!(find_hid(0x1462, 0x7D91, 0xFF00).is_none());
        assert_eq!(
            find_hid(0x0DB0, 0xA74B, 0xFF0A).unwrap().role,
            HidRole::Ignore
        );
    }

    #[test]
    fn matches_vendor_apps_but_not_windows_installer() {
        assert_eq!(
            match_conflict("LEDKeeper2.exe", None).unwrap().name,
            "MSI Center / Mystic Light"
        );
        assert_eq!(
            match_conflict("SignalRgb.exe", None).unwrap().name,
            "SignalRGB"
        );
        assert!(match_conflict("msiexec.exe", Some(r"C:\Windows\System32\msiexec.exe")).is_none());
        assert!(match_conflict("explorer.exe", None).is_none());
    }
}
