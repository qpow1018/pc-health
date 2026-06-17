import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import Dashboard from "./Dashboard";
import type {
  SensorDiagnostics,
  SensorSnapshot,
} from "@/features/sensors/types";

const snapshot: SensorSnapshot = {
  collectedAt: "2026-06-15T12:00:00Z",
  devices: [
    { kind: "cpu", name: "Mock CPU", readings: [] },
    { kind: "gpu", name: "Mock GPU", readings: [] },
    { kind: "memory", name: "Mock Memory", readings: [] },
  ],
};

const diagnostics: SensorDiagnostics = {
  collectedAt: "now",
  collector: "windows-native",
  durationMs: 10,
  snapshot,
};

describe("Dashboard", () => {
  it("renders the CPU card and changes scenarios", async () => {
    const onScenarioChange = vi.fn();
    render(
      <Dashboard
        mode="development"
        snapshot={snapshot}
        error={null}
        scenario="normal"
        onModeChange={() => {}}
        onScenarioChange={onScenarioChange}
      />,
    );

    expect(screen.getByTestId("cpu-card")).toBeInTheDocument();
    expect(screen.queryByTestId("gpu-card")).not.toBeInTheDocument();
    expect(screen.queryByTestId("memory-card")).not.toBeInTheDocument();

    expect(
      screen.getByRole("button", { name: "실제 모드" }),
    ).toHaveAttribute("aria-pressed", "false");
    expect(
      screen.getByRole("button", { name: "개발 모드" }),
    ).toHaveAttribute("aria-pressed", "true");

    await userEvent.selectOptions(
      screen.getByLabelText("개발 시나리오"),
      "error",
    );
    expect(onScenarioChange).toHaveBeenCalledWith("error");
  });

  it("keeps cards visible while showing a command error", () => {
    render(
      <Dashboard
        mode="live"
        snapshot={snapshot}
        error="센서 데이터를 불러오지 못했습니다. 다시 시도합니다."
        scenario="normal"
        onModeChange={() => {}}
        onScenarioChange={() => {}}
      />,
    );
    expect(screen.getByRole("alert")).toHaveTextContent("다시 시도합니다");
    expect(screen.getByTestId("cpu-card")).toBeInTheDocument();
    expect(screen.queryByLabelText("개발 시나리오")).not.toBeInTheDocument();
  });

  it("switches between live and development modes", async () => {
    const onModeChange = vi.fn();
    render(
      <Dashboard
        mode="live"
        snapshot={snapshot}
        error={null}
        scenario="normal"
        onModeChange={onModeChange}
        onScenarioChange={() => {}}
      />,
    );

    await userEvent.click(screen.getByRole("button", { name: "개발 모드" }));
    expect(onModeChange).toHaveBeenCalledWith("development");
  });

  it("shows diagnostics controls only in live mode", async () => {
    const onCaptureDiagnostics = vi.fn();
    const { rerender } = render(
      <Dashboard
        mode="live"
        snapshot={snapshot}
        error={null}
        scenario="normal"
        diagnostics={diagnostics}
        diagnosticsError={null}
        isDiagnosticsLoading={false}
        onCaptureDiagnostics={onCaptureDiagnostics}
        onModeChange={() => {}}
        onScenarioChange={() => {}}
      />,
    );

    await userEvent.click(screen.getByRole("button", { name: "진단 캡처" }));
    expect(onCaptureDiagnostics).toHaveBeenCalledTimes(1);
    expect(screen.getByLabelText("센서 진단")).toBeInTheDocument();

    rerender(
      <Dashboard
        mode="development"
        snapshot={snapshot}
        error={null}
        scenario="normal"
        diagnostics={diagnostics}
        diagnosticsError={null}
        isDiagnosticsLoading={false}
        onCaptureDiagnostics={onCaptureDiagnostics}
        onModeChange={() => {}}
        onScenarioChange={() => {}}
      />,
    );

    expect(screen.queryByLabelText("센서 진단")).not.toBeInTheDocument();
  });
});
