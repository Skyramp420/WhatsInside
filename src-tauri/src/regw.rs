//! Registry writes done by WhatsInside itself instead of `reg.exe`.
//!
//! Windows 11 protects some settings (e.g. Widgets) with a driver (UCPD)
//! that refuses writes from known tools like reg.exe or PowerShell, even as
//! admin. Writing through the API from our own process avoids that, and
//! the error messages are clearer.
//!
//! Commands are still described as `reg add/delete` argument lists, so
//! they stay readable and testable everywhere.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    Set { hive: Hive, path: String, name: String, data: Data },
    DeleteValue { hive: Hive, path: String, name: String },
    DeleteKey { hive: Hive, path: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hive {
    User,
    Machine,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Data {
    Dword(u32),
    Sz(String),
    Binary(Vec<u8>),
}

fn split_key(key: &str) -> Option<(Hive, String)> {
    let (h, rest) = key.split_once('\\')?;
    let hive = match h.to_uppercase().as_str() {
        "HKCU" | "HKEY_CURRENT_USER" => Hive::User,
        "HKLM" | "HKEY_LOCAL_MACHINE" => Hive::Machine,
        _ => return None,
    };
    Some((hive, rest.to_string()))
}

fn hex_bytes(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 {
        return None;
    }
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok()).collect()
}

/// Reads `["add", key, "/v", name | "/ve", "/t", type, "/d", data, "/f"]` or
/// `["delete", key, ("/v", name | "/ve")?, "/f"]`.
pub fn parse(args: &[String]) -> Option<Op> {
    let verb = args.first()?.to_lowercase();
    let (hive, path) = split_key(args.get(1)?)?;
    let flag = |f: &str| args.iter().position(|a| a.eq_ignore_ascii_case(f));
    let value_after = |f: &str| flag(f).and_then(|i| args.get(i + 1)).cloned();
    let name = if flag("/ve").is_some() { Some(String::new()) } else { value_after("/v") };
    match verb.as_str() {
        "add" => {
            let ty = value_after("/t").unwrap_or_else(|| "REG_SZ".into()).to_uppercase();
            let raw = value_after("/d").unwrap_or_default();
            let data = match ty.as_str() {
                "REG_DWORD" => Data::Dword(match raw.strip_prefix("0x") {
                    Some(h) => u32::from_str_radix(h, 16).ok()?,
                    None => raw.parse().ok()?,
                }),
                "REG_BINARY" => Data::Binary(hex_bytes(&raw)?),
                _ => Data::Sz(raw),
            };
            Some(Op::Set { hive, path, name: name?, data })
        }
        "delete" => Some(match name {
            Some(name) => Op::DeleteValue { hive, path, name },
            None => Op::DeleteKey { hive, path },
        }),
        _ => None,
    }
}

#[cfg(windows)]
pub fn run(op: &Op) -> Result<(), String> {
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_SET_VALUE, KEY_WOW64_64KEY, KEY_WRITE, REG_BINARY};
    use winreg::{RegKey, RegValue};
    let root = |h: Hive| RegKey::predef(if h == Hive::Machine { HKEY_LOCAL_MACHINE } else { HKEY_CURRENT_USER });
    let err = |e: std::io::Error| -> String {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            "Windows zápis do tohto nastavenia nedovolil (prístup zamietnutý).".into()
        } else {
            e.to_string()
        }
    };
    match op {
        Op::Set { hive, path, name, data } => {
            let (k, _) = root(*hive).create_subkey_with_flags(path, KEY_WRITE | KEY_WOW64_64KEY).map_err(err)?;
            match data {
                Data::Dword(v) => k.set_value(name, v),
                Data::Sz(s) => k.set_value(name, s),
                Data::Binary(b) => k.set_raw_value(name, &RegValue { vtype: REG_BINARY, bytes: b.clone().into() }),
            }
            .map_err(err)
        }
        Op::DeleteValue { hive, path, name } => {
            let k = root(*hive).open_subkey_with_flags(path, KEY_SET_VALUE | KEY_WOW64_64KEY).map_err(err)?;
            k.delete_value(name).map_err(err)
        }
        Op::DeleteKey { hive, path } => root(*hive).delete_subkey_all(path).map_err(err),
    }
}

#[cfg(not(windows))]
pub fn run(_: &Op) -> Result<(), String> {
    Err("Len na Windows.".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn v(a: &[&str]) -> Vec<String> {
        a.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn reads_reg_style_commands() {
        assert_eq!(
            parse(&v(&["add", r"HKLM\SOFTWARE\Policies\Microsoft\Dsh", "/v", "AllowNewsAndInterests", "/t", "REG_DWORD", "/d", "0", "/f"])),
            Some(Op::Set { hive: Hive::Machine, path: r"SOFTWARE\Policies\Microsoft\Dsh".into(), name: "AllowNewsAndInterests".into(), data: Data::Dword(0) })
        );
        assert_eq!(
            parse(&v(&["add", r"HKCU\Software\Classes\CLSID\{x}\InprocServer32", "/ve", "/t", "REG_SZ", "/d", "", "/f"])),
            Some(Op::Set { hive: Hive::User, path: r"Software\Classes\CLSID\{x}\InprocServer32".into(), name: String::new(), data: Data::Sz(String::new()) })
        );
        assert_eq!(
            parse(&v(&["add", r"HKCU\X", "/v", "Discord", "/t", "REG_BINARY", "/d", "030000000000000000000000", "/f"])),
            Some(Op::Set { hive: Hive::User, path: "X".into(), name: "Discord".into(), data: Data::Binary(vec![3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]) })
        );
        assert_eq!(parse(&v(&["delete", r"HKCU\Software\Classes\CLSID\{x}", "/f"])), Some(Op::DeleteKey { hive: Hive::User, path: r"Software\Classes\CLSID\{x}".into() }));
        assert_eq!(parse(&v(&["delete", r"HKLM\A\B", "/v", "C", "/f"])), Some(Op::DeleteValue { hive: Hive::Machine, path: r"A\B".into(), name: "C".into() }));
        assert!(parse(&v(&["add", r"HKCR\x", "/v", "a", "/d", "1", "/f"])).is_none());
    }

    #[test]
    fn every_tweak_command_parses() {
        for t in crate::tweaks::TWEAKS {
            for on in [true, false] {
                for c in crate::tweaks::commands(t, on) {
                    if c.exe == "reg" {
                        assert!(parse(&c.args).is_some(), "{} {:?}", t.key, c.args);
                    }
                }
            }
        }
    }
}
