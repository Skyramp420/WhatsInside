//! Hardware detection carried over from v0.1. Read-only: devices are only
//! enumerated, never opened for I/O.

use crate::known::{self, HidRole};
use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct Device {
    pub kind: &'static str,
    pub name: String,
    pub id: String,
    pub note: &'static str,
    pub supported: bool,
}

#[derive(Serialize, Default)]
pub struct HwReport {
    pub devices: Vec<Device>,
    pub nvapi: bool,
    pub errors: Vec<String>,
}

pub struct RawHid {
    pub vid: u16,
    pub pid: u16,
    pub usage_page: u16,
    pub product: Option<String>,
}

/// Turns raw enumeration results into the list the UI shows.
pub fn summarize(hid: &[RawHid], gpus: &[String], nvapi: bool, errors: Vec<String>) -> HwReport {
    let mut devices = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for d in hid {
        if !seen.insert((d.vid, d.pid, d.usage_page)) {
            continue;
        }
        match known::find_hid(d.vid, d.pid, d.usage_page) {
            Some(k) if k.role == HidRole::Rgb => devices.push(Device {
                kind: "RGB radič",
                name: k.name.into(),
                id: format!("USB {:04X}:{:04X}", d.vid, d.pid),
                note: k.note,
                supported: true,
            }),
            Some(_) => {}
            None => {
                if let Some(vendor) = known::rgb_vendor(d.vid) {
                    if known::KNOWN_HID
                        .iter()
                        .any(|k| k.vid == d.vid && k.pid == d.pid)
                    {
                        continue;
                    }
                    if !seen.insert((d.vid, d.pid, 0xFFFF)) {
                        continue;
                    }
                    devices.push(Device {
                        kind: "Neznáme zariadenie",
                        name: format!("{} {}", vendor, d.product.as_deref().unwrap_or("").trim()),
                        id: format!("USB {:04X}:{:04X}", d.vid, d.pid),
                        note: "Zatiaľ nepodporované.",
                        supported: false,
                    });
                }
            }
        }
    }
    for inst in gpus {
        let Some(id) = known::parse_pci(inst) else {
            continue;
        };
        match known::find_gpu(&id) {
            Some(k) => devices.push(Device {
                kind: "Grafika",
                name: k.name.into(),
                id: format!(
                    "PCI {:04X}:{:04X} SUBSYS {:04X}{:04X} · RGB na I2C 0x{:02X}",
                    id.ven, id.dev, id.sub_dev, id.sub_ven, k.rgb_i2c_addr
                ),
                note: k.note,
                supported: true,
            }),
            None => devices.push(Device {
                kind: "Grafika",
                name: "Neznáma grafická karta".into(),
                id: format!(
                    "PCI {:04X}:{:04X} SUBSYS {:04X}{:04X}",
                    id.ven, id.dev, id.sub_dev, id.sub_ven
                ),
                note: "Zatiaľ nepodporované.",
                supported: false,
            }),
        }
    }
    HwReport {
        devices,
        nvapi,
        errors,
    }
}

#[cfg(windows)]
pub fn scan() -> HwReport {
    let mut errors = Vec::new();
    let hid = win::hid().unwrap_or_else(|e| {
        errors.push(format!("USB zariadenia: {e}"));
        Vec::new()
    });
    let gpus = win::display_adapters().unwrap_or_else(|e| {
        errors.push(format!("Grafické karty: {e}"));
        Vec::new()
    });
    summarize(&hid, &gpus, win::nvapi_present(), errors)
}

#[cfg(not(windows))]
pub fn scan() -> HwReport {
    HwReport {
        errors: vec!["Detekcia hardvéru funguje len na Windows.".into()],
        ..Default::default()
    }
}

#[cfg(windows)]
mod win {
    use super::RawHid;
    use windows::core::PCWSTR;
    use windows::Win32::Devices::DeviceAndDriverInstallation::{
        SetupDiDestroyDeviceInfoList, SetupDiEnumDeviceInfo, SetupDiGetClassDevsW,
        SetupDiGetDeviceInstanceIdW, DIGCF_PRESENT, GUID_DEVCLASS_DISPLAY, SP_DEVINFO_DATA,
    };

    pub fn hid() -> Result<Vec<RawHid>, String> {
        let api = hidapi::HidApi::new().map_err(|e| e.to_string())?;
        Ok(api
            .device_list()
            .map(|d| RawHid {
                vid: d.vendor_id(),
                pid: d.product_id(),
                usage_page: d.usage_page(),
                product: d.product_string().map(str::to_owned),
            })
            .collect())
    }

    pub fn display_adapters() -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        unsafe {
            let set = SetupDiGetClassDevsW(
                Some(&GUID_DEVCLASS_DISPLAY as *const _),
                PCWSTR::null(),
                None,
                DIGCF_PRESENT,
            )
            .map_err(|e| e.to_string())?;
            let mut i = 0u32;
            loop {
                let mut info = SP_DEVINFO_DATA {
                    cbSize: std::mem::size_of::<SP_DEVINFO_DATA>() as u32,
                    ..Default::default()
                };
                if SetupDiEnumDeviceInfo(set, i, &mut info).is_err() {
                    break;
                }
                let mut buf = [0u16; 512];
                if SetupDiGetDeviceInstanceIdW(set, &info, Some(&mut buf), None).is_ok() {
                    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
                    out.push(String::from_utf16_lossy(&buf[..end]));
                }
                i += 1;
            }
            let _ = SetupDiDestroyDeviceInfoList(set);
        }
        Ok(out)
    }

    pub fn nvapi_present() -> bool {
        let root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into());
        std::path::Path::new(&root)
            .join(r"System32\nvapi64.dll")
            .exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarizes_alex_pc() {
        let hid = vec![
            RawHid {
                vid: 0x1462,
                pid: 0x7D91,
                usage_page: 1,
                product: Some("MYSTIC LIGHT".into()),
            },
            RawHid {
                vid: 0x0DB0,
                pid: 0xA74B,
                usage_page: 0xFFC0,
                product: Some("USB Audio".into()),
            },
            RawHid {
                vid: 0x046D,
                pid: 0xC54D,
                usage_page: 1,
                product: Some("USB Receiver".into()),
            },
        ];
        let gpus = vec![
            r"ROOT\DISPLAY\0000".to_string(),
            r"PCI\VEN_10DE&DEV_2C02&SUBSYS_418C1458&REV_A1\X".to_string(),
        ];
        let r = summarize(&hid, &gpus, true, vec![]);
        let names: Vec<_> = r.devices.iter().map(|d| d.name.as_str()).collect();
        assert_eq!(
            names,
            vec![
                "MSI MAG Z790 TOMAHAWK WIFI – Mystic Light",
                "GIGABYTE AORUS GeForce RTX 5080 MASTER ICE"
            ]
        );
    }
}
