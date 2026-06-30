import { useEffect, useMemo, useState } from "react";
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

const areaLabels: Record<NetworkIncident["area"], string> = {
  local_connection: "로컬 연결 구간",
  gateway_or_local: "공유기 또는 로컬 연결 구간",
  dns: "DNS",
  external: "외부 연결 구간",
  unknown: "확인 불가",
};

function formatTime(value: string) {
  return new Intl.DateTimeFormat("ko-KR", {
    dateStyle: "short",
    timeStyle: "medium",
  }).format(new Date(value));
}

export default function NetworkIncidentHistoryPage({
  onBack,
}: {
  onBack: () => void;
}) {
  const [available] = useState(canUseNetworkIncidents);
  const [incidents, setIncidents] = useState<NetworkIncident[]>([]);
  const [filters, setFilters] = useState<IncidentHistoryFilters>(
    defaultIncidentHistoryFilters,
  );
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

  const setFilter = <K extends keyof IncidentHistoryFilters>(
    key: K,
    value: IncidentHistoryFilters[K],
  ) => {
    setFilters((current) => ({ ...current, [key]: value }));
  };

  return (
    <main className={styles["shell"]}>
      <header className={styles["page-header"]}>
        <button className={styles["back-button"]} type="button" onClick={onBack}>
          돌아가기
        </button>
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
          <ul className={styles["list"]}>
            {filtered.map((incident) => {
              const endAt = incident.resolvedAt ?? incident.lastObservedAt;
              return (
                <li className={styles["row"]} key={incident.id}>
                  <strong>{statusLabels[incident.status]}</strong>
                  <span>{areaLabels[incident.area]}</span>
                  <time dateTime={incident.startedAt}>
                    {formatTime(incident.startedAt)}
                  </time>
                  <time dateTime={endAt}>{formatTime(endAt)}</time>
                  <span>{formatIncidentDuration(incident)}</span>
                  <p>{incident.summary}</p>
                </li>
              );
            })}
          </ul>
        ) : null}
      </section>
    </main>
  );
}
