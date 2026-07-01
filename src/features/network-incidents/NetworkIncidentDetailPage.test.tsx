import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ongoingIncidentFixture } from "./fixture";
import NetworkIncidentDetailPage from "./NetworkIncidentDetailPage";

describe("NetworkIncidentDetailPage", () => {
  it("renders a selected incident as a read-only detail page", () => {
    render(
      <NetworkIncidentDetailPage
        incident={ongoingIncidentFixture}
        onBack={vi.fn()}
      />,
    );

    expect(
      screen.getByRole("heading", { name: "장애 상세" }),
    ).toBeInTheDocument();
    expect(screen.getByText("진행 중")).toBeInTheDocument();
    expect(screen.getByText("내 PC 또는 공유기")).toBeInTheDocument();
    expect(screen.getByText("아직 복구 기록 없음")).toBeInTheDocument();
    expect(screen.getByText("대표 근거")).toBeInTheDocument();
    expect(screen.getByText("게이트웨이")).toBeInTheDocument();
    expect(screen.getByText("시간 초과")).toBeInTheDocument();
    expect(screen.getByText("1000ms")).toBeInTheDocument();
    expect(screen.getByText("gateway timeout")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /삭제|복구/ })).not.toBeInTheDocument();
  });

  it("returns to the history page from the detail page", () => {
    const onBack = vi.fn();

    render(
      <NetworkIncidentDetailPage
        incident={ongoingIncidentFixture}
        onBack={onBack}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: "장애 이력으로 돌아가기" }));

    expect(onBack).toHaveBeenCalledTimes(1);
  });
});
