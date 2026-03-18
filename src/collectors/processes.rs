use anyhow::Result;
use serde::Deserialize;

use crate::collectors::{ps_json_to_vec, run_powershell};
use crate::snapshot::ProcessEntry;

#[derive(Deserialize)]
struct PsProcess {
    #[serde(rename = "ProcessId")]
    process_id: Option<serde_json::Value>,
    #[serde(rename = "Name")]
    name: Option<String>,
    #[serde(rename = "ExecutablePath")]
    executable_path: Option<String>,
    #[serde(rename = "CommandLine")]
    command_line: Option<String>,
    #[serde(rename = "ParentProcessId")]
    parent_process_id: Option<serde_json::Value>,
}

pub fn collect() -> Result<Vec<ProcessEntry>> {
    let script = concat!(
        "@(Get-CimInstance Win32_Process | ",
        "Select-Object ProcessId,Name,ExecutablePath,CommandLine,ParentProcessId) | ",
        "ConvertTo-Json -Compress -Depth 2"
    );

    let raw = run_powershell(script)?;
    let ps_list: Vec<PsProcess> = ps_json_to_vec(&raw)?;

    let mut entries: Vec<ProcessEntry> = ps_list
        .into_iter()
        .filter_map(|p| {
            let pid = parse_u32(&p.process_id?)?;
            Some(ProcessEntry {
                pid,
                name: p.name.unwrap_or_default(),
                executable: p.executable_path.unwrap_or_default(),
                command_line: p.command_line.unwrap_or_default(),
                parent_pid: p.parent_process_id.as_ref().and_then(parse_u32),
            })
        })
        .collect();

    entries.sort_by_key(|p| p.pid);
    Ok(entries)
}

fn parse_u32(val: &serde_json::Value) -> Option<u32> {
    match val {
        serde_json::Value::Number(n) => n.as_u64().map(|v| v as u32),
        serde_json::Value::String(s) => s.parse().ok(),
        _ => None,
    }
}
