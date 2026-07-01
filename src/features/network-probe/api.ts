import { invoke } from "@tauri-apps/api/core";
import type { NetworkProbeSnapshot } from "./types";

export function getNetworkProbeSnapshot() {
  return invoke<NetworkProbeSnapshot>("get_network_probe_snapshot");
}
