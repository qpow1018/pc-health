import { useEffect, useState } from "react";
import { getSensorSnapshot, SensorRuntimeUnavailableError } from "./api";
import type { MockScenario, SensorSnapshot } from "./types";

type SnapshotLoader = (scenario: MockScenario) => Promise<SensorSnapshot>;

export function useSensorSnapshot(
  scenario: MockScenario,
  load: SnapshotLoader = getSensorSnapshot,
) {
  const [snapshot, setSnapshot] = useState<SensorSnapshot | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    let timer: number | undefined;

    const poll = async () => {
      try {
        const next = await load(scenario);
        if (!active) return;
        setSnapshot(next);
        setError(null);
      } catch (caughtError) {
        if (!active) return;
        if (caughtError instanceof SensorRuntimeUnavailableError) {
          setError("Tauri 앱에서 실행해야 센서 데이터를 불러올 수 있습니다.");
          return;
        }
        setError("센서 데이터를 불러오지 못했습니다. 다시 시도합니다.");
      }

      if (active) timer = window.setTimeout(poll, 1_000);
    };

    void poll();

    return () => {
      active = false;
      if (timer !== undefined) window.clearTimeout(timer);
    };
  }, [scenario, load]);

  return { snapshot, error };
}
