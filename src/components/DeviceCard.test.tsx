import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import DeviceCard from "./DeviceCard";
import type { DeviceSnapshot } from "../sensors/types";

const device: DeviceSnapshot = {
  kind: "gpu",
  name: "Mock RTX 4070",
  readings: [
    {
      kind: "gpu_usage",
      label: "사용률",
      value: { status: "available", value: 96, unit: "%" },
      indication: { level: "high-load", message: "높은 부하" },
    },
    {
      kind: "gpu_temperature",
      label: "온도",
      value: { status: "unsupported-device" },
    },
    {
      kind: "gpu_power",
      label: "전력",
      value: { status: "unsupported-app" },
    },
    { kind: "gpu_fan", label: "팬", value: { status: "waiting" } },
    {
      kind: "gpu_vram",
      label: "VRAM",
      value: { status: "error", message: "읽기 실패" },
    },
  ],
};

describe("DeviceCard", () => {
  it("renders available values and every explicit status label", () => {
    render(<DeviceCard device={device} />);
    expect(screen.getByText("96%")).toBeInTheDocument();
    expect(screen.getByText("지원하지 않음")).toBeInTheDocument();
    expect(screen.getByText("현재 버전 미지원")).toBeInTheDocument();
    expect(screen.getByText("데이터 대기 중")).toBeInTheDocument();
    expect(screen.getByText("측정 실패")).toBeInTheDocument();
    expect(screen.getByText("높은 부하")).toBeInTheDocument();
  });

  it("uses the strongest indication as the card state", () => {
    const warningDevice: DeviceSnapshot = {
      ...device,
      readings: [
        {
          kind: "gpu_temperature",
          label: "온도",
          value: { status: "available", value: 87, unit: "C" },
          indication: {
            level: "warning",
            message: "온도가 일반적인 권장 범위보다 높습니다.",
          },
        },
      ],
    };
    render(<DeviceCard device={warningDevice} />);
    expect(screen.getByTestId("gpu-card")).toHaveAttribute(
      "data-level",
      "warning",
    );
  });
});
