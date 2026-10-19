//! Facts about this PC: components for the share card, and the settings the
//! helper looks at. Read-only: we only query, never change anything.

use serde::Serialize;

// ---------------------------------------------------------------- reg query parsing

#[derive(Debug, Clone, PartialEq)]
pub struct RegValue {
    pub name: String,
    pub kind: String,
    pub data: String,
}

/// Parses `reg query` output into (key, values) blocks. Works for single
/// keys and for `/s` recursive output.
pub fn parse_reg(text: &str) -> Vec<(String, Vec<RegValue>)> {
    let mut out: Vec<(String, Vec<RegValue>)> = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim_end();
        if trimmed.starts_with("HKEY_") {
            out.push((trimmed.to_string(), Vec::new()));
            continue;
        }
        let l = trimmed.trim_start();
        let Some(idx) = l.find("    REG_") else {
            continue;
        };
        let name = l[..idx].trim().to_string();
        let rest = l[idx..].trim_start();
        let (kind, data) = rest
            .split_once(char::is_whitespace)
            .map(|(k, d)| (k, d.trim()))
            .unwrap_or((rest, ""));
        if out.is_empty() {
            out.push((String::new(), Vec::new()));
        }
        out.last_mut().unwrap().1.push(RegValue {
            name,
            kind: kind.to_string(),
            data: data.to_string(),
        });
    }
    out
}

/// REG_DWORD / REG_QWORD data like `0x2` → 2.
pub fn reg_number(data: &str) -> Option<u64> {
    let d = data.trim();
    match d.strip_prefix("0x").or_else(|| d.strip_prefix("0X")) {
        Some(hex) => u64::from_str_radix(hex, 16).ok(),
        None => d.parse().ok(),
    }
}

fn value<'a>(blocks: &'a [(String, Vec<RegValue>)], name: &str) -> Option<&'a str> {
    blocks
        .iter()
        .flat_map(|(_, v)| v)
        .find(|v| v.name.eq_ignore_ascii_case(name))
        .map(|v| v.data.as_str())
}

// ---------------------------------------------------------------- SMBIOS (RAM)

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct RamModule {
    pub slot: String,
    pub size_mb: u64,
    pub speed: u32,
    pub configured_speed: u32,
    /// DDR4, DDR5…
    pub kind: String,
    pub manufacturer: String,
    pub part: String,
}

/// Parses the raw SMBIOS table returned by `GetSystemFirmwareTable('RSMB')`
/// and returns the populated memory slots (structure type 17).
pub fn parse_smbios_memory(raw: &[u8]) -> Vec<RamModule> {
    // RawSMBIOSData: 4 bytes of version info, u32 length, then the table.
    if raw.len() < 8 {
        return Vec::new();
    }
    let len = u32::from_le_bytes([raw[4], raw[5], raw[6], raw[7]]) as usize;
    let table = &raw[8..(8 + len).min(raw.len())];
    let mut out = Vec::new();
    let mut i = 0;
    while i + 4 <= table.len() {
        let typ = table[i];
        let flen = table[i + 1] as usize;
        if flen < 4 || i + flen > table.len() {
            break;
        }
        let formatted = &table[i..i + flen];
        // Strings follow the formatted area, each NUL-terminated; the set
        // ends with an extra NUL (two NULs right away if there are none).
        let mut j = i + flen;
        let mut strings: Vec<String> = Vec::new();
        if j + 1 < table.len() && table[j] == 0 && table[j + 1] == 0 {
            j += 2;
        } else {
            while j < table.len() {
                let start = j;
                while j < table.len() && table[j] != 0 {
                    j += 1;
                }
                strings.push(String::from_utf8_lossy(&table[start..j]).trim().to_string());
                j += 1;
                if j >= table.len() || table[j] == 0 {
                    j += 1;
                    break;
                }
            }
        }
        let s = |idx: usize| -> String {
            let n = formatted.get(idx).copied().unwrap_or(0) as usize;
            if n == 0 {
                String::new()
            } else {
                strings.get(n - 1).cloned().unwrap_or_default()
            }
        };
        let w = |idx: usize| -> u16 {
            if idx + 1 < formatted.len() {
                u16::from_le_bytes([formatted[idx], formatted[idx + 1]])
            } else {
                0
            }
        };
        let d = |idx: usize| -> u32 {
            if idx + 3 < formatted.len() {
                u32::from_le_bytes([
                    formatted[idx],
                    formatted[idx + 1],
                    formatted[idx + 2],
                    formatted[idx + 3],
                ])
            } else {
                0
            }
        };
        if typ == 17 {
            let raw_size = w(0x0C);
            let size_mb = match raw_size {
                0 | 0xFFFF => 0,
                0x7FFF => (d(0x1C) & 0x7FFF_FFFF) as u64,
                s if s & 0x8000 != 0 => ((s & 0x7FFF) as u64) / 1024,
                s => s as u64,
            };
            if size_mb > 0 {
                out.push(RamModule {
                    slot: s(0x10),
                    size_mb,
                    speed: w(0x15) as u32,
                    configured_speed: w(0x20) as u32,
                    kind: memory_kind(formatted.get(0x12).copied().unwrap_or(0)).to_string(),
                    manufacturer: s(0x17),
                    part: s(0x1A),
                });
            }
        }
        if typ == 127 {
            break;
        }
        i = j;
    }
    out
}

/// SMBIOS memory type byte → name.
fn memory_kind(b: u8) -> &'static str {
    match b {
        0x12 => "DDR",
        0x13 => "DDR2",
        0x18 => "DDR3",
        0x1A => "DDR4",
        0x1B => "LPDDR",
        0x1C => "LPDDR2",
        0x1D => "LPDDR3",
        0x1E => "LPDDR4",
        0x22 => "DDR5",
        0x23 => "LPDDR5",
        _ => "",
    }
}

/// Turns vendor strings into what people call the brand.
pub fn pretty_vendor(s: &str) -> String {
    let l = s.to_lowercase();
    let map: &[(&str, &str)] = &[
        ("micro-star", "MSI"),
        ("asustek", "ASUS"),
        ("gigabyte", "GIGABYTE"),
        ("asrock", "ASRock"),
        ("g skill", "G.Skill"),
        ("g.skill", "G.Skill"),
        ("kingston", "Kingston"),
        ("corsair", "Corsair"),
        ("crucial", "Crucial"),
        ("micron", "Micron"),
        ("samsung", "Samsung"),
        ("sk hynix", "SK hynix"),
        ("hynix", "SK hynix"),
        ("teamgroup", "TeamGroup"),
        ("team group", "TeamGroup"),
        ("patriot", "Patriot"),
        ("adata", "ADATA"),
        ("biostar", "BIOSTAR"),
        ("lenovo", "Lenovo"),
        ("dell", "Dell"),
        ("hp", "HP"),
    ];
    for (k, v) in map {
        if l.starts_with(k) || l.contains(&format!("{k} ")) {
            return v.to_string();
        }
    }
    s.trim().to_string()
}

// ---------------------------------------------------------------- results

#[derive(Serialize, Clone, Debug, Default)]
pub struct Gpu {
    pub name: String,
    pub vram: u64,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct Specs {
    pub cpu: String,
    pub threads: usize,
    pub gpus: Vec<Gpu>,
    pub board: String,
    pub ram_total: u64,
    pub ram_modules: Vec<RamModule>,
    pub disks: Vec<String>,
    pub os: String,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct Facts {
    /// Active power plan GUID and its name as Windows shows it.
    pub power_guid: Option<String>,
    pub power_name: Option<String>,
    pub game_mode: Option<bool>,
    pub hags: Option<bool>,
    pub hibernation: Option<bool>,
    pub hiberfil_bytes: Option<u64>,
    pub os: String,
    pub build: u32,
}

/// Windows 11 still reports "Windows 10" in ProductName; the build tells the truth.
pub fn os_name(product: &str, build: u32, display: &str) -> String {
    let mut name = product.trim().to_string();
    if build >= 22000 {
        name = name.replace("Windows 10", "Windows 11");
    }
    if display.is_empty() {
        name
    } else {
        format!("{name} {display}")
    }
}

/// Picks the GUID and the name out of `powercfg /getactivescheme`, whatever the language.
pub fn parse_powercfg(text: &str) -> (Option<String>, Option<String>) {
    let guid = text
        .split(|c: char| !(c.is_ascii_hexdigit() || c == '-'))
        .find(|w| w.len() == 36 && w.matches('-').count() == 4)
        .map(|g| g.to_lowercase());
    let name = text.find('(').and_then(|a| {
        text[a + 1..]
            .find(')')
            .map(|b| text[a + 1..a + 1 + b].trim().to_string())
    });
    (guid, name)
}

pub const POWER_SAVER: &str = "a1841308-3541-4fab-bc81-f71556f20b4a";
pub const BALANCED: &str = "381b4222-f694-41f0-9685-ff5bb260df2e";
pub const HIGH_PERF: &str = "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c";
pub const ULTIMATE: &str = "e9a42b02-d5df-448d-aa00-03f14749eb61";

#[cfg(windows)]
mod platform {
    use super::*;
    use std::os::windows::process::CommandExt;
    use std::process::Command;
    use windows::core::PCWSTR;
    use windows::Win32::Devices::DeviceAndDriverInstallation::{
        SetupDiDestroyDeviceInfoList, SetupDiEnumDeviceInfo, SetupDiGetClassDevsW,
        SetupDiGetDeviceRegistryPropertyW, DIGCF_PRESENT, GUID_DEVCLASS_DISKDRIVE,
        SPDRP_FRIENDLYNAME, SP_DEVINFO_DATA,
    };
    use windows::Win32::System::SystemInformation::{
        GetSystemFirmwareTable, GlobalMemoryStatusEx, MEMORYSTATUSEX, RSMB,
    };

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    fn run(cmd: &str, args: &[&str]) -> String {
        Command::new(cmd)
            .args(args)
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
            .unwrap_or_default()
    }

    fn reg(key: &str, extra: &[&str]) -> Vec<(String, Vec<RegValue>)> {
        let mut args = vec!["query", key];
        args.extend_from_slice(extra);
        parse_reg(&run("reg", &args))
    }

    fn os() -> (String, u32) {
        let b = reg(r"HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion", &[]);
        let build = value(&b, "CurrentBuild")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        (
            os_name(
                value(&b, "ProductName").unwrap_or("Windows"),
                build,
                value(&b, "DisplayVersion").unwrap_or(""),
            ),
            build,
        )
    }

    pub fn facts() -> Facts {
        let (power_guid, power_name) = parse_powercfg(&run("powercfg", &["/getactivescheme"]));
        let gm = reg(
            r"HKCU\Software\Microsoft\GameBar",
            &["/v", "AutoGameModeEnabled"],
        );
        let hags = reg(
            r"HKLM\SYSTEM\CurrentControlSet\Control\GraphicsDrivers",
            &["/v", "HwSchMode"],
        );
        let hib = reg(
            r"HKLM\SYSTEM\CurrentControlSet\Control\Power",
            &["/v", "HibernateEnabled"],
        );
        let sysdrive = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".into());
        let hiberfil_bytes = std::fs::metadata(format!(r"{sysdrive}\hiberfil.sys"))
            .ok()
            .map(|m| m.len());
        let (os, build) = os();
        Facts {
            power_guid,
            power_name,
            // Missing value = Windows default, which is on.
            game_mode: Some(
                value(&gm, "AutoGameModeEnabled")
                    .and_then(reg_number)
                    .map(|v| v != 0)
                    .unwrap_or(true),
            ),
            hags: value(&hags, "HwSchMode")
                .and_then(reg_number)
                .map(|v| v == 2),
            hibernation: value(&hib, "HibernateEnabled")
                .and_then(reg_number)
                .map(|v| v != 0),
            hiberfil_bytes,
            os,
            build,
        }
    }

    fn smbios() -> Vec<u8> {
        unsafe {
            let need = GetSystemFirmwareTable(RSMB, 0, None);
            if need == 0 {
                return Vec::new();
            }
            let mut buf = vec![0u8; need as usize];
            let got = GetSystemFirmwareTable(RSMB, 0, Some(&mut buf));
            buf.truncate(got as usize);
            buf
        }
    }

    fn disks() -> Vec<String> {
        let mut out = Vec::new();
        unsafe {
            let Ok(set) = SetupDiGetClassDevsW(
                Some(&GUID_DEVCLASS_DISKDRIVE as *const _),
                PCWSTR::null(),
                None,
                DIGCF_PRESENT,
            ) else {
                return out;
            };
            let mut i = 0u32;
            loop {
                let mut info = SP_DEVINFO_DATA {
                    cbSize: std::mem::size_of::<SP_DEVINFO_DATA>() as u32,
                    ..Default::default()
                };
                if SetupDiEnumDeviceInfo(set, i, &mut info).is_err() {
                    break;
                }
                let mut buf = [0u8; 512];
                if SetupDiGetDeviceRegistryPropertyW(
                    set,
                    &info,
                    SPDRP_FRIENDLYNAME,
                    None,
                    Some(&mut buf),
                    None,
                )
                .is_ok()
                {
                    let wide: Vec<u16> = buf
                        .chunks_exact(2)
                        .map(|c| u16::from_le_bytes([c[0], c[1]]))
                        .take_while(|&c| c != 0)
                        .collect();
                    let name = String::from_utf16_lossy(&wide).trim().to_string();
                    if !name.is_empty() {
                        out.push(name);
                    }
                }
                i += 1;
            }
            let _ = SetupDiDestroyDeviceInfoList(set);
        }
        out
    }

    fn gpus() -> Vec<Gpu> {
        let blocks = reg(
            r"HKLM\SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}",
            &["/s"],
        );
        let mut out = Vec::new();
        for (key, vals) in &blocks {
            // Only the numbered adapter keys (…\0000), not their sub-keys.
            let last = key.rsplit('\\').next().unwrap_or("");
            if last.len() != 4 || !last.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }
            let get = |n: &str| {
                vals.iter()
                    .find(|v| v.name.eq_ignore_ascii_case(n))
                    .map(|v| v.data.clone())
            };
            let Some(name) = get("DriverDesc") else {
                continue;
            };
            let id = get("MatchingDeviceId").unwrap_or_default().to_lowercase();
            if !id.starts_with("pci\\") {
                continue; // virtual displays (Parsec, Meta, remote desktop…)
            }
            let vram = get("HardwareInformation.qwMemorySize")
                .and_then(|v| reg_number(&v))
                .unwrap_or(0);
            out.push(Gpu { name, vram });
        }
        out
    }

    pub fn specs() -> Specs {
        let cpu = reg(
            r"HKLM\HARDWARE\DESCRIPTION\System\CentralProcessor\0",
            &["/v", "ProcessorNameString"],
        );
        let bios = reg(r"HKLM\HARDWARE\DESCRIPTION\System\BIOS", &[]);
        let board = format!(
            "{} {}",
            pretty_vendor(value(&bios, "BaseBoardManufacturer").unwrap_or("")),
            value(&bios, "BaseBoardProduct").unwrap_or("")
        )
        .trim()
        .to_string();
        let ram_total = unsafe {
            let mut m = MEMORYSTATUSEX {
                dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
                ..Default::default()
            };
            if GlobalMemoryStatusEx(&mut m).is_ok() {
                m.ullTotalPhys
            } else {
                0
            }
        };
        Specs {
            cpu: value(&cpu, "ProcessorNameString")
                .unwrap_or("")
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" "),
            threads: std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(0),
            gpus: gpus(),
            board,
            ram_total,
            ram_modules: parse_smbios_memory(&smbios()),
            disks: disks(),
            os: os().0,
        }
    }
}

#[cfg(windows)]
pub use platform::{facts, specs};

#[cfg(not(windows))]
pub fn facts() -> Facts {
    Facts::default()
}

#[cfg(not(windows))]
pub fn specs() -> Specs {
    Specs::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_reg_blocks() {
        let text = "\r\nHKEY_LOCAL_MACHINE\\SYSTEM\\Class\\{x}\\0000\r\n    DriverDesc    REG_SZ    NVIDIA GeForce RTX 5080\r\n    HardwareInformation.qwMemorySize    REG_QWORD    0x400000000\r\n\r\nHKEY_LOCAL_MACHINE\\SYSTEM\\Class\\{x}\\0001\r\n    DriverDesc    REG_SZ    Parsec Virtual Display Adapter\r\n";
        let b = parse_reg(text);
        assert_eq!(b.len(), 2);
        assert_eq!(b[0].1[0].data, "NVIDIA GeForce RTX 5080");
        assert_eq!(reg_number(&b[0].1[1].data), Some(16 << 30));
        assert_eq!(value(&b, "DriverDesc"), Some("NVIDIA GeForce RTX 5080"));
        assert_eq!(reg_number("0x2"), Some(2));
    }

    #[test]
    fn reads_power_plan_in_any_language() {
        let sk = "GUID schémy napájania: 381b4222-f694-41f0-9685-ff5bb260df2e  (Vyvážený)";
        assert_eq!(
            parse_powercfg(sk),
            (Some(BALANCED.into()), Some("Vyvážený".into()))
        );
        assert_eq!(parse_powercfg("").0, None);
    }

    #[test]
    fn names_windows_11_correctly() {
        assert_eq!(
            os_name("Windows 10 Pro", 26100, "24H2"),
            "Windows 11 Pro 24H2"
        );
        assert_eq!(
            os_name("Windows 10 Home", 19045, "22H2"),
            "Windows 10 Home 22H2"
        );
    }

    #[test]
    fn prettifies_vendors() {
        assert_eq!(pretty_vendor("Micro-Star International Co., Ltd."), "MSI");
        assert_eq!(pretty_vendor("G Skill Intl"), "G.Skill");
        assert_eq!(pretty_vendor("Unknown Vendor"), "Unknown Vendor");
    }

    fn smbios_with(structs: &[Vec<u8>]) -> Vec<u8> {
        let table: Vec<u8> = structs.concat();
        let mut raw = vec![0, 3, 4, 0];
        raw.extend_from_slice(&(table.len() as u32).to_le_bytes());
        raw.extend_from_slice(&table);
        raw
    }

    fn type17(size: u16, speed: u16, conf: u16, strings: &[&str]) -> Vec<u8> {
        let mut f = vec![0u8; 0x28];
        f[0] = 17;
        f[1] = 0x28;
        f[0x0C..0x0E].copy_from_slice(&size.to_le_bytes());
        f[0x10] = 1; // device locator
        f[0x12] = 0x22; // DDR5
        f[0x15..0x17].copy_from_slice(&speed.to_le_bytes());
        f[0x17] = 2; // manufacturer
        f[0x1A] = 3; // part number
        f[0x20..0x22].copy_from_slice(&conf.to_le_bytes());
        for s in strings {
            f.extend_from_slice(s.as_bytes());
            f.push(0);
        }
        f.push(0);
        f
    }

    #[test]
    fn parses_memory_modules_from_smbios() {
        let mut bios = vec![0u8; 0x12];
        bios[0] = 0;
        bios[1] = 0x12;
        bios.extend_from_slice(b"American Megatrends\0\0");
        let empty_slot = type17(0, 0, 0, &["DIMMA1", "Unknown", "Unknown"]);
        let end = vec![127, 4, 0, 0, 0, 0];
        let raw = smbios_with(&[
            bios,
            type17(
                16384,
                6400,
                6000,
                &["DIMMA2", "G Skill Intl", "F5-6400J3239G16G"],
            ),
            empty_slot,
            type17(
                16384,
                6400,
                6000,
                &["DIMMB2", "G Skill Intl", "F5-6400J3239G16G"],
            ),
            end,
        ]);
        let m = parse_smbios_memory(&raw);
        assert_eq!(m.len(), 2);
        assert_eq!(m[0].size_mb, 16384);
        assert_eq!(m[0].speed, 6400);
        assert_eq!(m[0].configured_speed, 6000);
        assert_eq!(m[0].kind, "DDR5");
        assert_eq!(m[0].manufacturer, "G Skill Intl");
        assert_eq!(m[0].part, "F5-6400J3239G16G");
        assert_eq!(m[1].slot, "DIMMB2");
    }
}
