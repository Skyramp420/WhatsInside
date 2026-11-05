//! Things Windows only tells an administrator: SSD wear and temperature,
//! how long boots take, and the restore points. Collected in one elevated
//! run (`--deep out.json`, one UAC prompt) by a PowerShell script.
//!
//! The basic disk list (model, health status, SSD/HDD) works without admin.

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Disk {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    /// "SSD", "HDD", "Unspecified"
    #[serde(default)]
    pub media: String,
    /// "NVMe", "SATA", "USB"…
    #[serde(default)]
    pub bus: String,
    /// "Healthy", "Warning", "Unhealthy"
    #[serde(default)]
    pub health: String,
    #[serde(default)]
    pub size: u64,
    /// Windows is installed on it.
    #[serde(default)]
    pub system: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Reliability {
    #[serde(default)]
    pub id: String,
    /// Percent of rated endurance used (NVMe "Percentage Used").
    pub wear: Option<u32>,
    pub temp: Option<u32>,
    pub temp_max: Option<u32>,
    pub hours: Option<u64>,
    pub read_err: Option<u64>,
    pub write_err: Option<u64>,
    pub read_unc: Option<u64>,
    pub write_unc: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Boot {
    /// ISO time of the boot.
    pub at: String,
    /// Milliseconds until the desktop was usable.
    pub boot: u64,
    pub main: u64,
    pub post: u64,
}

/// Something that made a boot slower, as Windows logs it (events 101–109).
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct SlowStart {
    pub at: String,
    /// "app", "driver", "service", "device", "other"
    pub kind: String,
    pub name: String,
    /// How long it took in total and how much longer than usual (ms).
    pub total: u64,
    pub extra: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct RestorePoint {
    pub at: String,
    pub desc: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Deep {
    #[serde(default)]
    pub disks: Vec<Reliability>,
    #[serde(default)]
    pub boots: Vec<Boot>,
    #[serde(default)]
    pub restore: Vec<RestorePoint>,
    #[serde(default)]
    pub slow: Vec<SlowStart>,
}

/// PowerShell's ConvertTo-Json writes a single object instead of a list of
/// one, and `null` for empty lists; accept all of it.
fn list<T: for<'de> Deserialize<'de>>(v: &serde_json::Value) -> Vec<T> {
    match v {
        serde_json::Value::Array(a) => a.iter().filter_map(|x| serde_json::from_value(x.clone()).ok()).collect(),
        serde_json::Value::Null => Vec::new(),
        other => serde_json::from_value(other.clone()).map(|x| vec![x]).unwrap_or_default(),
    }
}

pub fn parse_disks(json: &str) -> Vec<Disk> {
    serde_json::from_str::<serde_json::Value>(json.trim_start_matches('\u{feff}')).map(|v| list(&v)).unwrap_or_default()
}

pub fn parse_deep(json: &str) -> Option<Deep> {
    let v: serde_json::Value = serde_json::from_str(json.trim_start_matches('\u{feff}').trim()).ok()?;
    Some(Deep {
        disks: list(v.get("disks").unwrap_or(&serde_json::Value::Null)),
        boots: list(v.get("boots").unwrap_or(&serde_json::Value::Null)),
        restore: list(v.get("restore").unwrap_or(&serde_json::Value::Null)),
        slow: list(v.get("slow").unwrap_or(&serde_json::Value::Null)),
    })
}

pub const DISKS_PS: &str = r#"$sys=$null; try { $sys=(Get-Partition -DriveLetter $env:SystemDrive.Substring(0,1) -ErrorAction Stop).DiskNumber } catch {}
ConvertTo-Json -Compress -InputObject @(Get-PhysicalDisk | ForEach-Object { [pscustomobject]@{ id=[string]$_.DeviceId; name=[string]$_.FriendlyName; media=[string]$_.MediaType; bus=[string]$_.BusType; health=[string]$_.HealthStatus; size=[int64]$_.Size; system=([string]$_.DeviceId -eq [string]$sys) } })"#;

pub const DEEP_PS: &str = r#"$ErrorActionPreference='SilentlyContinue'
$d=@(Get-PhysicalDisk | ForEach-Object { $r=$_ | Get-StorageReliabilityCounter; [pscustomobject]@{ id=[string]$_.DeviceId; wear=$r.Wear; temp=$r.Temperature; temp_max=$r.TemperatureMax; hours=$r.PowerOnHours; read_err=$r.ReadErrorsTotal; write_err=$r.WriteErrorsTotal; read_unc=$r.ReadErrorsUncorrected; write_unc=$r.WriteErrorsUncorrected } })
$b=@(Get-WinEvent -FilterHashtable @{LogName='Microsoft-Windows-Diagnostics-Performance/Operational'; Id=100} -MaxEvents 40 | ForEach-Object { $x=[xml]$_.ToXml(); $v=@{}; foreach ($n in $x.Event.EventData.Data) { $v[$n.Name]=$n.'#text' }; [pscustomobject]@{ at=$_.TimeCreated.ToUniversalTime().ToString('o'); boot=[int64]$v['BootTime']; main=[int64]$v['MainPathBootTime']; post=[int64]$v['BootPostBootTime'] } })
$rp=@(Get-ComputerRestorePoint | ForEach-Object { [pscustomobject]@{ at=$_.ConvertToDateTime($_.CreationTime).ToUniversalTime().ToString('o'); desc=[string]$_.Description } })
$kinds=@{101='app';102='driver';103='service';106='other';107='other';108='other';109='device'}
$sl=@(Get-WinEvent -FilterHashtable @{LogName='Microsoft-Windows-Diagnostics-Performance/Operational'; Id=101,102,103,106,107,108,109; StartTime=(Get-Date).AddDays(-60)} -MaxEvents 300 | ForEach-Object { $x=[xml]$_.ToXml(); $v=@{}; foreach ($n in $x.Event.EventData.Data) { $v[$n.Name]=$n.'#text' }; $nm=$v['FriendlyName']; if (-not $nm) { $nm=$v['Name'] }; [pscustomobject]@{ at=$_.TimeCreated.ToUniversalTime().ToString('o'); kind=$kinds[[int]$_.Id]; name=[string]$nm; total=[int64]$v['TotalTime']; extra=[int64]$v['DegradationTime'] } })
[pscustomobject]@{ disks=$d; boots=$b; restore=$rp; slow=$sl } | ConvertTo-Json -Depth 4 -Compress"#;

/// Whether System Protection (restore points) is on for the system drive.
/// Readable without admin.
pub fn protection_on(spp_value: &str, sys_letter: char) -> bool {
    spp_value.to_uppercase().contains(&format!("({}%3A)", sys_letter.to_ascii_uppercase()))
}

#[cfg(windows)]
mod platform {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    pub fn ps(script: &str) -> Result<String, String> {
        let o = std::process::Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command",
                &format!("[Console]::OutputEncoding=[Text.Encoding]::UTF8; {script}")])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| e.to_string())?;
        Ok(String::from_utf8_lossy(&o.stdout).into_owned())
    }

    pub fn spp() -> String {
        let o = std::process::Command::new("reg")
            .args(["query", r"HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\SPP\Clients"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
        o.map(|o| String::from_utf8_lossy(&o.stdout).into_owned()).unwrap_or_default()
    }
}

#[cfg(windows)]
pub fn disks() -> Vec<Disk> {
    platform::ps(DISKS_PS).map(|s| parse_disks(&s)).unwrap_or_default()
}

#[cfg(not(windows))]
pub fn disks() -> Vec<Disk> {
    Vec::new()
}

#[cfg(windows)]
pub fn protection() -> bool {
    let l = std::env::var("SystemDrive").ok().and_then(|s| s.chars().next()).unwrap_or('C');
    protection_on(&platform::spp(), l)
}

#[cfg(not(windows))]
pub fn protection() -> bool {
    false
}

/// Elevated helper: run the deep script and write its JSON.
#[cfg(windows)]
pub fn run_helper(out: &str) -> i32 {
    match platform::ps(DEEP_PS) {
        Ok(json) if parse_deep(&json).is_some() => std::fs::write(out, json).map(|_| 0).unwrap_or(3),
        Ok(_) => 4,
        Err(_) => 5,
    }
}

#[cfg(not(windows))]
pub fn run_helper(_: &str) -> i32 {
    1
}

/// Creates a restore point (admin). Windows allows one per 24 hours by default.
#[cfg(windows)]
pub fn create_restore_point() -> Result<(), String> {
    let out = platform::ps("try { Checkpoint-Computer -Description 'WhatsInside' -RestorePointType 'MODIFY_SETTINGS' -ErrorAction Stop; 'ok' } catch { $_.Exception.Message }")?;
    if out.trim().ends_with("ok") {
        Ok(())
    } else if out.contains("1440") {
        Ok(()) // one already exists from the last 24 hours, that's fine
    } else {
        Err(if out.trim().is_empty() { "Bod obnovenia sa nepodarilo vytvoriť.".into() } else { out.trim().to_string() })
    }
}

#[cfg(not(windows))]
pub fn create_restore_point() -> Result<(), String> {
    Err("Len na Windows.".into())
}

#[cfg(windows)]
pub fn enable_protection() -> Result<(), String> {
    let out = platform::ps("try { Enable-ComputerRestore -Drive \"$env:SystemDrive\\\" -ErrorAction Stop; 'ok' } catch { $_.Exception.Message }")?;
    if out.trim().ends_with("ok") { Ok(()) } else { Err(out.trim().to_string()) }
}

#[cfg(not(windows))]
pub fn enable_protection() -> Result<(), String> {
    Err("Len na Windows.".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_powershell_json() {
        let one = r#"{"id":"0","name":"Samsung SSD 990 PRO 2TB","media":"SSD","bus":"NVMe","health":"Healthy","size":2000398934016,"system":true}"#;
        let d = parse_disks(one);
        assert_eq!(d.len(), 1);
        assert!(d[0].system && d[0].bus == "NVMe");
        let deep = parse_deep(r#"{"disks":{"id":"0","wear":3,"temp":41,"temp_max":70,"hours":5120,"read_err":0,"write_err":0,"read_unc":0,"write_unc":0},"boots":[{"at":"2026-11-01T08:00:00Z","boot":22000,"main":12000,"post":10000},{"at":"2026-11-02T08:00:00Z","boot":48000,"main":13000,"post":35000}],"restore":null}"#).unwrap();
        assert_eq!(deep.disks[0].wear, Some(3));
        assert_eq!(deep.boots.len(), 2);
        assert!(deep.restore.is_empty());
        // Counters PowerShell couldn't read come back as null.
        let n = parse_deep(r#"{"disks":[{"id":"1","wear":null,"temp":null}],"boots":[],"restore":[{"at":"x","desc":"WhatsInside"}]}"#).unwrap();
        assert_eq!(n.disks[0].wear, None);
        assert_eq!(n.restore[0].desc, "WhatsInside");
        let sl = parse_deep(r#"{"disks":[],"boots":[],"restore":[],"slow":{"at":"2026-11-01T08:00:00Z","kind":"app","name":"Discord","total":6200,"extra":4200}}"#).unwrap();
        assert_eq!(sl.slow[0].name, "Discord");
        assert_eq!(sl.slow[0].extra, 4200);
    }

    #[test]
    fn reads_system_protection() {
        let v = "    {09F7EDC5-294E-4180-AF6A-FB0E6A0E9513}    REG_MULTI_SZ    \\\\?\\Volume{abc}\\:(C%3A)";
        assert!(protection_on(v, 'C'));
        assert!(!protection_on(v, 'D'));
        assert!(!protection_on("", 'C'));
    }
}
