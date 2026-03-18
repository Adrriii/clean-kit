use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::Deserialize;

use super::{EndpointStats, ProcessData, WatchState};

// ── PowerShell data shapes ────────────────────────────────────────────────────

#[derive(Deserialize)]
struct PollResult {
    procs: Vec<PsProc>,
    tcp:   Vec<PsTcpConn>,
    udp:   Vec<PsUdpConn>,
}

#[derive(Deserialize)]
struct PsProc {
    #[serde(rename = "Id")]          id:           u32,
    #[serde(rename = "ProcessName")] process_name: String,
}

#[derive(Deserialize)]
struct PsTcpConn {
    #[serde(rename = "RemoteAddress")] remote_address: String,
    #[serde(rename = "RemotePort")]    remote_port:    u16,
    #[serde(rename = "OwningProcess")] owning_process: u32,
}

#[derive(Deserialize)]
struct PsUdpConn {
    #[serde(rename = "RemoteAddress")] remote_address: Option<String>,
    #[serde(rename = "RemotePort")]    remote_port:    Option<u16>,
    #[serde(rename = "OwningProcess")] owning_process: u32,
}

// ── Public helpers ────────────────────────────────────────────────────────────

/// Snapshot current PIDs to build the baseline (processes to ignore).
pub fn get_current_pids() -> anyhow::Result<HashSet<u32>> {
    #[derive(Deserialize)]
    struct P { #[serde(rename = "Id")] id: u32 }

    let raw = crate::collectors::run_powershell(
        "@(Get-Process | Select-Object Id) | ConvertTo-Json -Compress"
    )?;

    let procs: Vec<P> = if raw.starts_with('[') {
        serde_json::from_str(&raw)?
    } else if raw.starts_with('{') {
        vec![serde_json::from_str(&raw)?]
    } else {
        return Ok(HashSet::new());
    };

    Ok(procs.into_iter().map(|p| p.id).collect())
}

/// List names of active network adapters for the interface picker.
pub fn list_interfaces() -> anyhow::Result<Vec<String>> {
    #[derive(Deserialize)]
    struct Adapter { #[serde(rename = "Name")] name: String }

    let raw = crate::collectors::run_powershell(concat!(
        "@(Get-NetAdapter | Where-Object { $_.Status -eq 'Up' } | ",
        "Select-Object Name) | ConvertTo-Json -Compress"
    ))?;

    if raw.is_empty() || raw == "null" || raw == "[]" {
        return Ok(vec!["All interfaces".to_string()]);
    }

    let adapters: Vec<Adapter> = if raw.starts_with('[') {
        serde_json::from_str(&raw)?
    } else if raw.starts_with('{') {
        vec![serde_json::from_str(&raw)?]
    } else {
        return Ok(vec!["All interfaces".to_string()]);
    };

    let mut names: Vec<String> = adapters.into_iter().map(|a| a.name).collect();
    names.insert(0, "All interfaces".to_string());
    Ok(names)
}

// ── Poll loop ─────────────────────────────────────────────────────────────────

pub fn run_poll_loop(state: Arc<Mutex<WatchState>>, stop_rx: std::sync::mpsc::Receiver<()>) {
    loop {
        if stop_rx.try_recv().is_ok() {
            break;
        }

        if let Ok(result) = poll_once() {
            apply_poll_result(&state, result);
        }

        std::thread::sleep(Duration::from_millis(500));
    }
}

fn apply_poll_result(state: &Arc<Mutex<WatchState>>, result: PollResult) {
    let mut s = state.lock().unwrap();
    let now = s.started_at.elapsed();

    let proc_map: HashMap<u32, String> = result.procs.iter()
        .map(|p| (p.id, p.process_name.clone()))
        .collect();

    // TCP connections
    for conn in &result.tcp {
        let pid = conn.owning_process;
        if s.baseline_pids.contains(&pid) || s.ignored_pids.contains(&pid) { continue; }

        // Skip unconnected sockets (remote is 0.0.0.0 / ::)
        if is_zero_addr(&conn.remote_address) || conn.remote_port == 0 { continue; }

        record_event(
            &mut s,
            pid,
            &proc_map,
            conn.remote_address.clone(),
            conn.remote_port,
            "TCP".to_string(),
            now,
        );
    }

    // UDP endpoints
    for conn in &result.udp {
        let pid = conn.owning_process;
        if s.baseline_pids.contains(&pid) || s.ignored_pids.contains(&pid) { continue; }

        let ip   = match &conn.remote_address { Some(a) => a.clone(), None => continue };
        let port = match conn.remote_port      { Some(p) => p,         None => continue };
        if is_zero_addr(&ip) || port == 0 { continue; }

        record_event(&mut s, pid, &proc_map, ip, port, "UDP".to_string(), now);
    }
}

fn record_event(
    s: &mut WatchState,
    pid: u32,
    proc_map: &HashMap<u32, String>,
    remote_ip: String,
    remote_port: u16,
    protocol: String,
    now: std::time::Duration,
) {
    let name = proc_map.get(&pid)
        .cloned()
        .unwrap_or_else(|| format!("PID {}", pid));

    let proc_data = s.processes.entry(pid).or_insert_with(|| ProcessData {
        name: name.clone(),
        pid,
        endpoints:    std::collections::HashMap::new(),
        total_events: 0,
    });
    proc_data.name = name;

    let key = (remote_ip.clone(), remote_port, protocol.clone());
    let ep = proc_data.endpoints.entry(key).or_insert_with(|| EndpointStats {
        remote_ip:      remote_ip.clone(),
        remote_port,
        protocol,
        observed_count: 0,
        _first_seen:    now,
        last_seen:      now,
    });
    ep.observed_count += 1;
    ep.last_seen = now;
    proc_data.total_events += 1;
}

fn is_zero_addr(addr: &str) -> bool {
    addr.is_empty() || addr == "0.0.0.0" || addr == "::" || addr == "*"
}

// ── PowerShell query ──────────────────────────────────────────────────────────

fn poll_once() -> anyhow::Result<PollResult> {
    // Single PowerShell invocation for all three data sources.
    let script = concat!(
        "$r = @{ ",
        "procs = @(Get-Process -ErrorAction SilentlyContinue | Select-Object Id,ProcessName); ",
        "tcp   = @(Get-NetTCPConnection -ErrorAction SilentlyContinue | Select-Object RemoteAddress,RemotePort,OwningProcess); ",
        "udp   = @(Get-NetUDPEndpoint   -ErrorAction SilentlyContinue | Select-Object RemoteAddress,RemotePort,OwningProcess) ",
        "}; $r | ConvertTo-Json -Compress -Depth 3"
    );

    let raw = crate::collectors::run_powershell(script)?;
    Ok(serde_json::from_str(&raw)?)
}
