import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import HomePage from "./HomePage";

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
  default: ({ onBack }: { onBack?: () => void }) => (
    <section>
      <h1>장애 이력</h1>
      {onBack ? (
        <button type="button" onClick={onBack}>
          돌아가기
        </button>
      ) : null}
    </section>
  ),
}));

describe("HomePage", () => {
  it("switches between home and incident history views from the app header", () => {
    render(<HomePage />);

    expect(screen.getByRole("banner")).toHaveTextContent("PC Health");
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
});
