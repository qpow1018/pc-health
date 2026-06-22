mod commands;
mod network;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::get_network_probe_snapshot
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
