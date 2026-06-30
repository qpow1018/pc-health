import type {
  DiagnosticArea,
  DiagnosticEvidence,
} from "@/features/network-diagnostics/types";

export type NetworkIncidentStatus = "ongoing" | "recovering" | "resolved";

export interface NetworkIncidentEvidence extends DiagnosticEvidence {
  observedAt: string;
}

export interface NetworkIncident {
  id: number;
  status: NetworkIncidentStatus;
  area: DiagnosticArea;
  startedAt: string;
  lastObservedAt: string;
  resolvedAt: string | null;
  summary: string;
  representativeEvidence: NetworkIncidentEvidence[];
}
