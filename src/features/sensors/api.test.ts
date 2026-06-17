import { invoke } from "@tauri-apps/api/core";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  getLiveSensorSnapshot,
  getSensorDiagnostics,
  SensorRuntimeUnavailableError,
} from "./api";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

describe("getSensorSnapshot", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
    Object.defineProperty(window, "__TAURI_INTERNALS__", {
      configurable: true,
      value: {},
    });
  });

  afterEach(() => {
    Reflect.deleteProperty(window, "__TAURI_INTERNALS__");
  });

  it("invokes the stable Tauri command with the selected scenario", async () => {
    vi.mocked(invoke).mockResolvedValue({ collectedAt: "now", devices: [] });
    await getLiveSensorSnapshot();
    expect(invoke).toHaveBeenCalledWith("get_sensor_snapshot", {
      scenario: "live",
    });
  });

  it("rejects clearly when the Tauri runtime is unavailable", async () => {
    Reflect.deleteProperty(window, "__TAURI_INTERNALS__");

    await expect(getLiveSensorSnapshot()).rejects.toBeInstanceOf(
      SensorRuntimeUnavailableError,
    );
    expect(invoke).not.toHaveBeenCalled();
  });

  it("invokes the diagnostics command when Tauri is available", async () => {
    vi.mocked(invoke).mockResolvedValue({
      collectedAt: "now",
      collector: "windows-powershell",
      durationMs: 10,
      snapshot: { collectedAt: "now", devices: [] },
    });

    await getSensorDiagnostics();

    expect(invoke).toHaveBeenCalledWith("get_sensor_diagnostics");
  });

  it("rejects diagnostics clearly outside Tauri", async () => {
    Reflect.deleteProperty(window, "__TAURI_INTERNALS__");

    await expect(getSensorDiagnostics()).rejects.toBeInstanceOf(
      SensorRuntimeUnavailableError,
    );
    expect(invoke).not.toHaveBeenCalled();
  });
});
