import type {
  DeviceSnapshot,
  IndicationLevel,
  SensorValue,
} from "@/features/sensors/types";
import styles from "./DeviceCard.module.css";

const levelRank: Record<IndicationLevel, number> = {
  "high-load": 1,
  advisory: 2,
  warning: 3,
};

function formatValue(value: SensorValue) {
  switch (value.status) {
    case "available":
      return `${Number.isInteger(value.value) ? value.value : value.value.toFixed(1)}${value.unit}`;
    case "unsupported-device":
      return "지원하지 않음";
    case "unsupported-app":
      return "현재 버전 미지원";
    case "waiting":
      return "데이터 대기 중";
    case "error":
      return "측정 실패";
  }
}

function cardLevel(device: DeviceSnapshot) {
  return device.readings.reduce<IndicationLevel | undefined>(
    (strongest, reading) => {
      const next = reading.indication?.level;
      if (!next) return strongest;
      if (!strongest || levelRank[next] > levelRank[strongest]) return next;
      return strongest;
    },
    undefined,
  );
}

export default function DeviceCard({ device }: { device: DeviceSnapshot }) {
  const level = cardLevel(device);

  return (
    <section
      className={styles["card"]}
      data-level={level}
      data-testid={`${device.kind}-card`}
    >
      <header className={styles["header"]}>
        <span className={styles["kind"]}>{device.kind.toUpperCase()}</span>
        <h2 className={styles["title"]}>{device.name}</h2>
      </header>
      <dl className={styles["list"]}>
        {device.readings.map((reading) => (
          <div
            className={styles["reading"]}
            data-level={reading.indication?.level}
            key={reading.kind}
          >
            <dt>{reading.label}</dt>
            <dd>
              <span>{formatValue(reading.value)}</span>
              {reading.indication && <small>{reading.indication.message}</small>}
              {reading.value.status === "error" && (
                <small>{reading.value.message}</small>
              )}
            </dd>
          </div>
        ))}
      </dl>
    </section>
  );
}
