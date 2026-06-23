import { invoke, isTauri } from "@tauri-apps/api/core";
import type { NetworkDiagnosticStatus } from "./types";

export const canUseNetworkDiagnostics = () => isTauri();
export const getNetworkDiagnosticStatus = () =>
  invoke<NetworkDiagnosticStatus>("get_network_diagnostic_status");
