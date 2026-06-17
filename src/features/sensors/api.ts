import { invoke } from "@tauri-apps/api/core";
import type { SensorDiagnostics, SensorSnapshot } from "./types";

export class SensorRuntimeUnavailableError extends Error {
  constructor() {
    super("Tauri runtime is unavailable.");
    this.name = "SensorRuntimeUnavailableError";
  }
}

export function getLiveSensorSnapshot() {
  if (!("__TAURI_INTERNALS__" in window)) {
    return Promise.reject(new SensorRuntimeUnavailableError());
  }

  return invoke<SensorSnapshot>("get_sensor_snapshot", { scenario: "live" });
}

export function getSensorDiagnostics() {
  if (!("__TAURI_INTERNALS__" in window)) {
    return Promise.reject(new SensorRuntimeUnavailableError());
  }

  return invoke<SensorDiagnostics>("get_sensor_diagnostics");
}
