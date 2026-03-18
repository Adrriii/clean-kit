use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const SNAPSHOT_VERSION: u8 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessEntry {
    pub pid: u32,
    pub name: String,
    pub executable: String,
    pub command_line: String,
    pub parent_pid: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEntry {
    pub protocol: String,
    pub local_addr: String,
    pub local_port: u16,
    pub remote_addr: String,
    pub remote_port: u16,
    pub state: String,
    pub pid: u32,
    pub process_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub hive: String,
    pub key_path: String,
    pub name: String,
    pub data: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub size: u64,
    pub modified: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskEntry {
    pub task_name: String,
    pub task_path: String,
    pub state: String,
    pub execute: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEntry {
    pub name: String,
    pub display_name: String,
    pub state: String,
    pub start_mode: String,
    pub path_name: String,
    pub start_name: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CollectedData {
    pub processes: Option<Vec<ProcessEntry>>,
    pub network: Option<Vec<NetworkEntry>>,
    pub registry: Option<Vec<RegistryEntry>>,
    pub files: Option<Vec<FileEntry>>,
    pub tasks: Option<Vec<TaskEntry>>,
    pub services: Option<Vec<ServiceEntry>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub version: u8,
    pub timestamp: DateTime<Utc>,
    pub hostname: String,
    pub data: CollectedData,
}

impl Snapshot {
    pub fn new(data: CollectedData) -> Self {
        let hostname = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "unknown".to_string());
        Snapshot {
            version: SNAPSHOT_VERSION,
            timestamp: Utc::now(),
            hostname,
            data,
        }
    }

    pub fn save(&self, output_dir: &Path, name: &str) -> Result<()> {
        let path = output_dir.join(format!("{}.json", name));
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
        println!("  → {}", path.display());
        Ok(())
    }

    pub fn load(output_dir: &Path, name: &str) -> Result<Self> {
        let path = output_dir.join(format!("{}.json", name));
        let json = std::fs::read_to_string(&path)
            .map_err(|_| anyhow!("Snapshot not found at {} — run 'ck before' first", path.display()))?;
        Ok(serde_json::from_str(&json)?)
    }
}
