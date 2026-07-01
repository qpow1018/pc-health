import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const { invokeMock, isTauriMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
  isTauriMock: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
  isTauri: isTauriMock,
}));

describe("network incidents api mock mode", () => {
  beforeEach(() => {
    window.history.replaceState(null, "", "/");
    invokeMock.mockReset();
    isTauriMock.mockReset();
  });

  afterEach(() => {
    window.history.replaceState(null, "", "/");
  });

  it("uses browser mock incidents when a mock scenario is selected", async () => {
    window.history.replaceState(null, "", "/?mock=external-incident");
    isTauriMock.mockReturnValue(false);
    const {
      canUseNetworkIncidents,
      getNetworkIncidents,
      getRecentNetworkIncidents,
    } = await import("./api");

    expect(canUseNetworkIncidents()).toBe(true);
    await expect(getRecentNetworkIncidents()).resolves.toHaveLength(3);
    await expect(getNetworkIncidents()).resolves.toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          status: "ongoing",
          area: "external",
        }),
      ]),
    );
    expect(invokeMock).not.toHaveBeenCalled();
  });

  it("keeps Tauri runtime on the native command path", async () => {
    window.history.replaceState(null, "", "/?mock=normal");
    isTauriMock.mockReturnValue(true);
    invokeMock.mockResolvedValue([]);
    const { canUseNetworkIncidents, getNetworkIncidents } = await import("./api");

    expect(canUseNetworkIncidents()).toBe(true);
    await expect(getNetworkIncidents()).resolves.toEqual([]);
    expect(invokeMock).toHaveBeenCalledWith("get_network_incidents");
  });

  it("rejects through the async API for mock error states", async () => {
    window.history.replaceState(null, "", "/?mock=error");
    isTauriMock.mockReturnValue(false);
    const { getRecentNetworkIncidents } = await import("./api");

    await expect(getRecentNetworkIncidents()).rejects.toThrow(
      "mock incident history unavailable",
    );
  });
});
