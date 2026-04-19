use crate::scanner::{self, ScanHandle};
use crate::tree::{FileNode, Volume};
use parking_lot::Mutex;
use std::path::PathBuf;
use std::sync::Arc;
use sysinfo::Disks;
use tauri::{AppHandle, State};

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
pub fn start_scan(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
    show_hidden: bool,
    max_depth: Option<u32>,
) -> Result<FileNode, String> {
    // Cancel any in-flight scan.
    if let Some(prev) = state.active_scan.lock().take() {
        prev.cancel();
    }

    let handle = Arc::new(ScanHandle::new());
    *state.active_scan.lock() = Some(handle.clone());

    let root = PathBuf::from(&path);
    let depth = max_depth.unwrap_or(6);

    let result = scanner::scan(&root, app, handle.clone(), show_hidden, depth);

    // Clear state if this scan is still the active one.
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
    trash::delete(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn home_dir() -> Option<String> {
    std::env::var("HOME")
        .ok()
        .or_else(|| std::env::var("USERPROFILE").ok())
}
