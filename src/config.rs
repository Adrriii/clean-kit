use anyhow::Result;
use serde::Deserialize;
use std::path::PathBuf;

pub const DEFAULT_TOML: &str = r#"output_dir = "./snapshots"

[collectors]
processes = true
network   = true
registry  = true
files     = true
tasks     = true
services  = true

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
}

#[derive(Debug, Deserialize)]
pub struct CollectorConfig {
    #[serde(default = "default_true")] pub processes: bool,
    #[serde(default = "default_true")] pub network:   bool,
    #[serde(default = "default_true")] pub registry:  bool,
    #[serde(default = "default_true")] pub files:     bool,
    #[serde(default = "default_true")] pub tasks:     bool,
    #[serde(default = "default_true")] pub services:  bool,
}

impl Default for CollectorConfig {
    fn default() -> Self {
        Self { processes: true, network: true, registry: true, files: true, tasks: true, services: true }
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

pub fn expand_env(s: &str) -> String {
    let mut result = s.to_string();
    for (key, val) in std::env::vars() {
        result = result.replace(&format!("%{}%", key), &val);
        result = result.replace(&format!("%{}%", key.to_uppercase()), &val);
    }
    result
}
