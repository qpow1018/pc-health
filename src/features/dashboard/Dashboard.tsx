import DeviceCard from "./DeviceCard";
import SensorDiagnosticsPanel from "./SensorDiagnosticsPanel";
import styles from "./Dashboard.module.css";
import type {
  MockScenario,
  SensorDiagnostics,
  SensorMode,
  SensorSnapshot,
} from "@/features/sensors/types";

const scenarios: Array<{ value: MockScenario; label: string }> = [
  { value: "normal", label: "정상" },
  { value: "threshold", label: "임계값 초과" },
  { value: "unsupported", label: "미지원" },
  { value: "waiting", label: "대기" },
  { value: "error", label: "오류" },
];

type DashboardProps = {
  mode: SensorMode;
  snapshot: SensorSnapshot | null;
  error: string | null;
  diagnostics?: SensorDiagnostics | null;
  diagnosticsError?: string | null;
  isDiagnosticsLoading?: boolean;
  scenario: MockScenario;
  onCaptureDiagnostics?: () => void;
  onModeChange: (mode: SensorMode) => void;
  onScenarioChange: (scenario: MockScenario) => void;
};

export default function Dashboard({
  mode,
  snapshot,
  error,
  diagnostics = null,
  diagnosticsError = null,
  isDiagnosticsLoading = false,
  scenario,
  onCaptureDiagnostics = () => {},
  onModeChange,
  onScenarioChange,
}: DashboardProps) {
  const cpuDevice = snapshot?.devices.find((device) => device.kind === "cpu");

  return (
    <main className={styles["shell"]}>
      <header className={styles["header"]}>
        <div>
          <p className={styles["eyebrow"]}>READ-ONLY PERFORMANCE MONITOR</p>
          <h1 className={styles["title"]}>PC Health</h1>
          <p className={styles["updated-at"]}>
            {snapshot
              ? `마지막 측정 ${new Date(snapshot.collectedAt).toLocaleTimeString("ko-KR")}`
              : "센서 데이터 연결 중"}
          </p>
        </div>
        <div className={styles["controls"]}>
          <div className={styles["mode-control"]} aria-label="실행 모드">
            <button
              aria-pressed={mode === "live"}
              type="button"
              onClick={() => onModeChange("live")}
            >
              실제 모드
            </button>
            <button
              aria-pressed={mode === "development"}
              type="button"
              onClick={() => onModeChange("development")}
            >
              개발 모드
            </button>
          </div>
          {mode === "development" && (
            <label className={styles["scenario-control"]}>
              <span>개발 시나리오</span>
              <select
                aria-label="개발 시나리오"
                value={scenario}
                onChange={(event) =>
                  onScenarioChange(event.target.value as MockScenario)
                }
              >
                {scenarios.map((item) => (
                  <option key={item.value} value={item.value}>
                    {item.label}
                  </option>
                ))}
              </select>
            </label>
          )}
        </div>
      </header>

      {error && (
        <div className={styles["error"]} role="alert">
          {error}
        </div>
      )}

      <div className={styles["cpu-panel"]}>
        {cpuDevice && <DeviceCard device={cpuDevice} />}
      </div>

      <p className={styles["guidance-note"]}>
        표시 기준은 일반적인 권장 범위이며 장치 제조사의 공식 한계값이
        우선합니다.
      </p>

      {mode === "live" && (
        <SensorDiagnosticsPanel
          diagnostics={diagnostics}
          error={diagnosticsError}
          isLoading={isDiagnosticsLoading}
          onCapture={onCaptureDiagnostics}
        />
      )}
    </main>
  );
}
