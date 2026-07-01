import { invoke, isTauri } from "@tauri-apps/api/core";
import {
  getMockNetworkIncidents,
  getNetworkMockScenario,
} from "@/features/network-mock/scenarios";
import type { NetworkIncident } from "./types";

export const canUseNetworkIncidents = () =>
  isTauri() || getNetworkMockScenario() !== null;

export const getRecentNetworkIncidents = () => {
  const scenario = getNetworkMockScenario();
  if (!isTauri() && scenario) {
    try {
      return Promise.resolve(getMockNetworkIncidents(scenario).slice(0, 3));
    } catch (caught) {
      return Promise.reject(caught);
    }
  }

  return invoke<NetworkIncident[]>("get_recent_network_incidents");
};

export const getNetworkIncidents = () => {
  const scenario = getNetworkMockScenario();
  if (!isTauri() && scenario) {
    try {
      return Promise.resolve(getMockNetworkIncidents(scenario));
    } catch (caught) {
      return Promise.reject(caught);
    }
  }

  return invoke<NetworkIncident[]>("get_network_incidents");
};
