mod commands;
mod network;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let coordinator = network::service::NetworkProbeCoordinator::platform();
            let incident_store = app
                .path()
                .app_data_dir()
                .ok()
                .and_then(|data_dir| {
                    std::fs::create_dir_all(&data_dir)
                        .ok()
                        .map(|_| data_dir.join("network.sqlite"))
                })
                .and_then(|path| network::incident_store::NetworkIncidentStore::open(path).ok());
            let recorder = incident_store.as_ref().map(|store| {
                network::incident_recorder::NetworkIncidentRecorder::new(store.clone())
            });
            let runtime = network::runtime::NetworkDiagnosticsRuntime::platform(
                coordinator.clone(),
                recorder,
            );
            app.manage(coordinator);
            if let Some(store) = incident_store {
                app.manage(store);
            }
            app.manage(runtime);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_network_probe_snapshot,
            commands::get_network_diagnostic_status,
            commands::get_recent_network_incidents,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
