use anyhow::Result;
use serde::Deserialize;

use crate::collectors::{ps_json_to_vec, run_powershell};
use crate::snapshot::ServiceEntry;

#[derive(Deserialize)]
struct PsService {
    #[serde(rename = "Name")]
    name: Option<String>,
    #[serde(rename = "DisplayName")]
    display_name: Option<String>,
    #[serde(rename = "State")]
    state: Option<String>,
    #[serde(rename = "StartMode")]
    start_mode: Option<String>,
    #[serde(rename = "PathName")]
    path_name: Option<String>,
    #[serde(rename = "StartName")]
    start_name: Option<String>,
}

pub fn collect() -> Result<Vec<ServiceEntry>> {
    let script = concat!(
        "@(Get-CimInstance Win32_Service | ",
        "Select-Object Name,DisplayName,State,StartMode,PathName,StartName) | ",
        "ConvertTo-Json -Compress -Depth 2"
    );

    let raw = run_powershell(script)?;
    let ps_list: Vec<PsService> = ps_json_to_vec(&raw)?;

    Ok(ps_list
        .into_iter()
        .map(|s| ServiceEntry {
            name: s.name.unwrap_or_default(),
            display_name: s.display_name.unwrap_or_default(),
            state: s.state.unwrap_or_default(),
            start_mode: s.start_mode.unwrap_or_default(),
            path_name: s.path_name.unwrap_or_default(),
            start_name: s.start_name.unwrap_or_default(),
        })
        .collect())
}
