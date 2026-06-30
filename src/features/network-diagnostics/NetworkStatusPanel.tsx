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
    return (item.checkedAt ?? "") >= (selected.checkedAt ?? "") ? item : selected;
  });
}

type PathStage = {
  key: string;
  label: string;
  evidence: DiagnosticEvidence;
};

function formatDuration(value: number | null) {
  return value === null ? "-" : `${value}ms`;
}

function statusTone(status: EvidenceStatus) {
  if (status === "failure" || status === "timeout") return "danger";
  if (status === "unavailable" || status === "not_checked") return "unknown";
  return "normal";
}

function buildPathStages(evidence: DiagnosticEvidence[]): PathStage[] {
  return [
    {
      key: "pc",
      label: "PC/어댑터",
      evidence: latestEvidence(evidence, ["ethernet"]),
    },
    {
      key: "route",
      label: "IPv4/Route",
      evidence: latestEvidence(evidence, ["ipv4", "default_route"]),
    },
    {
      key: "gateway",
      label: "게이트웨이",
      evidence: latestEvidence(evidence, ["gateway"]),
    },
    {
      key: "dns",
      label: "DNS",
      evidence: latestEvidence(evidence, ["dns_microsoft", "dns_google"]),
    },
    {
      key: "external",
      label: "외부 연결",
      evidence: latestEvidence(evidence, ["http_microsoft", "http_google"]),
    },
  ];
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
      <span className={styles["evidence-duration"]}>
        {formatDuration(evidence.durationMs)}
      </span>
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

function areaLabelForStatus(status: NetworkDiagnosticStatus) {
  if (status.availability === "starting") return "첫 확인 중";
  if (status.availability === "unavailable" || status.availability === "error") {
    return "확인 불가";
  }
  if (status.suspectedArea) return areaLabels[status.suspectedArea];
  if (status.lifecycle === "normal" || status.lifecycle === "resolved") {
    return "해당 없음";
  }
  return "확인 불가";
}

function descriptionForStatus(status: NetworkDiagnosticStatus) {
  if (status.availability !== "running" || !status.lifecycle) {
    return "진단 근거가 부족하거나 Windows 앱에서만 확인할 수 있습니다.";
  }

  if (status.lifecycle === "normal") {
    return "현재 확인된 구간에서 반복 이상이 없습니다.";
  }

  if (status.lifecycle === "suspected") {
    return "이상이 감지됐지만 장애로 확정하려면 추가 근거가 필요합니다.";
  }

  if (status.lifecycle === "incident") {
    return "호환되는 이상이 반복 확인되었습니다.";
  }

  if (status.lifecycle === "recovering") {
    return "정상 근거가 확인되어 추가 확인 중입니다.";
  }

  return "현재 연결은 복구된 상태입니다.";
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
  const areaLabel = areaLabelForStatus(status);
  const statusDescription = descriptionForStatus(status);
  const visualLifecycle =
    status.availability === "running" ? status.lifecycle : null;
  const pathStages = buildPathStages(status.evidence);
  const rows = [
    {
      label: "PC/어댑터",
      testId: "evidence-pc",
      evidence: latestEvidence(status.evidence, ["ethernet"]),
    },
    {
      label: "IPv4/Route",
      testId: "evidence-route",
      evidence: latestEvidence(status.evidence, ["ipv4", "default_route"]),
    },
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
      className={`${styles["panel"]} ${styles[`state-${visualLifecycle ?? "unknown"}`]}`}
      aria-labelledby="network-status-title"
    >
      <div className={styles["panel-header"]}>
        <div>
          <p className={styles["label"]}>CURRENT NETWORK STATUS</p>
          <h2 id="network-status-title">현재 네트워크 진단</h2>
        </div>
        <strong
          className={styles["status"]}
          role="status"
          aria-live="polite"
          aria-atomic="true"
        >
          {statusLabel}
        </strong>
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
        <p className={styles["availability"]}>
          {status.error.message}
        </p>
      )}
      <p className={styles["status-description"]}>{statusDescription}</p>

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

      <div className={styles["path"]} aria-label="구간별 진단 경로">
        {pathStages.map((stage) => (
          <div
            className={styles["path-stage"]}
            data-tone={statusTone(stage.evidence.status)}
            data-testid={`path-${stage.key}`}
            key={stage.key}
          >
            <span className={styles["path-label"]}>{stage.label}</span>
            <strong>{evidenceLabels[stage.evidence.status]}</strong>
          </div>
        ))}
      </div>

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
