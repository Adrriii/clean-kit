mod collectors;
mod config;
mod diff;
mod menu;
mod output;
mod snapshot;

use anyhow::Result;

fn main() -> Result<()> {
    let config = config::load(None)?;
    std::fs::create_dir_all(&config.output_dir)?;
    menu::run(config)
}
