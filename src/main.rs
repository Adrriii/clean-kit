mod collectors;
mod config;
mod diff;
mod menu;
mod netwatch;
mod output;
mod snapshot;

use anyhow::Result;

fn main() -> Result<()> {
    // Enable ANSI/VT processing on Windows before any colored output
    #[cfg(windows)]
    colored::control::set_virtual_terminal(true).ok();

    let config = config::load(None)?;
    std::fs::create_dir_all(&config.output_dir)?;
    menu::run(config)
}
