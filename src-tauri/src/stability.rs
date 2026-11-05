//! Why the PC or an app crashed, from the Windows event log.
//!
//! Windows records blue screens, unexpected shutdowns, hardware errors
//! (WHEA), graphics driver resets, disk problems and app crashes, but never
//! shows them to anyone. We read the last 30 days (as a normal user, no
//! admin) and translate them.

use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Event {
    pub provider: String,
    pub id: u32,
    /// Unix seconds.
    pub time: i64,
    /// (name, value); unnamed `<Data>` gets an empty name.
    pub data: Vec<(String, String)>,
}

impl Event {
    fn get(&self, name: &str, pos: usize) -> Option<&str> {
        self.data
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .or_else(|| self.data.get(pos))
            .map(|(_, v)| v.as_str())
            .filter(|v| !v.is_empty())
    }
}

fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let i = tag.find(&format!("{name}="))? + name.len() + 1;
    let q = tag[i..].chars().next()?;
    let rest = &tag[i + 1..];
    Some(&rest[..rest.find(q)?])
}

fn unescape(s: &str) -> String {
    s.replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&apos;", "'").replace("&amp;", "&")
}

/// Days from 1970 for a civil date (proleptic Gregorian).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// "2026-11-01T10:20:30.1234567Z" → unix seconds.
pub fn parse_time(s: &str) -> Option<i64> {
    let n = |a: usize, b: usize| s.get(a..b)?.parse::<i64>().ok();
    let days = days_from_civil(n(0, 4)?, n(5, 7)?, n(8, 10)?);
    Some(days * 86_400 + n(11, 13)? * 3600 + n(14, 16)? * 60 + n(17, 19)?)
}

/// Parses `wevtutil qe … /f:xml` output: `<Event>` elements one after another.
pub fn parse_events(xml: &str) -> Vec<Event> {
    let mut out = Vec::new();
    for chunk in xml.split("<Event ").skip(1) {
        let Some(end) = chunk.find("</Event>") else { continue };
        let ev = &chunk[..end];
        let mut e = Event::default();
        if let Some(i) = ev.find("<Provider ") {
            let tag = &ev[i..ev[i..].find('>').map(|j| i + j).unwrap_or(ev.len())];
            e.provider = attr(tag, "Name").unwrap_or("").to_string();
        }
        if let Some(i) = ev.find("<EventID") {
            let rest = &ev[i..];
            if let (Some(a), Some(b)) = (rest.find('>'), rest.find("</EventID>")) {
                e.id = rest[a + 1..b].trim().parse().unwrap_or(0);
            }
        }
        if let Some(i) = ev.find("<TimeCreated ") {
            let tag = &ev[i..ev[i..].find('>').map(|j| i + j).unwrap_or(ev.len())];
            e.time = attr(tag, "SystemTime").and_then(parse_time).unwrap_or(0);
        }
        let mut rest = ev;
        while let Some(i) = rest.find("<Data") {
            let after = &rest[i + 5..];
            let Some(gt) = after.find('>') else { break };
            let tag = &after[..gt];
            if tag.ends_with('/') {
                e.data.push((attr(tag, "Name").unwrap_or("").to_string(), String::new()));
                rest = &after[gt + 1..];
                continue;
            }
            let body = &after[gt + 1..];
            let Some(close) = body.find("</Data>") else { break };
            e.data.push((attr(tag, "Name").unwrap_or("").to_string(), unescape(body[..close].trim())));
            rest = &body[close + 7..];
        }
        out.push(e);
    }
    out
}

/// Blue screen codes people actually see, in plain words.
pub fn bugcheck(code: u32) -> (&'static str, &'static str) {
    match code {
        0x0A => ("IRQL_NOT_LESS_OR_EQUAL", "Ovládač alebo pamäť siahli, kam nemali. Najčastejšie zlý ovládač, menej často nestabilná RAM (XMP)."),
        0x1A => ("MEMORY_MANAGEMENT", "Problém so správou pamäte. Často nestabilná RAM: skús vypnúť XMP/EXPO alebo otestovať pamäť (Windows Memory Diagnostic, MemTest86)."),
        0x1E => ("KMODE_EXCEPTION_NOT_HANDLED", "Chyba v ovládači. Pozri, čo si naposledy inštaloval alebo aktualizoval."),
        0x3B => ("SYSTEM_SERVICE_EXCEPTION", "Chyba v ovládači alebo systémovej službe. Často grafika, antivírus alebo anti-cheat."),
        0x50 => ("PAGE_FAULT_IN_NONPAGED_AREA", "Ovládač alebo RAM. Ak sa opakuje, otestuj pamäť a aktualizuj ovládače."),
        0x7A => ("KERNEL_DATA_INPAGE_ERROR", "Windows nevedel načítať dáta z disku. Skontroluj zdravie disku a kábel (pri SATA)."),
        0x7E => ("SYSTEM_THREAD_EXCEPTION_NOT_HANDLED", "Chyba v ovládači, ktorý bežal v systéme."),
        0x9F => ("DRIVER_POWER_STATE_FAILURE", "Ovládač zlyhal pri uspávaní alebo prebúdzaní. Často Wi-Fi, USB alebo grafika."),
        0xC2 => ("BAD_POOL_CALLER", "Chyba v ovládači."),
        0x19 => ("BAD_POOL_HEADER", "Poškodená pamäť systému, ovládač alebo RAM."),
        0xD1 => ("DRIVER_IRQL_NOT_LESS_OR_EQUAL", "Chyba v ovládači, často sieťová karta, Wi-Fi alebo grafika."),
        0xEF => ("CRITICAL_PROCESS_DIED", "Zlyhal dôležitý proces Windowsu. Môže to byť disk alebo poškodené systémové súbory (sfc /scannow)."),
        0x101 => ("CLOCK_WATCHDOG_TIMEOUT", "Jadro procesora prestalo odpovedať. Typicky nestabilné pretaktovanie, undervolt alebo napätie procesora."),
        0x109 => ("CRITICAL_STRUCTURE_CORRUPTION", "Poškodenie jadra systému, často ovládač alebo RAM."),
        0x116 => ("VIDEO_TDR_FAILURE", "Grafika prestala odpovedať a nepodarilo sa ju obnoviť. Ovládač grafiky, pretaktovanie grafiky alebo prehrievanie."),
        0x117 => ("VIDEO_TDR_TIMEOUT_DETECTED", "Grafika prestala odpovedať. Ovládač, pretaktovanie alebo teplota grafiky."),
        0x124 => ("WHEA_UNCORRECTABLE_ERROR", "Hardvérová chyba, ktorú nebolo možné opraviť. Najčastejšie procesor: pretaktovanie, napätie alebo starý BIOS."),
        0x133 => ("DPC_WATCHDOG_VIOLATION", "Ovládač alebo firmvér disku nestihol odpovedať. Aktualizuj ovládače a firmvér SSD."),
        0x139 => ("KERNEL_SECURITY_CHECK_FAILURE", "Ovládač alebo RAM poškodili dáta jadra."),
        0x154 => ("UNEXPECTED_STORE_EXCEPTION", "Problém s diskom alebo ovládačom úložiska."),
        0x1000007E => ("SYSTEM_THREAD_EXCEPTION_NOT_HANDLED", "Chyba v ovládači, ktorý bežal v systéme."),
        0x1000008E => ("KERNEL_MODE_EXCEPTION_NOT_HANDLED", "Chyba v ovládači."),
        0x100000EA => ("THREAD_STUCK_IN_DEVICE_DRIVER", "Ovládač grafiky uviazol. Aktualizuj alebo preinštaluj ovládač grafiky."),
        _ => ("", "Neznámy kód. Presnú príčinu ukáže výpis pádu (napríklad v appke WinDbg alebo BlueScreenView)."),
    }
}

/// What a crashing module usually means.
pub fn module_hint(module: &str) -> Option<&'static str> {
    let m = module.to_lowercase();
    if m.starts_with("nvwgf2um") || m.starts_with("nvoglv") || m.starts_with("nvlddmkm") || m.starts_with("nvd3dum") {
        Some("Ovládač grafiky NVIDIA")
    } else if m.starts_with("amdxx") || m.starts_with("atiumd") || m.starts_with("amdvlk") || m.starts_with("atio6axx") || m.starts_with("amdkmdag") {
        Some("Ovládač grafiky AMD")
    } else if m.starts_with("igd") || m.starts_with("igc64") || m.starts_with("igxelp") {
        Some("Ovládač grafiky Intel")
    } else if m.starts_with("d3d12") || m.starts_with("d3d11") || m.starts_with("dxgi") {
        Some("DirectX")
    } else if m.starts_with("vulkan-1") {
        Some("Vulkan")
    } else if m == "kernelbase.dll" || m == "ntdll.dll" || m == "ucrtbase.dll" || m == "msvcrt.dll" {
        Some("Všeobecná chyba appky (nie je jasné, kto ju spôsobil)")
    } else if m.starts_with("clr.dll") || m.starts_with("coreclr") {
        Some(".NET runtime")
    } else {
        None
    }
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct BlueScreen {
    pub time: i64,
    pub code: String,
    pub name: &'static str,
    pub meaning: &'static str,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct AppCrash {
    pub app: String,
    pub count: u32,
    pub hangs: u32,
    pub last: i64,
    pub module: Option<String>,
    pub hint: Option<&'static str>,
}

#[derive(Serialize, Clone, Debug, Default, PartialEq)]
pub struct Report {
    pub days: u32,
    pub blue_screens: Vec<BlueScreen>,
    /// Kernel-Power 41 without a blue screen: power loss, reset button, or a freeze.
    pub sudden_offs: Vec<i64>,
    pub whea_fatal: u32,
    pub whea_corrected: u32,
    /// When hardware errors happened, so tuning can be checked against them.
    pub whea_times: Vec<i64>,
    /// Graphics driver stopped responding and recovered (event 4101).
    pub gpu_resets: Vec<i64>,
    pub gpu_driver: Option<String>,
    pub disk_errors: u32,
    pub disk_resets: u32,
    pub apps: Vec<AppCrash>,
    pub read_ok: bool,
}

fn parse_code(s: &str) -> Option<u32> {
    let t = s.trim();
    let hex = t.split(|c: char| c == ' ' || c == '(').next()?.trim_start_matches("0x").trim_start_matches("0X");
    u32::from_str_radix(hex, 16).ok().or_else(|| t.parse::<u32>().ok())
}

pub fn analyze(system: &[Event], apps: &[Event], days: u32) -> Report {
    let mut r = Report { days, read_ok: true, ..Default::default() };
    for e in system {
        let p = e.provider.to_lowercase();
        match e.id {
            1001 if p.contains("wer-systemerrorreporting") || p.contains("bugcheck") => {
                let code = e.get("param1", 0).and_then(parse_code).unwrap_or(0);
                let (name, meaning) = bugcheck(code);
                r.blue_screens.push(BlueScreen { time: e.time, code: format!("0x{code:08X}"), name, meaning });
            }
            41 if p.contains("kernel-power") => {
                let code = e.get("BugcheckCode", 0).and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
                if code == 0 {
                    r.sudden_offs.push(e.time);
                }
            }
            18 if p.contains("whea") => {
                r.whea_fatal += 1;
                r.whea_times.push(e.time);
            }
            17 | 19 | 47 if p.contains("whea") => {
                r.whea_corrected += 1;
                r.whea_times.push(e.time);
            }
            4101 if p == "display" => {
                r.gpu_resets.push(e.time);
                if let Some(d) = e.get("", 0) {
                    r.gpu_driver = Some(d.to_string());
                }
            }
            7 | 51 | 153 if p == "disk" => r.disk_errors += 1,
            129 if p.contains("stornvme") || p.contains("storahci") || p.contains("iastor") => r.disk_resets += 1,
            _ => {}
        }
    }
    let mut by_app: HashMap<String, (AppCrash, HashMap<String, u32>)> = HashMap::new();
    for e in apps {
        let p = e.provider.to_lowercase();
        let (hang, app, module) = match e.id {
            1000 if p == "application error" => (false, e.get("AppName", 0), e.get("ModuleName", 3)),
            1002 if p == "application hang" => (true, e.get("AppName", 0), None),
            _ => continue,
        };
        let Some(app) = app else { continue };
        let key = app.to_lowercase();
        let entry = by_app.entry(key).or_insert_with(|| {
            (AppCrash { app: app.to_string(), count: 0, hangs: 0, last: 0, module: None, hint: None }, HashMap::new())
        });
        if hang {
            entry.0.hangs += 1;
        } else {
            entry.0.count += 1;
        }
        entry.0.last = entry.0.last.max(e.time);
        if let Some(m) = module {
            *entry.1.entry(m.to_string()).or_default() += 1;
        }
    }
    r.apps = by_app
        .into_values()
        .map(|(mut a, mods)| {
            a.module = mods.into_iter().max_by_key(|(_, n)| *n).map(|(m, _)| m);
            a.hint = a.module.as_deref().and_then(module_hint);
            a
        })
        .collect();
    r.apps.sort_by(|a, b| (b.count + b.hangs).cmp(&(a.count + a.hangs)).then(b.last.cmp(&a.last)));
    r.apps.truncate(15);
    r.blue_screens.sort_by(|a, b| b.time.cmp(&a.time));
    r
}

#[cfg(windows)]
pub fn report() -> Report {
    use std::os::windows::process::CommandExt;
    let days: u32 = 30;
    let ms = days as u64 * 86_400_000;
    let q = |log: &str, ids: &str| -> Option<String> {
        let query = format!("*[System[({ids}) and TimeCreated[timediff(@SystemTime) <= {ms}]]]");
        let o = std::process::Command::new("wevtutil")
            .args(["qe", log, "/rd:true", "/c:500", "/f:xml", &format!("/q:{query}")])
            .creation_flags(0x0800_0000)
            .output()
            .ok()?;
        Some(crate::autostart::decode(&o.stdout))
    };
    let ids = |list: &[u32]| list.iter().map(|i| format!("EventID={i}")).collect::<Vec<_>>().join(" or ");
    let sys = q("System", &ids(&[1001, 41, 17, 18, 19, 47, 4101, 7, 51, 153, 129]));
    let app = q("Application", &ids(&[1000, 1002]));
    let mut r = analyze(
        &parse_events(sys.as_deref().unwrap_or("")),
        &parse_events(app.as_deref().unwrap_or("")),
        days,
    );
    r.read_ok = sys.is_some() && app.is_some();
    r
}

#[cfg(not(windows))]
pub fn report() -> Report {
    Report { days: 30, ..Default::default() }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SYS: &str = r#"<Event xmlns='http://schemas.microsoft.com/win/2004/08/events/event'><System><Provider Name='Microsoft-Windows-WER-SystemErrorReporting' Guid='{x}' EventSourceName='BugCheck'/><EventID Qualifiers='16384'>1001</EventID><TimeCreated SystemTime='2026-11-01T10:20:30.1234567Z'/></System><EventData><Data Name='param1'>0x0000001a (0x0000000000041792, 0xffff, 0x0, 0x0)</Data><Data Name='param2'>C:\WINDOWS\Minidump\110126-1.dmp</Data></EventData></Event><Event xmlns='http://schemas.microsoft.com/win/2004/08/events/event'><System><Provider Name='Microsoft-Windows-Kernel-Power' Guid='{y}'/><EventID>41</EventID><TimeCreated SystemTime='2026-11-02T08:00:00.0Z'/></System><EventData><Data Name='BugcheckCode'>0</Data><Data Name='PowerButtonTimestamp'>0</Data></EventData></Event><Event xmlns='x'><System><Provider Name='Microsoft-Windows-WHEA-Logger'/><EventID>18</EventID><TimeCreated SystemTime='2026-11-03T08:00:00Z'/></System><EventData><Data Name='ErrorSource'>3</Data></EventData></Event><Event xmlns='x'><System><Provider Name='Display'/><EventID Qualifiers='0'>4101</EventID><TimeCreated SystemTime='2026-11-03T09:00:00Z'/></System><EventData><Data>nvlddmkm</Data><Data></Data></EventData></Event>"#;

    const APP: &str = r#"<Event xmlns='x'><System><Provider Name='Application Error'/><EventID Qualifiers='0'>1000</EventID><TimeCreated SystemTime='2026-11-03T20:00:00Z'/></System><EventData><Data Name='AppName'>Cyberpunk2077.exe</Data><Data Name='AppVersion'>3.0</Data><Data Name='AppTimeStamp'>0</Data><Data Name='ModuleName'>nvwgf2umx.dll</Data><Data Name='ModuleVersion'>32.0</Data><Data Name='ModuleTimeStamp'>0</Data><Data Name='ExceptionCode'>c0000005</Data></EventData></Event><Event xmlns='x'><System><Provider Name='Application Error'/><EventID>1000</EventID><TimeCreated SystemTime='2026-11-04T20:00:00Z'/></System><EventData><Data>Cyberpunk2077.exe</Data><Data>3.0</Data><Data>0</Data><Data>nvwgf2umx.dll</Data><Data>32.0</Data><Data>0</Data><Data>c0000005</Data></EventData></Event><Event xmlns='x'><System><Provider Name='Application Hang'/><EventID>1002</EventID><TimeCreated SystemTime='2026-11-04T21:00:00Z'/></System><EventData><Data>explorer.exe</Data></EventData></Event>"#;

    #[test]
    fn reads_event_xml() {
        let e = parse_events(SYS);
        assert_eq!(e.len(), 4);
        assert_eq!(e[0].provider, "Microsoft-Windows-WER-SystemErrorReporting");
        assert_eq!(e[0].id, 1001);
        assert_eq!(e[0].time, parse_time("2026-11-01T10:20:30Z").unwrap());
        assert_eq!(e[3].get("", 0), Some("nvlddmkm"));
        assert_eq!(parse_time("1970-01-02T00:00:00Z"), Some(86_400));
    }

    #[test]
    fn translates_crashes() {
        let r = analyze(&parse_events(SYS), &parse_events(APP), 30);
        assert_eq!(r.blue_screens.len(), 1);
        assert_eq!(r.blue_screens[0].code, "0x0000001A");
        assert_eq!(r.blue_screens[0].name, "MEMORY_MANAGEMENT");
        assert_eq!(r.sudden_offs.len(), 1);
        assert_eq!(r.whea_fatal, 1);
        assert_eq!(r.gpu_resets.len(), 1);
        assert_eq!(r.gpu_driver.as_deref(), Some("nvlddmkm"));
        assert_eq!(r.apps[0].app, "Cyberpunk2077.exe");
        assert_eq!(r.apps[0].count, 2); // named and positional Data both work
        assert_eq!(r.apps[0].hint, Some("Ovládač grafiky NVIDIA"));
        assert_eq!(r.apps[1].hangs, 1);
        assert_eq!(bugcheck(0x124).0, "WHEA_UNCORRECTABLE_ERROR");
    }
}
