import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const { invokeMock, isTauriMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
  isTauriMock: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
  isTauri: isTauriMock,
}));

describe("network diagnostics api mock mode", () => {
  beforeEach(() => {
    window.history.replaceState(null, "", "/");
    invokeMock.mockReset();
    isTauriMock.mockReset();
  });

  afterEach(() => {
    window.history.replaceState(null, "", "/");
  });

  it("uses browser mock data when a mock scenario is selected", async () => {
    window.history.replaceState(null, "", "/?mock=dns-incident");
    isTauriMock.mockReturnValue(false);
    const { canUseNetworkDiagnostics, getNetworkDiagnosticStatus } = await import(
      "./api"
    );

    expect(canUseNetworkDiagnostics()).toBe(true);
    await expect(getNetworkDiagnosticStatus()).resolves.toMatchObject({
      availability: "running",
      lifecycle: "incident",
      suspectedArea: "dns",
    });
    expect(invokeMock).not.toHaveBeenCalled();
  });

  it("keeps Tauri runtime on the native command path", async () => {
    window.history.replaceState(null, "", "/?mock=gateway-incident");
    isTauriMock.mockReturnValue(true);
    invokeMock.mockResolvedValue({ availability: "running" });
    const { canUseNetworkDiagnostics, getNetworkDiagnosticStatus } = await import(
      "./api"
    );

    expect(canUseNetworkDiagnostics()).toBe(true);
    await expect(getNetworkDiagnosticStatus()).resolves.toEqual({
      availability: "running",
    });
    expect(invokeMock).toHaveBeenCalledWith("get_network_diagnostic_status");
  });
});
