use std::io::{self, Write as _};
use std::time::Duration;

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{
        disable_raw_mode, enable_raw_mode, size, Clear, ClearType,
        EnterAlternateScreen, LeaveAlternateScreen,
    },
};

use super::{EndpointStats, ProcessData, WatchSession};

// ── Public return value ───────────────────────────────────────────────────────

pub enum ViewResult {
    Left,       // Q / Esc — return to submenu, keep watching
    Stopped,    // S — stop the session
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn run(session: &WatchSession) -> anyhow::Result<ViewResult> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, Hide)?;

    let result = event_loop(session);

    execute!(stdout, Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;

    result
}

// ── Event loop ────────────────────────────────────────────────────────────────

fn event_loop(session: &WatchSession) -> anyhow::Result<ViewResult> {
    let mut cursor: usize = 0;  // index into sorted process list
    let mut scroll: usize = 0;  // first visible line in the content area

    loop {
        let (cols, rows) = size()?;
        let cols = cols as usize;
        let rows = rows as usize;

        // snapshot state for this frame
        let (sorted_procs, interface, elapsed, proc_count) = {
            let s = session.state.lock().unwrap();
            let mut procs: Vec<ProcessData> = s.processes.values().cloned().collect();
            procs.sort_by(|a, b| b.total_events.cmp(&a.total_events));
            let iface = s.interface.clone();
            let elapsed = format_elapsed(s.started_at.elapsed().as_secs());
            let count = procs.len();
            (procs, iface, elapsed, count)
        };

        // Clamp cursor
        if proc_count > 0 && cursor >= proc_count { cursor = proc_count - 1; }

        // Build render lines from sorted process list
        let lines = build_lines(&sorted_procs, cursor, cols);

        // Auto-scroll to keep cursor line visible
        let header_rows = 2usize;
        let footer_rows = 2usize;
        let viewport_h = rows.saturating_sub(header_rows + footer_rows);
        scroll = adjust_scroll(scroll, cursor, &sorted_procs, viewport_h);

        // ── Render ────────────────────────────────────────────
        render(
            &sorted_procs, &lines, &interface, &elapsed, proc_count,
            cursor, scroll, cols, rows, viewport_h,
        )?;

        // ── Input (300 ms timeout = refresh rate) ─────────────
        if event::poll(Duration::from_millis(300))? {
            match event::read()? {
                // Leave view
                Event::Key(KeyEvent { code: KeyCode::Char('q') | KeyCode::Esc, .. }) => {
                    return Ok(ViewResult::Left);
                }
                // Stop watching
                Event::Key(KeyEvent { code: KeyCode::Char('s') | KeyCode::Char('S'), .. }) => {
                    return Ok(ViewResult::Stopped);
                }
                // Ctrl+C
                Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL, ..
                }) => return Ok(ViewResult::Left),

                // Suppress highlighted process
                Event::Key(KeyEvent { code: KeyCode::Char('d') | KeyCode::Char('D'), .. }) => {
                    if let Some(proc) = sorted_procs.get(cursor) {
                        session.suppress_pid(proc.pid);
                        if cursor > 0 { cursor -= 1; }
                    }
                }

                // Navigation
                Event::Key(KeyEvent { code: KeyCode::Up, .. }) => {
                    if cursor > 0 { cursor -= 1; }
                }
                Event::Key(KeyEvent { code: KeyCode::Down, .. }) => {
                    if proc_count > 0 && cursor + 1 < proc_count { cursor += 1; }
                }

                _ => {}
            }
        }
    }
}

// ── Render ────────────────────────────────────────────────────────────────────

struct RenderLine {
    text:      String,
    color:     Option<Color>,
    bold:      bool,
    _is_cursor: bool,
}

fn build_lines(procs: &[ProcessData], cursor: usize, cols: usize) -> Vec<RenderLine> {
    let mut lines = Vec::new();

    for (i, proc) in procs.iter().enumerate() {
        let is_cursor = i == cursor;
        let arrow = if is_cursor { "▶" } else { " " };

        lines.push(RenderLine {
            text: format!(
                "  {} {} (PID {})   {} poll-hits",
                arrow, proc.name, proc.pid, proc.total_events
            ),
            color:     Some(Color::White),
            bold:      is_cursor,
            _is_cursor: is_cursor,
        });

        // Sort endpoints by observed_count desc
        let mut eps: Vec<&EndpointStats> = proc.endpoints.values().collect();
        eps.sort_by(|a, b| b.observed_count.cmp(&a.observed_count));

        for ep in eps {
            let dest = format!("{}:{}", ep.remote_ip, ep.remote_port);
            lines.push(RenderLine {
                text: format!(
                    "       {:<42} {:<5}  {:>5} hits",
                    dest, ep.protocol, ep.observed_count,
                ),
                color:     Some(Color::DarkGrey),
                bold:      false,
                _is_cursor: false,
            });
        }

        // Blank separator between processes
        lines.push(RenderLine {
            text: String::new(),
            color: None,
            bold: false,
            _is_cursor: false,
        });
    }

    // Truncate lines to terminal width
    for line in &mut lines {
        if line.text.len() > cols {
            line.text.truncate(cols);
        }
    }

    lines
}

fn adjust_scroll(
    mut scroll: usize,
    cursor: usize,
    procs: &[ProcessData],
    viewport_h: usize,
) -> usize {
    // Find the line index where the cursor process header starts
    let mut cursor_line = 0usize;
    for (i, proc) in procs.iter().enumerate() {
        if i == cursor { break; }
        cursor_line += 1 + proc.endpoints.len() + 1; // header + endpoints + blank
    }

    if cursor_line < scroll {
        scroll = cursor_line;
    } else if cursor_line >= scroll + viewport_h {
        scroll = cursor_line.saturating_sub(viewport_h.saturating_sub(1));
    }

    scroll
}

fn render(
    procs:      &[ProcessData],
    lines:      &[RenderLine],
    interface:  &str,
    elapsed:    &str,
    proc_count: usize,
    _cursor:    usize,
    scroll:     usize,
    cols:       usize,
    rows:       usize,
    viewport_h: usize,
) -> anyhow::Result<()> {
    let mut out = io::stdout();
    queue!(out, MoveTo(0, 0), Clear(ClearType::All))?;

    // ── Header line ───────────────────────────────────────────
    let noun = if proc_count == 1 { "process" } else { "processes" };
    let header = format!(
        "  NETWORK WATCH  ──  {}  ──  {}  ──  {} new {}",
        elapsed, interface, proc_count, noun
    );
    queue!(
        out,
        SetForegroundColor(Color::Cyan),
        SetAttribute(Attribute::Bold),
        Print(truncate(&header, cols)),
        SetAttribute(Attribute::Reset),
        ResetColor,
        Print("\r\n"),
        SetForegroundColor(Color::DarkGrey),
        Print("─".repeat(cols)),
        ResetColor,
        Print("\r\n"),
    )?;

    // ── Content area ──────────────────────────────────────────
    if procs.is_empty() {
        queue!(
            out,
            SetForegroundColor(Color::DarkGrey),
            Print("  Waiting for new processes to make network connections..."),
            ResetColor,
        )?;
        // fill remaining lines
        for _ in 1..viewport_h {
            queue!(out, Print("\r\n"))?;
        }
    } else {
        let visible = lines.iter().skip(scroll).take(viewport_h);
        let mut rendered = 0usize;
        for line in visible {
            if line.bold {
                queue!(out, SetAttribute(Attribute::Bold))?;
            }
            if let Some(c) = line.color {
                queue!(out, SetForegroundColor(c))?;
            }
            queue!(out, Print(&line.text), ResetColor, SetAttribute(Attribute::Reset), Print("\r\n"))?;
            rendered += 1;
        }
        // fill empty lines in viewport
        for _ in rendered..viewport_h {
            queue!(out, Print("\r\n"))?;
        }
    }

    // ── Footer ────────────────────────────────────────────────
    queue!(out, MoveTo(0, (rows - 2) as u16))?;
    queue!(
        out,
        SetForegroundColor(Color::DarkGrey),
        Print("─".repeat(cols)),
        Print("\r\n"),
        Print("  [Q] leave view  [S] stop watching  [D] suppress process  [↑↓] navigate"),
        ResetColor,
    )?;

    out.flush()?;
    Ok(())
}

// ── Utilities ─────────────────────────────────────────────────────────────────

fn format_elapsed(secs: u64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    if h > 0 { format!("{:02}:{:02}:{:02}", h, m, s) } else { format!("{:02}:{:02}", m, s) }
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max { s } else { &s[..max] }
}
