//! Monitors: which graphics card drives them and at what refresh rate,
//! compared to the best refresh rate they offer at the same resolution.
//!
//! Two very common mistakes on new PCs: a 144/165 Hz monitor left at 60 Hz,
//! and the cable plugged into the motherboard instead of the graphics card.

use serde::Serialize;

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Display {
    /// Graphics adapter driving it, e.g. "NVIDIA GeForce RTX 4070".
    pub adapter: String,
    pub primary: bool,
    pub width: u32,
    pub height: u32,
    pub hz: u32,
    /// Highest refresh rate offered at the current resolution.
    pub max_hz: u32,
}

/// Best refresh rate among modes `(w, h, hz)` at the given resolution.
pub fn max_hz_at(modes: &[(u32, u32, u32)], w: u32, h: u32) -> u32 {
    modes
        .iter()
        .filter(|m| m.0 == w && m.1 == h)
        .map(|m| m.2)
        .max()
        .unwrap_or(0)
}

#[cfg(windows)]
pub fn list() -> Vec<Display> {
    use windows::core::PCWSTR;
    use windows::Win32::Graphics::Gdi::{
        EnumDisplayDevicesW, EnumDisplaySettingsW, DEVMODEW, DISPLAY_DEVICEW,
        DISPLAY_DEVICE_ATTACHED_TO_DESKTOP, DISPLAY_DEVICE_PRIMARY_DEVICE,
        ENUM_CURRENT_SETTINGS, ENUM_DISPLAY_SETTINGS_MODE,
    };
    fn wide(b: &[u16]) -> String {
        let end = b.iter().position(|&c| c == 0).unwrap_or(b.len());
        String::from_utf16_lossy(&b[..end]).trim().to_string()
    }
    let mut out = Vec::new();
    unsafe {
        let mut i = 0u32;
        loop {
            let mut dd = DISPLAY_DEVICEW {
                cb: std::mem::size_of::<DISPLAY_DEVICEW>() as u32,
                ..Default::default()
            };
            if !EnumDisplayDevicesW(PCWSTR::null(), i, &mut dd, 0).as_bool() {
                break;
            }
            i += 1;
            if dd.StateFlags.0 & DISPLAY_DEVICE_ATTACHED_TO_DESKTOP.0 == 0 {
                continue;
            }
            let name = PCWSTR(dd.DeviceName.as_ptr());
            let mut cur = DEVMODEW {
                dmSize: std::mem::size_of::<DEVMODEW>() as u16,
                ..Default::default()
            };
            if !EnumDisplaySettingsW(name, ENUM_CURRENT_SETTINGS, &mut cur).as_bool() {
                continue;
            }
            let mut modes = Vec::new();
            let mut k = 0u32;
            loop {
                let mut m = DEVMODEW {
                    dmSize: std::mem::size_of::<DEVMODEW>() as u16,
                    ..Default::default()
                };
                if !EnumDisplaySettingsW(name, ENUM_DISPLAY_SETTINGS_MODE(k), &mut m).as_bool() {
                    break;
                }
                modes.push((m.dmPelsWidth, m.dmPelsHeight, m.dmDisplayFrequency));
                k += 1;
                if k > 2000 {
                    break;
                }
            }
            let (w, h, hz) = (cur.dmPelsWidth, cur.dmPelsHeight, cur.dmDisplayFrequency);
            out.push(Display {
                adapter: wide(&dd.DeviceString),
                primary: dd.StateFlags.0 & DISPLAY_DEVICE_PRIMARY_DEVICE.0 != 0,
                width: w,
                height: h,
                hz,
                max_hz: max_hz_at(&modes, w, h).max(hz),
            });
        }
    }
    out
}

#[cfg(not(windows))]
pub fn list() -> Vec<Display> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_best_refresh_rate_at_resolution() {
        let modes = [(2560, 1440, 60), (2560, 1440, 144), (2560, 1440, 165), (1920, 1080, 240)];
        assert_eq!(max_hz_at(&modes, 2560, 1440), 165);
        assert_eq!(max_hz_at(&modes, 3840, 2160), 0);
    }
}
