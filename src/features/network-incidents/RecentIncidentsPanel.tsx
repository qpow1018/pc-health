import { useEffect, useState } from "react";
import { canUseNetworkIncidents, getRecentNetworkIncidents } from "./api";
import type {
  NetworkIncident,
  NetworkIncidentStatus,
} from "./types";
import styles from "./RecentIncidentsPanel.module.css";

const statusLabels: Record<NetworkIncidentStatus, string> = {
  ongoing: "진행 중",
  recovering: "복구 확인 중",
  resolved: "복구됨",
};

const areaLabels: Record<NetworkIncident["area"], string> = {
  local_connection: "내 PC 연결",
  gateway_or_local: "내 PC 또는 공유기",
  dns: "DNS",
  external: "외부 연결",
  unknown: "확인 불가",
};

function formatTime(value: string) {
  return new Intl.DateTimeFormat("ko-KR", {
    dateStyle: "short",
    timeStyle: "medium",
  }).format(new Date(value));
}

export default function RecentIncidentsPanel({
  onOpenHistory,
}: {
  onOpenHistory?: () => void;
}) {
  const [available] = useState(canUseNetworkIncidents);
  const [incidents, setIncidents] = useState<NetworkIncident[]>([]);
  const [failed, setFailed] = useState(!available);
  const [loading, setLoading] = useState(available);

  useEffect(() => {
    if (!available) return;

    let cancelled = false;

    getRecentNetworkIncidents()
      .then((items) => {
        if (!cancelled) setIncidents(items.slice(0, 3));
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

  return (
    <section className={styles["panel"]} aria-labelledby="recent-incidents-title">
      <header className={styles["header"]}>
        <h2 id="recent-incidents-title">최근 장애</h2>
        {onOpenHistory ? (
          <button type="button" onClick={onOpenHistory}>
            전체 이력 보기
          </button>
        ) : null}
      </header>

      {loading ? <p className={styles["muted"]}>장애 기록 확인 중</p> : null}
      {!loading && failed ? (
        <p className={styles["muted"]}>장애 기록을 불러올 수 없습니다.</p>
      ) : null}
      {!loading && !failed && incidents.length === 0 ? (
        <p className={styles["muted"]}>저장된 장애 기록이 없습니다.</p>
      ) : null}
      {!loading && !failed && incidents.length > 0 ? (
        <ul className={styles["list"]}>
          {incidents.map((incident) => {
            const endAt = incident.resolvedAt ?? incident.lastObservedAt;
            return (
              <li className={styles["row"]} key={incident.id}>
                <div className={styles["row-header"]}>
                  <strong>{statusLabels[incident.status]}</strong>
                  <span>{areaLabels[incident.area]}</span>
                </div>
                <p>{incident.summary}</p>
                <dl>
                  <div>
                    <dt>시작</dt>
                    <dd>
                      <time dateTime={incident.startedAt}>
                        {formatTime(incident.startedAt)}
                      </time>
                    </dd>
                  </div>
                  <div>
                    <dt>{incident.resolvedAt ? "복구" : "마지막 확인"}</dt>
                    <dd>
                      <time dateTime={endAt}>{formatTime(endAt)}</time>
                    </dd>
                  </div>
                </dl>
              </li>
            );
          })}
        </ul>
      ) : null}
    </section>
  );
}
