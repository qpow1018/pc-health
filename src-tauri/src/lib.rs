mod commands;
mod network;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let coordinator = network::service::NetworkProbeCoordinator::platform();
            let recorder = app
                .path()
                .app_data_dir()
                .ok()
                .and_then(|data_dir| {
                    std::fs::create_dir_all(&data_dir)
                        .ok()
                        .map(|_| data_dir.join("network.sqlite"))
                })
                .and_then(|path| network::incident_store::NetworkIncidentStore::open(path).ok())
                .map(network::incident_recorder::NetworkIncidentRecorder::new);
            let runtime = network::runtime::NetworkDiagnosticsRuntime::platform(
                coordinator.clone(),
                recorder,
            );
            app.manage(coordinator);
            app.manage(runtime);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_network_probe_snapshot,
            commands::get_network_diagnostic_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
