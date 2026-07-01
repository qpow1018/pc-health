import type {
  DiagnosticArea,
  EvidenceSource,
  EvidenceStatus,
} from "@/features/network-diagnostics/types";
import { formatIncidentDuration } from "./filter";
import type { NetworkIncident } from "./types";
import styles from "./NetworkIncidentDetailPage.module.css";

const statusLabels: Record<NetworkIncident["status"], string> = {
  ongoing: "진행 중",
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

const sourceLabels: Record<EvidenceSource, string> = {
  ethernet: "PC/어댑터",
  ipv4: "IPv4",
  default_route: "기본 경로",
  gateway: "게이트웨이",
  dns_microsoft: "DNS Microsoft",
  dns_google: "DNS Google",
  http_microsoft: "Microsoft 연결",
  http_google: "Google 연결",
};

const evidenceStatusLabels: Record<EvidenceStatus, string> = {
  success: "성공",
  failure: "실패",
  timeout: "시간 초과",
  unavailable: "확인 불가",
  not_checked: "확인하지 않음",
};

function formatTime(value: string) {
  return new Intl.DateTimeFormat("ko-KR", {
    dateStyle: "short",
    timeStyle: "medium",
  }).format(new Date(value));
}

function formatOptionalTime(value: string | null | undefined, fallback: string) {
  return value ? formatTime(value) : fallback;
}

function formatDurationMs(value: number | null | undefined) {
  return typeof value === "number" ? `${value}ms` : "-";
}

export default function NetworkIncidentDetailPage({
  incident,
  onBack,
}: {
  incident: NetworkIncident;
  onBack: () => void;
}) {
  return (
    <main className={styles["shell"]}>
      <header className={styles["page-header"]}>
        <button className={styles["back-button"]} type="button" onClick={onBack}>
          장애 이력으로 돌아가기
        </button>
        <div>
          <p className={styles["eyebrow"]}>INCIDENT DETAIL</p>
          <h1>장애 상세</h1>
          <p>{incident.summary}</p>
        </div>
      </header>

      <section
        className={styles["panel"]}
        aria-labelledby="incident-detail-summary-title"
      >
        <h2 id="incident-detail-summary-title">요약</h2>
        <dl className={styles["summary"]}>
          <div>
            <dt>상태</dt>
            <dd>{statusLabels[incident.status]}</dd>
          </div>
          <div>
            <dt>추정 구간</dt>
            <dd>{areaLabels[incident.area]}</dd>
          </div>
          <div>
            <dt>시작</dt>
            <dd>{formatTime(incident.startedAt)}</dd>
          </div>
          <div>
            <dt>마지막 확인</dt>
            <dd>{formatTime(incident.lastObservedAt)}</dd>
          </div>
          <div>
            <dt>복구</dt>
            <dd>
              {formatOptionalTime(incident.resolvedAt, "아직 복구 기록 없음")}
            </dd>
          </div>
          <div>
            <dt>지속 시간</dt>
            <dd>{formatIncidentDuration(incident)}</dd>
          </div>
        </dl>
      </section>

      <section
        className={styles["panel"]}
        aria-labelledby="incident-detail-evidence-title"
      >
        <h2 id="incident-detail-evidence-title">대표 근거</h2>
        {incident.representativeEvidence.length === 0 ? (
          <p className={styles["muted"]}>저장된 대표 근거가 없습니다.</p>
        ) : (
          <ul className={styles["evidence-list"]}>
            {incident.representativeEvidence.map((evidence) => (
              <li
                className={styles["evidence-row"]}
                key={`${evidence.source}-${evidence.observedAt}-${evidence.detail ?? ""}`}
              >
                <strong>{sourceLabels[evidence.source]}</strong>
                <span>{evidenceStatusLabels[evidence.status]}</span>
                <span>{formatDurationMs(evidence.durationMs)}</span>
                <p>{evidence.detail ?? "세부 정보 없음"}</p>
                <time dateTime={evidence.checkedAt ?? undefined}>
                  {formatOptionalTime(evidence.checkedAt, "확인 시각 없음")}
                </time>
                <time dateTime={evidence.observedAt}>
                  {formatTime(evidence.observedAt)}
                </time>
              </li>
            ))}
          </ul>
        )}
      </section>
    </main>
  );
}
