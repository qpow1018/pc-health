mod commands;
mod network;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let coordinator = network::service::NetworkProbeCoordinator::platform();
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let store = network::incident_store::NetworkIncidentStore::open(
                data_dir.join("network.sqlite"),
            )
            .map_err(std::io::Error::other)?;
            let recorder = network::incident_recorder::NetworkIncidentRecorder::new(store);
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
