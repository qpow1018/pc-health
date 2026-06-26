import type { ProbeError } from "@/features/network-probe/types";

export type RuntimeAvailability =
  | "starting"
  | "running"
  | "unavailable"
  | "error";

export type DiagnosticLifecycle =
  | "normal"
  | "suspected"
  | "incident"
  | "recovering"
  | "resolved";

export type DiagnosticArea =
  | "local_connection"
  | "gateway_or_local"
  | "dns"
  | "external"
  | "unknown";

export type EvidenceSource =
  | "ethernet"
  | "ipv4"
  | "default_route"
  | "gateway"
  | "dns_microsoft"
  | "dns_google"
  | "http_microsoft"
  | "http_google";

export type EvidenceStatus =
  | "success"
  | "failure"
  | "timeout"
  | "unavailable"
  | "not_checked";

export interface DiagnosticEvidence {
  source: EvidenceSource;
  status: EvidenceStatus;
  checkedAt: string | null;
  durationMs: number | null;
  detail: string | null;
}

export interface NetworkDiagnosticStatus {
  availability: RuntimeAvailability;
  lifecycle: DiagnosticLifecycle | null;
  suspectedArea: DiagnosticArea | null;
  observedAt: string | null;
  lastFullProbeAt: string | null;
  evidence: DiagnosticEvidence[];
  error: ProbeError | null;
}
