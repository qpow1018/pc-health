import { invoke, isTauri } from "@tauri-apps/api/core";
import {
  getMockNetworkDiagnosticStatus,
  getNetworkMockScenario,
} from "@/features/network-mock/scenarios";
import type { NetworkDiagnosticStatus } from "./types";

export const canUseNetworkDiagnostics = () =>
  isTauri() || getNetworkMockScenario() !== null;

export const getNetworkDiagnosticStatus = () => {
  const scenario = getNetworkMockScenario();
  if (!isTauri() && scenario) {
    return Promise.resolve(getMockNetworkDiagnosticStatus(scenario));
  }

  return invoke<NetworkDiagnosticStatus>("get_network_diagnostic_status");
};
