import type { NetworkDiagnosticStatus } from "@/features/network-diagnostics/types";
import type { NetworkIncident } from "@/features/network-incidents/types";

export type NetworkMockScenario =
  | "normal"
  | "gateway-incident"
  | "dns-incident"
  | "external-incident"
  | "recovering"
  | "empty"
  | "error";

export const networkMockScenarios: NetworkMockScenario[] = [
  "normal",
  "gateway-incident",
  "dns-incident",
  "external-incident",
  "recovering",
  "empty",
  "error",
];

const observedAt = "2026-07-01T01:24:20Z";
const lastFullProbeAt = "2026-07-01T01:24:18Z";

const baseEvidence: NetworkDiagnosticStatus["evidence"] = [
  {
    source: "ethernet",
    status: "success",
    checkedAt: "2026-07-01T01:24:14Z",
    durationMs: null,
    detail: "Intel Ethernet Controller 감지",
  },
  {
    source: "ipv4",
    status: "success",
    checkedAt: "2026-07-01T01:24:14Z",
    durationMs: null,
    detail: "192.168.0.23",
  },
  {
    source: "default_route",
    status: "success",
    checkedAt: "2026-07-01T01:24:14Z",
    durationMs: null,
    detail: "기본 경로 192.168.0.1",
  },
  {
    source: "gateway",
    status: "success",
    checkedAt: "2026-07-01T01:24:15Z",
    durationMs: 3,
    detail: "192.168.0.1 응답",
  },
  {
    source: "dns_microsoft",
    status: "success",
    checkedAt: "2026-07-01T01:24:16Z",
    durationMs: 11,
    detail: "www.msftconnecttest.com 확인",
  },
  {
    source: "dns_google",
    status: "success",
    checkedAt: "2026-07-01T01:24:16Z",
    durationMs: 10,
    detail: "connectivitycheck.gstatic.com 확인",
  },
  {
    source: "http_microsoft",
    status: "success",
    checkedAt: "2026-07-01T01:24:17Z",
    durationMs: 42,
    detail: "connecttest.txt 응답 일치",
  },
  {
    source: "http_google",
    status: "success",
    checkedAt: "2026-07-01T01:24:18Z",
    durationMs: 39,
    detail: "HTTP 204",
  },
];

const normalStatus: NetworkDiagnosticStatus = {
  availability: "running",
  lifecycle: "normal",
  suspectedArea: null,
  observedAt,
  lastFullProbeAt,
  evidence: baseEvidence,
  error: null,
};

const gatewayIncidentStatus: NetworkDiagnosticStatus = {
  ...normalStatus,
  lifecycle: "incident",
  suspectedArea: "gateway_or_local",
  observedAt: "2026-07-01T01:25:20Z",
  lastFullProbeAt: "2026-07-01T01:25:19Z",
  evidence: baseEvidence.map((item) =>
    item.source === "gateway"
      ? {
          ...item,
          status: "timeout",
          checkedAt: "2026-07-01T01:25:20Z",
          durationMs: 1000,
          detail: "기본 게이트웨이 응답 시간 초과",
        }
      : item,
  ),
};

const dnsIncidentStatus: NetworkDiagnosticStatus = {
  ...normalStatus,
  lifecycle: "incident",
  suspectedArea: "dns",
  observedAt: "2026-07-01T01:26:12Z",
  lastFullProbeAt: "2026-07-01T01:26:10Z",
  evidence: baseEvidence.map((item) => {
    if (item.source === "dns_microsoft" || item.source === "dns_google") {
      return {
        ...item,
        status: "timeout",
        checkedAt: "2026-07-01T01:26:11Z",
        durationMs: 1000,
        detail: "DNS 응답 시간 초과",
      };
    }
    if (item.source === "http_microsoft" || item.source === "http_google") {
      return {
        ...item,
        status: "not_checked",
        checkedAt: null,
        durationMs: null,
        detail: null,
      };
    }
    return item;
  }),
};

const externalIncidentStatus: NetworkDiagnosticStatus = {
  ...normalStatus,
  lifecycle: "incident",
  suspectedArea: "external",
  observedAt: "2026-07-01T01:27:08Z",
  lastFullProbeAt: "2026-07-01T01:27:06Z",
  evidence: baseEvidence.map((item) =>
    item.source === "http_microsoft" || item.source === "http_google"
      ? {
          ...item,
          status: "timeout",
          checkedAt: "2026-07-01T01:27:08Z",
          durationMs: 1000,
          detail: "연결 확인 endpoint 응답 시간 초과",
        }
      : item,
  ),
};

const recoveringStatus: NetworkDiagnosticStatus = {
  ...normalStatus,
  lifecycle: "recovering",
  suspectedArea: "gateway_or_local",
  observedAt: "2026-07-01T01:28:24Z",
  lastFullProbeAt: "2026-07-01T01:28:22Z",
};

const errorStatus: NetworkDiagnosticStatus = {
  availability: "error",
  lifecycle: null,
  suspectedArea: null,
  observedAt: null,
  lastFullProbeAt: null,
  evidence: [],
  error: {
    stage: "command",
    code: "mock_error",
    message: "목업 진단 데이터를 불러올 수 없습니다.",
    nativeCode: null,
  },
};

const gatewayIncident: NetworkIncident = {
  id: 301,
  status: "ongoing",
  area: "gateway_or_local",
  startedAt: "2026-07-01T01:25:00Z",
  lastObservedAt: "2026-07-01T01:25:20Z",
  resolvedAt: null,
  summary: "내 PC 또는 공유기에서 시간 초과 근거가 확인되었습니다.",
  representativeEvidence: [
    {
      source: "gateway",
      status: "timeout",
      checkedAt: "2026-07-01T01:25:20Z",
      durationMs: 1000,
      detail: "기본 게이트웨이 응답 시간 초과",
      observedAt: "2026-07-01T01:25:20Z",
    },
  ],
};

const dnsIncident: NetworkIncident = {
  id: 302,
  status: "ongoing",
  area: "dns",
  startedAt: "2026-07-01T01:26:00Z",
  lastObservedAt: "2026-07-01T01:26:12Z",
  resolvedAt: null,
  summary: "DNS에서 시간 초과 근거가 확인되었습니다.",
  representativeEvidence: [
    {
      source: "dns_microsoft",
      status: "timeout",
      checkedAt: "2026-07-01T01:26:11Z",
      durationMs: 1000,
      detail: "www.msftconnecttest.com 확인 시간 초과",
      observedAt: "2026-07-01T01:26:12Z",
    },
    {
      source: "dns_google",
      status: "timeout",
      checkedAt: "2026-07-01T01:26:11Z",
      durationMs: 1000,
      detail: "connectivitycheck.gstatic.com 확인 시간 초과",
      observedAt: "2026-07-01T01:26:12Z",
    },
  ],
};

const externalIncident: NetworkIncident = {
  id: 303,
  status: "ongoing",
  area: "external",
  startedAt: "2026-07-01T01:27:00Z",
  lastObservedAt: "2026-07-01T01:27:08Z",
  resolvedAt: null,
  summary: "외부 연결에서 시간 초과 근거가 확인되었습니다.",
  representativeEvidence: [
    {
      source: "http_microsoft",
      status: "timeout",
      checkedAt: "2026-07-01T01:27:08Z",
      durationMs: 1000,
      detail: "connecttest.txt 응답 시간 초과",
      observedAt: "2026-07-01T01:27:08Z",
    },
    {
      source: "http_google",
      status: "timeout",
      checkedAt: "2026-07-01T01:27:08Z",
      durationMs: 1000,
      detail: "HTTP 204 확인 시간 초과",
      observedAt: "2026-07-01T01:27:08Z",
    },
  ],
};

const recoveringIncident: NetworkIncident = {
  ...gatewayIncident,
  id: 304,
  status: "recovering",
  startedAt: "2026-07-01T01:22:00Z",
  lastObservedAt: "2026-07-01T01:28:24Z",
  summary: "내 PC 또는 공유기가 정상으로 돌아왔는지 확인하고 있습니다.",
  representativeEvidence: [
    {
      source: "gateway",
      status: "success",
      checkedAt: "2026-07-01T01:28:24Z",
      durationMs: 4,
      detail: "192.168.0.1 응답",
      observedAt: "2026-07-01T01:28:24Z",
    },
  ],
};

const resolvedIncident: NetworkIncident = {
  ...gatewayIncident,
  id: 305,
  status: "resolved",
  startedAt: "2026-07-01T00:50:00Z",
  lastObservedAt: "2026-07-01T00:54:20Z",
  resolvedAt: "2026-07-01T00:54:20Z",
  summary: "네트워크 장애가 복구되었습니다.",
};

const unknownIncident: NetworkIncident = {
  ...gatewayIncident,
  id: 306,
  status: "resolved",
  area: "unknown",
  startedAt: "2026-07-01T00:35:00Z",
  lastObservedAt: "2026-07-01T00:36:00Z",
  resolvedAt: "2026-07-01T00:36:00Z",
  summary: "확인 불가 상태로 저장된 장애 기록입니다.",
  representativeEvidence: [],
};

export function getNetworkMockScenario() {
  if (typeof window === "undefined") return null;

  const scenario = new URLSearchParams(window.location.search).get("mock");
  return networkMockScenarios.includes(scenario as NetworkMockScenario)
    ? (scenario as NetworkMockScenario)
    : null;
}

export function getMockNetworkDiagnosticStatus(
  scenario: NetworkMockScenario,
): NetworkDiagnosticStatus {
  if (scenario === "gateway-incident") return gatewayIncidentStatus;
  if (scenario === "dns-incident") return dnsIncidentStatus;
  if (scenario === "external-incident") return externalIncidentStatus;
  if (scenario === "recovering") return recoveringStatus;
  if (scenario === "error") return errorStatus;
  return normalStatus;
}

export function getMockNetworkIncidents(
  scenario: NetworkMockScenario,
): NetworkIncident[] {
  if (scenario === "empty") return [];
  if (scenario === "gateway-incident") {
    return [gatewayIncident, recoveringIncident, resolvedIncident];
  }
  if (scenario === "dns-incident") {
    return [dnsIncident, resolvedIncident, unknownIncident];
  }
  if (scenario === "external-incident") {
    return [externalIncident, resolvedIncident, unknownIncident];
  }
  if (scenario === "recovering") {
    return [recoveringIncident, resolvedIncident, unknownIncident];
  }
  if (scenario === "error") {
    throw new Error("mock incident history unavailable");
  }
  return [resolvedIncident, recoveringIncident, unknownIncident];
}
