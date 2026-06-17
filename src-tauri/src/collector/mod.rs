mod mock;
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
mod windows;

use crate::domain::{SensorDiagnostics, SensorSnapshot};

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

#[cfg(not(target_os = "windows"))]
pub fn unsupported_platform_diagnostics(collected_at: String) -> SensorDiagnostics {
    let mut mock = MockCollector::new();
    SensorDiagnostics {
        collected_at: collected_at.clone(),
        collector: "unsupported-platform".into(),
        duration_ms: 0,
        raw_payload: None,
        raw_error: None,
        parsed_telemetry: None,
        snapshot: mock.collect(MockScenario::Waiting, collected_at),
    }
}
