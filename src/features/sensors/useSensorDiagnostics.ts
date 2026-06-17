import { useState } from "react";
import { getSensorDiagnostics, SensorRuntimeUnavailableError } from "./api";
import type { SensorDiagnostics } from "./types";

type DiagnosticsLoader = () => Promise<SensorDiagnostics>;

export function useSensorDiagnostics(
  loader: DiagnosticsLoader = getSensorDiagnostics,
) {
  const [diagnostics, setDiagnostics] = useState<SensorDiagnostics | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  const capture = async () => {
    setIsLoading(true);
    setError(null);

    try {
      setDiagnostics(await loader());
    } catch (caughtError) {
      if (caughtError instanceof SensorRuntimeUnavailableError) {
        setError("Tauri 앱에서 실행해야 진단 정보를 불러올 수 있습니다.");
      } else {
        setError("진단 정보를 불러오지 못했습니다.");
      }
    } finally {
      setIsLoading(false);
    }
  };

  return { diagnostics, error, isLoading, capture };
}
