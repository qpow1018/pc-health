import { useEffect, useState } from "react";
import { getLiveSensorSnapshot, SensorRuntimeUnavailableError } from "./api";
import { getMockSensorSnapshot } from "./mockSnapshot";
import type { MockScenario, SensorMode, SensorSnapshot } from "./types";

type SnapshotLoaders = {
  live: () => Promise<SensorSnapshot>;
  development: (
    scenario: MockScenario,
    sampleIndex: number,
  ) => Promise<SensorSnapshot>;
};

const defaultLoaders: SnapshotLoaders = {
  live: getLiveSensorSnapshot,
  development: getMockSensorSnapshot,
};

export function useSensorSnapshot(
  mode: SensorMode,
  scenario: MockScenario,
  loaders: SnapshotLoaders = defaultLoaders,
) {
  const [snapshot, setSnapshot] = useState<SensorSnapshot | null>(null);
  const [error, setError] = useState<string | null>(null);
  const { live, development } = loaders;

  useEffect(() => {
    let active = true;
    let sampleIndex = 0;
    let timer: number | undefined;

    const poll = async () => {
      try {
        const next =
          mode === "live"
            ? await live()
            : await development(scenario, sampleIndex);
        if (!active) return;
        sampleIndex += 1;
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
  }, [mode, scenario, live, development]);

  return { snapshot, error };
}
