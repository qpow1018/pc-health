import { beforeEach, describe, expect, it, vi } from "vitest";
import { normalStatusFixture } from "./fixture";

const { invokeMock, isTauriMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
  isTauriMock: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
  isTauri: isTauriMock,
}));

import {
  canUseNetworkDiagnostics,
  getNetworkDiagnosticStatus,
} from "./api";

describe("network diagnostics API", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    isTauriMock.mockReset();
  });

  it("reads the current in-memory status", async () => {
    invokeMock.mockResolvedValue(normalStatusFixture);

    await expect(getNetworkDiagnosticStatus()).resolves.toBe(
      normalStatusFixture,
    );
    expect(invokeMock).toHaveBeenCalledWith("get_network_diagnostic_status");
  });

  it("reports browser mode without invoking Tauri", () => {
    isTauriMock.mockReturnValue(false);

    expect(canUseNetworkDiagnostics()).toBe(false);
    expect(invokeMock).not.toHaveBeenCalled();
  });
});
