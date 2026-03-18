use anyhow::Result;
use chrono::{DateTime, Utc};
use std::path::Path;
use walkdir::WalkDir;

use crate::config::{expand_env, FilesConfig};
use crate::snapshot::FileEntry;

pub fn collect(config: &FilesConfig) -> Result<Vec<FileEntry>> {
    let extensions: Vec<String> = config
        .extensions
        .iter()
        .map(|e| e.trim_start_matches('.').to_lowercase())
        .collect();

    let mut entries = Vec::new();

    for raw_path in &config.paths {
        let expanded = expand_env(raw_path);
        let path = Path::new(&expanded);

        if !path.exists() {
            continue;
        }

        for entry in WalkDir::new(path)
            .max_depth(config.max_depth)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }

            let p = entry.path();
            let ext = p
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase())
                .unwrap_or_default();

            if extensions.is_empty() || extensions.contains(&ext) {
                let (size, modified) = match entry.metadata() {
                    Ok(m) => {
                        let mtime: Option<DateTime<Utc>> = m
                            .modified()
                            .ok()
                            .map(|t| t.into());
                        (m.len(), mtime)
                    }
                    Err(_) => (0, None),
                };

                entries.push(FileEntry {
                    path: p.to_string_lossy().to_string(),
                    size,
                    modified,
                });
            }
        }
    }

    Ok(entries)
}
