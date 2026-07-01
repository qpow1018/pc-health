import type { DiagnosticArea } from "@/features/network-diagnostics/types";
import type { NetworkIncident, NetworkIncidentStatus } from "./types";

export type IncidentHistoryPeriod = "all" | "24h" | "7d" | "30d";
export type IncidentHistoryStatus = "all" | NetworkIncidentStatus;
export type IncidentHistoryArea = "all" | DiagnosticArea;

export interface IncidentHistoryFilters {
  period: IncidentHistoryPeriod;
  status: IncidentHistoryStatus;
  area: IncidentHistoryArea;
}

const periodDurations: Record<Exclude<IncidentHistoryPeriod, "all">, number> = {
  "24h": 24 * 60 * 60 * 1000,
  "7d": 7 * 24 * 60 * 60 * 1000,
  "30d": 30 * 24 * 60 * 60 * 1000,
};

export const defaultIncidentHistoryFilters: IncidentHistoryFilters = {
  period: "all",
  status: "all",
  area: "all",
};

export function filterNetworkIncidents(
  incidents: NetworkIncident[],
  filters: IncidentHistoryFilters,
  now = new Date(),
) {
  return incidents.filter((incident) => {
    if (filters.status !== "all" && incident.status !== filters.status) {
      return false;
    }
    if (filters.area !== "all" && incident.area !== filters.area) {
      return false;
    }
    if (filters.period === "all") return true;

    const startedAt = new Date(incident.startedAt).getTime();
    if (Number.isNaN(startedAt)) return false;
    return startedAt >= now.getTime() - periodDurations[filters.period];
  });
}

export function formatIncidentDuration(incident: NetworkIncident) {
  const startedAt = new Date(incident.startedAt).getTime();
  const endedAt = new Date(incident.resolvedAt ?? incident.lastObservedAt).getTime();
  if (Number.isNaN(startedAt) || Number.isNaN(endedAt) || endedAt < startedAt) {
    return "-";
  }

  const totalMinutes = Math.max(1, Math.round((endedAt - startedAt) / 60000));
  const hours = Math.floor(totalMinutes / 60);
  const minutes = totalMinutes % 60;
  if (hours > 0 && minutes > 0) return `${hours}시간 ${minutes}분`;
  if (hours > 0) return `${hours}시간`;
  return `${minutes}분`;
}
