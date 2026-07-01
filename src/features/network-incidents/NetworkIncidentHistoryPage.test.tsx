import { fireEvent, render, screen, within } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  incidentWithoutEvidenceFixture,
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

  it("does not render row links before or after selecting a detail", async () => {
    getIncidentsMock.mockResolvedValue([ongoingIncidentFixture]);

    render(<NetworkIncidentHistoryPage onBack={vi.fn()} />);

    fireEvent.click(await screen.findByRole("button", { name: "상세 보기" }));

    expect(screen.queryByRole("link")).not.toBeInTheDocument();
  });

  it("opens a read-only detail panel for a selected incident", async () => {
    getIncidentsMock.mockResolvedValue([ongoingIncidentFixture]);

    render(<NetworkIncidentHistoryPage onBack={vi.fn()} />);
    fireEvent.click(await screen.findByRole("button", { name: "상세 보기" }));

    const detail = screen.getByRole("region", { name: "선택한 장애 상세" });
    expect(within(detail).getByText("진행 중")).toBeInTheDocument();
    expect(
      within(detail).getByText("공유기 또는 로컬 연결 구간"),
    ).toBeInTheDocument();
    expect(within(detail).getByText("아직 복구 기록 없음")).toBeInTheDocument();
    expect(within(detail).getByText("게이트웨이")).toBeInTheDocument();
    expect(within(detail).getByText("시간 초과")).toBeInTheDocument();
    expect(within(detail).getByText("1000ms")).toBeInTheDocument();
    expect(within(detail).getByText("gateway timeout")).toBeInTheDocument();
  });

  it("opens the selected incident through the detail callback when provided", async () => {
    getIncidentsMock.mockResolvedValue([ongoingIncidentFixture]);
    const onOpenDetail = vi.fn();

    render(
      <NetworkIncidentHistoryPage
        onBack={vi.fn()}
        onOpenDetail={onOpenDetail}
      />,
    );
    fireEvent.click(await screen.findByRole("button", { name: "상세 보기" }));

    expect(onOpenDetail).toHaveBeenCalledWith(ongoingIncidentFixture);
    expect(
      screen.queryByRole("region", { name: "선택한 장애 상세" }),
    ).not.toBeInTheDocument();
  });

  it("closes the detail panel when the close action is pressed", async () => {
    getIncidentsMock.mockResolvedValue([ongoingIncidentFixture]);

    render(<NetworkIncidentHistoryPage onBack={vi.fn()} />);
    fireEvent.click(await screen.findByRole("button", { name: "상세 보기" }));
    fireEvent.click(screen.getByRole("button", { name: "상세 닫기" }));

    expect(
      screen.queryByRole("region", { name: "선택한 장애 상세" }),
    ).not.toBeInTheDocument();
  });

  it("shows an empty evidence message when the selected incident has no representative evidence", async () => {
    getIncidentsMock.mockResolvedValue([incidentWithoutEvidenceFixture]);

    render(<NetworkIncidentHistoryPage onBack={vi.fn()} />);
    fireEvent.click(await screen.findByRole("button", { name: "상세 보기" }));

    expect(screen.getByText("저장된 대표 근거가 없습니다.")).toBeInTheDocument();
  });

  it("clears the selected incident when filters remove it from the list", async () => {
    getIncidentsMock.mockResolvedValue([ongoingIncidentFixture]);

    render(<NetworkIncidentHistoryPage onBack={vi.fn()} />);
    fireEvent.click(await screen.findByRole("button", { name: "상세 보기" }));
    expect(
      screen.getByRole("region", { name: "선택한 장애 상세" }),
    ).toBeInTheDocument();

    fireEvent.change(screen.getByLabelText("상태"), {
      target: { value: "resolved" },
    });

    expect(
      await screen.findByText("조건에 맞는 장애 기록이 없습니다."),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("region", { name: "선택한 장애 상세" }),
    ).not.toBeInTheDocument();
  });

  it("does not expose destructive or repair actions in the detail panel", async () => {
    getIncidentsMock.mockResolvedValue([ongoingIncidentFixture]);

    render(<NetworkIncidentHistoryPage onBack={vi.fn()} />);
    fireEvent.click(await screen.findByRole("button", { name: "상세 보기" }));

    expect(screen.queryByRole("button", { name: /삭제/ })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /복구/ })).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /내보내기|export/i }),
    ).not.toBeInTheDocument();
  });

  it("keeps the detail area stacked below the incident list to avoid overlap", async () => {
    // @ts-expect-error Node fs types are not part of the browser app build.
    const { readFileSync } = await import("node:fs");
    const historyPageCss = readFileSync(
      "src/features/network-incidents/NetworkIncidentHistoryPage.module.css",
      "utf8",
    );

    expect(historyPageCss).toMatch(
      /\.content\s*{[^}]*grid-template-columns:\s*1fr;/s,
    );
    expect(historyPageCss).not.toMatch(/\.content\s*{[^}]*minmax\(300px/s);
  });
});
