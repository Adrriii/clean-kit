use anyhow::Result;

use crate::snapshot::RegistryEntry;

pub fn collect(keys: &[String]) -> Result<Vec<RegistryEntry>> {
    collect_inner(keys)
}

// ── Windows implementation ────────────────────────────────────────────────────

#[cfg(windows)]
fn collect_inner(keys: &[String]) -> Result<Vec<RegistryEntry>> {
    use winreg::enums::*;
    use winreg::RegKey;

    let mut entries = Vec::new();

    for key_spec in keys {
        let (hive_name, subkey) = match split_hive(key_spec) {
            Some(v) => v,
            None => {
                eprintln!("  [!] Unrecognised registry key spec: {key_spec}");
                continue;
            }
        };

        let hive = match hive_name {
            "HKCU" => RegKey::predef(HKEY_CURRENT_USER),
            "HKLM" => RegKey::predef(HKEY_LOCAL_MACHINE),
            "HKCR" => RegKey::predef(HKEY_CLASSES_ROOT),
            "HKU"  => RegKey::predef(HKEY_USERS),
            other  => { eprintln!("  [!] Unknown hive: {other}"); continue; }
        };

        let opened = match hive.open_subkey_with_flags(subkey, KEY_READ) {
            Ok(k) => k,
            Err(_) => continue, // key may not exist on all systems
        };

        for result in opened.enum_values() {
            match result {
                Ok((name, value)) => {
                    entries.push(RegistryEntry {
                        hive: hive_name.to_string(),
                        key_path: subkey.to_string(),
                        name,
                        data: value.to_string(),
                        kind: format!("{:?}", value.vtype),
                    });
                }
                Err(e) => eprintln!("  [!] Registry value read error: {e}"),
            }
        }
    }

    Ok(entries)
}

#[cfg(not(windows))]
fn collect_inner(_keys: &[String]) -> Result<Vec<RegistryEntry>> {
    Ok(vec![])
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Split "HKCU\Software\..." into ("HKCU", "Software\...")
fn split_hive(key: &str) -> Option<(&str, &str)> {
    let pos = key.find('\\')?;
    Some((&key[..pos], &key[pos + 1..]))
}
