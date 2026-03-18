use anyhow::Result;
use colored::Colorize;
use crossterm::{cursor::MoveTo, execute, terminal::{Clear, ClearType}};
use inquire::{InquireError, Select};

use crate::config;
use super::{poll, view, WatchSession};

// ── Entry point (called from main menu) ──────────────────────────────────────

/// Run the Network Watcher submenu. Takes ownership of an existing session (if
/// one is running) so it can be stopped or passed to the live view.
pub fn run(session: &mut Option<WatchSession>, cfg: &config::Config) -> Result<()> {
    loop {
        clear();
        print_header(session.as_ref(), &cfg.netwatch.interface);

        let items = menu_items(session.is_some());
        let choice = Select::new("", items)
            .without_help_message()
            .prompt();

        match choice {
            Ok(Item::Back) => break,
            Ok(item) => {
                if let Err(e) = dispatch(item, session, cfg) {
                    println!("\n  {}: {}", "Error".red().bold(), e);
                    wait_for_enter();
                }
            }
            Err(InquireError::OperationCanceled | InquireError::OperationInterrupted) => break,
            Err(e) => return Err(e.into()),
        }
    }

    Ok(())
}

// ── Menu items ────────────────────────────────────────────────────────────────

#[derive(Clone)]
enum Item {
    Start,
    LiveView,
    Stop,
    Configure,
    Back,
}

impl std::fmt::Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Item::Start     => write!(f, "Start watching"),
            Item::LiveView  => write!(f, "Live view"),
            Item::Stop      => write!(f, "Stop watching"),
            Item::Configure => write!(f, "Configure network interface"),
            Item::Back      => write!(f, "Back"),
        }
    }
}

fn menu_items(running: bool) -> Vec<Item> {
    let mut items = Vec::new();
    if running {
        items.push(Item::LiveView);
        items.push(Item::Stop);
    } else {
        items.push(Item::Start);
    }
    items.push(Item::Configure);
    items.push(Item::Back);
    items
}

// ── Dispatch ──────────────────────────────────────────────────────────────────

fn dispatch(item: Item, session: &mut Option<WatchSession>, cfg: &config::Config) -> Result<()> {
    match item {
        Item::Start => {
            let iface = cfg.netwatch.interface.clone();
            println!("\n  {}", "[*] Snapshotting baseline process list...".cyan().bold());
            let s = WatchSession::start(iface)?;
            *session = Some(s);
            // Immediately drop into live view
            if let Some(s) = session.as_ref() {
                if let view::ViewResult::Stopped = view::run(s)? {
                    *session = None;
                }
            }
        }

        Item::LiveView => {
            if let Some(s) = session.as_ref() {
                match view::run(s)? {
                    view::ViewResult::Left    => {}
                    view::ViewResult::Stopped => {
                        *session = None;
                    }
                }
            }
        }

        Item::Stop => {
            if session.take().is_some() {
                println!("\n  {}", "[✓] Network watch stopped.".yellow());
                wait_for_enter();
            }
        }

        Item::Configure => {
            configure_interface(session, &cfg.netwatch.interface)?;
        }

        Item::Back => {}
    }

    Ok(())
}

// ── Interface configuration ───────────────────────────────────────────────────

fn configure_interface(session: &mut Option<WatchSession>, current: &str) -> Result<()> {
    clear();
    println!("{}", "  Configure network interface".bold().white());
    println!("{}", "  Saved to cleankit.toml. Applies on next watch start.\n".dimmed());

    let interfaces = match poll::list_interfaces() {
        Ok(i) => i,
        Err(_) => vec!["All interfaces".to_string()],
    };

    let default_idx = interfaces.iter().position(|i| i == current).unwrap_or(0);

    let choice = match Select::new("Interface:", interfaces)
        .with_starting_cursor(default_idx)
        .without_help_message()
        .prompt()
    {
        Ok(c) => c,
        Err(InquireError::OperationCanceled | InquireError::OperationInterrupted) => return Ok(()),
        Err(e) => return Err(e.into()),
    };

    config::save_netwatch_interface(&choice)?;

    // Update a running session's label live
    if let Some(s) = session.as_ref() {
        s.state.lock().unwrap().interface = choice.clone();
    }

    println!("\n  {} Interface set to: {}", "[✓]".green(), choice.bold());
    wait_for_enter();
    Ok(())
}

// ── Header ────────────────────────────────────────────────────────────────────

fn print_header(session: Option<&WatchSession>, configured_interface: &str) {
    let w = 58usize;
    println!("{}", "━".repeat(w).dimmed());
    println!(
        "  {}  {}",
        "Network Watcher".bold().cyan(),
        "live network activity monitor".dimmed()
    );
    println!("{}", "━".repeat(w).dimmed());

    let (status_str, iface_str) = if let Some(s) = session {
        let count = s.process_count();
        let status = format!(
            "{}  {}  ──  {} process{}",
            "RUNNING".green().bold(),
            s.elapsed_str(),
            count,
            if count == 1 { "" } else { "es" },
        );
        (status, s.interface())
    } else {
        ("stopped".dimmed().to_string(), configured_interface.to_string())
    };

    println!("  {:<11}{}", "Status".dimmed(), status_str);
    println!("  {:<11}{}", "Interface".dimmed(), iface_str.dimmed());
    println!("{}", "━".repeat(w).dimmed());
    println!();
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn clear() {
    execute!(std::io::stdout(), Clear(ClearType::All), MoveTo(0, 0)).ok();
}

fn wait_for_enter() {
    use std::io::Write as _;
    print!("\n  {}", "Press Enter to continue...".dimmed());
    let _ = std::io::stdout().flush();
    let mut buf = String::new();
    let _ = std::io::stdin().read_line(&mut buf);
}
