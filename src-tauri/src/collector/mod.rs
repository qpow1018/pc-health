mod mock;

use crate::domain::SensorSnapshot;

pub use mock::{MockCollector, MockScenario};

pub trait SensorCollector {
    fn collect(&mut self, scenario: MockScenario, collected_at: String) -> SensorSnapshot;
}
