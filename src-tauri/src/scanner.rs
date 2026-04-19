use crate::tree::{FileNode, ScanProgress};
use jwalk::{Parallelism, WalkDirGeneric};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
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
        self.cancelled.store(true, Ordering::SeqCst);
    }
}

/// Raw entry collected during walk (flat).
struct Entry {
    path: PathBuf,
    parent: Option<PathBuf>,
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
    let root = root.canonicalize().map_err(|e| format!("cannot resolve path: {e}"))?;
    let cancelled = handle.cancelled.clone();
    let scanned_files = handle.scanned_files.clone();
    let scanned_bytes = handle.scanned_bytes.clone();

    // Shared collector across rayon threads.
    let entries: Arc<Mutex<Vec<Entry>>> = Arc::new(Mutex::new(Vec::with_capacity(4096)));
    let last_emit: Arc<Mutex<Instant>> = Arc::new(Mutex::new(Instant::now()));
    let current_path: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));

    let walker = WalkDirGeneric::<((), ())>::new(&root)
        .parallelism(Parallelism::RayonNewPool(0)) // 0 = auto = all cores
        .skip_hidden(!show_hidden)
        .follow_links(false);

    for entry in walker {
        if cancelled.load(Ordering::Relaxed) {
            return Err("cancelled".into());
        }
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue, // permission denied, broken symlink, etc.
        };

        let path = entry.path();
        let is_dir = entry.file_type().is_dir();

        // Skip the root itself — we'll synthesize it.
        if path == root {
            continue;
        }

        let (size, modified) = if is_dir {
            (0u64, None)
        } else {
            let md = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            let size = md.len();
            let modified = md
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs());

            scanned_files.fetch_add(1, Ordering::Relaxed);
            scanned_bytes.fetch_add(size, Ordering::Relaxed);
            (size, modified)
        };

        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();

        let parent = path.parent().map(|p| p.to_path_buf());

        entries.lock().push(Entry {
            path: path.clone(),
            parent,
            name,
            size,
            is_dir,
            modified,
        });

        // Throttle progress emits to ~5/sec.
        let now = Instant::now();
        let should_emit = {
            let mut last = last_emit.lock();
            if now.duration_since(*last).as_millis() >= 200 {
                *last = now;
                true
            } else {
                false
            }
        };
        if should_emit {
            *current_path.lock() = path.to_string_lossy().into_owned();
            let _ = app.emit(
                "scan:progress",
                ScanProgress {
                    scanned_files: scanned_files.load(Ordering::Relaxed),
                    scanned_bytes: scanned_bytes.load(Ordering::Relaxed),
                    current_path: current_path.lock().clone(),
                },
            );
        }
    }

    if cancelled.load(Ordering::Relaxed) {
        return Err("cancelled".into());
    }

    let flat = Arc::try_unwrap(entries)
        .map_err(|_| "entries still referenced".to_string())?
        .into_inner();

    // Build tree from flat entries.
    let tree = build_tree(&root, flat, max_depth);
    Ok(tree)
}

/// Build a nested FileNode tree from the flat entries, summing sizes up.
fn build_tree(root: &Path, entries: Vec<Entry>, max_depth: u32) -> FileNode {
    // Group by parent path.
    let mut by_parent: HashMap<PathBuf, Vec<Entry>> = HashMap::new();
    for e in entries {
        let parent = e.parent.clone().unwrap_or_else(|| root.to_path_buf());
        by_parent.entry(parent).or_default().push(e);
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
        id: format!("{depth}:{}", path.to_string_lossy()),
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
    children.sort_by(|a, b| b.size.cmp(&a.size));

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
