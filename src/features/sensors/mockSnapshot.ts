import type { MockScenario, SensorReading, SensorSnapshot, SensorValue } from "./types";

function available(
  kind: string,
  label: string,
  value: number,
  unit: string,
): SensorReading {
  return {
    kind,
    label,
    value: { status: "available", value, unit },
  };
}

function status(
  kind: string,
  label: string,
  value: SensorValue,
): SensorReading {
  return { kind, label, value };
}

export function createMockSensorSnapshot(
  scenario: MockScenario,
  sampleIndex: number,
): SensorSnapshot {
  const wave = sampleIndex % 5;
  const threshold = scenario === "threshold";
  const cpuTemperature = threshold ? 92 : 61 + wave;
  const gpuTemperature = threshold ? 87 : 67 + wave;
  const memoryUsage = threshold ? 96 : 54 + wave;

  const cpu: SensorReading[] = [
    available("cpu_usage", "사용률", 42 + wave, "%"),
    available("cpu_temperature", "온도", cpuTemperature, "C"),
    available("cpu_clock", "클럭", 4.2, "GHz"),
    available("cpu_power", "전력", 72 + wave, "W"),
  ];
  const gpu: SensorReading[] = [
    available("gpu_usage", "사용률", threshold ? 96 : 71 + wave, "%"),
    available("gpu_temperature", "온도", gpuTemperature, "C"),
    available("gpu_clock", "클럭", 2.5, "GHz"),
    available("gpu_power", "전력", 185 + wave, "W"),
    available("gpu_fan", "팬", 68, "%"),
    available("gpu_vram", "VRAM", 7.2, "GB"),
  ];
  const memory: SensorReading[] = [
    available("memory_usage", "사용률", memoryUsage, "%"),
    available("memory_used", "사용 중", 17.3 + wave / 10, "GB"),
  ];

  if (threshold) {
    cpu[1].indication = {
      level: "warning",
      message: "온도가 일반적인 권장 범위보다 높습니다.",
    };
    gpu[0].indication = { level: "high-load", message: "높은 부하" };
    gpu[1].indication = {
      level: "warning",
      message: "온도가 일반적인 권장 범위보다 높습니다.",
    };
    memory[0].indication = {
      level: "warning",
      message: "메모리 사용률이 일반적인 권장 범위보다 높습니다.",
    };
  }

  if (scenario === "unsupported") {
    gpu[1] = status("gpu_temperature", "온도", {
      status: "unsupported-device",
    });
    gpu[3] = status("gpu_power", "전력", { status: "unsupported-app" });
    gpu[4] = status("gpu_fan", "팬", { status: "unsupported-device" });
  }

  if (scenario === "waiting") {
    for (const reading of [...cpu, ...gpu, ...memory]) {
      reading.value = { status: "waiting" };
      reading.indication = undefined;
    }
  }

  if (scenario === "error") {
    cpu[1] = status("cpu_temperature", "온도", {
      status: "error",
      message: "온도 센서 응답이 없습니다.",
    });
    gpu[4] = status("gpu_fan", "팬", {
      status: "error",
      message: "팬 속도를 읽지 못했습니다.",
    });
  }

  return {
    collectedAt: new Date().toISOString(),
    devices: [
      {
        kind: "cpu",
        name: "Mock AMD Ryzen 7 7800X3D",
        readings: cpu,
      },
      {
        kind: "gpu",
        name: "Mock NVIDIA GeForce RTX 4070",
        readings: gpu,
      },
      {
        kind: "memory",
        name: "Mock System Memory 32 GB",
        readings: memory,
      },
    ],
  };
}

export function getMockSensorSnapshot(
  scenario: MockScenario,
  sampleIndex: number,
) {
  return Promise.resolve(createMockSensorSnapshot(scenario, sampleIndex));
}
