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
  local_connection: "내 PC 연결",
  gateway_or_local: "내 PC 또는 공유기",
  dns: "DNS",
  external: "외부 연결",
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

type EvidenceDisplay = {
  label: string;
  tone: "normal" | "danger" | "unknown";
  durationMs: number | null;
  detail: string | null;
  checkedAt: string | null;
};

function formatDuration(value: number | null) {
  return value === null ? "-" : `${value}ms`;
}

function statusTone(status: EvidenceStatus) {
  if (status === "failure" || status === "timeout") return "danger";
  if (status === "unavailable" || status === "not_checked") return "unknown";
  return "normal";
}

function isRemoteSource(source: EvidenceSource) {
  return (
    source === "dns_microsoft" ||
    source === "dns_google" ||
    source === "http_microsoft" ||
    source === "http_google"
  );
}

function isStaleRemoteFailure(
  status: NetworkDiagnosticStatus,
  evidence: DiagnosticEvidence,
) {
  if (status.lifecycle !== "normal" && status.lifecycle !== "resolved") {
    return false;
  }
  if (evidence.status !== "failure" && evidence.status !== "timeout") {
    return false;
  }
  if (!isRemoteSource(evidence.source)) {
    return false;
  }
  return evidence.checkedAt !== null && evidence.checkedAt !== status.observedAt;
}

function displayForEvidence(
  status: NetworkDiagnosticStatus,
  evidence: DiagnosticEvidence,
): EvidenceDisplay {
  if (isStaleRemoteFailure(status, evidence)) {
    const target = evidence.source.startsWith("dns_") ? "DNS" : "외부 연결";
    return {
      label: "전체 확인 필요",
      tone: "unknown",
      durationMs: null,
      detail: `현재 기본 확인에는 ${target}를 다시 검사하지 않았습니다.`,
      checkedAt: evidence.checkedAt,
    };
  }

  return {
    label: evidenceLabels[evidence.status],
    tone: statusTone(evidence.status),
    durationMs: evidence.durationMs,
    detail: evidence.detail,
    checkedAt: evidence.checkedAt,
  };
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
      label: "IP/경로",
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
  status,
  testId,
}: {
  evidence: DiagnosticEvidence;
  label: string;
  status: NetworkDiagnosticStatus;
  testId: string;
}) {
  const display = displayForEvidence(status, evidence);
  return (
    <li className={styles["evidence-row"]} data-testid={testId}>
      <span className={styles["evidence-name"]}>{label}</span>
      <strong data-tone={display.tone}>{display.label}</strong>
      <span className={styles["evidence-duration"]}>
        {formatDuration(display.durationMs)}
      </span>
      <span className={styles["evidence-detail"]}>
        {display.detail ?? "세부 정보 없음"}
      </span>
      <span className={styles["evidence-time"]}>
        {display.checkedAt ? (
          <time dateTime={display.checkedAt}>{formatTime(display.checkedAt)}</time>
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

function evidenceForArea(status: NetworkDiagnosticStatus) {
  if (!status.suspectedArea || status.suspectedArea === "unknown") return null;

  if (status.suspectedArea === "local_connection") {
    return latestEvidence(status.evidence, ["ethernet", "ipv4", "default_route"]);
  }

  if (status.suspectedArea === "gateway_or_local") {
    return latestEvidence(status.evidence, ["gateway"]);
  }

  if (status.suspectedArea === "dns") {
    return latestEvidence(status.evidence, ["dns_microsoft", "dns_google"]);
  }

  return latestEvidence(status.evidence, ["http_microsoft", "http_google"]);
}

function unavailableMessageForStatus(status: NetworkDiagnosticStatus) {
  if (status.availability === "starting") {
    return "첫 확인 결과를 기다리고 있습니다.";
  }
  if (status.availability === "unavailable") {
    return "Windows 앱에서만 확인할 수 있습니다.";
  }
  if (status.availability === "error") {
    return "진단 상태를 확인할 수 없습니다.";
  }
  return "아직 확인할 수 없습니다.";
}

function descriptionForStatus(status: NetworkDiagnosticStatus) {
  if (status.availability !== "running" || !status.lifecycle) {
    return unavailableMessageForStatus(status);
  }

  if (status.lifecycle === "normal") {
    return "현재 확인된 구간에서 반복 이상이 없습니다.";
  }

  if (status.lifecycle === "suspected") {
    return "이상이 감지됐지만 장애로 확정하려면 추가 근거가 필요합니다.";
  }

  if (status.lifecycle === "incident") {
    return "문제 근거가 확인되었습니다.";
  }

  if (status.lifecycle === "recovering") {
    return "정상 근거가 확인되어 추가 확인 중입니다.";
  }

  return "현재 연결은 복구된 상태입니다.";
}

function reasonForStatus(status: NetworkDiagnosticStatus) {
  if (status.availability !== "running" || !status.lifecycle) {
    return unavailableMessageForStatus(status);
  }

  if (status.lifecycle === "normal") {
    return "최근 확인된 주요 구간이 정상입니다.";
  }

  if (status.lifecycle === "resolved") {
    return "이전 장애가 복구된 뒤 현재 확인은 정상입니다.";
  }

  const evidence = evidenceForArea(status);
  if (!status.suspectedArea || status.suspectedArea === "unknown" || !evidence) {
    return "근거가 부족하거나 서로 충돌해 원인 구간을 확정하지 않았습니다.";
  }

  const areaLabel = areaLabels[status.suspectedArea];
  const evidenceLabel = evidenceLabels[evidence.status];

  if (status.lifecycle === "incident") {
    return `${areaLabel}에서 ${evidenceLabel} 근거가 확인되었습니다.`;
  }

  if (status.lifecycle === "recovering") {
    return `${areaLabel}의 정상 근거를 추가 확인하고 있습니다.`;
  }

  return `${areaLabel}에서 ${evidenceLabel} 근거가 있어 추가 확인 중입니다.`;
}

function progressForStatus(status: NetworkDiagnosticStatus) {
  if (status.availability === "starting") return "첫 확인 대기";
  if (status.availability === "unavailable") return "진단 사용 불가";
  if (status.availability === "error") return "진단 상태 확인 실패";
  if (!status.lifecycle) return "확인 상태 없음";

  if (status.lifecycle === "normal" || status.lifecycle === "resolved") {
    return "기본 확인 완료";
  }

  if (status.lifecycle === "suspected") return "일시적인 문제인지 확인 중";
  if (status.lifecycle === "incident") return "문제가 확인됨";
  return "정상으로 돌아왔는지 확인 중";
}

function recommendedCheckForStatus(status: NetworkDiagnosticStatus) {
  if (status.availability === "starting") {
    return "첫 확인이 끝날 때까지 잠시 기다리세요.";
  }
  if (status.availability !== "running" || !status.lifecycle) {
    return "Windows 앱에서 다시 확인하세요.";
  }
  if (status.lifecycle === "normal" || status.lifecycle === "resolved") {
    return "현재는 추가 확인이 필요하지 않습니다.";
  }
  if (status.suspectedArea === "local_connection") {
    return "랜 케이블 연결과 Windows 어댑터 사용 상태를 확인하세요.";
  }
  if (status.suspectedArea === "gateway_or_local") {
    return "같은 공유기의 다른 기기도 함께 끊기는지 확인하세요.";
  }
  if (status.suspectedArea === "dns") {
    return "웹사이트 주소 대신 IP 연결이나 다른 앱 연결도 함께 확인하세요.";
  }
  if (status.suspectedArea === "external") {
    return "다른 기기에서도 외부 사이트 접속이 느리거나 끊기는지 확인하세요.";
  }
  return "근거가 더 쌓일 때까지 현재 상태를 지켜보세요.";
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
  const reasonLabel = reasonForStatus(status);
  const progressLabel = progressForStatus(status);
  const recommendedCheck = recommendedCheckForStatus(status);
  const visualLifecycle =
    status.availability === "running" ? status.lifecycle : null;
  const pathStages = buildPathStages(status.evidence);
  const displayPathStages = pathStages.map((stage) => ({
    ...stage,
    display: displayForEvidence(status, stage.evidence),
  }));
  const rows = [
    {
      label: "PC/어댑터",
      testId: "evidence-pc",
      evidence: latestEvidence(status.evidence, ["ethernet"]),
    },
    {
      label: "IP/경로",
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
          <dt>문제가 의심되는 곳</dt>
          <dd>{areaLabel}</dd>
        </div>
        <div>
          <dt>판단 이유</dt>
          <dd>{reasonLabel}</dd>
        </div>
        <div>
          <dt>진행 상태</dt>
          <dd>{progressLabel}</dd>
        </div>
        <div>
          <dt>권장 확인</dt>
          <dd>{recommendedCheck}</dd>
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
        <div>
          <dt>전체 확인</dt>
          <dd>
            {status.lastFullProbeAt ? (
              <time dateTime={status.lastFullProbeAt}>
                {formatTime(status.lastFullProbeAt)}
              </time>
            ) : (
              "확인 시각 없음"
            )}
          </dd>
        </div>
      </dl>

      <div className={styles["path-section"]}>
        <h3>연결 경로</h3>
        <div className={styles["path"]} aria-label="구간별 진단 경로">
          {displayPathStages.map((stage) => (
            <div
              className={styles["path-stage"]}
              data-tone={stage.display.tone}
              data-testid={`path-${stage.key}`}
              key={stage.key}
            >
              <span className={styles["path-label"]}>{stage.label}</span>
              <strong>{stage.display.label}</strong>
            </div>
          ))}
        </div>
      </div>

      <div className={styles["evidence"]}>
        <h3>최근 확인 내용</h3>
        <ul>
          {rows.map((row) => (
            <EvidenceRow key={row.testId} status={status} {...row} />
          ))}
        </ul>
      </div>
    </section>
  );
}
