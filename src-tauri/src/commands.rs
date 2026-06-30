use crate::network::{
    domain::{NetworkDiagnosticStatus, NetworkIncident, NetworkProbeSnapshot},
    incident_store::NetworkIncidentStore,
    runtime::NetworkDiagnosticsRuntime,
    service::NetworkProbeCoordinator,
};

pub struct NetworkIncidentHistoryState {
    store: Option<NetworkIncidentStore>,
}

impl NetworkIncidentHistoryState {
    pub fn new(store: Option<NetworkIncidentStore>) -> Self {
        Self { store }
    }
}

fn clone_diagnostic_status(
    runtime: &NetworkDiagnosticsRuntime,
) -> Result<NetworkDiagnosticStatus, String> {
    runtime.status()
}

fn latest_network_incidents(
    state: &NetworkIncidentHistoryState,
) -> Result<Vec<NetworkIncident>, String> {
    let Some(store) = &state.store else {
        return Err("network incident history unavailable".into());
    };
    store.recent_incidents(3)
}

#[tauri::command]
pub async fn get_network_probe_snapshot(
    coordinator: tauri::State<'_, NetworkProbeCoordinator>,
) -> Result<NetworkProbeSnapshot, String> {
    let coordinator = coordinator.inner().clone();
    tauri::async_runtime::spawn_blocking(move || coordinator.collect_full())
        .await
        .map_err(|error| format!("network probe task failed: {error}"))?
}

#[tauri::command]
pub fn get_network_diagnostic_status(
    runtime: tauri::State<'_, NetworkDiagnosticsRuntime>,
) -> Result<NetworkDiagnosticStatus, String> {
    clone_diagnostic_status(runtime.inner())
}

#[tauri::command]
pub fn get_recent_network_incidents(
    state: tauri::State<'_, NetworkIncidentHistoryState>,
) -> Result<Vec<NetworkIncident>, String> {
    latest_network_incidents(state.inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::{
        domain::{
            DiagnosticArea, DiagnosticEvidence, EvidenceSource, EvidenceStatus, RuntimeAvailability,
        },
        incident_store::NetworkIncidentStore,
        runtime::NetworkDiagnosticsRuntime,
    };

    #[test]
    fn status_command_reads_memory_without_collecting_a_probe() {
        let runtime = NetworkDiagnosticsRuntime::unavailable_for_test();
        let status = clone_diagnostic_status(&runtime).unwrap();

        assert_eq!(status.availability, RuntimeAvailability::Unavailable);
    }

    #[test]
    fn recent_incidents_command_limits_to_three() {
        let store = NetworkIncidentStore::open_in_memory().unwrap();
        for index in 0..4 {
            store
                .create_incident(
                    DiagnosticArea::External,
                    &format!("2026-06-30T00:00:0{index}Z"),
                    "외부 연결 구간에서 이상 근거가 반복 확인되었습니다.",
                    &[DiagnosticEvidence {
                        source: EvidenceSource::HttpGoogle,
                        status: EvidenceStatus::Timeout,
                        checked_at: Some("2026-06-30T00:00:00Z".into()),
                        duration_ms: Some(10),
                        detail: Some("test".into()),
                    }],
                )
                .unwrap();
        }

        let state = NetworkIncidentHistoryState::new(Some(store));

        assert_eq!(latest_network_incidents(&state).unwrap().len(), 3);
    }

    #[test]
    fn recent_incidents_command_returns_unavailable_when_store_is_absent() {
        let state = NetworkIncidentHistoryState::new(None);

        assert_eq!(
            latest_network_incidents(&state).unwrap_err(),
            "network incident history unavailable"
        );
    }
}
