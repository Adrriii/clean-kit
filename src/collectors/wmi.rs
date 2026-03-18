use anyhow::Result;
use serde::Deserialize;

use crate::collectors::{ps_json_to_vec, run_powershell};
use crate::snapshot::WmiSubscriptionEntry;

#[derive(Deserialize)]
struct PsWmiEntry {
    #[serde(rename = "Kind")]
    kind: Option<String>,
    #[serde(rename = "Name")]
    name: Option<String>,
    #[serde(rename = "Detail")]
    detail: Option<String>,
}

pub fn collect() -> Result<Vec<WmiSubscriptionEntry>> {
    // Enumerate WMI event filters and consumers in root\subscription namespace.
    // Three object types matter:
    //   __EventFilter          — the trigger condition (query)
    //   CommandLineEventConsumer — executes a shell command
    //   ActiveScriptEventConsumer — runs a VBS/JS script
    let script = concat!(
        "$r = @(); ",
        "@(Get-CimInstance -Namespace 'root\\subscription' -ClassName __EventFilter ",
        "  -ErrorAction SilentlyContinue) | ForEach-Object { ",
        "  $r += [PSCustomObject]@{ Kind='Filter'; Name=$_.Name; Detail=$_.Query } ",
        "}; ",
        "@(Get-CimInstance -Namespace 'root\\subscription' -ClassName CommandLineEventConsumer ",
        "  -ErrorAction SilentlyContinue) | ForEach-Object { ",
        "  $r += [PSCustomObject]@{ Kind='CommandLine'; Name=$_.Name; Detail=$_.CommandLineTemplate } ",
        "}; ",
        "@(Get-CimInstance -Namespace 'root\\subscription' -ClassName ActiveScriptEventConsumer ",
        "  -ErrorAction SilentlyContinue) | ForEach-Object { ",
        "  $r += [PSCustomObject]@{ Kind='ActiveScript'; Name=$_.Name; Detail=$_.ScriptText } ",
        "}; ",
        "if ($r.Count -gt 0) { $r | ConvertTo-Json -Compress } else { '[]' }"
    );

    let raw = run_powershell(script)?;
    let ps_list: Vec<PsWmiEntry> = ps_json_to_vec(&raw)?;

    Ok(ps_list
        .into_iter()
        .map(|e| WmiSubscriptionEntry {
            kind: e.kind.unwrap_or_default(),
            name: e.name.unwrap_or_default(),
            detail: e.detail.unwrap_or_default(),
        })
        .collect())
}
