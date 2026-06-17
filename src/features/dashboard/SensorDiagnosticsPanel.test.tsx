import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import SensorDiagnosticsPanel from "./SensorDiagnosticsPanel";
import type { SensorDiagnostics } from "@/features/sensors/types";

const diagnostics: SensorDiagnostics = {
  collectedAt: "2026-06-17T12:00:00Z",
  collector: "windows-native",
  durationMs: 42,
  rawPayload: '{"CpuUsage":37}',
  rawError: "parse failed",
  parsedTelemetry: {
    cpuName: "AMD Ryzen",
    cpuUsage: 37,
    cpuClockMhz: 4200,
    totalMemoryKb: 33554432,
    freeMemoryKb: 16777216,
  },
  snapshot: {
    collectedAt: "2026-06-17T12:00:00Z",
    devices: [{ kind: "cpu", name: "AMD Ryzen", readings: [] }],
  },
};

describe("SensorDiagnosticsPanel", () => {
  it("captures diagnostics on user action", async () => {
    const onCapture = vi.fn();

    render(
      <SensorDiagnosticsPanel
        diagnostics={null}
        error={null}
        isLoading={false}
        onCapture={onCapture}
      />,
    );

    await userEvent.click(screen.getByRole("button", { name: "진단 캡처" }));

    expect(onCapture).toHaveBeenCalledTimes(1);
  });

  it("disables capture while loading", () => {
    render(
      <SensorDiagnosticsPanel
        diagnostics={null}
        error={null}
        isLoading
        onCapture={() => {}}
      />,
    );

    expect(screen.getByRole("button", { name: "진단 중" })).toBeDisabled();
  });

  it("renders raw, parsed, final snapshot, and error details", () => {
    render(
      <SensorDiagnosticsPanel
        diagnostics={diagnostics}
        error={null}
        isLoading={false}
        onCapture={() => {}}
      />,
    );

    expect(screen.getByText("windows-native")).toBeInTheDocument();
    expect(screen.getByText("42ms")).toBeInTheDocument();
    expect(screen.getByText(/CpuUsage/)).toBeInTheDocument();
    expect(screen.getAllByText(/AMD Ryzen/).length).toBeGreaterThan(1);
    expect(screen.getByText(/parse failed/)).toBeInTheDocument();
    expect(screen.getByText(/devices/)).toBeInTheDocument();
  });

  it("shows diagnostics errors", () => {
    render(
      <SensorDiagnosticsPanel
        diagnostics={null}
        error="진단 정보를 불러오지 못했습니다."
        isLoading={false}
        onCapture={() => {}}
      />,
    );

    expect(screen.getByRole("alert")).toHaveTextContent(
      "진단 정보를 불러오지 못했습니다.",
    );
  });

  it("labels sub-millisecond diagnostics duration clearly", () => {
    render(
      <SensorDiagnosticsPanel
        diagnostics={{ ...diagnostics, durationMs: 0 }}
        error={null}
        isLoading={false}
        onCapture={() => {}}
      />,
    );

    expect(screen.getByText("<1ms")).toBeInTheDocument();
  });
});
