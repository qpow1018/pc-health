import { act, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useSensorSnapshot } from "./useSensorSnapshot";
import type { MockScenario, SensorSnapshot } from "./types";

const snapshot: SensorSnapshot = { collectedAt: "now", devices: [] };

describe("useSensorSnapshot", () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it("loads immediately and schedules only after the request settles", async () => {
    let resolveRequest: (value: SensorSnapshot) => void = () => {};
    const load = vi.fn(
      () =>
        new Promise<SensorSnapshot>((resolve) => {
          resolveRequest = resolve;
        }),
    );

    renderHook(() => useSensorSnapshot("normal", load));
    expect(load).toHaveBeenCalledTimes(1);

    await act(async () => vi.advanceTimersByTimeAsync(5_000));
    expect(load).toHaveBeenCalledTimes(1);

    await act(async () => resolveRequest(snapshot));
    await act(async () => vi.advanceTimersByTimeAsync(999));
    expect(load).toHaveBeenCalledTimes(1);

    await act(async () => vi.advanceTimersByTimeAsync(1));
    expect(load).toHaveBeenCalledTimes(2);
  });

  it("preserves the last snapshot when a later request fails", async () => {
    const load = vi
      .fn()
      .mockResolvedValueOnce(snapshot)
      .mockRejectedValueOnce(new Error("offline"));

    const { result } = renderHook(() => useSensorSnapshot("normal", load));
    await act(async () => Promise.resolve());
    expect(result.current.snapshot).toEqual(snapshot);

    await act(async () => vi.advanceTimersByTimeAsync(1_000));
    expect(result.current.snapshot).toEqual(snapshot);
    expect(result.current.error).toBe(
      "센서 데이터를 불러오지 못했습니다. 다시 시도합니다.",
    );
  });

  it("loads immediately when the scenario changes", async () => {
    const load = vi.fn().mockResolvedValue(snapshot);
    const { rerender } = renderHook(
      ({ scenario }: { scenario: MockScenario }) =>
        useSensorSnapshot(scenario, load),
      { initialProps: { scenario: "normal" as MockScenario } },
    );
    await act(async () => Promise.resolve());

    rerender({ scenario: "error" });
    expect(load).toHaveBeenLastCalledWith("error");
  });
});
