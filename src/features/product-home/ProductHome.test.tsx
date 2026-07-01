import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import ProductHome from "./ProductHome";

const { openHistoryMock } = vi.hoisted(() => ({
  openHistoryMock: vi.fn(),
}));

vi.mock("@/features/network-diagnostics/NetworkStatusPanel", () => ({
  default: () => <section>현재 네트워크 진단 상태</section>,
}));

vi.mock("@/features/network-incidents/RecentIncidentsPanel", () => ({
  default: ({ onOpenHistory }: { onOpenHistory?: () => void }) => {
    openHistoryMock.mockImplementation(onOpenHistory ?? vi.fn());
    return <section>최근 장애</section>;
  },
}));

describe("ProductHome", () => {
  it("shows network diagnostics as a compact header status", () => {
    render(<ProductHome onOpenIncidentHistory={vi.fn()} />);

    expect(screen.getByText("인터넷 장애 진단")).toBeInTheDocument();
    expect(screen.getByText("활성")).toBeInTheDocument();
    expect(screen.queryByLabelText("제품 영역")).not.toBeInTheDocument();
    expect(
      screen.queryByRole("heading", { name: "드라이버 관리" }),
    ).not.toBeInTheDocument();
    expect(screen.queryByText("기획 중")).not.toBeInTheDocument();
    expect(screen.getByText("현재 네트워크 진단 상태")).toBeInTheDocument();
    expect(screen.getByText("최근 장애")).toBeInTheDocument();
    expect(screen.queryByText(/성능 모니터/)).not.toBeInTheDocument();
  });

  it("passes the incident history action to the recent incidents panel", () => {
    const onOpenIncidentHistory = vi.fn();

    render(<ProductHome onOpenIncidentHistory={onOpenIncidentHistory} />);
    openHistoryMock();

    expect(onOpenIncidentHistory).toHaveBeenCalledTimes(1);
  });
});
