use crate::{
    collector::{MockCollector, MockScenario, SensorCollector},
    domain::SensorSnapshot,
    warning::WarningEvaluator,
};

pub struct SnapshotService<C = MockCollector> {
    collector: C,
    warnings: WarningEvaluator,
}

impl Default for SnapshotService {
    fn default() -> Self {
        Self::new(MockCollector::new())
    }
}

impl<C: SensorCollector> SnapshotService<C> {
    pub fn new(collector: C) -> Self {
        Self {
            collector,
            warnings: WarningEvaluator::default(),
        }
    }

    pub fn snapshot_at(
        &mut self,
        scenario: MockScenario,
        collected_at: String,
        now_seconds: i64,
    ) -> SensorSnapshot {
        let mut snapshot = self.collector.collect(scenario, collected_at);
        self.warnings.evaluate(&mut snapshot, now_seconds);
        snapshot
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{DeviceKind, DeviceSnapshot, IndicationLevel};

    struct StaticCollector;

    impl SensorCollector for StaticCollector {
        fn collect(&mut self, _scenario: MockScenario, collected_at: String) -> SensorSnapshot {
            SensorSnapshot {
                collected_at,
                devices: vec![DeviceSnapshot {
                    kind: DeviceKind::Memory,
                    name: "Injected Memory".into(),
                    readings: vec![],
                }],
            }
        }
    }

    #[test]
    fn service_combines_collection_and_stateful_evaluation() {
        let mut service = SnapshotService::default();
        let first = service.snapshot_at(MockScenario::Threshold, "first".into(), 100);
        let second = service.snapshot_at(MockScenario::Threshold, "second".into(), 110);

        assert_eq!(first.collected_at, "first");
        let cpu_temperature = second.devices[0]
            .readings
            .iter()
            .find(|reading| reading.kind == "cpu_temperature")
            .unwrap();
        assert_eq!(
            cpu_temperature
                .indication
                .as_ref()
                .map(|item| item.level.clone()),
            Some(IndicationLevel::Warning),
        );
    }

    #[test]
    fn service_accepts_an_injected_collector() {
        let mut service = SnapshotService::new(StaticCollector);

        let snapshot = service.snapshot_at(MockScenario::Normal, "collected".into(), 100);

        assert_eq!(snapshot.collected_at, "collected");
        assert_eq!(snapshot.devices[0].name, "Injected Memory");
    }
}
