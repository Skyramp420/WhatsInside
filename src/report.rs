//! What a scan found, and how we print it.

use crate::known::{self, HidRole};
use serde::Serialize;

#[derive(Serialize, Default)]
pub struct Report {
    pub version: &'static str,
    pub hid: Vec<HidDevice>,
    pub gpus: Vec<Gpu>,
    pub nvapi_present: bool,
    pub processes: Vec<ConflictProcess>,
    pub autostart: Vec<AutostartEntry>,
    pub errors: Vec<String>,
}

#[derive(Serialize, Clone)]
pub struct HidDevice {
    pub vid: u16,
    pub pid: u16,
    pub usage_page: u16,
    pub usage: u16,
    pub interface: i32,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct Gpu {
    pub instance_id: String,
}

#[derive(Serialize, Clone)]
pub struct ConflictProcess {
    pub app: &'static str,
    pub controls: &'static str,
    pub exe: String,
    pub pid: u32,
    pub path: Option<String>,
    pub ram_bytes: Option<u64>,
}

#[derive(Serialize, Clone)]
pub struct AutostartEntry {
    pub app: &'static str,
    pub location: String,
    pub name: String,
    pub command: String,
}

fn mb(bytes: u64) -> String {
    format!("{:.0} MB", bytes as f64 / 1_048_576.0)
}

impl Report {
    pub fn render(&self) -> String {
        let mut o = String::new();
        let mut line = |s: &str| {
            o.push_str(s);
            o.push('\n');
        };

        line(&format!(
            "WhatsInside v{}  (iba čítanie, nič sa nezapisuje)",
            self.version
        ));
        line("");

        // --- RGB controllers -------------------------------------------
        line("■ RGB radiče (USB)");
        let mut seen_any = false;
        for d in &self.hid {
            match known::find_hid(d.vid, d.pid, d.usage_page) {
                Some(k) if k.role == HidRole::Rgb => {
                    seen_any = true;
                    line(&format!("  ✔ {}  [{:04X}:{:04X}]", k.name, d.vid, d.pid));
                    line(&format!("      {}", k.note));
                }
                Some(_) => {}
                None => {
                    // Unknown device from a vendor that makes RGB gear: worth a look.
                    if let Some(vendor) = known::rgb_vendor(d.vid) {
                        if known::KNOWN_HID
                            .iter()
                            .any(|k| k.vid == d.vid && k.pid == d.pid)
                        {
                            continue; // known device, just another interface
                        }
                        seen_any = true;
                        line(&format!(
                            "  ? Neznáme zariadenie od {}  [{:04X}:{:04X}] page {:04X}  {}",
                            vendor,
                            d.vid,
                            d.pid,
                            d.usage_page,
                            d.product.as_deref().unwrap_or("")
                        ));
                    }
                }
            }
        }
        if !seen_any {
            line("  – nenašiel som žiadny RGB radič");
        }
        line("");

        // --- GPUs ---------------------------------------------------------
        line("■ Grafika");
        let mut gpu_any = false;
        for g in &self.gpus {
            let Some(id) = known::parse_pci(&g.instance_id) else {
                continue;
            };
            gpu_any = true;
            match known::find_gpu(&id) {
                Some(k) => {
                    line(&format!("  ✔ {}", k.name));
                    line(&format!(
                        "      RGB radič na I2C adrese 0x{:02X}. {}",
                        k.rgb_i2c_addr, k.note
                    ));
                }
                None => line(&format!(
                    "  ? Neznáma karta  VEN {:04X} DEV {:04X} SUBSYS {:04X}{:04X}",
                    id.ven, id.dev, id.sub_dev, id.sub_ven
                )),
            }
        }
        if !gpu_any {
            line("  – nenašiel som fyzickú grafiku");
        }
        line(&format!(
            "  NVIDIA NVAPI: {}",
            if self.nvapi_present {
                "k dispozícii (cez neho pôjde RGB grafiky)"
            } else {
                "nenájdené"
            }
        ));
        line("");

        // --- Conflicting software ------------------------------------------
        line("■ Appky, ktoré sa môžu biť o RGB");
        if self.processes.is_empty() && self.autostart.is_empty() {
            line("  ✔ nič nebeží a nič sa nespúšťa pri štarte");
        } else {
            let mut apps: Vec<&str> = self
                .processes
                .iter()
                .map(|p| p.app)
                .chain(self.autostart.iter().map(|a| a.app))
                .collect();
            apps.sort_unstable();
            apps.dedup();
            for app in apps {
                let procs: Vec<_> = self.processes.iter().filter(|p| p.app == app).collect();
                let ram: u64 = procs.iter().filter_map(|p| p.ram_bytes).sum();
                let controls = procs
                    .first()
                    .map(|p| p.controls)
                    .or_else(|| {
                        known::CONFLICT_APPS
                            .iter()
                            .find(|a| a.name == app)
                            .map(|a| a.controls)
                    })
                    .unwrap_or("");
                line(&format!("  ⚠ {}  (ovláda {})", app, controls));
                if !procs.is_empty() {
                    let ram_txt = if ram > 0 {
                        format!(", spolu {}", mb(ram))
                    } else {
                        String::new()
                    };
                    line(&format!("      beží {} procesov{}:", procs.len(), ram_txt));
                    for p in procs {
                        let r = p.ram_bytes.map(mb).unwrap_or_else(|| "? MB".into());
                        line(&format!("        {} (PID {}, {})", p.exe, p.pid, r));
                    }
                }
                for a in self.autostart.iter().filter(|a| a.app == app) {
                    line(&format!(
                        "      spúšťa sa pri štarte: {} → {}",
                        a.location, a.name
                    ));
                }
            }
            line("");
            line("  Kým tieto appky bežia, naša appka do hardvéru zapisovať nebude.");
        }

        if !self.errors.is_empty() {
            line("");
            line("■ Čo sa nepodarilo zistiť");
            for e in &self.errors {
                line(&format!("  · {e}"));
            }
        }
        o
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_alex_setup() {
        let r = Report {
            version: "0.1.0",
            hid: vec![HidDevice {
                vid: 0x1462,
                pid: 0x7D91,
                usage_page: 0x0001,
                usage: 0,
                interface: 0,
                manufacturer: None,
                product: None,
            }],
            gpus: vec![Gpu {
                instance_id: r"PCI\VEN_10DE&DEV_2C02&SUBSYS_418C1458&REV_A1\X".into(),
            }],
            nvapi_present: true,
            processes: vec![ConflictProcess {
                app: "MSI Center / Mystic Light",
                controls: "RGB dosky a ARGB headerov",
                exe: "LEDKeeper2.exe".into(),
                pid: 42,
                path: None,
                ram_bytes: Some(50 * 1_048_576),
            }],
            autostart: vec![],
            errors: vec![],
        };
        let out = r.render();
        assert!(out.contains("MAG Z790 TOMAHAWK"));
        assert!(out.contains("MASTER ICE"));
        assert!(out.contains("50 MB"));
    }
}
