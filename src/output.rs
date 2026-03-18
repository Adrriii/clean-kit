use colored::Colorize;

use crate::diff::*;
use crate::snapshot::*;

const SEP_LEN: usize = 64;

// ── Public entry point ────────────────────────────────────────────────────────

pub fn print_diff(diff: &SnapshotDiff, before: &Snapshot, after: &Snapshot, show_removed: bool) {
    print_header(before, after);

    let any = print_processes(&diff.processes, show_removed)
        | print_network(&diff.network, show_removed)
        | print_registry(&diff.registry, show_removed)
        | print_files(&diff.files, show_removed)
        | print_tasks(&diff.tasks, show_removed)
        | print_services(&diff.services, show_removed);

    if !any {
        println!("\n  {}", "No changes detected.".green());
    }

    print_summary(diff);
    print_hints();
}

// ── Header ────────────────────────────────────────────────────────────────────

fn print_header(before: &Snapshot, after: &Snapshot) {
    let bf = before.timestamp.format("%Y-%m-%d %H:%M UTC");
    let af = after.timestamp.format("%Y-%m-%d %H:%M UTC");
    let delta = after.timestamp - before.timestamp;
    let mins = delta.num_minutes();
    let secs = delta.num_seconds() % 60;
    let delta_str = if mins > 0 {
        format!("Δ {}m {:02}s", mins, secs)
    } else {
        format!("Δ {}s", secs)
    };

    println!();
    println!("{}", "─".repeat(SEP_LEN).dimmed());
    println!(
        "  {}  {}  →  {}  ({})",
        "ck diff".bold().white(),
        bf.to_string().dimmed(),
        af.to_string().dimmed(),
        delta_str.dimmed()
    );
    if before.hostname != after.hostname {
        println!(
            "  {}  host mismatch: {} vs {}",
            "WARN".yellow().bold(),
            before.hostname,
            after.hostname
        );
    }
    println!("{}", "─".repeat(SEP_LEN).dimmed());
}

// ── Section helpers ───────────────────────────────────────────────────────────

fn section(title: &str) {
    let pad = SEP_LEN.saturating_sub(title.len() + 4);
    println!("\n  {} {}", title.bold().white(), "─".repeat(pad).dimmed());
}

fn row_added(line: &str)   { println!("  {} {}", "[+]".green().bold(),  line); }
fn row_removed(line: &str) { println!("  {} {}", "[-]".red().bold(),    line); }
fn row_changed(line: &str) { println!("  {} {}", "[~]".yellow().bold(), line); }

// ── Per-collector printers ────────────────────────────────────────────────────

fn print_processes(d: &CollectorDiff<ProcessEntry>, show_removed: bool) -> bool {
    if d.skipped { return false; }
    let has = d.has_changes() || (show_removed && !d.removed.is_empty());
    if !has { return false; }

    section("PROCESSES");
    for p in &d.added {
        let exe = if p.executable.is_empty() { "(no path)".to_string() } else { p.executable.clone() };
        row_added(&format!("{:<28} {}", p.name, exe));
        if !p.command_line.is_empty() && p.command_line != p.executable {
            println!("       cmd: {}", p.command_line.dimmed());
        }
    }
    if show_removed {
        for p in &d.removed {
            row_removed(&format!("{:<28} {}", p.name, p.executable));
        }
    }
    for (before, after) in &d.changed {
        row_changed(&format!("{} — command_line changed", before.name));
        println!("       was: {}", before.command_line.dimmed());
        println!("       now: {}", after.command_line.yellow());
    }
    true
}

fn print_network(d: &CollectorDiff<NetworkEntry>, show_removed: bool) -> bool {
    if d.skipped { return false; }
    let has = d.has_changes() || (show_removed && !d.removed.is_empty());
    if !has { return false; }

    section("NETWORK CONNECTIONS");
    for n in &d.added {
        let proc = n.process_name.as_deref().unwrap_or("?");
        let remote = if n.remote_addr == "*" {
            "*:*".to_string()
        } else {
            format!("{}:{}", n.remote_addr, n.remote_port)
        };
        row_added(&format!(
            "{:<4} {}:{} → {}  {}  ({})",
            n.protocol, n.local_addr, n.local_port,
            remote, n.state, proc
        ));
    }
    if show_removed {
        for n in &d.removed {
            let proc = n.process_name.as_deref().unwrap_or("?");
            row_removed(&format!(
                "{} {}:{} ({})",
                n.protocol, n.remote_addr, n.remote_port, proc
            ));
        }
    }
    for (before, after) in &d.changed {
        row_changed(&format!(
            "{} {}:{} → {}:{} — state {} → {}",
            before.protocol, before.local_addr, before.local_port,
            before.remote_addr, before.remote_port,
            before.state.dimmed(), after.state.yellow()
        ));
    }
    true
}

fn print_registry(d: &CollectorDiff<RegistryEntry>, show_removed: bool) -> bool {
    if d.skipped { return false; }
    let has = d.has_changes() || (show_removed && !d.removed.is_empty());
    if !has { return false; }

    section("REGISTRY STARTUP");
    for r in &d.added {
        row_added(&format!("{}\\{}  {}  =  {}", r.hive, r.key_path, r.name, r.data));
    }
    if show_removed {
        for r in &d.removed {
            row_removed(&format!("{}\\{}  {}", r.hive, r.key_path, r.name));
        }
    }
    for (before, after) in &d.changed {
        row_changed(&format!("{}\\{}  {}", before.hive, before.key_path, before.name));
        println!("       was: {}", before.data.dimmed());
        println!("       now: {}", after.data.yellow());
    }
    true
}

fn print_files(d: &CollectorDiff<FileEntry>, show_removed: bool) -> bool {
    if d.skipped { return false; }
    let has = d.has_changes() || (show_removed && !d.removed.is_empty());
    if !has { return false; }

    section("FILES");
    for f in &d.added {
        row_added(&format!("{}  ({} bytes)", f.path, fmt_size(f.size)));
    }
    if show_removed {
        for f in &d.removed {
            row_removed(&f.path);
        }
    }
    for (before, after) in &d.changed {
        row_changed(&format!("{}  {} → {} bytes", before.path, fmt_size(before.size), fmt_size(after.size)));
    }
    true
}

fn print_tasks(d: &CollectorDiff<TaskEntry>, show_removed: bool) -> bool {
    if d.skipped { return false; }
    let has = d.has_changes() || (show_removed && !d.removed.is_empty());
    if !has { return false; }

    section("SCHEDULED TASKS");
    for t in &d.added {
        row_added(&format!(
            "{}{} ({})",
            t.task_path, t.task_name, t.state
        ));
        if !t.execute.is_empty() {
            let args = if t.arguments.is_empty() { String::new() } else { format!(" {}", t.arguments) };
            println!("       exec: {}{}", t.execute, args);
        }
    }
    if show_removed {
        for t in &d.removed {
            row_removed(&format!("{}{}", t.task_path, t.task_name));
        }
    }
    for (before, after) in &d.changed {
        row_changed(&format!("{}{}", before.task_path, before.task_name));
        if before.execute != after.execute {
            println!("       exec was: {}", before.execute.dimmed());
            println!("       exec now: {}", after.execute.yellow());
        }
        if before.state != after.state {
            println!("       state: {} → {}", before.state.dimmed(), after.state.yellow());
        }
    }
    true
}

fn print_services(d: &CollectorDiff<ServiceEntry>, show_removed: bool) -> bool {
    if d.skipped { return false; }
    let has = d.has_changes() || (show_removed && !d.removed.is_empty());
    if !has { return false; }

    section("SERVICES");
    for s in &d.added {
        row_added(&format!(
            "{}  \"{}\"  {}  {}",
            s.name, s.display_name, s.state, s.start_mode
        ));
        if !s.path_name.is_empty() {
            println!("       path: {}", s.path_name);
        }
    }
    if show_removed {
        for s in &d.removed {
            row_removed(&format!("{}  \"{}\"", s.name, s.display_name));
        }
    }
    for (before, after) in &d.changed {
        row_changed(&format!("{}  \"{}\"", before.name, before.display_name));
        if before.state != after.state {
            println!("       state: {} → {}", before.state.dimmed(), after.state.yellow());
        }
        if before.start_mode != after.start_mode {
            println!("       start: {} → {}", before.start_mode.dimmed(), after.start_mode.yellow());
        }
        if before.path_name != after.path_name {
            println!("       path was: {}", before.path_name.dimmed());
            println!("       path now: {}", after.path_name.yellow());
        }
    }
    true
}

// ── Summary and hints ─────────────────────────────────────────────────────────

fn print_summary(diff: &SnapshotDiff) {
    section("SUMMARY");

    fn count<T>(d: &CollectorDiff<T>) -> String {
        if d.skipped {
            return "skipped".dimmed().to_string();
        }
        let mut parts = Vec::new();
        if !d.added.is_empty()   { parts.push(format!("{} added",   d.added.len()).green().to_string()); }
        if !d.changed.is_empty() { parts.push(format!("{} changed", d.changed.len()).yellow().to_string()); }
        if !d.removed.is_empty() { parts.push(format!("{} removed", d.removed.len()).red().to_string()); }
        if parts.is_empty() { "clean".dimmed().to_string() } else { parts.join(", ") }
    }

    println!("  processes:  {}", count(&diff.processes));
    println!("  network:    {}", count(&diff.network));
    println!("  registry:   {}", count(&diff.registry));
    println!("  files:      {}", count(&diff.files));
    println!("  tasks:      {}", count(&diff.tasks));
    println!("  services:   {}", count(&diff.services));
}

fn print_hints() {
    println!("\n  {}", "Analysis hints:".dimmed());
    println!("  {}", "• .exe or .ps1 in %AppData%, %Temp%, %Public%".dimmed());
    println!("  {}", "• Random-looking filenames (abc123.exe)".dimmed());
    println!("  {}", "• Connections to unknown IPs on unusual ports".dimmed());
    println!("  {}", "• New services/tasks with paths outside System32".dimmed());
    println!("{}", "─".repeat(SEP_LEN).dimmed());
}

// ── Utilities ─────────────────────────────────────────────────────────────────

fn fmt_size(bytes: u64) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1_024 {
        format!("{:.1} KB", bytes as f64 / 1_024.0)
    } else {
        format!("{} B", bytes)
    }
}
