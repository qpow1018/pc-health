import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import ProductHome from "./ProductHome";

vi.mock("@/features/network-diagnostics/NetworkStatusPanel", () => ({
  default: () => <section>현재 네트워크 진단 상태</section>,
}));

describe("ProductHome", () => {
  it("shows network diagnostics as the only product area", () => {
    render(<ProductHome />);

    expect(
      screen.getByRole("heading", { name: "인터넷 장애 진단" }),
    ).toBeInTheDocument();
    expect(screen.getByText("활성")).toBeInTheDocument();
    expect(
      screen.queryByRole("heading", { name: "드라이버 관리" }),
    ).not.toBeInTheDocument();
    expect(screen.queryByText("기획 중")).not.toBeInTheDocument();
    expect(screen.getByText("현재 네트워크 진단 상태")).toBeInTheDocument();
    expect(screen.queryByText(/성능 모니터/)).not.toBeInTheDocument();
  });
});
