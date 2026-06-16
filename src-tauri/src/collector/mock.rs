use serde::Deserialize;

use crate::collector::SensorCollector;
use crate::domain::{DeviceKind, DeviceSnapshot, SensorReading, SensorSnapshot, SensorValue};

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum MockScenario {
    #[default]
    Normal,
    Threshold,
    Unsupported,
    Waiting,
    Error,
}

#[derive(Default)]
pub struct MockCollector {
    sample_index: u64,
}

impl MockCollector {
    pub fn new() -> Self {
        Self::default()
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

fn status(kind: &str, label: &str, value: SensorValue) -> SensorReading {
    SensorReading {
        kind: kind.into(),
        label: label.into(),
        value,
        indication: None,
    }
}

impl SensorCollector for MockCollector {
    fn collect(&mut self, scenario: MockScenario, collected_at: String) -> SensorSnapshot {
        let wave = (self.sample_index % 5) as f64;
        self.sample_index += 1;

        let (cpu_temp, gpu_temp, memory_usage) = match scenario {
            MockScenario::Threshold => (92.0, 87.0, 96.0),
            _ => (61.0 + wave, 67.0 + wave, 54.0 + wave),
        };

        let mut cpu = vec![
            available("cpu_usage", "사용률", 42.0 + wave, "%"),
            available("cpu_temperature", "온도", cpu_temp, "C"),
            available("cpu_clock", "클럭", 4.2, "GHz"),
            available("cpu_power", "전력", 72.0 + wave, "W"),
        ];
        let mut gpu = vec![
            available(
                "gpu_usage",
                "사용률",
                if scenario == MockScenario::Threshold {
                    96.0
                } else {
                    71.0 + wave
                },
                "%",
            ),
            available("gpu_temperature", "온도", gpu_temp, "C"),
            available("gpu_clock", "클럭", 2.5, "GHz"),
            available("gpu_power", "전력", 185.0 + wave, "W"),
            available("gpu_fan", "팬", 68.0, "%"),
            available("gpu_vram", "VRAM", 7.2, "GB"),
        ];
        let mut memory = vec![
            available("memory_usage", "사용률", memory_usage, "%"),
            available("memory_used", "사용 중", 17.3 + wave / 10.0, "GB"),
        ];

        match scenario {
            MockScenario::Unsupported => {
                gpu[1] = status("gpu_temperature", "온도", SensorValue::UnsupportedDevice);
                gpu[3] = status("gpu_power", "전력", SensorValue::UnsupportedApp);
                gpu[4] = status("gpu_fan", "팬", SensorValue::UnsupportedDevice);
            }
            MockScenario::Waiting => {
                for reading in cpu
                    .iter_mut()
                    .chain(gpu.iter_mut())
                    .chain(memory.iter_mut())
                {
                    reading.value = SensorValue::Waiting;
                }
            }
            MockScenario::Error => {
                cpu[1] = status(
                    "cpu_temperature",
                    "온도",
                    SensorValue::Error {
                        message: "온도 센서 응답이 없습니다.".into(),
                    },
                );
                gpu[4] = status(
                    "gpu_fan",
                    "팬",
                    SensorValue::Error {
                        message: "팬 속도를 읽지 못했습니다.".into(),
                    },
                );
            }
            MockScenario::Normal | MockScenario::Threshold => {}
        }

        SensorSnapshot {
            collected_at,
            devices: vec![
                DeviceSnapshot {
                    kind: DeviceKind::Cpu,
                    name: "Mock AMD Ryzen 7 7800X3D".into(),
                    readings: cpu,
                },
                DeviceSnapshot {
                    kind: DeviceKind::Gpu,
                    name: "Mock NVIDIA GeForce RTX 4070".into(),
                    readings: gpu,
                },
                DeviceSnapshot {
                    kind: DeviceKind::Memory,
                    name: "Mock System Memory 32 GB".into(),
                    readings: memory,
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{DeviceKind, SensorValue};

    fn collect(scenario: MockScenario) -> SensorSnapshot {
        MockCollector::new().collect(scenario, "2026-06-15T00:00:00Z".into())
    }

    fn available_value(value: &SensorValue) -> f64 {
        match value {
            SensorValue::Available { value, .. } => *value,
            other => panic!("expected available value, got {other:?}"),
        }
    }

    #[test]
    fn scenario_defaults_and_deserializes_from_kebab_case() {
        assert_eq!(MockScenario::default(), MockScenario::Normal);
        assert_eq!(
            serde_json::from_str::<MockScenario>(r#""unsupported""#).unwrap(),
            MockScenario::Unsupported
        );
    }

    #[test]
    fn every_scenario_keeps_device_and_reading_order_stable() {
        for scenario in [
            MockScenario::Normal,
            MockScenario::Threshold,
            MockScenario::Unsupported,
            MockScenario::Waiting,
            MockScenario::Error,
        ] {
            let snapshot = collect(scenario);

            assert_eq!(snapshot.devices.len(), 3);
            assert_eq!(snapshot.devices[0].kind, DeviceKind::Cpu);
            assert_eq!(snapshot.devices[1].kind, DeviceKind::Gpu);
            assert_eq!(snapshot.devices[2].kind, DeviceKind::Memory);
            assert_eq!(snapshot.devices[0].readings.len(), 4);
            assert_eq!(snapshot.devices[1].readings.len(), 6);
            assert_eq!(snapshot.devices[2].readings.len(), 2);

            let kinds: Vec<&str> = snapshot
                .devices
                .iter()
                .flat_map(|device| device.readings.iter())
                .map(|reading| reading.kind.as_str())
                .collect();
            assert_eq!(
                kinds,
                [
                    "cpu_usage",
                    "cpu_temperature",
                    "cpu_clock",
                    "cpu_power",
                    "gpu_usage",
                    "gpu_temperature",
                    "gpu_clock",
                    "gpu_power",
                    "gpu_fan",
                    "gpu_vram",
                    "memory_usage",
                    "memory_used",
                ]
            );
        }
    }

    #[test]
    fn normal_scenario_uses_named_devices_and_deterministic_wave() {
        let mut collector = MockCollector::new();
        let first = collector.collect(MockScenario::Normal, "first".into());
        let second = collector.collect(MockScenario::Normal, "second".into());

        assert_eq!(first.collected_at, "first");
        assert_eq!(first.devices[0].name, "Mock AMD Ryzen 7 7800X3D");
        assert_eq!(first.devices[1].name, "Mock NVIDIA GeForce RTX 4070");
        assert_eq!(first.devices[2].name, "Mock System Memory 32 GB");

        let first_values: Vec<f64> = first
            .devices
            .iter()
            .flat_map(|device| device.readings.iter())
            .map(|reading| available_value(&reading.value))
            .collect();
        assert_eq!(
            first_values,
            [42.0, 61.0, 4.2, 72.0, 71.0, 67.0, 2.5, 185.0, 68.0, 7.2, 54.0, 17.3,]
        );
        assert_eq!(available_value(&second.devices[0].readings[0].value), 43.0);
        assert!((available_value(&second.devices[2].readings[1].value) - 17.4).abs() < 1e-12);
    }

    #[test]
    fn same_scenario_and_sample_index_are_reproducible() {
        let first = collect(MockScenario::Normal);
        let second = collect(MockScenario::Normal);

        assert_eq!(first, second);
    }

    #[test]
    fn threshold_scenario_sets_required_values() {
        let snapshot = collect(MockScenario::Threshold);

        assert_eq!(
            available_value(&snapshot.devices[0].readings[1].value),
            92.0
        );
        assert_eq!(
            available_value(&snapshot.devices[1].readings[0].value),
            96.0
        );
        assert_eq!(
            available_value(&snapshot.devices[1].readings[1].value),
            87.0
        );
        assert_eq!(
            available_value(&snapshot.devices[2].readings[0].value),
            96.0
        );
    }

    #[test]
    fn unsupported_scenario_marks_only_selected_gpu_readings() {
        let snapshot = collect(MockScenario::Unsupported);
        let gpu = &snapshot.devices[1].readings;

        assert!(matches!(gpu[0].value, SensorValue::Available { .. }));
        assert_eq!(gpu[1].value, SensorValue::UnsupportedDevice);
        assert!(matches!(gpu[2].value, SensorValue::Available { .. }));
        assert_eq!(gpu[3].value, SensorValue::UnsupportedApp);
        assert_eq!(gpu[4].value, SensorValue::UnsupportedDevice);
        assert!(matches!(gpu[5].value, SensorValue::Available { .. }));
    }

    #[test]
    fn waiting_scenario_marks_every_reading_waiting() {
        let snapshot = collect(MockScenario::Waiting);

        assert!(snapshot
            .devices
            .iter()
            .flat_map(|device| device.readings.iter())
            .all(|reading| reading.value == SensorValue::Waiting));
    }

    #[test]
    fn error_scenario_fails_only_selected_readings() {
        let snapshot = collect(MockScenario::Error);
        let cpu = &snapshot.devices[0].readings;
        let gpu = &snapshot.devices[1].readings;

        assert_eq!(
            cpu[1].value,
            SensorValue::Error {
                message: "온도 센서 응답이 없습니다.".into()
            }
        );
        assert_eq!(
            gpu[4].value,
            SensorValue::Error {
                message: "팬 속도를 읽지 못했습니다.".into()
            }
        );
        assert!(cpu
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != 1)
            .all(|(_, reading)| matches!(reading.value, SensorValue::Available { .. })));
        assert!(gpu
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != 4)
            .all(|(_, reading)| matches!(reading.value, SensorValue::Available { .. })));
        assert!(snapshot.devices[2]
            .readings
            .iter()
            .all(|reading| matches!(reading.value, SensorValue::Available { .. })));
    }
}
