use crate::network::{
    domain::{NetworkDiagnosticStatus, NetworkProbeSnapshot},
    runtime::NetworkDiagnosticsRuntime,
    service::NetworkProbeCoordinator,
};

fn clone_diagnostic_status(
    runtime: &NetworkDiagnosticsRuntime,
) -> Result<NetworkDiagnosticStatus, String> {
    runtime.status()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::{domain::RuntimeAvailability, runtime::NetworkDiagnosticsRuntime};

    #[test]
    fn status_command_reads_memory_without_collecting_a_probe() {
        let runtime = NetworkDiagnosticsRuntime::unavailable_for_test();
        let status = clone_diagnostic_status(&runtime).unwrap();

        assert_eq!(status.availability, RuntimeAvailability::Unavailable);
    }
}
