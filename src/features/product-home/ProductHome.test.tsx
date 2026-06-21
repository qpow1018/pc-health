import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import ProductHome from "./ProductHome";

describe("ProductHome", () => {
  it("shows network diagnostics as active and driver management as planning", () => {
    render(<ProductHome />);

    expect(
      screen.getByRole("heading", { name: "인터넷 장애 진단" }),
    ).toBeInTheDocument();
    expect(screen.getByText("다음 구현 영역")).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "드라이버 관리" }),
    ).toBeInTheDocument();
    expect(screen.getByText("기획 중")).toBeInTheDocument();
    expect(screen.queryByText(/성능 모니터/)).not.toBeInTheDocument();
  });
});
