use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub mod poll;
pub mod submenu;
pub mod view;

// ── Data model ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct EndpointStats {
    pub remote_ip:      String,
    pub remote_port:    u16,
    pub protocol:       String,   // "TCP" | "UDP"
    pub observed_count: u32,      // times seen across polls — proxy for activity
    pub _first_seen:    Duration, // reserved for future "first seen" display
    pub last_seen:      Duration,
}

#[derive(Debug, Clone)]
pub struct ProcessData {
    pub name:         String,
    pub pid:          u32,
    pub endpoints:    HashMap<(String, u16, String), EndpointStats>,
    pub total_events: u32, // sum of all endpoint observed_counts
}

// ── Shared session state ──────────────────────────────────────────────────────

pub struct WatchState {
    pub baseline_pids: HashSet<u32>,
    pub processes:     HashMap<u32, ProcessData>,
    pub ignored_pids:  HashSet<u32>,
    pub started_at:    Instant,
    pub interface:     String,
}

// ── WatchSession (owns background thread) ────────────────────────────────────

pub struct WatchSession {
    pub state: Arc<Mutex<WatchState>>,
    stop_tx:   std::sync::mpsc::SyncSender<()>,
    thread:    Option<std::thread::JoinHandle<()>>,
}

impl WatchSession {
    pub fn start(interface: String) -> anyhow::Result<Self> {
        let baseline_pids = poll::get_current_pids()?;

        let state = Arc::new(Mutex::new(WatchState {
            baseline_pids,
            processes:    HashMap::new(),
            ignored_pids: HashSet::new(),
            started_at:   Instant::now(),
            interface,
        }));

        let (stop_tx, stop_rx) = std::sync::mpsc::sync_channel(1);
        let state_clone = Arc::clone(&state);
        let thread = std::thread::spawn(move || poll::run_poll_loop(state_clone, stop_rx));

        Ok(WatchSession { state, stop_tx, thread: Some(thread) })
    }

    pub fn suppress_pid(&self, pid: u32) {
        let mut s = self.state.lock().unwrap();
        s.ignored_pids.insert(pid);
        s.processes.remove(&pid);
    }


    pub fn elapsed_str(&self) -> String {
        let secs = self.state.lock().unwrap().started_at.elapsed().as_secs();
        let h = secs / 3600;
        let m = (secs % 3600) / 60;
        let s = secs % 60;
        if h > 0 { format!("{:02}:{:02}:{:02}", h, m, s) } else { format!("{:02}:{:02}", m, s) }
    }

    pub fn process_count(&self) -> usize {
        self.state.lock().unwrap().processes.len()
    }

    pub fn interface(&self) -> String {
        self.state.lock().unwrap().interface.clone()
    }

    fn stop_thread(&mut self) {
        let _ = self.stop_tx.try_send(());
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

impl Drop for WatchSession {
    fn drop(&mut self) { self.stop_thread(); }
}
