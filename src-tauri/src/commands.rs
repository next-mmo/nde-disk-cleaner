use crate::scanner::{self, ScanHandle};
use crate::tree::{FileNode, Volume};
use parking_lot::Mutex;
use std::path::PathBuf;
use std::sync::Arc;
use sysinfo::Disks;
use tauri::State;

/// Held in Tauri state; allows cancelling the active scan.
pub struct AppState {
    pub active_scan: Mutex<Option<Arc<ScanHandle>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            active_scan: Mutex::new(None),
        }
    }
}

#[tauri::command]
pub fn list_volumes() -> Vec<Volume> {
    let disks = Disks::new_with_refreshed_list();
    disks
        .iter()
        .map(|d| {
            let total = d.total_space();
            let avail = d.available_space();
            Volume {
                name: d.name().to_string_lossy().into_owned(),
                mount_point: d.mount_point().to_string_lossy().into_owned(),
                total_bytes: total,
                available_bytes: avail,
                used_bytes: total.saturating_sub(avail),
                file_system: d.file_system().to_string_lossy().into_owned(),
                is_removable: d.is_removable(),
            }
        })
        .collect()
}

#[tauri::command]
pub async fn start_scan(
    app: tauri::AppHandle,
    path: String,
    show_hidden: bool,
    max_depth: Option<u32>,
) -> Result<FileNode, String> {
    use tauri::Manager;
    let handle = {
        let state = app.state::<AppState>();
        // Cancel any in-flight scan.
        if let Some(prev) = state.active_scan.lock().take() {
            prev.cancel();
        }
        let h = Arc::new(ScanHandle::new());
        *state.active_scan.lock() = Some(h.clone());
        h
    };

    let root = PathBuf::from(&path);
    let depth = max_depth.unwrap_or(6);

    let handle_clone = handle.clone();
    let app_clone = app.clone();

    // Run the heavy scan on a blocking thread so the main/webview thread stays responsive.
    let result = tauri::async_runtime::spawn_blocking(move || {
        scanner::scan(&root, app_clone, handle_clone, show_hidden, depth)
    })
    .await
    .map_err(|e| format!("scan task failed: {e}"))?;

    // Clear state safely after await
    let state = app.state::<AppState>();
    let mut active = state.active_scan.lock();
    if let Some(h) = active.as_ref() {
        if Arc::ptr_eq(h, &handle) {
            *active = None;
        }
    }

    result
}

#[tauri::command]
pub fn cancel_scan(state: State<'_, AppState>) {
    if let Some(h) = state.active_scan.lock().as_ref() {
        h.cancel();
    }
}

#[tauri::command]
pub fn trash_path(path: String) -> Result<(), String> {
    let target = PathBuf::from(&path);
    if is_protected_system_path(&target) {
        return Err(format!(
            "Refusing to delete protected system path: {}. Removing it could prevent the OS from booting.",
            target.display()
        ));
    }
    trash::delete(&target).map_err(|e| e.to_string())
}

/// Returns true if `path` is — or lives inside — a directory critical to the OS,
/// or if it *is* a disk mount point. Deleting any of these can leave the machine
/// unbootable or destroy an entire volume, so we hard-block the trash command.
fn is_protected_system_path(path: &std::path::Path) -> bool {
    // Canonicalize when possible so callers can't bypass the check with `.`,
    // `..`, or symlinks. Fall back to the raw path on failure (broken symlink,
    // missing file) — better to over-refuse than under-refuse.
    let canon = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());

    // Filesystem root itself is always protected.
    if canon.as_os_str().is_empty() || canon.parent().is_none() {
        return true;
    }

    // Block deleting a mount point itself (e.g. `/`, `/Volumes/USB`, `C:\`).
    // Trashing a mount root tries to remove an entire volume from under the OS.
    for disk in Disks::new_with_refreshed_list().iter() {
        let mount = disk.mount_point();
        if canon == mount {
            return true;
        }
    }

    #[cfg(target_os = "macos")]
    let roots: &[&str] = &[
        "/System", "/Library", "/bin", "/sbin", "/usr", "/private",
        "/etc", "/var", "/Applications", "/cores", "/opt", "/Volumes",
    ];
    #[cfg(target_os = "linux")]
    let roots: &[&str] = &[
        "/bin", "/sbin", "/boot", "/dev", "/etc", "/lib", "/lib32", "/lib64",
        "/libx32", "/proc", "/root", "/run", "/srv", "/sys", "/usr", "/var",
    ];
    #[cfg(target_os = "windows")]
    let roots: &[&str] = &[
        r"C:\Windows",
        r"C:\Program Files",
        r"C:\Program Files (x86)",
        r"C:\ProgramData",
        r"C:\System Volume Information",
        r"C:\$Recycle.Bin",
        r"C:\Recovery",
        r"C:\Boot",
    ];

    for root in roots {
        let root_path = std::path::Path::new(root);
        if canon == root_path || canon.starts_with(root_path) {
            return true;
        }
        // Windows paths are case-insensitive; compare lowercased forms too.
        #[cfg(target_os = "windows")]
        {
            let canon_lower = canon.to_string_lossy().to_ascii_lowercase();
            let root_lower = root.to_ascii_lowercase();
            if canon_lower == root_lower
                || canon_lower.starts_with(&format!("{}\\", root_lower))
            {
                return true;
            }
        }
    }
    false
}

#[tauri::command]
pub fn home_dir() -> Option<String> {
    std::env::var("HOME")
        .ok()
        .or_else(|| std::env::var("USERPROFILE").ok())
}
