use std::collections::HashMap;
use std::hash::Hash;

use crate::snapshot::*;

// ── Result types ──────────────────────────────────────────────────────────────

pub struct CollectorDiff<T> {
    /// Both snapshots had no data for this collector (it was skipped in both)
    pub skipped: bool,
    pub added: Vec<T>,
    pub removed: Vec<T>,
    pub changed: Vec<(T, T)>, // (before, after)
}

impl<T> CollectorDiff<T> {
    pub fn skipped() -> Self {
        Self { skipped: true, added: vec![], removed: vec![], changed: vec![] }
    }

    pub fn has_changes(&self) -> bool {
        !self.added.is_empty() || !self.removed.is_empty() || !self.changed.is_empty()
    }
}

pub struct SnapshotDiff {
    pub processes: CollectorDiff<ProcessEntry>,
    pub network:   CollectorDiff<NetworkEntry>,
    pub registry:  CollectorDiff<RegistryEntry>,
    pub files:     CollectorDiff<FileEntry>,
    pub tasks:     CollectorDiff<TaskEntry>,
    pub services:  CollectorDiff<ServiceEntry>,
}

// ── Main entry point ──────────────────────────────────────────────────────────

pub fn compute(before: &Snapshot, after: &Snapshot) -> SnapshotDiff {
    SnapshotDiff {
        processes: diff_processes(&before.data.processes, &after.data.processes),
        network:   diff_network(&before.data.network, &after.data.network),
        registry:  diff_registry(&before.data.registry, &after.data.registry),
        files:     diff_files(&before.data.files, &after.data.files),
        tasks:     diff_tasks(&before.data.tasks, &after.data.tasks),
        services:  diff_services(&before.data.services, &after.data.services),
    }
}

// ── Per-collector diffs ───────────────────────────────────────────────────────

fn diff_processes(
    before: &Option<Vec<ProcessEntry>>,
    after: &Option<Vec<ProcessEntry>>,
) -> CollectorDiff<ProcessEntry> {
    with_data(before, after, |b, a| {
        diff_entries(
            b, a,
            |p| (p.name.to_lowercase(), p.executable.to_lowercase()),
            |a, b| a.command_line == b.command_line,
        )
    })
}

fn diff_network(
    before: &Option<Vec<NetworkEntry>>,
    after: &Option<Vec<NetworkEntry>>,
) -> CollectorDiff<NetworkEntry> {
    with_data(before, after, |b, a| {
        diff_entries(
            b, a,
            |n| (
                n.protocol.to_lowercase(),
                n.local_port,
                n.remote_addr.to_lowercase(),
                n.remote_port,
            ),
            |a, b| a.state == b.state,
        )
    })
}

fn diff_registry(
    before: &Option<Vec<RegistryEntry>>,
    after: &Option<Vec<RegistryEntry>>,
) -> CollectorDiff<RegistryEntry> {
    with_data(before, after, |b, a| {
        diff_entries(
            b, a,
            |r| (r.hive.to_lowercase(), r.key_path.to_lowercase(), r.name.to_lowercase()),
            |a, b| a.data == b.data,
        )
    })
}

fn diff_files(
    before: &Option<Vec<FileEntry>>,
    after: &Option<Vec<FileEntry>>,
) -> CollectorDiff<FileEntry> {
    with_data(before, after, |b, a| {
        diff_entries(
            b, a,
            |f| f.path.to_lowercase(),
            |a, b| a.size == b.size && a.modified == b.modified,
        )
    })
}

fn diff_tasks(
    before: &Option<Vec<TaskEntry>>,
    after: &Option<Vec<TaskEntry>>,
) -> CollectorDiff<TaskEntry> {
    with_data(before, after, |b, a| {
        diff_entries(
            b, a,
            |t| format!("{}{}", t.task_path, t.task_name).to_lowercase(),
            |a, b| a.execute == b.execute && a.arguments == b.arguments && a.state == b.state,
        )
    })
}

fn diff_services(
    before: &Option<Vec<ServiceEntry>>,
    after: &Option<Vec<ServiceEntry>>,
) -> CollectorDiff<ServiceEntry> {
    with_data(before, after, |b, a| {
        diff_entries(
            b, a,
            |s| s.name.to_lowercase(),
            |a, b| a.state == b.state && a.start_mode == b.start_mode && a.path_name == b.path_name,
        )
    })
}

// ── Generic helpers ───────────────────────────────────────────────────────────

fn with_data<T, F>(
    before: &Option<Vec<T>>,
    after: &Option<Vec<T>>,
    f: F,
) -> CollectorDiff<T>
where
    T: Clone,
    F: FnOnce(&[T], &[T]) -> CollectorDiff<T>,
{
    match (before, after) {
        (None, None) => CollectorDiff::skipped(),
        (b, a) => f(
            b.as_deref().unwrap_or(&[]),
            a.as_deref().unwrap_or(&[]),
        ),
    }
}

fn diff_entries<T, K>(
    before: &[T],
    after: &[T],
    key_fn: impl Fn(&T) -> K,
    same_fn: impl Fn(&T, &T) -> bool,
) -> CollectorDiff<T>
where
    T: Clone,
    K: Eq + Hash,
{
    let before_map: HashMap<K, &T> = before.iter().map(|e| (key_fn(e), e)).collect();
    let after_map: HashMap<K, &T> = after.iter().map(|e| (key_fn(e), e)).collect();

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut changed = Vec::new();

    for (key, after_entry) in &after_map {
        if let Some(before_entry) = before_map.get(key) {
            if !same_fn(before_entry, after_entry) {
                changed.push(((*before_entry).clone(), (*after_entry).clone()));
            }
        } else {
            added.push((*after_entry).clone());
        }
    }

    for (key, before_entry) in &before_map {
        if !after_map.contains_key(key) {
            removed.push((*before_entry).clone());
        }
    }

    CollectorDiff { skipped: false, added, removed, changed }
}
