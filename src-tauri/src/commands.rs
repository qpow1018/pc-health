use crate::network::{domain::NetworkProbeSnapshot, service::NetworkProbeService};

#[tauri::command]
pub async fn get_network_probe_snapshot() -> Result<NetworkProbeSnapshot, String> {
    tauri::async_runtime::spawn_blocking(|| NetworkProbeService::platform().collect())
        .await
        .map_err(|error| format!("network probe task failed: {error}"))
}
