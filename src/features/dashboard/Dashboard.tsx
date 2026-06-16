import DeviceCard from "./DeviceCard";
import type { MockScenario, SensorSnapshot } from "@/features/sensors/types";

const scenarios: Array<{ value: MockScenario; label: string }> = [
  { value: "normal", label: "정상" },
  { value: "threshold", label: "임계값 초과" },
  { value: "unsupported", label: "미지원" },
  { value: "waiting", label: "대기" },
  { value: "error", label: "오류" },
];

type DashboardProps = {
  snapshot: SensorSnapshot | null;
  error: string | null;
  scenario: MockScenario;
  onScenarioChange: (scenario: MockScenario) => void;
};

export default function Dashboard({
  snapshot,
  error,
  scenario,
  onScenarioChange,
}: DashboardProps) {
  return (
    <main className="app-shell">
      <header className="dashboard-header">
        <div>
          <p className="eyebrow">READ-ONLY PERFORMANCE MONITOR</p>
          <h1>PC Health</h1>
          <p className="updated-at">
            {snapshot
              ? `마지막 측정 ${new Date(snapshot.collectedAt).toLocaleTimeString("ko-KR")}`
              : "센서 데이터 연결 중"}
          </p>
        </div>
        <label className="scenario-control">
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
      </header>

      {error && (
        <div className="dashboard-error" role="alert">
          {error}
        </div>
      )}

      <div className="device-grid">
        {snapshot?.devices.map((device) => (
          <DeviceCard device={device} key={device.kind} />
        ))}
      </div>

      <p className="guidance-note">
        표시 기준은 일반적인 권장 범위이며 장치 제조사의 공식 한계값이
        우선합니다.
      </p>
    </main>
  );
}
