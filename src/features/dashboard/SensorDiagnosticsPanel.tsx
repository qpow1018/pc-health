import type { SensorDiagnostics } from "@/features/sensors/types";
import styles from "./Dashboard.module.css";

type SensorDiagnosticsPanelProps = {
  diagnostics: SensorDiagnostics | null;
  error: string | null;
  isLoading: boolean;
  onCapture: () => void;
};

function formatJson(value: unknown) {
  return JSON.stringify(value, null, 2);
}

function formatDuration(durationMs: number) {
  return durationMs === 0 ? "<1ms" : `${durationMs}ms`;
}

export default function SensorDiagnosticsPanel({
  diagnostics,
  error,
  isLoading,
  onCapture,
}: SensorDiagnosticsPanelProps) {
  const copyDiagnostics = async () => {
    if (!diagnostics) return;
    await navigator.clipboard.writeText(formatJson(diagnostics));
  };

  return (
    <section className={styles["diagnostics"]} aria-label="센서 진단">
      <header className={styles["diagnostics-header"]}>
        <div>
          <h2>센서 진단</h2>
          <p>Windows 수집 원본과 앱에 전달된 최종 값을 한 번만 확인합니다.</p>
        </div>
        <div className={styles["diagnostics-actions"]}>
          <button type="button" onClick={onCapture} disabled={isLoading}>
            {isLoading ? "진단 중" : "진단 캡처"}
          </button>
          <button type="button" onClick={copyDiagnostics} disabled={!diagnostics}>
            복사
          </button>
        </div>
      </header>

      {error && (
        <div className={styles["diagnostics-error"]} role="alert">
          {error}
        </div>
      )}

      {diagnostics && (
        <div className={styles["diagnostics-grid"]}>
          <div>
            <span>collector</span>
            <strong>{diagnostics.collector}</strong>
          </div>
          <div>
            <span>duration</span>
            <strong>{formatDuration(diagnostics.durationMs)}</strong>
          </div>
          <div>
            <span>collected</span>
            <strong>
              {new Date(diagnostics.collectedAt).toLocaleTimeString("ko-KR")}
            </strong>
          </div>
          {diagnostics.rawError && (
            <div className={styles["diagnostics-wide"]}>
              <span>raw error</span>
              <pre>{diagnostics.rawError}</pre>
            </div>
          )}
          {diagnostics.rawPayload && (
            <div className={styles["diagnostics-wide"]}>
              <span>raw payload</span>
              <pre>{diagnostics.rawPayload}</pre>
            </div>
          )}
          {diagnostics.parsedTelemetry && (
            <div className={styles["diagnostics-wide"]}>
              <span>parsed telemetry</span>
              <pre>{formatJson(diagnostics.parsedTelemetry)}</pre>
            </div>
          )}
          <div className={styles["diagnostics-wide"]}>
            <span>final snapshot</span>
            <pre>{formatJson(diagnostics.snapshot)}</pre>
          </div>
        </div>
      )}
    </section>
  );
}
