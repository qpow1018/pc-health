import { invoke } from "@tauri-apps/api/core";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { getSensorSnapshot, SensorRuntimeUnavailableError } from "./api";

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
    await getSensorSnapshot("unsupported");
    expect(invoke).toHaveBeenCalledWith("get_sensor_snapshot", {
      scenario: "unsupported",
    });
  });

  it("rejects clearly when the Tauri runtime is unavailable", async () => {
    Reflect.deleteProperty(window, "__TAURI_INTERNALS__");

    await expect(getSensorSnapshot("normal")).rejects.toBeInstanceOf(
      SensorRuntimeUnavailableError,
    );
    expect(invoke).not.toHaveBeenCalled();
  });
});
