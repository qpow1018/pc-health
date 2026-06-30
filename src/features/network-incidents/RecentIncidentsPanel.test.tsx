import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  ongoingIncidentFixture,
  recoveringIncidentFixture,
  resolvedIncidentFixture,
  unknownIncidentFixture,
} from "./fixture";
import RecentIncidentsPanel from "./RecentIncidentsPanel";

const { canUseMock, getRecentMock } = vi.hoisted(() => ({
  canUseMock: vi.fn(),
  getRecentMock: vi.fn(),
}));

vi.mock("./api", () => ({
  canUseNetworkIncidents: canUseMock,
  getRecentNetworkIncidents: getRecentMock,
}));

describe("RecentIncidentsPanel", () => {
  beforeEach(() => {
    canUseMock.mockReturnValue(true);
    getRecentMock.mockReset();
  });

  it("shows an empty state when there are no saved incidents", async () => {
    getRecentMock.mockResolvedValue([]);

    render(<RecentIncidentsPanel />);

    expect(
      await screen.findByText("저장된 장애 기록이 없습니다."),
    ).toBeInTheDocument();
  });

  it("renders recent incidents with stable status and area labels", async () => {
    getRecentMock.mockResolvedValue([
      ongoingIncidentFixture,
      recoveringIncidentFixture,
      resolvedIncidentFixture,
    ]);

    render(<RecentIncidentsPanel />);

    expect(await screen.findByText("진행 중")).toBeInTheDocument();
    expect(screen.getByText("복구 확인 중")).toBeInTheDocument();
    expect(screen.getByText("복구됨")).toBeInTheDocument();
    expect(
      screen.getAllByText("공유기 또는 로컬 연결 구간").length,
    ).toBeGreaterThan(0);
  });

  it("limits the main panel to three rows", async () => {
    getRecentMock.mockResolvedValue([
      ongoingIncidentFixture,
      recoveringIncidentFixture,
      resolvedIncidentFixture,
      unknownIncidentFixture,
    ]);

    render(<RecentIncidentsPanel />);

    expect(await screen.findAllByRole("listitem")).toHaveLength(3);
    expect(
      screen.queryByText("확인 불가 구간에서 이상 근거가 반복 확인되었습니다."),
    ).not.toBeInTheDocument();
  });

  it("shows unknown area as unavailable without claiming a cause", async () => {
    getRecentMock.mockResolvedValue([unknownIncidentFixture]);

    render(<RecentIncidentsPanel />);

    expect(await screen.findByText("확인 불가")).toBeInTheDocument();
  });

  it("shows unavailable state in browser mode", async () => {
    canUseMock.mockReturnValue(false);

    render(<RecentIncidentsPanel />);

    expect(
      await screen.findByText("장애 기록을 불러올 수 없습니다."),
    ).toBeInTheDocument();
    expect(getRecentMock).not.toHaveBeenCalled();
  });

  it("shows command failure without claiming a current outage", async () => {
    getRecentMock.mockRejectedValue(new Error("db unavailable"));

    render(<RecentIncidentsPanel />);

    expect(
      await screen.findByText("장애 기록을 불러올 수 없습니다."),
    ).toBeInTheDocument();
    expect(screen.queryByText("장애 확인")).not.toBeInTheDocument();
  });
});
