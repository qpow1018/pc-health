import { invoke, isTauri } from "@tauri-apps/api/core";
import type { NetworkIncident } from "./types";

export const canUseNetworkIncidents = () => isTauri();
export const getRecentNetworkIncidents = () =>
  invoke<NetworkIncident[]>("get_recent_network_incidents");
export const getNetworkIncidents = () =>
  invoke<NetworkIncident[]>("get_network_incidents");
