use std::collections::HashMap;

use crate::domain::{Indication, IndicationLevel, SensorReading, SensorSnapshot, SensorValue};

const SUSTAINED_SECONDS: i64 = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ThermalLevel {
    Advisory,
    Warning,
}

#[derive(Default)]
pub struct WarningEvaluator {
    crossed_at: HashMap<String, (ThermalLevel, i64)>,
}

impl WarningEvaluator {
    pub fn evaluate(&mut self, snapshot: &mut SensorSnapshot, now_seconds: i64) {
        self.crossed_at.retain(|kind, _| {
            snapshot
                .devices
                .iter()
                .flat_map(|device| device.readings.iter())
                .any(|reading| reading.kind == *kind)
        });

        for reading in snapshot
            .devices
            .iter_mut()
            .flat_map(|device| device.readings.iter_mut())
        {
            reading.indication = None;

            let SensorValue::Available { value, .. } = &reading.value else {
                self.crossed_at.remove(&reading.kind);
                continue;
            };

            match reading.kind.as_str() {
                "cpu_temperature" => {
                    self.evaluate_temperature(reading, *value, 80.0, 90.0, now_seconds)
                }
                "gpu_temperature" => {
                    self.evaluate_temperature(reading, *value, 80.0, 85.0, now_seconds)
                }
                "memory_usage" => {
                    self.crossed_at.remove(&reading.kind);
                    reading.indication = if *value >= 95.0 {
                        Some(memory_indication(IndicationLevel::Warning))
                    } else if *value >= 85.0 {
                        Some(memory_indication(IndicationLevel::Advisory))
                    } else {
                        None
                    };
                }
                "cpu_usage" | "gpu_usage" => {
                    self.crossed_at.remove(&reading.kind);
                    if *value >= 90.0 {
                        reading.indication = Some(Indication {
                            level: IndicationLevel::HighLoad,
                            message: "높은 부하".into(),
                        });
                    }
                }
                _ => {
                    self.crossed_at.remove(&reading.kind);
                }
            }
        }
    }

    fn evaluate_temperature(
        &mut self,
        reading: &mut SensorReading,
        value: f64,
        advisory_threshold: f64,
        warning_threshold: f64,
        now_seconds: i64,
    ) {
        let current_level = if value >= warning_threshold {
            Some(ThermalLevel::Warning)
        } else if value >= advisory_threshold {
            Some(ThermalLevel::Advisory)
        } else {
            None
        };

        let Some(current_level) = current_level else {
            self.crossed_at.remove(&reading.kind);
            return;
        };

        let crossed_at = match self.crossed_at.get(&reading.kind) {
            Some((stored_level, crossed_at)) if *stored_level == current_level => *crossed_at,
            _ => {
                self.crossed_at
                    .insert(reading.kind.clone(), (current_level, now_seconds));
                now_seconds
            }
        };

        if now_seconds - crossed_at >= SUSTAINED_SECONDS {
            reading.indication = Some(Indication {
                level: match current_level {
                    ThermalLevel::Advisory => IndicationLevel::Advisory,
                    ThermalLevel::Warning => IndicationLevel::Warning,
                },
                message: format!("{}가 일반적인 권장 범위보다 높습니다.", reading.label),
            });
        }
    }
}

fn memory_indication(level: IndicationLevel) -> Indication {
    Indication {
        level,
        message: "메모리 사용률이 일반적인 권장 범위보다 높습니다.".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collector::{MockCollector, MockScenario, SensorCollector};
    use crate::domain::{IndicationLevel, SensorReading, SensorSnapshot, SensorValue};

    fn collect(scenario: MockScenario) -> SensorSnapshot {
        MockCollector::new().collect(scenario, "2026-06-15T00:00:00Z".into())
    }

    fn reading<'a>(snapshot: &'a SensorSnapshot, kind: &str) -> &'a SensorReading {
        snapshot
            .devices
            .iter()
            .flat_map(|device| device.readings.iter())
            .find(|reading| reading.kind == kind)
            .unwrap()
    }

    fn reading_mut<'a>(snapshot: &'a mut SensorSnapshot, kind: &str) -> &'a mut SensorReading {
        snapshot
            .devices
            .iter_mut()
            .flat_map(|device| device.readings.iter_mut())
            .find(|reading| reading.kind == kind)
            .unwrap()
    }

    fn level(reading: &SensorReading) -> Option<&IndicationLevel> {
        reading
            .indication
            .as_ref()
            .map(|indication| &indication.level)
    }

    #[test]
    fn cpu_temperature_requires_ten_sustained_seconds_before_warning() {
        let mut evaluator = WarningEvaluator::default();
        let mut snapshot = collect(MockScenario::Threshold);

        evaluator.evaluate(&mut snapshot, 100);
        assert_eq!(level(reading(&snapshot, "cpu_temperature")), None);

        evaluator.evaluate(&mut snapshot, 109);
        assert_eq!(level(reading(&snapshot, "cpu_temperature")), None);

        evaluator.evaluate(&mut snapshot, 110);
        let cpu_temperature = reading(&snapshot, "cpu_temperature");
        assert_eq!(level(cpu_temperature), Some(&IndicationLevel::Warning));
        assert_eq!(
            cpu_temperature.indication.as_ref().unwrap().message,
            "온도가 일반적인 권장 범위보다 높습니다."
        );
    }

    #[test]
    fn unavailable_gpu_temperature_clears_its_timer() {
        let mut evaluator = WarningEvaluator::default();
        let mut snapshot = collect(MockScenario::Threshold);

        evaluator.evaluate(&mut snapshot, 100);
        snapshot = collect(MockScenario::Unsupported);
        evaluator.evaluate(&mut snapshot, 109);
        assert_eq!(level(reading(&snapshot, "gpu_temperature")), None);

        snapshot = collect(MockScenario::Threshold);
        evaluator.evaluate(&mut snapshot, 110);
        evaluator.evaluate(&mut snapshot, 119);
        assert_eq!(level(reading(&snapshot, "gpu_temperature")), None);

        evaluator.evaluate(&mut snapshot, 120);
        assert_eq!(
            level(reading(&snapshot, "gpu_temperature")),
            Some(&IndicationLevel::Warning)
        );
    }

    #[test]
    fn temperature_level_transition_restarts_the_sustained_timer() {
        let mut evaluator = WarningEvaluator::default();
        let mut snapshot = collect(MockScenario::Normal);
        reading_mut(&mut snapshot, "cpu_temperature").value = SensorValue::Available {
            value: 82.0,
            unit: "C".into(),
        };

        evaluator.evaluate(&mut snapshot, 100);
        evaluator.evaluate(&mut snapshot, 110);
        assert_eq!(
            level(reading(&snapshot, "cpu_temperature")),
            Some(&IndicationLevel::Advisory)
        );

        reading_mut(&mut snapshot, "cpu_temperature").value = SensorValue::Available {
            value: 90.0,
            unit: "C".into(),
        };
        evaluator.evaluate(&mut snapshot, 111);
        evaluator.evaluate(&mut snapshot, 120);
        assert_eq!(level(reading(&snapshot, "cpu_temperature")), None);

        evaluator.evaluate(&mut snapshot, 121);
        assert_eq!(
            level(reading(&snapshot, "cpu_temperature")),
            Some(&IndicationLevel::Warning)
        );
    }

    #[test]
    fn memory_warning_is_immediate_and_gpu_usage_reports_high_load() {
        let mut evaluator = WarningEvaluator::default();
        let mut snapshot = collect(MockScenario::Threshold);

        evaluator.evaluate(&mut snapshot, 100);

        let memory_usage = reading(&snapshot, "memory_usage");
        assert_eq!(level(memory_usage), Some(&IndicationLevel::Warning));
        assert_eq!(
            memory_usage.indication.as_ref().unwrap().message,
            "메모리 사용률이 일반적인 권장 범위보다 높습니다."
        );
        let gpu_usage = reading(&snapshot, "gpu_usage");
        assert_eq!(level(gpu_usage), Some(&IndicationLevel::HighLoad));
        assert_eq!(gpu_usage.indication.as_ref().unwrap().message, "높은 부하");
    }

    #[test]
    fn unmatched_kind_clears_timer_and_indication() {
        let mut evaluator = WarningEvaluator::default();
        let mut snapshot = collect(MockScenario::Threshold);

        evaluator.evaluate(&mut snapshot, 100);
        reading_mut(&mut snapshot, "cpu_temperature").kind = "unmatched_temperature".into();
        evaluator.evaluate(&mut snapshot, 109);
        assert_eq!(level(reading(&snapshot, "unmatched_temperature")), None);

        reading_mut(&mut snapshot, "unmatched_temperature").kind = "cpu_temperature".into();
        evaluator.evaluate(&mut snapshot, 110);
        evaluator.evaluate(&mut snapshot, 119);
        assert_eq!(level(reading(&snapshot, "cpu_temperature")), None);

        evaluator.evaluate(&mut snapshot, 120);
        assert_eq!(
            level(reading(&snapshot, "cpu_temperature")),
            Some(&IndicationLevel::Warning)
        );
    }
}
