import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import HomePage from "./HomePage";

vi.mock("@/features/product-home/ProductHome", () => ({
  default: ({ onOpenIncidentHistory }: { onOpenIncidentHistory: () => void }) => (
    <button type="button" onClick={onOpenIncidentHistory}>
      open history
    </button>
  ),
}));

vi.mock("@/features/network-incidents/NetworkIncidentHistoryPage", () => ({
  default: ({ onBack }: { onBack: () => void }) => (
    <section>
      <h1>장애 이력</h1>
      <button type="button" onClick={onBack}>
        돌아가기
      </button>
    </section>
  ),
}));

describe("HomePage", () => {
  it("switches between home and incident history views", () => {
    render(<HomePage />);

    fireEvent.click(screen.getByRole("button", { name: "open history" }));
    expect(screen.getByRole("heading", { name: "장애 이력" })).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "돌아가기" }));
    expect(screen.getByRole("button", { name: "open history" })).toBeInTheDocument();
  });
});
