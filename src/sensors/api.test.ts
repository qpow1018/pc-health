import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { getSensorSnapshot } from "./api";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

describe("getSensorSnapshot", () => {
  beforeEach(() => vi.mocked(invoke).mockReset());

  it("invokes the stable Tauri command with the selected scenario", async () => {
    vi.mocked(invoke).mockResolvedValue({ collectedAt: "now", devices: [] });
    await getSensorSnapshot("unsupported");
    expect(invoke).toHaveBeenCalledWith("get_sensor_snapshot", {
      scenario: "unsupported",
    });
  });
});
