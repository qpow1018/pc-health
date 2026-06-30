use crate::network::{
    domain::{NetworkDiagnosticStatus, NetworkIncident, NetworkProbeSnapshot},
    incident_store::NetworkIncidentStore,
    runtime::NetworkDiagnosticsRuntime,
    service::NetworkProbeCoordinator,
};

fn clone_diagnostic_status(
    runtime: &NetworkDiagnosticsRuntime,
) -> Result<NetworkDiagnosticStatus, String> {
    runtime.status()
}

fn latest_network_incidents(store: &NetworkIncidentStore) -> Result<Vec<NetworkIncident>, String> {
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
    store: tauri::State<'_, NetworkIncidentStore>,
) -> Result<Vec<NetworkIncident>, String> {
    latest_network_incidents(store.inner())
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

        assert_eq!(latest_network_incidents(&store).unwrap().len(), 3);
    }
}
