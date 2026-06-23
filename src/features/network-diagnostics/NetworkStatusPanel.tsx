import { useEffect, useState } from "react";
import {
  canUseNetworkDiagnostics,
  getNetworkDiagnosticStatus,
} from "./api";
import {
  startingStatusFixture,
  unavailableStatusFixture,
} from "./fixture";
import type {
  DiagnosticArea,
  DiagnosticEvidence,
  DiagnosticLifecycle,
  EvidenceSource,
  EvidenceStatus,
  NetworkDiagnosticStatus,
} from "./types";
import styles from "./NetworkStatusPanel.module.css";

const lifecycleLabels: Record<DiagnosticLifecycle, string> = {
  normal: "정상",
  suspected: "장애 의심",
  incident: "장애 확인",
  recovering: "복구 확인 중",
  resolved: "복구됨",
};

const areaLabels: Record<DiagnosticArea, string> = {
  local_connection: "로컬 연결 구간",
  gateway_or_local: "공유기 또는 로컬 연결 구간",
  dns: "DNS",
  external: "외부 연결 구간",
  unknown: "확인 불가",
};

const evidenceLabels: Record<EvidenceStatus, string> = {
  success: "성공",
  failure: "실패",
  timeout: "시간 초과",
  unavailable: "확인 불가",
  not_checked: "확인하지 않음",
};

const evidencePriority: Record<EvidenceStatus, number> = {
  failure: 5,
  timeout: 4,
  unavailable: 3,
  not_checked: 2,
  success: 1,
};

const notCheckedEvidence: DiagnosticEvidence = {
  source: "gateway",
  status: "not_checked",
  checkedAt: null,
  durationMs: null,
  detail: null,
};

function formatTime(value: string) {
  return new Intl.DateTimeFormat("ko-KR", {
    dateStyle: "short",
    timeStyle: "medium",
  }).format(new Date(value));
}

function latestEvidence(
  evidence: DiagnosticEvidence[],
  sources: EvidenceSource[],
) {
  const matches = evidence.filter((item) => sources.includes(item.source));
  if (matches.length === 0) return notCheckedEvidence;

  return matches.reduce((selected, item) => {
    const priorityDifference =
      evidencePriority[item.status] - evidencePriority[selected.status];
    if (priorityDifference !== 0) return priorityDifference > 0 ? item : selected;
    return (item.checkedAt ?? "") > (selected.checkedAt ?? "") ? item : selected;
  });
}

function EvidenceRow({
  evidence,
  label,
  testId,
}: {
  evidence: DiagnosticEvidence;
  label: string;
  testId: string;
}) {
  return (
    <li className={styles["evidence-row"]} data-testid={testId}>
      <span className={styles["evidence-name"]}>{label}</span>
      <strong data-status={evidence.status}>
        {evidenceLabels[evidence.status]}
      </strong>
      <span className={styles["evidence-detail"]}>
        {evidence.detail ?? "세부 정보 없음"}
      </span>
      <span className={styles["evidence-time"]}>
        {evidence.checkedAt ? (
          <time dateTime={evidence.checkedAt}>{formatTime(evidence.checkedAt)}</time>
        ) : (
          "확인 시각 없음"
        )}
      </span>
    </li>
  );
}

function commandErrorStatus(caught: unknown): NetworkDiagnosticStatus {
  const message = caught instanceof Error ? caught.message : String(caught);
  return {
    ...unavailableStatusFixture,
    availability: "error",
    error: {
      stage: "command",
      code: "status_command_failed",
      message,
      nativeCode: null,
    },
  };
}

export default function NetworkStatusPanel() {
  const [available] = useState(canUseNetworkDiagnostics);
  const [status, setStatus] = useState<NetworkDiagnosticStatus>(
    available ? startingStatusFixture : unavailableStatusFixture,
  );

  useEffect(() => {
    if (!available) return;

    let cancelled = false;
    let inFlight = false;

    const load = async () => {
      if (inFlight) return;
      inFlight = true;
      try {
        const nextStatus = await getNetworkDiagnosticStatus();
        if (!cancelled) setStatus(nextStatus);
      } catch (caught) {
        if (!cancelled) setStatus(commandErrorStatus(caught));
      } finally {
        inFlight = false;
      }
    };

    void load();
    const interval = window.setInterval(() => void load(), 1000);

    return () => {
      cancelled = true;
      window.clearInterval(interval);
    };
  }, [available]);

  const isQuietUnavailable =
    status.availability === "unavailable" || status.availability === "error";
  const statusLabel =
    status.availability === "starting"
      ? "첫 확인 중"
      : isQuietUnavailable || !status.lifecycle
        ? "확인 불가"
        : lifecycleLabels[status.lifecycle];
  const areaLabel = status.suspectedArea
    ? areaLabels[status.suspectedArea]
    : "확인 불가";
  const rows = [
    {
      label: "게이트웨이",
      testId: "evidence-gateway",
      evidence: latestEvidence(status.evidence, ["gateway"]),
    },
    {
      label: "DNS",
      testId: "evidence-dns",
      evidence: latestEvidence(status.evidence, ["dns_microsoft", "dns_google"]),
    },
    {
      label: "Microsoft",
      testId: "evidence-microsoft",
      evidence: latestEvidence(status.evidence, ["http_microsoft"]),
    },
    {
      label: "Google",
      testId: "evidence-google",
      evidence: latestEvidence(status.evidence, ["http_google"]),
    },
  ];

  return (
    <section
      className={`${styles["panel"]} ${styles[`state-${status.lifecycle ?? "unknown"}`]}`}
      aria-labelledby="network-status-title"
    >
      <div className={styles["panel-header"]}>
        <div>
          <p className={styles["label"]}>CURRENT NETWORK STATUS</p>
          <h2 id="network-status-title">현재 네트워크 진단</h2>
        </div>
        <strong className={styles["status"]}>{statusLabel}</strong>
      </div>

      {!available && (
        <p className={styles["availability"]}>Windows 앱에서 확인 가능</p>
      )}
      {available && status.availability === "unavailable" && (
        <p className={styles["availability"]}>
          Windows 네트워크 진단을 사용할 수 없습니다.
        </p>
      )}
      {status.error && (
        <p className={styles["availability"]} role="alert">
          {status.error.message}
        </p>
      )}

      <dl className={styles["summary"]}>
        <div>
          <dt>추정 구간</dt>
          <dd>{areaLabel}</dd>
        </div>
        <div>
          <dt>마지막 확인</dt>
          <dd>
            {status.observedAt ? (
              <time dateTime={status.observedAt}>
                {formatTime(status.observedAt)}
              </time>
            ) : (
              "확인 시각 없음"
            )}
          </dd>
        </div>
      </dl>

      <div className={styles["evidence"]}>
        <h3>최신 근거</h3>
        <ul>
          {rows.map((row) => (
            <EvidenceRow key={row.testId} {...row} />
          ))}
        </ul>
      </div>
    </section>
  );
}
