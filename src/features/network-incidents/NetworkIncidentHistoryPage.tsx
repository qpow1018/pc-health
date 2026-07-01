import { useEffect, useMemo, useState } from "react";
import type {
  DiagnosticArea,
  EvidenceSource,
  EvidenceStatus,
} from "@/features/network-diagnostics/types";
import { canUseNetworkIncidents, getNetworkIncidents } from "./api";
import {
  defaultIncidentHistoryFilters,
  filterNetworkIncidents,
  formatIncidentDuration,
  type IncidentHistoryFilters,
} from "./filter";
import type { NetworkIncident, NetworkIncidentStatus } from "./types";
import styles from "./NetworkIncidentHistoryPage.module.css";

const statusLabels: Record<NetworkIncidentStatus, string> = {
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

export default function NetworkIncidentHistoryPage({
  onBack,
}: {
  onBack?: () => void;
}) {
  const [available] = useState(canUseNetworkIncidents);
  const [incidents, setIncidents] = useState<NetworkIncident[]>([]);
  const [filters, setFilters] = useState<IncidentHistoryFilters>(
    defaultIncidentHistoryFilters,
  );
  const [selectedIncidentId, setSelectedIncidentId] = useState<number | null>(null);
  const [failed, setFailed] = useState(!available);
  const [loading, setLoading] = useState(available);

  useEffect(() => {
    if (!available) return;

    let cancelled = false;

    getNetworkIncidents()
      .then((items) => {
        if (!cancelled) setIncidents(items);
      })
      .catch(() => {
        if (!cancelled) setFailed(true);
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [available]);

  const filtered = useMemo(
    () => filterNetworkIncidents(incidents, filters),
    [incidents, filters],
  );

  const selectedIncident = useMemo(
    () => filtered.find((incident) => incident.id === selectedIncidentId) ?? null,
    [filtered, selectedIncidentId],
  );

  useEffect(() => {
    if (selectedIncidentId !== null && !selectedIncident) {
      setSelectedIncidentId(null);
    }
  }, [selectedIncident, selectedIncidentId]);

  const setFilter = <K extends keyof IncidentHistoryFilters>(
    key: K,
    value: IncidentHistoryFilters[K],
  ) => {
    setFilters((current) => ({ ...current, [key]: value }));
  };

  return (
    <main className={styles["shell"]}>
      <header className={styles["page-header"]}>
        {onBack ? (
          <button className={styles["back-button"]} type="button" onClick={onBack}>
            돌아가기
          </button>
        ) : null}
        <div>
          <p className={styles["eyebrow"]}>INCIDENT HISTORY</p>
          <h1>장애 이력</h1>
          <p>앱 실행 중 확정된 네트워크 장애 기록을 보여줍니다.</p>
        </div>
      </header>

      <section className={styles["filters"]} aria-label="장애 이력 필터">
        <label>
          기간
          <select
            value={filters.period}
            onChange={(event) =>
              setFilter("period", event.target.value as IncidentHistoryFilters["period"])
            }
          >
            <option value="all">전체</option>
            <option value="24h">최근 24시간</option>
            <option value="7d">7일</option>
            <option value="30d">30일</option>
          </select>
        </label>
        <label>
          상태
          <select
            value={filters.status}
            onChange={(event) =>
              setFilter("status", event.target.value as IncidentHistoryFilters["status"])
            }
          >
            <option value="all">전체</option>
            <option value="ongoing">진행 중</option>
            <option value="recovering">복구 확인 중</option>
            <option value="resolved">복구됨</option>
          </select>
        </label>
        <label>
          구간
          <select
            value={filters.area}
            onChange={(event) =>
              setFilter("area", event.target.value as IncidentHistoryFilters["area"])
            }
          >
            <option value="all">전체</option>
            <option value="local_connection">로컬 연결</option>
            <option value="gateway_or_local">공유기 또는 로컬</option>
            <option value="dns">DNS</option>
            <option value="external">외부 연결</option>
            <option value="unknown">확인 불가</option>
          </select>
        </label>
      </section>

      <section className={styles["panel"]} aria-label="장애 이력 목록">
        {loading ? <p className={styles["muted"]}>장애 이력 확인 중</p> : null}
        {!loading && failed ? (
          <p className={styles["muted"]}>장애 이력을 불러올 수 없습니다.</p>
        ) : null}
        {!loading && !failed && incidents.length === 0 ? (
          <p className={styles["muted"]}>저장된 장애 기록이 없습니다.</p>
        ) : null}
        {!loading && !failed && incidents.length > 0 && filtered.length === 0 ? (
          <p className={styles["muted"]}>조건에 맞는 장애 기록이 없습니다.</p>
        ) : null}
        {!loading && !failed && filtered.length > 0 ? (
          <div className={styles["content"]}>
            <ul className={styles["list"]}>
              {filtered.map((incident) => {
                const endAt = incident.resolvedAt ?? incident.lastObservedAt;
                const selected = incident.id === selectedIncidentId;
                return (
                  <li
                    className={`${styles["row"]} ${
                      selected ? styles["row-selected"] : ""
                    }`}
                    key={incident.id}
                  >
                    <strong>{statusLabels[incident.status]}</strong>
                    <span>{areaLabels[incident.area]}</span>
                    <time dateTime={incident.startedAt}>
                      {formatTime(incident.startedAt)}
                    </time>
                    <time dateTime={endAt}>{formatTime(endAt)}</time>
                    <span>{formatIncidentDuration(incident)}</span>
                    <p>{incident.summary}</p>
                    <button
                      className={styles["detail-button"]}
                      type="button"
                      aria-pressed={selected}
                      onClick={() => setSelectedIncidentId(incident.id)}
                    >
                      상세 보기
                    </button>
                  </li>
                );
              })}
            </ul>
            {selectedIncident ? (
              <aside
                className={styles["detail"]}
                aria-label="선택한 장애 상세"
                role="region"
              >
                <div className={styles["detail-header"]}>
                  <div>
                    <p className={styles["eyebrow"]}>INCIDENT DETAIL</p>
                    <h2>선택한 장애 상세</h2>
                  </div>
                  <button
                    className={styles["close-button"]}
                    type="button"
                    onClick={() => setSelectedIncidentId(null)}
                  >
                    상세 닫기
                  </button>
                </div>

                <dl className={styles["detail-summary"]}>
                  <div>
                    <dt>상태</dt>
                    <dd>{statusLabels[selectedIncident.status]}</dd>
                  </div>
                  <div>
                    <dt>추정 구간</dt>
                    <dd>{areaLabels[selectedIncident.area]}</dd>
                  </div>
                  <div>
                    <dt>시작</dt>
                    <dd>{formatTime(selectedIncident.startedAt)}</dd>
                  </div>
                  <div>
                    <dt>마지막 확인</dt>
                    <dd>{formatTime(selectedIncident.lastObservedAt)}</dd>
                  </div>
                  <div>
                    <dt>복구</dt>
                    <dd>
                      {formatOptionalTime(
                        selectedIncident.resolvedAt,
                        "아직 복구 기록 없음",
                      )}
                    </dd>
                  </div>
                  <div>
                    <dt>지속 시간</dt>
                    <dd>{formatIncidentDuration(selectedIncident)}</dd>
                  </div>
                </dl>

                <p className={styles["detail-summary-text"]}>
                  {selectedIncident.summary}
                </p>

                <div className={styles["evidence-section"]}>
                  <h3>대표 근거</h3>
                  {selectedIncident.representativeEvidence.length === 0 ? (
                    <p className={styles["muted"]}>
                      저장된 대표 근거가 없습니다.
                    </p>
                  ) : (
                    <ul className={styles["evidence-list"]}>
                      {selectedIncident.representativeEvidence.map((evidence) => (
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
                </div>
              </aside>
            ) : (
              <p className={styles["selection-hint"]}>
                기록을 선택하면 대표 근거를 확인할 수 있습니다.
              </p>
            )}
          </div>
        ) : null}
      </section>
    </main>
  );
}
