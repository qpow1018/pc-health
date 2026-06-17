use std::process::Command;

use serde::Deserialize;

use crate::collector::{MockCollector, MockScenario, SensorCollector};
use crate::domain::{DeviceKind, DeviceSnapshot, SensorReading, SensorSnapshot, SensorValue};

const TELEMETRY_SCRIPT: &str = r#"
$cpu = Get-CimInstance Win32_Processor | Select-Object -First 1
$os = Get-CimInstance Win32_OperatingSystem
[pscustomobject]@{
  CpuName = $cpu.Name
  CpuUsage = $cpu.LoadPercentage
  CpuClockMhz = $cpu.MaxClockSpeed
  TotalMemoryKb = $os.TotalVisibleMemorySize
  FreeMemoryKb = $os.FreePhysicalMemory
} | ConvertTo-Json -Compress
"#;

#[derive(Default)]
pub struct WindowsCollector {
    mock: MockCollector,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct WindowsTelemetry {
    cpu_name: Option<String>,
    cpu_usage: Option<f64>,
    cpu_clock_mhz: Option<f64>,
    total_memory_kb: Option<f64>,
    free_memory_kb: Option<f64>,
}

impl WindowsCollector {
    pub fn new() -> Self {
        Self::default()
    }

    fn collect_live(&mut self, collected_at: String) -> SensorSnapshot {
        match Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                TELEMETRY_SCRIPT,
            ])
            .output()
        {
            Ok(output) if output.status.success() => {
                let payload = String::from_utf8_lossy(&output.stdout);
                Self::snapshot_from_json(collected_at.clone(), &payload)
                    .unwrap_or_else(|message| error_snapshot(collected_at, message))
            }
            Ok(output) => {
                let message = String::from_utf8_lossy(&output.stderr).trim().to_string();
                error_snapshot(collected_at, non_empty_or_default(message))
            }
            Err(error) => error_snapshot(collected_at, error.to_string()),
        }
    }

    fn snapshot_from_json(collected_at: String, payload: &str) -> Result<SensorSnapshot, String> {
        let telemetry: WindowsTelemetry =
            serde_json::from_str(payload).map_err(|error| error.to_string())?;
        snapshot_from_telemetry(collected_at, telemetry)
    }
}

fn available(kind: &str, label: &str, value: f64, unit: &str) -> SensorReading {
    SensorReading {
        kind: kind.into(),
        label: label.into(),
        value: SensorValue::Available {
            value,
            unit: unit.into(),
        },
        indication: None,
    }
}

fn unsupported_app(kind: &str, label: &str) -> SensorReading {
    SensorReading {
        kind: kind.into(),
        label: label.into(),
        value: SensorValue::UnsupportedApp,
        indication: None,
    }
}

fn error(kind: &str, label: &str, message: &str) -> SensorReading {
    SensorReading {
        kind: kind.into(),
        label: label.into(),
        value: SensorValue::Error {
            message: message.into(),
        },
        indication: None,
    }
}

fn snapshot_from_telemetry(
    collected_at: String,
    telemetry: WindowsTelemetry,
) -> Result<SensorSnapshot, String> {
    let cpu_name = telemetry
        .cpu_name
        .filter(|name| !name.trim().is_empty())
        .ok_or_else(|| "CPU 이름을 읽지 못했습니다.".to_string())?;
    let cpu_usage = telemetry
        .cpu_usage
        .ok_or_else(|| "CPU 사용률을 읽지 못했습니다.".to_string())?;
    let cpu_clock_mhz = telemetry
        .cpu_clock_mhz
        .ok_or_else(|| "CPU 클럭을 읽지 못했습니다.".to_string())?;
    let total_memory_kb = telemetry
        .total_memory_kb
        .ok_or_else(|| "전체 메모리를 읽지 못했습니다.".to_string())?;
    let free_memory_kb = telemetry
        .free_memory_kb
        .ok_or_else(|| "사용 가능 메모리를 읽지 못했습니다.".to_string())?;
    let used_memory_kb = total_memory_kb - free_memory_kb;
    let memory_usage = used_memory_kb / total_memory_kb * 100.0;
    let memory_used_gb = used_memory_kb / 1024.0 / 1024.0;

    Ok(SensorSnapshot {
        collected_at,
        devices: vec![
            DeviceSnapshot {
                kind: DeviceKind::Cpu,
                name: cpu_name,
                readings: vec![
                    available("cpu_usage", "사용률", cpu_usage, "%"),
                    unsupported_app("cpu_temperature", "온도"),
                    available("cpu_clock", "클럭", cpu_clock_mhz / 1000.0, "GHz"),
                    unsupported_app("cpu_power", "전력"),
                ],
            },
            DeviceSnapshot {
                kind: DeviceKind::Gpu,
                name: "NVIDIA GPU".into(),
                readings: vec![
                    unsupported_app("gpu_usage", "사용률"),
                    unsupported_app("gpu_temperature", "온도"),
                    unsupported_app("gpu_clock", "클럭"),
                    unsupported_app("gpu_power", "전력"),
                    unsupported_app("gpu_fan", "팬"),
                    unsupported_app("gpu_vram", "VRAM"),
                ],
            },
            DeviceSnapshot {
                kind: DeviceKind::Memory,
                name: "System Memory".into(),
                readings: vec![
                    available("memory_usage", "사용률", memory_usage, "%"),
                    available("memory_used", "사용 중", memory_used_gb, "GB"),
                ],
            },
        ],
    })
}

fn error_snapshot(collected_at: String, message: String) -> SensorSnapshot {
    SensorSnapshot {
        collected_at,
        devices: vec![
            DeviceSnapshot {
                kind: DeviceKind::Cpu,
                name: "Windows CPU".into(),
                readings: vec![
                    error("cpu_usage", "사용률", &message),
                    unsupported_app("cpu_temperature", "온도"),
                    error("cpu_clock", "클럭", &message),
                    unsupported_app("cpu_power", "전력"),
                ],
            },
            DeviceSnapshot {
                kind: DeviceKind::Gpu,
                name: "NVIDIA GPU".into(),
                readings: vec![
                    unsupported_app("gpu_usage", "사용률"),
                    unsupported_app("gpu_temperature", "온도"),
                    unsupported_app("gpu_clock", "클럭"),
                    unsupported_app("gpu_power", "전력"),
                    unsupported_app("gpu_fan", "팬"),
                    unsupported_app("gpu_vram", "VRAM"),
                ],
            },
            DeviceSnapshot {
                kind: DeviceKind::Memory,
                name: "System Memory".into(),
                readings: vec![
                    error("memory_usage", "사용률", &message),
                    error("memory_used", "사용 중", &message),
                ],
            },
        ],
    }
}

fn non_empty_or_default(message: String) -> String {
    if message.is_empty() {
        "Windows 센서 데이터를 읽지 못했습니다.".into()
    } else {
        message
    }
}

impl SensorCollector for WindowsCollector {
    fn collect(&mut self, scenario: MockScenario, collected_at: String) -> SensorSnapshot {
        match scenario {
            MockScenario::Live => self.collect_live(collected_at),
            scenario => self.mock.collect(scenario, collected_at),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn available_value(value: &SensorValue) -> f64 {
        match value {
            SensorValue::Available { value, .. } => *value,
            other => panic!("expected available value, got {other:?}"),
        }
    }

    #[test]
    fn windows_telemetry_json_maps_to_cpu_memory_snapshot() {
        let snapshot = WindowsCollector::snapshot_from_json(
            "2026-06-17T12:00:00Z".into(),
            r#"{
                "CpuName": "AMD Ryzen 7",
                "CpuUsage": 37,
                "CpuClockMhz": 4200,
                "TotalMemoryKb": 33554432,
                "FreeMemoryKb": 16777216
            }"#,
        )
        .unwrap();

        assert_eq!(snapshot.collected_at, "2026-06-17T12:00:00Z");
        assert_eq!(snapshot.devices[0].kind, DeviceKind::Cpu);
        assert_eq!(snapshot.devices[0].name, "AMD Ryzen 7");
        assert_eq!(
            available_value(&snapshot.devices[0].readings[0].value),
            37.0
        );
        assert_eq!(available_value(&snapshot.devices[0].readings[2].value), 4.2);
        assert_eq!(
            snapshot.devices[0].readings[1].value,
            SensorValue::UnsupportedApp
        );
        assert_eq!(
            snapshot.devices[1].readings[0].value,
            SensorValue::UnsupportedApp
        );
        assert_eq!(
            available_value(&snapshot.devices[2].readings[0].value),
            50.0
        );
        assert_eq!(
            available_value(&snapshot.devices[2].readings[1].value),
            16.0
        );
    }
}
