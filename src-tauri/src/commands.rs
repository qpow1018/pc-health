use std::sync::{Arc, Mutex};

use chrono::Utc;
use tauri::{async_runtime::spawn_blocking, State};

use crate::collector::MockScenario;
use crate::domain::{SensorDiagnostics, SensorSnapshot};
use crate::service::SnapshotService;

pub struct AppState {
    snapshots: Arc<Mutex<SnapshotService>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            snapshots: Arc::new(Mutex::new(SnapshotService::default())),
        }
    }
}

#[tauri::command]
pub async fn get_sensor_snapshot(
    scenario: MockScenario,
    state: State<'_, AppState>,
) -> Result<SensorSnapshot, String> {
    let now = Utc::now();
    let snapshots = Arc::clone(&state.snapshots);

    spawn_blocking(move || {
        let mut service = snapshots
            .lock()
            .map_err(|_| "센서 수집 상태를 잠그지 못했습니다.".to_string())?;

        Ok(service.snapshot_at(scenario, now.to_rfc3339(), now.timestamp()))
    })
    .await
    .map_err(|error| format!("센서 수집 작업을 완료하지 못했습니다: {error}"))?
}

#[tauri::command]
pub async fn get_sensor_diagnostics(
    state: State<'_, AppState>,
) -> Result<SensorDiagnostics, String> {
    let now = Utc::now();
    let snapshots = Arc::clone(&state.snapshots);

    spawn_blocking(move || {
        let mut service = snapshots
            .lock()
            .map_err(|_| "센서 수집 상태를 잠그지 못했습니다.".to_string())?;

        Ok(service.diagnostics_at(now.to_rfc3339()))
    })
    .await
    .map_err(|error| format!("센서 진단 작업을 완료하지 못했습니다: {error}"))?
}
