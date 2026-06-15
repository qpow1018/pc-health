import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import Dashboard from "./Dashboard";
import type { SensorSnapshot } from "../sensors/types";

const snapshot: SensorSnapshot = {
  collectedAt: "2026-06-15T12:00:00Z",
  devices: [
    { kind: "cpu", name: "Mock CPU", readings: [] },
    { kind: "gpu", name: "Mock GPU", readings: [] },
    { kind: "memory", name: "Mock Memory", readings: [] },
  ],
};

describe("Dashboard", () => {
  it("renders all device cards and changes scenarios", async () => {
    const onScenarioChange = vi.fn();
    render(
      <Dashboard
        snapshot={snapshot}
        error={null}
        scenario="normal"
        onScenarioChange={onScenarioChange}
      />,
    );

    expect(screen.getByTestId("cpu-card")).toBeInTheDocument();
    expect(screen.getByTestId("gpu-card")).toBeInTheDocument();
    expect(screen.getByTestId("memory-card")).toBeInTheDocument();

    await userEvent.selectOptions(
      screen.getByLabelText("개발 시나리오"),
      "error",
    );
    expect(onScenarioChange).toHaveBeenCalledWith("error");
  });

  it("keeps cards visible while showing a command error", () => {
    render(
      <Dashboard
        snapshot={snapshot}
        error="센서 데이터를 불러오지 못했습니다. 다시 시도합니다."
        scenario="normal"
        onScenarioChange={() => {}}
      />,
    );
    expect(screen.getByRole("alert")).toHaveTextContent("다시 시도합니다");
    expect(screen.getByTestId("cpu-card")).toBeInTheDocument();
  });
});
