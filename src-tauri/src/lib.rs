mod collector;
mod domain;
mod service;
mod warning;

use std::sync::Mutex;

use chrono::Utc;
use collector::MockScenario;
use domain::SensorSnapshot;
use service::SnapshotService;
use tauri::State;

struct AppState {
    snapshots: Mutex<SnapshotService>,
}

#[tauri::command]
fn get_sensor_snapshot(
    scenario: MockScenario,
    state: State<'_, AppState>,
) -> Result<SensorSnapshot, String> {
    let now = Utc::now();
    let mut service = state
        .snapshots
        .lock()
        .map_err(|_| "센서 수집 상태를 잠그지 못했습니다.".to_string())?;

    Ok(service.snapshot_at(scenario, now.to_rfc3339(), now.timestamp()))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            snapshots: Mutex::new(SnapshotService::default()),
        })
        .invoke_handler(tauri::generate_handler![get_sensor_snapshot])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
