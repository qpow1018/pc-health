import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { NetworkIncident } from "@/features/network-incidents/types";
import HomePage from "./HomePage";

const selectedIncident = {
  id: 3,
  status: "ongoing",
  area: "gateway_or_local",
  startedAt: "2026-06-30T00:02:00Z",
  lastObservedAt: "2026-06-30T00:02:20Z",
  resolvedAt: null,
  summary: "내 PC 또는 공유기에서 문제 근거가 확인되었습니다.",
  representativeEvidence: [],
} satisfies NetworkIncident;

vi.mock("@/features/product-home/ProductHome", () => ({
  default: ({ onOpenIncidentHistory }: { onOpenIncidentHistory: () => void }) => (
    <section>
      <h2>현재 진단 화면</h2>
      <button type="button" onClick={onOpenIncidentHistory}>
        open history
      </button>
    </section>
  ),
}));

vi.mock("@/features/network-incidents/NetworkIncidentHistoryPage", () => ({
  default: ({
    onBack,
    onOpenDetail,
  }: {
    onBack?: () => void;
    onOpenDetail?: (incident: NetworkIncident) => void;
  }) => (
    <section>
      <h1>장애 이력</h1>
      {onBack ? (
        <button type="button" onClick={onBack}>
          돌아가기
        </button>
      ) : null}
      {onOpenDetail ? (
        <button type="button" onClick={() => onOpenDetail(selectedIncident)}>
          상세 보기
        </button>
      ) : null}
    </section>
  ),
}));

vi.mock("@/features/network-incidents/NetworkIncidentDetailPage", () => ({
  default: ({
    incident,
    onBack,
  }: {
    incident: NetworkIncident;
    onBack: () => void;
  }) => (
    <section>
      <h1>장애 상세 #{incident.id}</h1>
      <button type="button" onClick={onBack}>
        장애 이력으로 돌아가기
      </button>
    </section>
  ),
}));

describe("HomePage", () => {
  it("switches between home and incident history views from the app header", () => {
    render(<HomePage />);

    expect(screen.getByRole("banner")).toHaveTextContent("Net Checker");
    expect(
      screen.getByRole("button", { name: "현재 진단" }),
    ).toHaveAttribute("aria-current", "page");
    expect(
      screen.getByRole("button", { name: "장애 이력" }),
    ).not.toHaveAttribute("aria-current");

    fireEvent.click(screen.getByRole("button", { name: "장애 이력" }));
    expect(screen.getByRole("heading", { name: "장애 이력" })).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "장애 이력" }),
    ).toHaveAttribute("aria-current", "page");
    expect(screen.queryByRole("button", { name: "돌아가기" })).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "현재 진단" }));
    expect(
      screen.getByRole("heading", { name: "현재 진단 화면" }),
    ).toBeInTheDocument();
  });

  it("keeps the recent incidents shortcut wired to the same history view", () => {
    render(<HomePage />);

    fireEvent.click(screen.getByRole("button", { name: "open history" }));
    expect(screen.getByRole("heading", { name: "장애 이력" })).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "현재 진단" }));
    expect(screen.getByRole("button", { name: "open history" })).toBeInTheDocument();
  });

  it("opens a selected incident as its own detail view", () => {
    render(<HomePage />);

    fireEvent.click(screen.getByRole("button", { name: "장애 이력" }));
    fireEvent.click(screen.getByRole("button", { name: "상세 보기" }));

    expect(
      screen.getByRole("heading", { name: "장애 상세 #3" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "장애 이력" }),
    ).toHaveAttribute("aria-current", "page");

    fireEvent.click(screen.getByRole("button", { name: "장애 이력으로 돌아가기" }));
    expect(screen.getByRole("heading", { name: "장애 이력" })).toBeInTheDocument();
  });
});
