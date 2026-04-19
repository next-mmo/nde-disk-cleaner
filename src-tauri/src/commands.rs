use crate::scanner::{self, ScanHandle};
use crate::tree::{FileNode, Volume};
use parking_lot::Mutex;
use serde::Serialize;
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
    if let Some(reason) = protected_reason(&target) {
        return Err(format!(
            "Refusing to delete {}: {}",
            target.display(),
            reason
        ));
    }
    trash::delete(&target).map_err(|e| e.to_string())
}

/// Permanently delete a file or directory (bypasses Trash).
/// Still refuses to touch protected paths.
#[tauri::command]
pub fn permanent_delete(path: String) -> Result<(), String> {
    let target = PathBuf::from(&path);
    if let Some(reason) = protected_reason(&target) {
        return Err(format!(
            "Refusing to delete {}: {}",
            target.display(),
            reason
        ));
    }
    if target.is_dir() {
        std::fs::remove_dir_all(&target).map_err(|e| e.to_string())
    } else {
        std::fs::remove_file(&target).map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub fn is_path_protected(path: String) -> Option<String> {
    protected_reason(&PathBuf::from(&path))
}

/// Returns `Some(reason)` when deleting `path` would endanger the OS, destroy
/// a whole volume, or wipe every user's home. Reason is shown in the UI so the
/// user understands why the action is blocked.
fn protected_reason(path: &std::path::Path) -> Option<String> {
    // Canonicalize when possible so callers can't bypass the check with `.`,
    // `..`, or symlinks. Fall back to the raw path on failure (broken symlink,
    // missing file) — better to over-refuse than under-refuse.
    let canon = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());

    if canon.as_os_str().is_empty() || canon.parent().is_none() {
        return Some("this is the filesystem root".into());
    }

    // Block deleting a mount point itself (e.g. `/`, `/Volumes/USB`, `C:\`).
    // Trashing a mount root tries to remove an entire volume from under the OS.
    for disk in Disks::new_with_refreshed_list().iter() {
        if canon == disk.mount_point() {
            return Some("this is a disk mount point — trashing it would remove the whole volume".into());
        }
    }

    // Recursive: deleting the dir *or anything inside it* can brick the OS.
    #[cfg(target_os = "macos")]
    let recursive: &[&str] = &[
        "/System", "/Library", "/bin", "/sbin", "/usr", "/private",
        "/etc", "/var", "/Applications", "/cores", "/opt", "/Volumes",
    ];
    #[cfg(target_os = "linux")]
    let recursive: &[&str] = &[
        "/bin", "/sbin", "/boot", "/dev", "/etc", "/lib", "/lib32", "/lib64",
        "/libx32", "/proc", "/root", "/run", "/srv", "/sys", "/usr", "/var",
    ];
    #[cfg(target_os = "windows")]
    let recursive: &[&str] = &[
        r"C:\Windows",
        r"C:\Program Files",
        r"C:\Program Files (x86)",
        r"C:\ProgramData",
        r"C:\System Volume Information",
        r"C:\$Recycle.Bin",
        r"C:\Recovery",
        r"C:\Boot",
    ];

    // Exact-only: only the directory itself is protected, not its contents.
    // `/Users` holds every user's home — deleting it destroys all user data,
    // but a subfolder like `/Users/alice/Downloads` is fine to trash.
    #[cfg(target_os = "macos")]
    let exact: &[&str] = &["/Users", "/home"];
    #[cfg(target_os = "linux")]
    let exact: &[&str] = &["/home"];
    #[cfg(target_os = "windows")]
    let exact: &[&str] = &[r"C:\Users"];

    for root in recursive {
        let root_path = std::path::Path::new(root);
        if path_matches(&canon, root_path, true) {
            return Some(format!(
                "{} is a system directory; removing it can prevent the OS from booting",
                root
            ));
        }
    }

    for root in exact {
        let root_path = std::path::Path::new(root);
        if path_matches(&canon, root_path, false) {
            return Some(format!(
                "{} contains every user's home folder — deleting it would destroy all user data",
                root
            ));
        }
    }

    None
}

fn path_matches(canon: &std::path::Path, root: &std::path::Path, recursive: bool) -> bool {
    if canon == root {
        return true;
    }
    if recursive && canon.starts_with(root) {
        return true;
    }
    #[cfg(target_os = "windows")]
    {
        let canon_lower = canon.to_string_lossy().to_ascii_lowercase();
        let root_lower = root.to_string_lossy().to_ascii_lowercase();
        if canon_lower == root_lower {
            return true;
        }
        if recursive && canon_lower.starts_with(&format!("{}\\", root_lower)) {
            return true;
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

// ── Check for updates ──────────────────────────────────────────────

#[derive(Serialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub has_update: bool,
    pub release_url: String,
    pub release_notes: String,
}

#[tauri::command]
pub async fn check_for_updates(app: tauri::AppHandle) -> Result<UpdateInfo, String> {
    let current = app
        .config()
        .version
        .clone()
        .unwrap_or_else(|| "0.0.0".into());

    let client = reqwest::Client::builder()
        .user_agent("nde-disk-cleaner")
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get("https://api.github.com/repos/next-mmo/nde-disk-cleaner/releases/latest")
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("GitHub API returned {}", resp.status()));
    }

    let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

    let tag = body["tag_name"]
        .as_str()
        .unwrap_or("0.0.0")
        .trim_start_matches('v')
        .to_string();

    let release_url = body["html_url"]
        .as_str()
        .unwrap_or("https://github.com/next-mmo/nde-disk-cleaner/releases")
        .to_string();

    let release_notes = body["body"].as_str().unwrap_or("").to_string();

    let has_update = version_is_newer(&current, &tag);

    Ok(UpdateInfo {
        current_version: current,
        latest_version: tag,
        has_update,
        release_url,
        release_notes,
    })
}

/// Simple semver comparison: returns true when `latest` is strictly newer than `current`.
fn version_is_newer(current: &str, latest: &str) -> bool {
    let parse = |v: &str| -> Vec<u64> {
        v.split('.')
            .filter_map(|p| p.parse::<u64>().ok())
            .collect()
    };
    let cur = parse(current);
    let lat = parse(latest);
    for i in 0..3 {
        let c = cur.get(i).copied().unwrap_or(0);
        let l = lat.get(i).copied().unwrap_or(0);
        if l > c {
            return true;
        }
        if l < c {
            return false;
        }
    }
    false
}
