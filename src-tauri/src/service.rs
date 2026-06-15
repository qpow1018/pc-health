use crate::{
    collector::{MockCollector, MockScenario, SensorCollector},
    domain::SensorSnapshot,
    warning::WarningEvaluator,
};

pub struct SnapshotService {
    collector: MockCollector,
    warnings: WarningEvaluator,
}

impl Default for SnapshotService {
    fn default() -> Self {
        Self {
            collector: MockCollector::new(),
            warnings: WarningEvaluator::default(),
        }
    }
}

impl SnapshotService {
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
    use crate::domain::IndicationLevel;

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
}
