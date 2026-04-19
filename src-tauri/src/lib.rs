mod commands;
mod scanner;
mod tree;

use commands::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::list_volumes,
            commands::start_scan,
            commands::cancel_scan,
            commands::trash_path,
            commands::permanent_delete,
            commands::is_path_protected,
            commands::home_dir,
            commands::check_for_updates,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
