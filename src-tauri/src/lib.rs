mod commands;
mod network;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let coordinator = network::service::NetworkProbeCoordinator::platform();
    let runtime = network::runtime::NetworkDiagnosticsRuntime::platform(coordinator.clone());
    tauri::Builder::default()
        .manage(coordinator)
        .manage(runtime)
        .invoke_handler(tauri::generate_handler![
            commands::get_network_probe_snapshot,
            commands::get_network_diagnostic_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
