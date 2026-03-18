use anyhow::Result;
use serde::Deserialize;
use std::path::PathBuf;

pub const DEFAULT_TOML: &str = r#"output_dir = "./snapshots"

[netwatch]
interface = "All interfaces"

[collectors]
processes = true
network   = true
registry  = true
files     = true
tasks     = true
services  = true
firewall  = true
wmi       = true
startup   = true

[registry]
keys = [
    "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
    "HKLM\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
    "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\RunOnce",
    "HKLM\\Software\\Microsoft\\Windows\\CurrentVersion\\RunOnce",
    "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\RunServices",
    "HKLM\\Software\\Microsoft\\Windows\\CurrentVersion\\RunServices",
]

[files]
paths = [
    "%APPDATA%",
    "%LOCALAPPDATA%",
    "%TEMP%",
    "%USERPROFILE%\\Downloads",
    "%USERPROFILE%\\Desktop",
    "C:\\ProgramData",
]
extensions = ["exe", "dll", "bat", "ps1", "vbs", "js", "hta", "scr", "com"]
max_depth  = 6
"#;

fn default_output_dir() -> PathBuf { PathBuf::from("./snapshots") }
fn default_true() -> bool { true }
fn default_interface() -> String { "All interfaces".to_string() }

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_output_dir")]
    pub output_dir: PathBuf,
    #[serde(default)]
    pub collectors: CollectorConfig,
    #[serde(default)]
    pub registry: RegistryConfig,
    #[serde(default)]
    pub files: FilesConfig,
    #[serde(default)]
    pub netwatch: NetwatchConfig,
}

#[derive(Debug, Deserialize)]
pub struct NetwatchConfig {
    #[serde(default = "default_interface")]
    pub interface: String,
}

impl Default for NetwatchConfig {
    fn default() -> Self { Self { interface: default_interface() } }
}

#[derive(Debug, Deserialize)]
pub struct CollectorConfig {
    #[serde(default = "default_true")] pub processes: bool,
    #[serde(default = "default_true")] pub network:   bool,
    #[serde(default = "default_true")] pub registry:  bool,
    #[serde(default = "default_true")] pub files:     bool,
    #[serde(default = "default_true")] pub tasks:     bool,
    #[serde(default = "default_true")] pub services:  bool,
    #[serde(default = "default_true")] pub firewall:  bool,
    #[serde(default = "default_true")] pub wmi:       bool,
    #[serde(default = "default_true")] pub startup:   bool,
}

impl Default for CollectorConfig {
    fn default() -> Self {
        Self {
            processes: true, network: true, registry: true, files: true,
            tasks: true, services: true, firewall: true, wmi: true, startup: true,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RegistryConfig {
    #[serde(default = "default_registry_keys")]
    pub keys: Vec<String>,
}

fn default_registry_keys() -> Vec<String> {
    vec![
        r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run".to_string(),
        r"HKLM\Software\Microsoft\Windows\CurrentVersion\Run".to_string(),
        r"HKCU\Software\Microsoft\Windows\CurrentVersion\RunOnce".to_string(),
        r"HKLM\Software\Microsoft\Windows\CurrentVersion\RunOnce".to_string(),
    ]
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self { keys: default_registry_keys() }
    }
}

#[derive(Debug, Deserialize)]
pub struct FilesConfig {
    #[serde(default = "default_scan_paths")]
    pub paths: Vec<String>,
    #[serde(default = "default_extensions")]
    pub extensions: Vec<String>,
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,
}

fn default_scan_paths() -> Vec<String> {
    vec![
        "%APPDATA%".to_string(),
        "%LOCALAPPDATA%".to_string(),
        "%TEMP%".to_string(),
        "%USERPROFILE%\\Downloads".to_string(),
        "%USERPROFILE%\\Desktop".to_string(),
        "C:\\ProgramData".to_string(),
    ]
}
fn default_extensions() -> Vec<String> {
    vec!["exe".to_string(), "dll".to_string(), "bat".to_string(), "ps1".to_string(),
         "vbs".to_string(), "js".to_string(), "hta".to_string(), "scr".to_string(), "com".to_string()]
}
fn default_max_depth() -> usize { 6 }

impl Default for FilesConfig {
    fn default() -> Self {
        Self { paths: default_scan_paths(), extensions: default_extensions(), max_depth: default_max_depth() }
    }
}

pub fn load(path: Option<&std::path::Path>) -> Result<Config> {
    let config_path = path.unwrap_or(std::path::Path::new("cleankit.toml"));
    if config_path.exists() {
        let content = std::fs::read_to_string(config_path)?;
        Ok(toml::from_str(&content)?)
    } else {
        Ok(toml::from_str(DEFAULT_TOML)?)
    }
}

/// Persist the chosen network interface back into cleankit.toml.
/// Reads the file as a TOML value, updates `netwatch.interface`, and writes it back.
/// If no config file exists, creates one from DEFAULT_TOML first.
pub fn save_netwatch_interface(interface: &str) -> Result<()> {
    let config_path = std::path::Path::new("cleankit.toml");

    let content = if config_path.exists() {
        std::fs::read_to_string(config_path)?
    } else {
        DEFAULT_TOML.to_string()
    };

    let mut value: toml::Value = toml::from_str(&content)?;

    if let Some(table) = value.as_table_mut() {
        let netwatch = table
            .entry("netwatch")
            .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
        if let Some(nt) = netwatch.as_table_mut() {
            nt.insert("interface".to_string(), toml::Value::String(interface.to_string()));
        }
    }

    std::fs::write(config_path, toml::to_string_pretty(&value)?)?;
    Ok(())
}

pub fn expand_env(s: &str) -> String {
    let mut result = s.to_string();
    for (key, val) in std::env::vars() {
        result = result.replace(&format!("%{}%", key), &val);
        result = result.replace(&format!("%{}%", key.to_uppercase()), &val);
    }
    result
}
