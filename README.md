# clean-kit

Windows system state snapshot and diff tool for malware analysis and dynamic analysis labs.

Run `ck` **before** executing a suspicious file, then again **after** — get a precise, structured diff of everything that changed: processes, network connections, registry persistence, files, scheduled tasks, services, firewall rules, WMI subscriptions, and startup folder contents.

---

## Download

Grab the latest `ck.exe` from the [Releases](../../releases/latest) page. No install, no runtime required — single binary.

**Requirements:** Windows 10+, PowerShell 5.1+

---

## Usage

Just run it:

```
ck
```

An interactive menu appears. Navigate with **↑↓**, select with **Enter**, exit with **Esc** or **Ctrl+C**.

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  clean-kit  Windows State Diff
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Baseline   ✓  2026-03-18 10:00 UTC  (DESKTOP-LAB01)
  After      not taken
  Collectors processes, network, registry, files, tasks, services, firewall, wmi, startup
  Net Watch  RUNNING  00:02  ──  1 process
  Output     ./snapshots
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

❯ Take baseline snapshot
  Take after snapshot
  Show diff
  Run: after + diff
  Reset: fresh baseline
  Configure collectors
  Network Watcher
  Quit
```

### Snapshot workflow

| Step | Action |
|------|--------|
| 1 | Select **Take baseline snapshot** — captures the clean system state |
| 2 | Run the file you want to analyze |
| 3 | Select **Run: after + diff** — captures post-execution state and immediately shows what changed |
| 4 | Select **Reset: fresh baseline** before the next sample |

### Network Watcher workflow

| Step | Action |
|------|--------|
| 1 | Select **Network Watcher → Start watching** — snapshots the current process list as baseline, then enters the live view |
| 2 | Run the file you want to analyze |
| 3 | Watch new processes and their connections appear in real time |
| 4 | Press **S** to stop, **Q** to leave the view and return to the menu |

---

## Diff output

New entries are shown in green `[+]`, removed in red `[-]`, changed in yellow `[~]`.

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  ck diff  2026-03-18 10:00 UTC  →  2026-03-18 10:05 UTC  (Δ 5m)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  PROCESSES ──────────────────────────────────────────────
  [+] updater.exe        C:\Users\user\AppData\Local\Temp\updater.exe

  NETWORK CONNECTIONS ────────────────────────────────────
  [+] TCP  0.0.0.0:4444  →  *:*            LISTEN       (updater.exe)
  [+] TCP  10.0.0.5:51234  →  185.220.0.1:443  ESTABLISHED  (updater.exe)

  REGISTRY STARTUP ───────────────────────────────────────
  [+] HKCU\Software\Microsoft\Windows\CurrentVersion\Run
      Updater  =  C:\Users\user\AppData\Local\Temp\updater.exe

  FILES ──────────────────────────────────────────────────
  [+] C:\Users\user\AppData\Roaming\updater.exe  (48.2 KB)

  SCHEDULED TASKS ────────────────────────────────────────
  [+] \Updater (Ready)
      exec: C:\Users\user\AppData\Local\Temp\updater.exe

  SERVICES ───────────────────────────────────────────────
  (clean)

  FIREWALL RULES ─────────────────────────────────────────
  [+] Inbound      Allow    enabled  "updater"
       id: {A1B2C3D4-...}

  WMI SUBSCRIPTIONS ──────────────────────────────────────
  [Filter]  SCM Event Log Filter
       detail: SELECT * FROM __InstanceModificationEvent ...

  STARTUP FOLDER ─────────────────────────────────────────
  [+] C:\Users\user\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup\run.lnk  (1.2 KB)

  SUMMARY ────────────────────────────────────────────────
  processes:  1 added
  network:    2 added
  registry:   1 added
  files:      1 added
  tasks:      1 added
  services:   clean
  firewall:   1 added
  wmi:        1 added
  startup:    1 added
```

---

## Collectors

| Collector | What it captures | Source |
|-----------|-----------------|--------|
| **processes** | PID, name, full executable path, command line | `Win32_Process` (WMI) |
| **network** | TCP/UDP connections with state, local/remote address, owning process | `netstat -ano` |
| **registry** | Values in Run / RunOnce / RunServices keys (configurable) | `winreg` (native) |
| **files** | Executable and script files in suspicious locations | `walkdir` (configurable) |
| **tasks** | Scheduled task name, path, state, execute action | `Get-ScheduledTask` (PS) |
| **services** | Name, display name, state, start mode, binary path | `Win32_Service` (WMI) |
| **firewall** | All Windows Firewall rules — catches C2 allow-rules and inbound backdoors | `Get-NetFirewallRule` (PS) |
| **wmi** | WMI event filters, command-line consumers, and script consumers in `root\subscription` | `Get-CimInstance` (PS) |
| **startup** | All files (any extension) in per-user and All Users startup folders | filesystem |

---

## Network Watcher

The Network Watcher is a live monitoring mode that runs independently of the snapshot workflow. It snapshots the current process list at start, then continuously polls network connections — only tracking processes that weren't running at that moment.

```
  NETWORK WATCH  ──  00:02:14  ──  All interfaces  ──  2 new processes
────────────────────────────────────────────────────────────────────────
  ▶ updater.exe (PID 4821)   312 poll-hits
       185.220.0.1:443                           TCP     312 hits
       8.8.8.8:53                                UDP      48 hits

    game.exe (PID 3190)   6 poll-hits
       192.168.1.1:80                            TCP       6 hits
────────────────────────────────────────────────────────────────────────
  [Q] leave view  [S] stop watching  [D] suppress process  [↑↓] navigate
```

Processes are sorted by total poll-hits (highest activity first). Endpoints within each process are sorted the same way, making outliers immediately visible. The **poll-hit** count reflects how many 500ms polling cycles a connection was observed active — higher values indicate longer-lived or more persistent connections.

### Controls

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate between processes |
| `D` | Suppress the highlighted process (ignored for the rest of the session) |
| `Q` / `Esc` | Leave the live view, keep watching in the background |
| `S` | Stop watching entirely |

### Submenu

Accessible from **Network Watcher** in the main menu.

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Network Watcher  live network activity monitor
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Status     RUNNING  00:02  ──  1 process
  Interface  Ethernet
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

❯ Live view
  Stop watching
  Configure network interface
  Back
```

The selected network interface is saved to `cleankit.toml` under `[netwatch]` and persists across sessions.

### Why this beats plain `netstat` + `tasklist`

- **Structured JSON snapshots** — precise set-difference diffing, no false positives from text matching
- **Process identity by executable path** — a fake `svchost.exe` in `%TEMP%` won't hide behind the name
- **Network connections enriched with process name** — see immediately which process opened a connection
- **Services collector** — catches service-based persistence that the original scripts missed
- **Firewall rules collector** — detects new allow-rules that open backdoor ports or whitelist malware traffic
- **WMI subscription collector** — surfaces fileless persistence via `__EventFilter` / `CommandLineEventConsumer` / `ActiveScriptEventConsumer` in `root\subscription`
- **Startup folder collector** — catches `.lnk` shortcuts and other non-binary drops that file extension filters would miss
- **Configurable file scan paths** — focused on high-value locations (`%AppData%`, `%Temp%`, etc.) instead of walking all of `C:\Users`

---

## Configuration

`cleankit.toml` is read from the current directory on startup. If it doesn't exist, defaults are used.

```toml
output_dir = "./snapshots"

[netwatch]
interface = "All interfaces"   # saved automatically when changed in the menu

[collectors]
processes = true
network   = true
registry  = true
files     = true     # slowest collector — disable for quick scans
tasks     = true
services  = true
firewall  = true
wmi       = true
startup   = true

[registry]
keys = [
    "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
    "HKLM\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
    "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\RunOnce",
    "HKLM\\Software\\Microsoft\\Windows\\CurrentVersion\\RunOnce",
    "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\RunServices",
    "HKLM\\Software\\Microsoft\\Windows\\CurrentVersion\\RunServices",
]

[files]
paths = [
    "%APPDATA%",
    "%LOCALAPPDATA%",
    "%TEMP%",
    "%USERPROFILE%\\Downloads",
    "%USERPROFILE%\\Desktop",
    "C:\\ProgramData",
]
extensions = ["exe", "dll", "bat", "ps1", "vbs", "js", "hta", "scr", "com"]
max_depth  = 6
```

You can also toggle collectors live from the **Configure collectors** menu — changes apply for the current session only.

### Snapshot files

Snapshots are saved as `snapshots/before.json` and `snapshots/after.json`. They contain structured JSON (version, timestamp, hostname, and per-collector data) so you can inspect or post-process them with any tool.

---

## Building from source

```
git clone https://github.com/YOUR_USERNAME/clean-kit
cd clean-kit
cargo build --release
```

Binary will be at `target/release/ck.exe`. Requires Rust stable and the `x86_64-pc-windows-msvc` toolchain.

---

## License

MIT
