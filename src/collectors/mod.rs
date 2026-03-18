pub mod files;
pub mod firewall;
pub mod network;
pub mod processes;
pub mod registry;
pub mod services;
pub mod startup;
pub mod tasks;
pub mod wmi;

use anyhow::Result;
use serde::de::DeserializeOwned;
use std::collections::HashMap;

use crate::config::Config;
use crate::snapshot::{CollectedData, Snapshot};

// ── Collector filter ─────────────────────────────────────────────────────────

pub const COLLECTOR_NAMES: &[&str] =
    &["processes", "network", "registry", "files", "tasks", "services", "firewall", "wmi", "startup"];

pub struct CollectorFilter {
    pub processes: bool,
    pub network: bool,
    pub registry: bool,
    pub files: bool,
    pub tasks: bool,
    pub services: bool,
    pub firewall: bool,
    pub wmi: bool,
    pub startup: bool,
}

impl CollectorFilter {
    /// Build from config file defaults
    pub fn from_config(config: &Config) -> Self {
        let c = &config.collectors;
        Self {
            processes: c.processes,
            network:   c.network,
            registry:  c.registry,
            files:     c.files,
            tasks:     c.tasks,
            services:  c.services,
            firewall:  c.firewall,
            wmi:       c.wmi,
            startup:   c.startup,
        }
    }

    /// Build from a list of enabled collector names (from the interactive selector)
    pub fn from_enabled_list(enabled: &[&str]) -> Self {
        Self {
            processes: enabled.contains(&"processes"),
            network:   enabled.contains(&"network"),
            registry:  enabled.contains(&"registry"),
            files:     enabled.contains(&"files"),
            tasks:     enabled.contains(&"tasks"),
            services:  enabled.contains(&"services"),
            firewall:  enabled.contains(&"firewall"),
            wmi:       enabled.contains(&"wmi"),
            startup:   enabled.contains(&"startup"),
        }
    }

    /// Return the enabled collector names in order
    pub fn enabled_names(&self) -> Vec<&'static str> {
        COLLECTOR_NAMES.iter().copied()
            .filter(|&n| self.is_enabled(n))
            .collect()
    }

    pub fn is_enabled(&self, name: &str) -> bool {
        match name {
            "processes" => self.processes,
            "network"   => self.network,
            "registry"  => self.registry,
            "files"     => self.files,
            "tasks"     => self.tasks,
            "services"  => self.services,
            "firewall"  => self.firewall,
            "wmi"       => self.wmi,
            "startup"   => self.startup,
            _           => false,
        }
    }
}

// ── Orchestrator ─────────────────────────────────────────────────────────────

pub fn run_collectors(config: &Config, filter: &CollectorFilter) -> Result<Snapshot> {
    use std::io::Write as _;
    let mut data = CollectedData::default();

    // Always collect processes when network is enabled (for PID → name resolution)
    let need_processes = filter.processes || filter.network;
    let process_list = if need_processes {
        print!("  [*] processes...");
        let _ = std::io::stdout().flush();
        match processes::collect() {
            Ok(p) => {
                println!(" {} entries", p.len());
                p
            }
            Err(e) => {
                println!();
                eprintln!("  [!] processes failed: {e}");
                vec![]
            }
        }
    } else {
        vec![]
    };

    if filter.processes {
        data.processes = Some(process_list.clone());
    }

    if filter.network {
        print!("  [*] network...");
        let _ = std::io::stdout().flush();
        let pid_map: HashMap<u32, String> =
            process_list.iter().map(|p| (p.pid, p.name.clone())).collect();
        match network::collect(&pid_map) {
            Ok(n) => {
                println!(" {} entries", n.len());
                data.network = Some(n);
            }
            Err(e) => {
                println!();
                eprintln!("  [!] network failed: {e}");
            }
        }
    }

    if filter.registry {
        print!("  [*] registry...");
        let _ = std::io::stdout().flush();
        match registry::collect(&config.registry.keys) {
            Ok(r) => {
                println!(" {} entries", r.len());
                data.registry = Some(r);
            }
            Err(e) => {
                println!();
                eprintln!("  [!] registry failed: {e}");
            }
        }
    }

    if filter.files {
        print!("  [*] files (may take a moment)...");
        let _ = std::io::stdout().flush();
        match files::collect(&config.files) {
            Ok(f) => {
                println!(" {} entries", f.len());
                data.files = Some(f);
            }
            Err(e) => {
                println!();
                eprintln!("  [!] files failed: {e}");
            }
        }
    }

    if filter.tasks {
        print!("  [*] tasks...");
        let _ = std::io::stdout().flush();
        match tasks::collect() {
            Ok(t) => {
                println!(" {} entries", t.len());
                data.tasks = Some(t);
            }
            Err(e) => {
                println!();
                eprintln!("  [!] tasks failed: {e}");
            }
        }
    }

    if filter.services {
        print!("  [*] services...");
        let _ = std::io::stdout().flush();
        match services::collect() {
            Ok(s) => {
                println!(" {} entries", s.len());
                data.services = Some(s);
            }
            Err(e) => {
                println!();
                eprintln!("  [!] services failed: {e}");
            }
        }
    }

    if filter.firewall {
        print!("  [*] firewall...");
        let _ = std::io::stdout().flush();
        match firewall::collect() {
            Ok(f) => {
                println!(" {} entries", f.len());
                data.firewall = Some(f);
            }
            Err(e) => {
                println!();
                eprintln!("  [!] firewall failed: {e}");
            }
        }
    }

    if filter.wmi {
        print!("  [*] wmi...");
        let _ = std::io::stdout().flush();
        match wmi::collect() {
            Ok(w) => {
                println!(" {} entries", w.len());
                data.wmi = Some(w);
            }
            Err(e) => {
                println!();
                eprintln!("  [!] wmi failed: {e}");
            }
        }
    }

    if filter.startup {
        print!("  [*] startup...");
        let _ = std::io::stdout().flush();
        match startup::collect() {
            Ok(s) => {
                println!(" {} entries", s.len());
                data.startup = Some(s);
            }
            Err(e) => {
                println!();
                eprintln!("  [!] startup failed: {e}");
            }
        }
    }

    Ok(Snapshot::new(data))
}

// ── PowerShell helpers ────────────────────────────────────────────────────────

pub fn run_powershell(script: &str) -> Result<String> {
    let output = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to launch powershell.exe: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("PowerShell error: {}", stderr.trim());
    }

    // Strip UTF-8 BOM if present
    let raw = String::from_utf8_lossy(&output.stdout);
    Ok(raw.trim_start_matches('\u{feff}').trim().to_string())
}

/// Parse PowerShell ConvertTo-Json output (handles both single object and array)
pub fn ps_json_to_vec<T: DeserializeOwned>(raw: &str) -> Result<Vec<T>> {
    if raw.is_empty() || raw == "null" {
        return Ok(vec![]);
    }
    if raw.starts_with('[') {
        Ok(serde_json::from_str(raw)?)
    } else if raw.starts_with('{') {
        Ok(vec![serde_json::from_str(raw)?])
    } else {
        anyhow::bail!("Unexpected PowerShell JSON output shape");
    }
}
