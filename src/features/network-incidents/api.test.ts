import { beforeEach, describe, expect, it, vi } from "vitest";
import { resolvedIncidentFixture } from "./fixture";

const { invokeMock, isTauriMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
  isTauriMock: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
  isTauri: isTauriMock,
}));

import { canUseNetworkIncidents, getRecentNetworkIncidents } from "./api";

describe("network incidents API", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    isTauriMock.mockReset();
  });

  it("reads recent incidents from Tauri", async () => {
    invokeMock.mockResolvedValue([resolvedIncidentFixture]);

    await expect(getRecentNetworkIncidents()).resolves.toEqual([
      resolvedIncidentFixture,
    ]);
    expect(invokeMock).toHaveBeenCalledWith("get_recent_network_incidents");
  });

  it("reports browser mode without invoking Tauri", () => {
    isTauriMock.mockReturnValue(false);

    expect(canUseNetworkIncidents()).toBe(false);
    expect(invokeMock).not.toHaveBeenCalled();
  });
});
