import { act, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { SensorRuntimeUnavailableError } from "./api";
import { useSensorDiagnostics } from "./useSensorDiagnostics";
import type { SensorDiagnostics } from "./types";

const diagnostics: SensorDiagnostics = {
  collectedAt: "now",
  collector: "windows-powershell",
  durationMs: 12,
  rawPayload: '{"CpuUsage":37}',
  parsedTelemetry: { cpuUsage: 37 },
  snapshot: { collectedAt: "now", devices: [] },
};

describe("useSensorDiagnostics", () => {
  it("does not request diagnostics before user action", () => {
    const load = vi.fn().mockResolvedValue(diagnostics);

    renderHook(() => useSensorDiagnostics(load));

    expect(load).not.toHaveBeenCalled();
  });

  it("captures diagnostics and stores the latest result", async () => {
    const load = vi.fn().mockResolvedValue(diagnostics);

    const { result } = renderHook(() => useSensorDiagnostics(load));

    await act(async () => {
      await result.current.capture();
    });

    expect(load).toHaveBeenCalledTimes(1);
    expect(result.current.diagnostics).toEqual(diagnostics);
    expect(result.current.error).toBeNull();
    expect(result.current.isLoading).toBe(false);
  });

  it("sets loading while diagnostics are in flight", async () => {
    let resolveRequest: (value: SensorDiagnostics) => void = () => {};
    const load = vi.fn(
      () =>
        new Promise<SensorDiagnostics>((resolve) => {
          resolveRequest = resolve;
        }),
    );

    const { result } = renderHook(() => useSensorDiagnostics(load));

    act(() => {
      void result.current.capture();
    });
    expect(result.current.isLoading).toBe(true);

    await act(async () => {
      resolveRequest(diagnostics);
    });
    expect(result.current.isLoading).toBe(false);
  });

  it("explains diagnostics outside Tauri", async () => {
    const load = vi.fn().mockRejectedValue(new SensorRuntimeUnavailableError());

    const { result } = renderHook(() => useSensorDiagnostics(load));

    await act(async () => {
      await result.current.capture();
    });

    expect(result.current.error).toBe(
      "Tauri 앱에서 실행해야 진단 정보를 불러올 수 있습니다.",
    );
  });

  it("reports generic diagnostics failures", async () => {
    const load = vi.fn().mockRejectedValue(new Error("offline"));

    const { result } = renderHook(() => useSensorDiagnostics(load));

    await act(async () => {
      await result.current.capture();
    });

    expect(result.current.error).toBe("진단 정보를 불러오지 못했습니다.");
  });
});
