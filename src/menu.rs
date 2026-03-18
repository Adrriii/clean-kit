use std::fmt;
use std::io::Write as _;

use anyhow::Result;
use colored::Colorize;
use inquire::{InquireError, MultiSelect, Select};

use crate::collectors::{run_collectors, CollectorFilter, COLLECTOR_NAMES};
use crate::config::Config;
use crate::diff;
use crate::output;
use crate::snapshot::Snapshot;

// ── Menu option enum ──────────────────────────────────────────────────────────

#[derive(Clone)]
enum Action {
    Before,
    After,
    Diff,
    Run,
    Reset,
    Collectors,
    Quit,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Action::Before     => write!(f, "Take baseline snapshot"),
            Action::After      => write!(f, "Take after snapshot"),
            Action::Diff       => write!(f, "Show diff"),
            Action::Run        => write!(f, "Run: after + diff"),
            Action::Reset      => write!(f, "Reset: fresh baseline"),
            Action::Collectors => write!(f, "Configure collectors"),
            Action::Quit       => write!(f, "Quit"),
        }
    }
}

// ── Main loop ─────────────────────────────────────────────────────────────────

pub fn run(config: Config) -> Result<()> {
    let mut filter = CollectorFilter::from_config(&config);

    loop {
        clear();
        print_header(&config, &filter);

        let choice = Select::new("", menu_items())
            .without_help_message()
            .prompt();

        match choice {
            Ok(action) => {
                println!();
                if let Err(e) = dispatch(action, &config, &mut filter) {
                    println!("  {}: {}", "Error".red().bold(), e);
                    wait_for_enter();
                }
            }
            // ESC or Ctrl+C — exit cleanly
            Err(InquireError::OperationCanceled | InquireError::OperationInterrupted) => break,
            Err(e) => return Err(e.into()),
        }
    }

    Ok(())
}

// ── Action dispatch ───────────────────────────────────────────────────────────

fn dispatch(action: Action, config: &Config, filter: &mut CollectorFilter) -> Result<()> {
    match action {
        Action::Before => {
            println!("{}", "[*] Collecting baseline snapshot...".cyan().bold());
            let snap = run_collectors(config, filter)?;
            snap.save(&config.output_dir, "before")?;
            println!("{}", "[✓] Baseline saved.".green());
            wait_for_enter();
        }

        Action::After => {
            println!("{}", "[*] Collecting after snapshot...".cyan().bold());
            let snap = run_collectors(config, filter)?;
            snap.save(&config.output_dir, "after")?;
            println!("{}", "[✓] After snapshot saved.".green());
            wait_for_enter();
        }

        Action::Diff => {
            let before = Snapshot::load(&config.output_dir, "before")?;
            let after  = Snapshot::load(&config.output_dir, "after")?;
            let d = diff::compute(&before, &after);
            output::print_diff(&d, &before, &after, true);
            wait_for_enter();
        }

        Action::Run => {
            println!("{}", "[*] Collecting after snapshot...".cyan().bold());
            let snap = run_collectors(config, filter)?;
            snap.save(&config.output_dir, "after")?;
            println!("{}", "[✓] After snapshot saved.".green());
            println!();
            let before = Snapshot::load(&config.output_dir, "before")?;
            let after  = Snapshot::load(&config.output_dir, "after")?;
            let d = diff::compute(&before, &after);
            output::print_diff(&d, &before, &after, true);
            wait_for_enter();
        }

        Action::Reset => {
            let after_path = config.output_dir.join("after.json");
            if after_path.exists() {
                std::fs::remove_file(&after_path)?;
                println!("{}", "[✓] Cleared after snapshot.".yellow());
            }
            println!("{}", "[*] Collecting fresh baseline...".cyan().bold());
            let snap = run_collectors(config, filter)?;
            snap.save(&config.output_dir, "before")?;
            println!("{}", "[✓] Fresh baseline saved.".green());
            wait_for_enter();
        }

        Action::Collectors => {
            configure_collectors(filter)?;
        }

        Action::Quit => std::process::exit(0),
    }

    Ok(())
}

// ── Collector configuration ───────────────────────────────────────────────────

fn configure_collectors(filter: &mut CollectorFilter) -> Result<()> {
    clear();
    println!("{}", "  Configure collectors".bold().white());
    println!("{}", "  Space to toggle, Enter to confirm, Esc to cancel\n".dimmed());

    let defaults: Vec<usize> = COLLECTOR_NAMES.iter().enumerate()
        .filter(|(_, &name)| filter.is_enabled(name))
        .map(|(i, _)| i)
        .collect();

    let selected = match MultiSelect::new("", COLLECTOR_NAMES.to_vec())
        .with_default(&defaults)
        .without_help_message()
        .prompt()
    {
        Ok(s) => s,
        Err(InquireError::OperationCanceled | InquireError::OperationInterrupted) => return Ok(()),
        Err(e) => return Err(e.into()),
    };

    *filter = CollectorFilter::from_enabled_list(&selected);
    Ok(())
}

// ── Header ────────────────────────────────────────────────────────────────────

fn print_header(config: &Config, filter: &CollectorFilter) {
    let w = 58;
    println!("{}", "━".repeat(w).dimmed());
    println!("  {}  {}", "clean-kit".bold().cyan(), "Windows State Diff".dimmed());
    println!("{}", "━".repeat(w).dimmed());

    println!(
        "  {:<11}{}",
        "Baseline".dimmed(),
        snapshot_status(&config.output_dir, "before")
    );
    println!(
        "  {:<11}{}",
        "After".dimmed(),
        snapshot_status(&config.output_dir, "after")
    );

    let enabled = filter.enabled_names().join(", ");
    let collectors_str = if enabled.is_empty() {
        "none".red().to_string()
    } else {
        enabled.dimmed().to_string()
    };
    println!("  {:<11}{}", "Collectors".dimmed(), collectors_str);
    println!("  {:<11}{}", "Output".dimmed(), config.output_dir.display().to_string().dimmed());
    println!("{}", "━".repeat(w).dimmed());
    println!();
}

fn snapshot_status(output_dir: &std::path::Path, name: &str) -> String {
    match Snapshot::load(output_dir, name) {
        Ok(s) => format!(
            "{}  {}  ({})",
            "✓".green(),
            s.timestamp.format("%Y-%m-%d %H:%M UTC"),
            s.hostname.dimmed()
        ),
        Err(_) => format!("{}", "not taken".dimmed()),
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn menu_items() -> Vec<Action> {
    vec![
        Action::Before,
        Action::After,
        Action::Diff,
        Action::Run,
        Action::Reset,
        Action::Collectors,
        Action::Quit,
    ]
}

fn clear() {
    print!("\x1b[2J\x1b[H");
    let _ = std::io::stdout().flush();
}

fn wait_for_enter() {
    print!("\n  {}", "Press Enter to return to menu...".dimmed());
    let _ = std::io::stdout().flush();
    let mut buf = String::new();
    let _ = std::io::stdin().read_line(&mut buf);
}
