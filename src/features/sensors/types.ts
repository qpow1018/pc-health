export type SensorMode = "live" | "development";

export type MockScenario =
  | "normal"
  | "threshold"
  | "unsupported"
  | "waiting"
  | "error";

export type SensorValue =
  | { status: "available"; value: number; unit: string }
  | { status: "unsupported-device" }
  | { status: "unsupported-app" }
  | { status: "waiting" }
  | { status: "error"; message: string };

export type IndicationLevel = "advisory" | "warning" | "high-load";

export type SensorReading = {
  kind: string;
  label: string;
  value: SensorValue;
  indication?: {
    level: IndicationLevel;
    message: string;
  };
};

export type DeviceSnapshot = {
  kind: "cpu" | "gpu" | "memory";
  name: string;
  readings: SensorReading[];
};

export type SensorSnapshot = {
  collectedAt: string;
  devices: DeviceSnapshot[];
};

export type ParsedTelemetry = {
  cpuName?: string;
  cpuUsage?: number;
  cpuTemperatureCelsius?: number;
  cpuClockMhz?: number;
  totalMemoryKb?: number;
  freeMemoryKb?: number;
};

export type SensorDiagnostics = {
  collectedAt: string;
  collector: string;
  durationMs: number;
  rawPayload?: string;
  rawError?: string;
  parsedTelemetry?: ParsedTelemetry;
  snapshot: SensorSnapshot;
};
