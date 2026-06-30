import { describe, expect, it } from "vitest";
import {
  ongoingIncidentFixture,
  recoveringIncidentFixture,
  resolvedIncidentFixture,
  unknownIncidentFixture,
} from "./fixture";
import {
  filterNetworkIncidents,
  formatIncidentDuration,
  type IncidentHistoryFilters,
} from "./filter";

const baseFilters: IncidentHistoryFilters = {
  period: "all",
  status: "all",
  area: "all",
};

describe("network incident history filters", () => {
  it("keeps all incidents when every filter is all", () => {
    expect(
      filterNetworkIncidents(
        [ongoingIncidentFixture, recoveringIncidentFixture, resolvedIncidentFixture],
        baseFilters,
        new Date("2026-06-30T12:00:00Z"),
      ),
    ).toHaveLength(3);
  });

  it("filters by status and area", () => {
    const incidents = [
      ongoingIncidentFixture,
      recoveringIncidentFixture,
      resolvedIncidentFixture,
      unknownIncidentFixture,
    ];

    expect(
      filterNetworkIncidents(
        incidents,
        { ...baseFilters, status: "resolved" },
        new Date("2026-06-30T12:00:00Z"),
      ),
    ).toEqual([resolvedIncidentFixture]);
    expect(
      filterNetworkIncidents(
        incidents,
        { ...baseFilters, area: "unknown" },
        new Date("2026-06-30T12:00:00Z"),
      ),
    ).toEqual([unknownIncidentFixture]);
  });

  it("filters period by startedAt", () => {
    const recent = {
      ...ongoingIncidentFixture,
      startedAt: "2026-06-30T11:30:00Z",
      lastObservedAt: "2026-06-30T11:40:00Z",
    };
    const old = {
      ...resolvedIncidentFixture,
      startedAt: "2026-06-28T11:30:00Z",
      lastObservedAt: "2026-06-30T11:40:00Z",
      resolvedAt: "2026-06-30T11:40:00Z",
    };

    expect(
      filterNetworkIncidents(
        [recent, old],
        { ...baseFilters, period: "24h" },
        new Date("2026-06-30T12:00:00Z"),
      ),
    ).toEqual([recent]);
  });

  it("formats duration from resolvedAt or lastObservedAt", () => {
    expect(
      formatIncidentDuration({
        ...resolvedIncidentFixture,
        startedAt: "2026-06-30T00:00:00Z",
        resolvedAt: "2026-06-30T01:05:00Z",
      }),
    ).toBe("1시간 5분");
    expect(
      formatIncidentDuration({
        ...ongoingIncidentFixture,
        startedAt: "2026-06-30T00:00:00Z",
        lastObservedAt: "2026-06-30T00:05:00Z",
      }),
    ).toBe("5분");
  });
});
