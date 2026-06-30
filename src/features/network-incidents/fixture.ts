import type { NetworkIncident } from "./types";

export const ongoingIncidentFixture: NetworkIncident = {
  id: 3,
  status: "ongoing",
  area: "gateway_or_local",
  startedAt: "2026-06-30T00:02:00Z",
  lastObservedAt: "2026-06-30T00:02:20Z",
  resolvedAt: null,
  summary: "공유기 또는 로컬 연결 구간에서 이상 근거가 반복 확인되었습니다.",
  representativeEvidence: [
    {
      source: "gateway",
      status: "timeout",
      checkedAt: "2026-06-30T00:02:20Z",
      durationMs: 1000,
      detail: "gateway timeout",
      observedAt: "2026-06-30T00:02:20Z",
    },
  ],
};

export const recoveringIncidentFixture: NetworkIncident = {
  ...ongoingIncidentFixture,
  id: 2,
  status: "recovering",
  startedAt: "2026-06-30T00:01:00Z",
  lastObservedAt: "2026-06-30T00:01:20Z",
  summary: "공유기 또는 로컬 연결 구간의 정상 근거를 추가 확인하고 있습니다.",
};

export const resolvedIncidentFixture: NetworkIncident = {
  ...ongoingIncidentFixture,
  id: 1,
  status: "resolved",
  startedAt: "2026-06-30T00:00:00Z",
  lastObservedAt: "2026-06-30T00:00:40Z",
  resolvedAt: "2026-06-30T00:00:40Z",
  summary: "네트워크 장애가 복구되었습니다.",
};

export const unknownIncidentFixture: NetworkIncident = {
  ...ongoingIncidentFixture,
  id: 4,
  area: "unknown",
  summary: "확인 불가 구간에서 이상 근거가 반복 확인되었습니다.",
};

export const incidentWithoutEvidenceFixture: NetworkIncident = {
  ...resolvedIncidentFixture,
  id: 5,
  representativeEvidence: [],
  summary: "대표 근거가 없는 저장 기록입니다.",
};
