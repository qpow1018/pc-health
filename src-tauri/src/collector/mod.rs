mod mock;
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
mod windows;

use crate::domain::SensorSnapshot;

pub use mock::{MockCollector, MockScenario};

#[cfg(target_os = "windows")]
pub use windows::WindowsCollector;

#[cfg(not(target_os = "windows"))]
pub type PlatformCollector = MockCollector;

#[cfg(target_os = "windows")]
pub type PlatformCollector = WindowsCollector;

pub trait SensorCollector {
    fn collect(&mut self, scenario: MockScenario, collected_at: String) -> SensorSnapshot;
}
