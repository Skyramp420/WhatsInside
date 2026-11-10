//! Watches for hardware trouble while the user is tuning (XMP, undervolt,
//! overclock).
//!
//! Corrected hardware errors are silent: Windows logs them and carries on.
//! While a tuning step is being tested we look at the event log once a
//! minute and say so right away. This is the only thing in WhatsInside that
//! runs in the background, and only between "Začínam ladiť" and "Ukončiť".

use crate::stability::Event;
use serde::Serialize;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Alert {
    pub time: i64,
    /// "bluescreen", "whea_fatal", "whea", "gpu_reset"
    pub kind: &'static str,
}

/// Events at or after `since` that mean the current settings aren't stable.
pub fn pick(events: &[Event], since: i64) -> Vec<Alert> {
    let mut out: Vec<Alert> = events
        .iter()
        .filter(|e| e.time >= since)
        .filter_map(|e| {
            let p = e.provider.to_lowercase();
            let kind = match e.id {
                1001 if p.contains("wer-systemerrorreporting") || p.contains("bugcheck") => "bluescreen",
                18 if p.contains("whea") => "whea_fatal",
                17 | 19 | 47 if p.contains("whea") => "whea",
                4101 if p == "display" => "gpu_reset",
                _ => return None,
            };
            Some(Alert { time: e.time, kind })
        })
        .collect();
    out.sort_by_key(|a| a.time);
    out
}

/// What to tell the user about a batch of alerts: (title, text).
pub fn message(alerts: &[Alert]) -> (&'static str, String) {
    let has = |k: &str| alerts.iter().any(|a| a.kind == k);
    if has("bluescreen") || has("whea_fatal") {
        ("Ladenie: vážna chyba hardvéru", "Windows zaznamenal pád alebo neopraviteľnú chybu. Posledný krok vráť späť.".into())
    } else if has("whea") {
        let n = alerts.iter().filter(|a| a.kind == "whea").count();
        (
            "Ladenie: chyba hardvéru",
            format!("Windows práve opravil {n}× chybu procesora alebo pamäte (WHEA). Nastavenie je na hrane, uber o krok."),
        )
    } else {
        ("Ladenie: reset grafiky", "Ovládač grafiky prestal odpovedať a obnovil sa. Undervolt alebo takt grafiky je príliš agresívny.".into())
    }
}

static SINCE: AtomicI64 = AtomicI64::new(0);
static SEEN: AtomicI64 = AtomicI64::new(0);
static RUNNING: AtomicBool = AtomicBool::new(false);
static HINTED: AtomicBool = AtomicBool::new(false);

pub fn active() -> bool {
    SINCE.load(Ordering::Relaxed) != 0
}

/// True the first time it's asked, so the "still running in the tray" note shows once.
pub fn first_hide() -> bool {
    !HINTED.swap(true, Ordering::Relaxed)
}

fn now() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

pub fn stop() {
    SINCE.store(0, Ordering::Relaxed);
}

/// Starts watching. `on_alert` is called from the watcher thread with new events.
pub fn start(since: i64, on_alert: impl Fn(Vec<Alert>) + Send + 'static) {
    SINCE.store(since.max(1), Ordering::Relaxed);
    SEEN.store(now(), Ordering::Relaxed);
    if RUNNING.swap(true, Ordering::Relaxed) {
        return;
    }
    std::thread::spawn(move || loop {
        for _ in 0..30 {
            std::thread::sleep(std::time::Duration::from_secs(2));
            if !active() {
                RUNNING.store(false, Ordering::Relaxed);
                return;
            }
        }
        let from = SEEN.load(Ordering::Relaxed) + 1;
        let alerts = since_events(from);
        if let Some(last) = alerts.last() {
            SEEN.store(last.time, Ordering::Relaxed);
            on_alert(alerts);
        }
    });
}

#[cfg(windows)]
fn since_events(from: i64) -> Vec<Alert> {
    use std::os::windows::process::CommandExt;
    let ms = ((now() - from).clamp(60, 30 * 86_400) + 30) * 1000;
    let query = format!(
        "*[System[(EventID=1001 or EventID=17 or EventID=18 or EventID=19 or EventID=47 or EventID=4101) and TimeCreated[timediff(@SystemTime) <= {ms}]]]"
    );
    let Ok(o) = std::process::Command::new("wevtutil")
        .args(["qe", "System", "/rd:true", "/c:50", "/f:xml", &format!("/q:{query}")])
        .creation_flags(0x0800_0000)
        .output()
    else {
        return Vec::new();
    };
    pick(&crate::stability::parse_events(&crate::autostart::decode(&o.stdout)), from)
}

#[cfg(not(windows))]
fn since_events(_: i64) -> Vec<Alert> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ev(provider: &str, id: u32, time: i64) -> Event {
        Event { provider: provider.into(), id, time, data: Vec::new() }
    }

    #[test]
    fn picks_only_new_hardware_trouble() {
        let events = [
            ev("Microsoft-Windows-WHEA-Logger", 19, 200),
            ev("Microsoft-Windows-WHEA-Logger", 19, 50),
            ev("Display", 4101, 150),
            ev("Service Control Manager", 7000, 300),
            ev("Microsoft-Windows-WER-SystemErrorReporting", 1001, 400),
        ];
        let a = pick(&events, 100);
        assert_eq!(a.iter().map(|x| x.kind).collect::<Vec<_>>(), ["gpu_reset", "whea", "bluescreen"]);
        assert!(message(&a).0.contains("vážna"));
        assert!(message(&a[1..2]).1.contains("1×"));
        assert!(message(&a[..1]).0.contains("grafiky"));
    }
}
