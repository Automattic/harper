// harper-desktop/src-tauri/src/mac_broker/accessibility_consent.rs
use std::collections::HashSet;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Simple persistent consent store for accessibility approvals.
/// Stores a JSON object with an "allowed" array of bundle IDs in the user's config directory.
#[derive(Serialize, Deserialize, Default)]
struct ConsentStore {
    allowed: HashSet<String>,
}

fn consent_file_path() -> Option<PathBuf> {
    // Prefer XDG config directory if available, otherwise fallback to HOME.
    if let Ok(cfg) = std::env::var("XDG_CONFIG_HOME") {
        let mut p = PathBuf::from(cfg);
        p.push("harper");
        let _ = fs::create_dir_all(&p);
        p.push("accessibility_consent.json");
        return Some(p);
    }

    if let Ok(home) = std::env::var("HOME") {
        let mut p = PathBuf::from(home);
        p.push(".config");
        p.push("harper");
        let _ = fs::create_dir_all(&p);
        p.push("accessibility_consent.json");
        return Some(p);
    }

    None
}

fn load_store() -> ConsentStore {
    let path = match consent_file_path() {
        Some(p) => p,
        None => return ConsentStore::default(),
    };

    let mut f = match fs::File::open(&path) {
        Ok(mut fh) => {
            let mut s = String::new();
            if fh.read_to_string(&mut s).is_ok() {
                if let Ok(store) = serde_json::from_str::<ConsentStore>(&s) {
                    return store;
                }
            }
            return ConsentStore::default();
        }
        Err(_) => return ConsentStore::default(),
    };
}

fn save_store(store: &ConsentStore) -> Result<(), String> {
    let path = consent_file_path().ok_or_else(|| "no config path".to_string())?;
    let s = serde_json::to_string_pretty(store).map_err(|e| e.to_string())?;
    let mut f = fs::File::create(&path).map_err(|e| e.to_string())?;
    f.write_all(s.as_bytes()).map_err(|e| e.to_string())
}

/// Returns true if the user has explicitly consented to allow accessibility access
/// for the given bundle_id.
pub(crate) fn user_has_consented(bundle_id: &str) -> bool {
    let b = bundle_id.trim();
    if b.is_empty() {
        return false;
    }
    let store = load_store();
    store.allowed.contains(b)
}

/// Sets user consent for a bundle_id. `allow == true` grants consent; false removes it.
pub(crate) fn set_user_consent(bundle_id: &str, allow: bool) -> Result<(), String> {
    let b = bundle_id.trim();
    if b.is_empty() {
        return Err("bundle_id empty".to_string());
    }
    let mut store = load_store();
    if allow {
        store.allowed.insert(b.to_string());
    } else {
        store.allowed.remove(b);
    }
    save_store(&store)
}
