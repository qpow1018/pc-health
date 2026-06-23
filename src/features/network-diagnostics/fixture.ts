import type { NetworkDiagnosticStatus } from "./types";

export const startingStatusFixture: NetworkDiagnosticStatus = {
  availability: "starting",
  lifecycle: null,
  suspectedArea: null,
  observedAt: null,
  lastFullProbeAt: null,
  evidence: [],
  error: null,
};

export const normalStatusFixture: NetworkDiagnosticStatus = {
  availability: "running",
  lifecycle: "normal",
  suspectedArea: null,
  observedAt: "2026-06-23T09:00:04Z",
  lastFullProbeAt: "2026-06-23T09:00:00Z",
  evidence: [
    {
      source: "gateway",
      status: "success",
      checkedAt: "2026-06-23T09:00:01Z",
      durationMs: 3,
      detail: "192.168.0.1 응답",
    },
    {
      source: "dns_microsoft",
      status: "success",
      checkedAt: "2026-06-23T09:00:01Z",
      durationMs: 4,
      detail: "www.msftconnecttest.com 확인",
    },
    {
      source: "dns_google",
      status: "success",
      checkedAt: "2026-06-23T09:00:02Z",
      durationMs: 5,
      detail: "connectivitycheck.gstatic.com 확인",
    },
    {
      source: "http_microsoft",
      status: "success",
      checkedAt: "2026-06-23T09:00:03Z",
      durationMs: 12,
      detail: "connecttest.txt 응답 일치",
    },
    {
      source: "http_google",
      status: "success",
      checkedAt: "2026-06-23T09:00:04Z",
      durationMs: 13,
      detail: "HTTP 204",
    },
  ],
  error: null,
};

export const incidentStatusFixture: NetworkDiagnosticStatus = {
  availability: "running",
  lifecycle: "incident",
  suspectedArea: "gateway_or_local",
  observedAt: "2026-06-23T09:01:00Z",
  lastFullProbeAt: "2026-06-23T09:00:58Z",
  evidence: [
    {
      source: "gateway",
      status: "timeout",
      checkedAt: "2026-06-23T09:01:00Z",
      durationMs: 1000,
      detail: "기본 게이트웨이 응답 시간 초과",
    },
  ],
  error: null,
};

export const unavailableStatusFixture: NetworkDiagnosticStatus = {
  availability: "unavailable",
  lifecycle: null,
  suspectedArea: null,
  observedAt: null,
  lastFullProbeAt: null,
  evidence: [],
  error: null,
};
