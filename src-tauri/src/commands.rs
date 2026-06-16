use std::sync::Mutex;

use chrono::Utc;
use tauri::State;

use crate::collector::MockScenario;
use crate::domain::SensorSnapshot;
use crate::service::SnapshotService;

pub struct AppState {
    snapshots: Mutex<SnapshotService>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            snapshots: Mutex::new(SnapshotService::default()),
        }
    }
}

#[tauri::command]
pub fn get_sensor_snapshot(
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
