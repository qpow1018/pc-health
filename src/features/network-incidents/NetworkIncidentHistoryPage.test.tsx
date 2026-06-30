import { fireEvent, render, screen, within } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  ongoingIncidentFixture,
  recoveringIncidentFixture,
  resolvedIncidentFixture,
  unknownIncidentFixture,
} from "./fixture";
import NetworkIncidentHistoryPage from "./NetworkIncidentHistoryPage";

const { canUseMock, getIncidentsMock } = vi.hoisted(() => ({
  canUseMock: vi.fn(),
  getIncidentsMock: vi.fn(),
}));

vi.mock("./api", () => ({
  canUseNetworkIncidents: canUseMock,
  getNetworkIncidents: getIncidentsMock,
}));

describe("NetworkIncidentHistoryPage", () => {
  beforeEach(() => {
    canUseMock.mockReturnValue(true);
    getIncidentsMock.mockReset();
  });

  it("renders the full incident history", async () => {
    getIncidentsMock.mockResolvedValue([
      ongoingIncidentFixture,
      recoveringIncidentFixture,
      resolvedIncidentFixture,
    ]);

    render(<NetworkIncidentHistoryPage onBack={vi.fn()} />);

    expect(await screen.findByText("진행 중")).toBeInTheDocument();
    const rows = screen.getAllByRole("listitem");
    expect(rows).toHaveLength(3);
    expect(within(rows[1]).getByText("복구 확인 중")).toBeInTheDocument();
    expect(within(rows[2]).getByText("복구됨")).toBeInTheDocument();
  });

  it("calls back when the back button is pressed", () => {
    getIncidentsMock.mockResolvedValue([]);
    const onBack = vi.fn();

    render(<NetworkIncidentHistoryPage onBack={onBack} />);
    fireEvent.click(screen.getByRole("button", { name: "돌아가기" }));

    expect(onBack).toHaveBeenCalledTimes(1);
  });

  it("shows an empty state when there are no saved incidents", async () => {
    getIncidentsMock.mockResolvedValue([]);

    render(<NetworkIncidentHistoryPage onBack={vi.fn()} />);

    expect(
      await screen.findByText("저장된 장애 기록이 없습니다."),
    ).toBeInTheDocument();
  });

  it("shows a filtered empty state when filters hide all incidents", async () => {
    getIncidentsMock.mockResolvedValue([ongoingIncidentFixture]);

    render(<NetworkIncidentHistoryPage onBack={vi.fn()} />);
    await screen.findByText("진행 중");
    fireEvent.change(await screen.findByLabelText("상태"), {
      target: { value: "resolved" },
    });

    expect(
      await screen.findByText("조건에 맞는 장애 기록이 없습니다."),
    ).toBeInTheDocument();
  });

  it("filters by status and area", async () => {
    getIncidentsMock.mockResolvedValue([
      ongoingIncidentFixture,
      resolvedIncidentFixture,
      unknownIncidentFixture,
    ]);

    render(<NetworkIncidentHistoryPage onBack={vi.fn()} />);
    await screen.findByText("진행 중");

    fireEvent.change(screen.getByLabelText("상태"), {
      target: { value: "resolved" },
    });
    let rows = screen.getAllByRole("listitem");
    expect(rows).toHaveLength(1);
    expect(within(rows[0]).getByText("복구됨")).toBeInTheDocument();

    fireEvent.change(screen.getByLabelText("상태"), {
      target: { value: "all" },
    });
    fireEvent.change(screen.getByLabelText("구간"), {
      target: { value: "unknown" },
    });
    rows = screen.getAllByRole("listitem");
    expect(rows).toHaveLength(1);
    expect(within(rows[0]).getByText("확인 불가")).toBeInTheDocument();
  });

  it("shows unavailable state without claiming a current outage", async () => {
    getIncidentsMock.mockRejectedValue(new Error("history unavailable"));

    render(<NetworkIncidentHistoryPage onBack={vi.fn()} />);

    expect(
      await screen.findByText("장애 이력을 불러올 수 없습니다."),
    ).toBeInTheDocument();
    expect(screen.queryByText("장애 확인")).not.toBeInTheDocument();
  });

  it("does not render row links or detail actions", async () => {
    getIncidentsMock.mockResolvedValue([ongoingIncidentFixture]);

    render(<NetworkIncidentHistoryPage onBack={vi.fn()} />);

    expect(await screen.findByText("진행 중")).toBeInTheDocument();
    expect(screen.queryByRole("link")).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /상세/ })).not.toBeInTheDocument();
  });
});
