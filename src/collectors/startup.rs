use anyhow::Result;
use chrono::{DateTime, Utc};
use std::fs;

use crate::config::expand_env;
use crate::snapshot::FileEntry;

// Startup folders to scan (all files regardless of extension).
// Per-user startup folder and the All Users startup folder.
const STARTUP_PATHS: &[&str] = &[
    r"%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup",
    r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs\StartUp",
];

pub fn collect() -> Result<Vec<FileEntry>> {
    let mut entries = Vec::new();

    for dir in STARTUP_PATHS {
        let expanded = expand_env(dir);
        let path = std::path::Path::new(&expanded);
        if !path.exists() {
            continue;
        }

        let read_dir = match fs::read_dir(path) {
            Ok(r) => r,
            Err(_) => continue,
        };

        for entry in read_dir.flatten() {
            let meta = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            if meta.is_file() {
                let modified = meta.modified().ok().map(DateTime::<Utc>::from);
                entries.push(FileEntry {
                    path: entry.path().to_string_lossy().to_string(),
                    size: meta.len(),
                    modified,
                });
            }
        }
    }

    Ok(entries)
}
