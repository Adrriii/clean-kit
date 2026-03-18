use anyhow::Result;
use serde::Deserialize;

use crate::collectors::{ps_json_to_vec, run_powershell};
use crate::snapshot::TaskEntry;

#[derive(Deserialize)]
struct PsTask {
    #[serde(rename = "TaskName")]
    task_name: Option<String>,
    #[serde(rename = "TaskPath")]
    task_path: Option<String>,
    #[serde(rename = "State")]
    state: Option<String>,
    #[serde(rename = "Execute")]
    execute: Option<String>,
    #[serde(rename = "Arguments")]
    arguments: Option<String>,
}

pub fn collect() -> Result<Vec<TaskEntry>> {
    // Flatten Actions[0] into top-level fields to avoid ConvertTo-Json depth issues
    let script = concat!(
        "@(Get-ScheduledTask | ForEach-Object { ",
        "$a = $_.Actions | Select-Object -First 1; ",
        "[PSCustomObject]@{ ",
            "TaskName  = $_.TaskName; ",
            "TaskPath  = $_.TaskPath; ",
            "State     = if ($_.State -ne $null) { $_.State.ToString() } else { 'Unknown' }; ",
            "Execute   = if ($a -and $a.PSObject.Properties['Execute'] -and $a.Execute)   { [string]$a.Execute }   else { '' }; ",
            "Arguments = if ($a -and $a.PSObject.Properties['Arguments'] -and $a.Arguments) { [string]$a.Arguments } else { '' } ",
        "} ",
        "}) | ConvertTo-Json -Compress"
    );

    let raw = run_powershell(script)?;
    let ps_list: Vec<PsTask> = ps_json_to_vec(&raw)?;

    Ok(ps_list
        .into_iter()
        .map(|t| TaskEntry {
            task_name: t.task_name.unwrap_or_default(),
            task_path: t.task_path.unwrap_or_default(),
            state: t.state.unwrap_or_default(),
            execute: t.execute.unwrap_or_default(),
            arguments: t.arguments.unwrap_or_default(),
        })
        .collect())
}
