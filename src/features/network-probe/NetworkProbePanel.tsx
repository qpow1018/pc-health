import { useState } from "react";
import { getNetworkProbeSnapshot } from "./api";
import styles from "./NetworkProbePanel.module.css";

type PanelState = "idle" | "loading" | "success" | "error";

export default function NetworkProbePanel() {
  const [state, setState] = useState<PanelState>("idle");
  const [json, setJson] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [copyMessage, setCopyMessage] = useState<string | null>(null);
  const [isJsonExpanded, setIsJsonExpanded] = useState(false);

  async function runProbe() {
    setIsJsonExpanded(false);
    setState("loading");
    setError(null);
    setCopyMessage(null);

    try {
      const snapshot = await getNetworkProbeSnapshot();
      setJson(JSON.stringify(snapshot, null, 2));
      setState("success");
    } catch (caught) {
      setJson(null);
      setError(caught instanceof Error ? caught.message : String(caught));
      setState("error");
    }
  }

  async function copyJson() {
    if (!json) return;

    try {
      await navigator.clipboard.writeText(json);
      setCopyMessage("복사했습니다.");
    } catch {
      setCopyMessage("복사하지 못했습니다.");
    }
  }

  return (
    <section className={styles["panel"]} aria-labelledby="network-probe-title">
      <div className={styles["panel-header"]}>
        <div>
          <p className={styles["label"]}>WINDOWS RAW PROBE</p>
          <h2 id="network-probe-title">원시 네트워크 확인</h2>
          <p>
            장애를 판정하지 않고 어댑터, 게이트웨이, DNS와 외부 연결의 원시
            결과를 한 번 수집합니다.
          </p>
        </div>
        <button type="button" onClick={runProbe} disabled={state === "loading"}>
          {state === "loading" ? "확인 중" : "네트워크 확인 실행"}
        </button>
      </div>

      {error && (
        <p className={styles["error"]} role="alert">
          {error}
        </p>
      )}

      {json && (
        <div className={styles["result"]}>
          <div className={styles["result-header"]}>
            <span>Raw JSON</span>
            <button
              type="button"
              onClick={() => setIsJsonExpanded((current) => !current)}
            >
              {isJsonExpanded ? "Raw JSON 접기" : "Raw JSON 펼치기"}
            </button>
          </div>
          {isJsonExpanded && (
            <>
              <pre>{json}</pre>
              <button type="button" onClick={copyJson}>
                JSON 복사
              </button>
              {copyMessage && (
                <p className={styles["copy-message"]}>{copyMessage}</p>
              )}
            </>
          )}
        </div>
      )}
    </section>
  );
}
