//! Live graphics card readings through `nvidia-smi`, which ships with every
//! NVIDIA driver. Temperature, clocks, load, power and *why* the card slows
//! down (heat or power limit). AMD and Intel have no such tool built in, so
//! there we honestly say we can't read it yet.

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Sample {
    pub name: String,
    pub temp: f32,
    pub clock: f32,
    pub clock_max: f32,
    pub load: f32,
    pub power: f32,
    pub power_limit: f32,
    pub fan: Option<f32>,
    /// Slowing down because of heat (hardware or software).
    pub thermal: bool,
    /// Holding back at the power limit. Normal under full load.
    pub power_cap: bool,
}

/// Fields asked from nvidia-smi, in this order.
pub const QUERY_NEW: &str = "name,temperature.gpu,clocks.gr,clocks.max.gr,utilization.gpu,power.draw,power.limit,fan.speed,clocks_event_reasons.hw_thermal_slowdown,clocks_event_reasons.sw_thermal_slowdown,clocks_event_reasons.sw_power_cap";
/// Older drivers call them "throttle reasons".
pub const QUERY_OLD: &str = "name,temperature.gpu,clocks.gr,clocks.max.gr,utilization.gpu,power.draw,power.limit,fan.speed,clocks_throttle_reasons.hw_thermal_slowdown,clocks_throttle_reasons.sw_thermal_slowdown,clocks_throttle_reasons.sw_power_cap";

fn num(s: &str) -> Option<f32> {
    s.trim().parse().ok()
}

fn active(s: &str) -> bool {
    let s = s.trim().to_lowercase();
    s == "active" || s == "1" || s == "yes"
}

/// Parses one line of `--format=csv,noheader,nounits` output.
pub fn parse_line(line: &str) -> Option<Sample> {
    let f: Vec<&str> = line.split(',').map(str::trim).collect();
    if f.len() < 11 {
        return None;
    }
    Some(Sample {
        name: f[0].to_string(),
        temp: num(f[1])?,
        clock: num(f[2]).unwrap_or(0.0),
        clock_max: num(f[3]).unwrap_or(0.0),
        load: num(f[4]).unwrap_or(0.0),
        power: num(f[5]).unwrap_or(0.0),
        power_limit: num(f[6]).unwrap_or(0.0),
        fan: num(f[7]),
        thermal: active(f[8]) || active(f[9]),
        power_cap: active(f[10]),
    })
}

/// What a measurement looked like on the graphics card.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Summary {
    pub name: String,
    pub temp_avg: f32,
    pub temp_max: f32,
    pub clock_avg: f32,
    pub clock_max: f32,
    pub load_avg: f32,
    pub power_avg: f32,
    pub power_limit: f32,
    /// Share of samples (0–1) where the card slowed down because of heat.
    pub thermal_share: f32,
    pub power_cap_share: f32,
    pub samples: u32,
}

pub fn summarize(s: &[Sample]) -> Option<Summary> {
    let first = s.first()?;
    let n = s.len() as f32;
    let avg = |f: fn(&Sample) -> f32| s.iter().map(f).sum::<f32>() / n;
    Some(Summary {
        name: first.name.clone(),
        temp_avg: avg(|x| x.temp),
        temp_max: s.iter().map(|x| x.temp).fold(0.0, f32::max),
        clock_avg: avg(|x| x.clock),
        clock_max: first.clock_max,
        load_avg: avg(|x| x.load),
        power_avg: avg(|x| x.power),
        power_limit: first.power_limit,
        thermal_share: s.iter().filter(|x| x.thermal).count() as f32 / n,
        power_cap_share: s.iter().filter(|x| x.power_cap).count() as f32 / n,
        samples: s.len() as u32,
    })
}

#[cfg(windows)]
pub fn sample() -> Option<Sample> {
    use std::os::windows::process::CommandExt;
    let exe = std::path::PathBuf::from(std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into()))
        .join("System32")
        .join("nvidia-smi.exe");
    for q in [QUERY_NEW, QUERY_OLD] {
        let out = std::process::Command::new(&exe)
            .arg(format!("--query-gpu={q}"))
            .arg("--format=csv,noheader,nounits")
            .creation_flags(0x0800_0000)
            .output()
            .ok()?;
        if out.status.success() {
            if let Some(s) = String::from_utf8_lossy(&out.stdout).lines().find_map(parse_line) {
                return Some(s);
            }
        }
    }
    None
}

#[cfg(not(windows))]
pub fn sample() -> Option<Sample> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_nvidia_smi() {
        let s = parse_line("NVIDIA GeForce RTX 2060, 84, 1365, 2100, 99, 158.32, 160.00, 78, Not Active, Active, Not Active").unwrap();
        assert_eq!(s.name, "NVIDIA GeForce RTX 2060");
        assert!((s.temp - 84.0).abs() < 0.01);
        assert!(s.thermal && !s.power_cap);
        assert_eq!(s.fan, Some(78.0));
        // Laptops report [N/A] for fan speed.
        let s = parse_line("NVIDIA GeForce RTX 3060 Laptop GPU, 70, 1500, 2100, 50, 60.1, 80.0, [N/A], Not Active, Not Active, Active").unwrap();
        assert_eq!(s.fan, None);
        assert!(s.power_cap);
        assert!(parse_line("garbage").is_none());

        let sum = summarize(&[
            parse_line("G, 80, 1800, 2100, 99, 150, 160, 70, Not Active, Not Active, Active").unwrap(),
            parse_line("G, 86, 1500, 2100, 99, 150, 160, 80, Not Active, Active, Not Active").unwrap(),
        ])
        .unwrap();
        assert!((sum.temp_max - 86.0).abs() < 0.01);
        assert!((sum.thermal_share - 0.5).abs() < 0.01);
        assert!((sum.clock_avg - 1650.0).abs() < 0.01);
    }
}
