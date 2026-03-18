use anyhow::Result;
use serde::Deserialize;

use crate::collectors::{ps_json_to_vec, run_powershell};
use crate::snapshot::FirewallRuleEntry;

#[derive(Deserialize)]
struct PsRule {
    #[serde(rename = "Name")]
    name: Option<String>,
    #[serde(rename = "Display")]
    display: Option<String>,
    #[serde(rename = "Direction")]
    direction: Option<String>,
    #[serde(rename = "Action")]
    action: Option<String>,
    #[serde(rename = "Enabled")]
    enabled: Option<String>,
    #[serde(rename = "Profile")]
    profile: Option<String>,
}

pub fn collect() -> Result<Vec<FirewallRuleEntry>> {
    let script = concat!(
        "@(Get-NetFirewallRule | ForEach-Object { ",
        "[PSCustomObject]@{ ",
            "Name      = $_.Name; ",
            "Display   = $_.DisplayName; ",
            "Direction = $_.Direction.ToString(); ",
            "Action    = $_.Action.ToString(); ",
            "Enabled   = $_.Enabled.ToString(); ",
            "Profile   = $_.Profile.ToString() ",
        "} ",
        "}) | ConvertTo-Json -Compress"
    );

    let raw = run_powershell(script)?;
    let ps_list: Vec<PsRule> = ps_json_to_vec(&raw)?;

    Ok(ps_list
        .into_iter()
        .map(|r| FirewallRuleEntry {
            name: r.name.unwrap_or_default(),
            display: r.display.unwrap_or_default(),
            direction: r.direction.unwrap_or_default(),
            action: r.action.unwrap_or_default(),
            enabled: r.enabled.unwrap_or_default(),
            profile: r.profile.unwrap_or_default(),
        })
        .collect())
}
