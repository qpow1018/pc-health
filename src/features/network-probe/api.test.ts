import { beforeEach, describe, expect, it, vi } from "vitest";
import { getNetworkProbeSnapshot } from "./api";

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

describe("getNetworkProbeSnapshot", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("invokes the one-shot network command", async () => {
    const snapshot = { collector: "windows-native" };
    invokeMock.mockResolvedValue(snapshot);

    await expect(getNetworkProbeSnapshot()).resolves.toBe(snapshot);
    expect(invokeMock).toHaveBeenCalledWith("get_network_probe_snapshot");
  });
});
