# clean-kit

Windows system state snapshot and diff tool for malware analysis and dynamic analysis labs.

Run `ck` **before** executing a suspicious file, then again **after** — get a precise, structured diff of everything that changed: processes, network connections, registry persistence, files, scheduled tasks, and services.

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
  Collectors processes, network, registry, files, tasks, services
  Output     ./snapshots
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

❯ Take baseline snapshot
  Take after snapshot
  Show diff
  Run: after + diff
  Reset: fresh baseline
  Configure collectors
  Quit
```

### Typical workflow

| Step | Action |
|------|--------|
| 1 | Select **Take baseline snapshot** — captures the clean system state |
| 2 | Run the file you want to analyze |
| 3 | Select **Run: after + diff** — captures post-execution state and immediately shows what changed |
| 4 | Select **Reset: fresh baseline** before the next sample |

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

  SUMMARY ────────────────────────────────────────────────
  processes:  1 added
  network:    2 added
  registry:   1 added
  files:      1 added
  tasks:      1 added
  services:   clean
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

### Why this beats plain `netstat` + `tasklist`

- **Structured JSON snapshots** — precise set-difference diffing, no false positives from text matching
- **Process identity by executable path** — a fake `svchost.exe` in `%TEMP%` won't hide behind the name
- **Network connections enriched with process name** — see immediately which process opened a connection
- **Services collector** — catches service-based persistence that the original scripts missed
- **Configurable file scan paths** — focused on high-value locations (`%AppData%`, `%Temp%`, etc.) instead of walking all of `C:\Users`

---

## Configuration

`cleankit.toml` is read from the current directory on startup. If it doesn't exist, defaults are used.

```toml
output_dir = "./snapshots"

[collectors]
processes = true
network   = true
registry  = true
files     = true     # slowest collector — disable for quick scans
tasks     = true
services  = true

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
