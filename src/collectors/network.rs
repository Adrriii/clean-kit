use anyhow::Result;
use std::collections::HashMap;

use crate::snapshot::NetworkEntry;

pub fn collect(pid_map: &HashMap<u32, String>) -> Result<Vec<NetworkEntry>> {
    let output = std::process::Command::new("netstat")
        .args(["-ano"])
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to run netstat: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut entries = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        match parts.len() {
            5 => {
                if let Some(e) = parse_tcp_line(&parts, pid_map) {
                    entries.push(e);
                }
            }
            4 => {
                if let Some(e) = parse_udp_line(&parts, pid_map) {
                    entries.push(e);
                }
            }
            _ => {}
        }
    }

    Ok(entries)
}

fn parse_tcp_line(parts: &[&str], pid_map: &HashMap<u32, String>) -> Option<NetworkEntry> {
    if !parts[0].eq_ignore_ascii_case("TCP") {
        return None;
    }
    let (local_addr, local_port) = split_addr(parts[1])?;
    let (remote_addr, remote_port) = split_addr(parts[2])?;
    let state = parts[3].to_string();
    let pid: u32 = parts[4].parse().ok()?;
    Some(NetworkEntry {
        protocol: "TCP".to_string(),
        local_addr,
        local_port,
        remote_addr,
        remote_port,
        state,
        pid,
        process_name: pid_map.get(&pid).cloned(),
    })
}

fn parse_udp_line(parts: &[&str], pid_map: &HashMap<u32, String>) -> Option<NetworkEntry> {
    if !parts[0].eq_ignore_ascii_case("UDP") {
        return None;
    }
    let (local_addr, local_port) = split_addr(parts[1])?;
    let pid: u32 = parts[3].parse().ok()?;
    Some(NetworkEntry {
        protocol: "UDP".to_string(),
        local_addr,
        local_port,
        remote_addr: "*".to_string(),
        remote_port: 0,
        state: String::new(),
        pid,
        process_name: pid_map.get(&pid).cloned(),
    })
}

/// Split "1.2.3.4:1234" or "[::1]:1234" into (ip, port)
fn split_addr(addr: &str) -> Option<(String, u16)> {
    let pos = addr.rfind(':')?;
    let ip = addr[..pos].trim_matches(|c| c == '[' || c == ']').to_string();
    let port: u16 = addr[pos + 1..].parse().ok()?;
    Some((ip, port))
}
