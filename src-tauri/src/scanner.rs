use crate::tree::{FileNode, ScanProgress};
use jwalk::{Parallelism, WalkDirGeneric};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Instant, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};

/// Shared state for an in-flight scan; lets the frontend cancel.
pub struct ScanHandle {
    pub cancelled: Arc<AtomicBool>,
    pub scanned_files: Arc<AtomicU64>,
    pub scanned_bytes: Arc<AtomicU64>,
}

impl ScanHandle {
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
            scanned_files: Arc::new(AtomicU64::new(0)),
            scanned_bytes: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
}

/// Per-entry stat snapshot, attached during jwalk's parallel readdir so we
/// don't pay a second `stat()` syscall on the iterator thread.
#[derive(Default, Clone, Debug)]
struct EntryMeta {
    size: u64,
    modified: Option<u64>,
}

/// Raw entry collected during walk (flat).
struct Entry {
    path: PathBuf,
    parent: PathBuf,
    name: String,
    size: u64,
    is_dir: bool,
    modified: Option<u64>,
}

/// Walk `root`, return the root FileNode with children populated up to `max_depth`.
/// `max_depth = 0` means root only. The scanner always walks the full tree to
/// compute accurate folder sizes; `max_depth` just trims what gets serialized.
pub fn scan(
    root: &Path,
    app: AppHandle,
    handle: Arc<ScanHandle>,
    show_hidden: bool,
    max_depth: u32,
) -> Result<FileNode, String> {
    let root = root
        .canonicalize()
        .map_err(|e| format!("cannot resolve path: {e}"))?;
    let cancelled = handle.cancelled.clone();
    let scanned_files = handle.scanned_files.clone();
    let scanned_bytes = handle.scanned_bytes.clone();

    // Stat files inside jwalk's parallel pool during readdir. This moves the
    // expensive metadata syscalls off the single-threaded iterator and onto
    // every CPU — the dominant speedup for IO-bound scans.
    let cancelled_for_walk = cancelled.clone();
    let scanned_files_walk = scanned_files.clone();
    let scanned_bytes_walk = scanned_bytes.clone();
    let walker = WalkDirGeneric::<((), EntryMeta)>::new(&root)
        .parallelism(Parallelism::RayonNewPool(0)) // 0 = auto = all cores
        .skip_hidden(!show_hidden)
        .follow_links(false)
        .process_read_dir(move |_depth, _path, _state, children| {
            // Bail early if cancelled — avoids wasted stats on huge trees.
            if cancelled_for_walk.load(Ordering::Relaxed) {
                children.clear();
                return;
            }
            for child in children.iter_mut().flatten() {
                if !child.file_type.is_file() {
                    continue;
                }
                if let Ok(md) = child.metadata() {
                    let size = md.len();
                    let modified = md
                        .modified()
                        .ok()
                        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                        .map(|d| d.as_secs());
                    child.client_state = EntryMeta { size, modified };
                    scanned_files_walk.fetch_add(1, Ordering::Relaxed);
                    scanned_bytes_walk.fetch_add(size, Ordering::Relaxed);
                }
            }
        });

    // Iterator runs on this single thread; no mutex needed for collectors.
    let mut entries: Vec<Entry> = Vec::with_capacity(8192);
    let mut last_emit = Instant::now();

    for entry in walker {
        if cancelled.load(Ordering::Relaxed) {
            return Err("cancelled".into());
        }
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue, // permission denied, broken symlink, etc.
        };

        let path = entry.path();
        if path == root {
            continue; // root is synthesized below
        }

        let is_dir = entry.file_type().is_dir();
        let (size, modified) = if is_dir {
            (0u64, None)
        } else {
            // Already statted in the parallel pool above.
            (entry.client_state.size, entry.client_state.modified)
        };

        let name = entry.file_name().to_string_lossy().into_owned();
        let parent = path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| root.clone());

        entries.push(Entry {
            path,
            parent,
            name,
            size,
            is_dir,
            modified,
        });

        // Throttle progress emits to ~5/sec.
        let now = Instant::now();
        if now.duration_since(last_emit).as_millis() >= 200 {
            last_emit = now;
            let _ = app.emit(
                "scan:progress",
                ScanProgress {
                    scanned_files: scanned_files.load(Ordering::Relaxed),
                    scanned_bytes: scanned_bytes.load(Ordering::Relaxed),
                    current_path: entries
                        .last()
                        .map(|e| e.path.to_string_lossy().into_owned())
                        .unwrap_or_default(),
                },
            );
        }
    }

    if cancelled.load(Ordering::Relaxed) {
        return Err("cancelled".into());
    }

    Ok(build_tree(&root, entries, max_depth))
}

/// Build a nested FileNode tree from the flat entries, summing sizes up.
fn build_tree(root: &Path, entries: Vec<Entry>, max_depth: u32) -> FileNode {
    // Group by parent path. Pre-size based on rough dir density.
    let mut by_parent: HashMap<PathBuf, Vec<Entry>> =
        HashMap::with_capacity(entries.len() / 16 + 16);
    for e in entries {
        by_parent.entry(e.parent.clone()).or_default().push(e);
    }

    let root_name = root
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| root.to_string_lossy().into_owned());

    build_node(root, &root_name, true, 0, None, &mut by_parent, 0, max_depth)
}

fn build_node(
    path: &Path,
    name: &str,
    is_dir: bool,
    own_size: u64,
    modified: Option<u64>,
    by_parent: &mut HashMap<PathBuf, Vec<Entry>>,
    depth: u32,
    max_depth: u32,
) -> FileNode {
    let extension = if !is_dir {
        path.extension().map(|e| e.to_string_lossy().into_owned())
    } else {
        None
    };

    let mut node = FileNode {
        name: name.to_string(),
        path: path.to_string_lossy().into_owned(),
        size: own_size,
        is_dir,
        file_count: if is_dir { 0 } else { 1 },
        dir_count: 0,
        children: Vec::new(),
        modified,
        extension,
        depth,
    };

    if !is_dir {
        return node;
    }

    let kids = by_parent.remove(path).unwrap_or_default();
    let mut children: Vec<FileNode> = kids
        .into_iter()
        .map(|e| {
            build_node(
                &e.path,
                &e.name,
                e.is_dir,
                e.size,
                e.modified,
                by_parent,
                depth + 1,
                max_depth,
            )
        })
        .collect();

    // Sort biggest first.
    children.sort_unstable_by(|a, b| b.size.cmp(&a.size));

    for c in &children {
        node.size += c.size;
        node.file_count += c.file_count;
        node.dir_count += c.dir_count + if c.is_dir { 1 } else { 0 };
    }

    // Trim serialized tree to max_depth to keep payload small.
    if depth < max_depth {
        node.children = children;
    }

    node
}
