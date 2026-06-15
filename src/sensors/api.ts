import { invoke } from "@tauri-apps/api/core";
import type { MockScenario, SensorSnapshot } from "./types";

export function getSensorSnapshot(scenario: MockScenario) {
  return invoke<SensorSnapshot>("get_sensor_snapshot", { scenario });
}
