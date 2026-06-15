# Mock Real-Time Dashboard Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a deterministic Rust mock sensor pipeline exposed through Tauri IPC and a tested React dashboard that refreshes once per second and renders explicit sensor, guidance, and failure states.

**Architecture:** Rust owns the authoritative sensor domain, deterministic mock collection, and stateful warning evaluation. A single Tauri command locks a short-lived application service call and returns one serialized snapshot; React wraps that command in an API module, polls only after each request settles, and renders three equal device cards. The temporary React polling boundary remains isolated so a later recording design can replace it with a Rust background sampler.

**Tech Stack:** Tauri 2, Rust, Serde, Chrono, React 19, TypeScript, Vite 6, Vitest, Testing Library, jsdom

---

## File Map

### Rust

- Create `src-tauri/src/domain.rs`: serialized sensor, device, indication, and snapshot types.
- Create `src-tauri/src/collector.rs`: `SensorCollector` trait and deterministic `MockCollector` scenarios.
- Create `src-tauri/src/warning.rs`: general-guidance thresholds and sustained-temperature evaluator.
- Create `src-tauri/src/service.rs`: combines collection and warning evaluation with an injectable timestamp.
- Modify `src-tauri/src/lib.rs`: owns Tauri state and exposes `get_sensor_snapshot`.
- Modify `src-tauri/Cargo.toml`: adds `chrono` for RFC 3339 timestamps.

### React

- Create `src/sensors/types.ts`: TypeScript mirror of the serialized Rust contract.
- Create `src/sensors/api.ts`: narrow Tauri invocation wrapper.
- Create `src/sensors/useSensorSnapshot.ts`: immediate and one-second non-overlapping polling.
- Create `src/components/DeviceCard.tsx`: stable device and reading presentation.
- Create `src/components/Dashboard.tsx`: header, selector, cards, and dashboard-level errors.
- Create `src/test/setup.ts`: Testing Library cleanup and DOM matchers.
- Create `src/components/DeviceCard.test.tsx`: rendering and indication tests.
- Create `src/components/Dashboard.test.tsx`: scenarios, polling, and command-failure tests.
- Modify `src/App.tsx`: replace the template with the dashboard.
- Modify `src/App.css`: balanced three-column dashboard and state styling.
- Modify `vite.config.ts`: Vitest jsdom configuration.
- Modify `tsconfig.json`: include Vitest globals.
- Modify `package.json` and `package-lock.json`: add test scripts and test dependencies.
- Modify `src-tauri/tauri.conf.json`: increase the default desktop window for the three-column layout.

## Contract Conventions

- Sensor kinds are stable snake-case identifiers such as `cpu_usage`, `gpu_temperature`, and `memory_used`.
- Serde enum tags use kebab-case values matching the approved design.
- Rust remains authoritative; TypeScript mirrors the wire format without frontend-only reinterpretation.
- All scenarios return CPU, GPU, and memory devices in that order, with stable reading order per device.
- `collectedAt` is an RFC 3339 UTC string.
- Guidance messages use Korean UI copy and never claim that hardware is damaged or unsafe.

---

### Task 1: Add Frontend Test Infrastructure

**Files:**
- Modify: `package.json`
- Modify: `package-lock.json`
- Modify: `vite.config.ts`
- Modify: `tsconfig.json`
- Create: `src/test/setup.ts`
- Create: `src/test/smoke.test.tsx`

- [ ] **Step 1: Install the minimum test dependencies**

Run:

```bash
npm install --save-dev vitest jsdom @testing-library/react @testing-library/jest-dom @testing-library/user-event
```

Expected: installation succeeds and `package-lock.json` records the new development dependencies.

- [ ] **Step 2: Add test scripts to `package.json`**

Add these scripts while preserving the existing scripts:

```json
"test": "vitest run",
"test:watch": "vitest"
```

- [ ] **Step 3: Configure Vitest in `vite.config.ts`**

Add the Vitest config reference and a `test` block:

```ts
/// <reference types="vitest/config" />

import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// @ts-expect-error process is a Node.js global supplied by Vite.
const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  test: {
    environment: "jsdom",
    setupFiles: "./src/test/setup.ts",
    clearMocks: true,
  },
}));
```

- [ ] **Step 4: Include Vitest globals in `tsconfig.json`**

Add this compiler option:

```json
"types": ["vitest/globals", "@testing-library/jest-dom"]
```

- [ ] **Step 5: Create the shared test setup**

Create `src/test/setup.ts`:

```ts
import "@testing-library/jest-dom/vitest";
import { cleanup } from "@testing-library/react";
import { afterEach } from "vitest";

afterEach(() => cleanup());
```

- [ ] **Step 6: Write a smoke test before changing the template app**

Create `src/test/smoke.test.tsx`:

```tsx
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import App from "../App";

describe("App", () => {
  it("renders the current application", () => {
    render(<App />);
    expect(screen.getByRole("heading", { name: /Welcome to Tauri/i })).toBeInTheDocument();
  });
});
```

- [ ] **Step 7: Run the smoke test and production build**

Run:

```bash
npm test
npm run build
```

Expected: the smoke test passes and the production build succeeds.

- [ ] **Step 8: Commit the test infrastructure**

```bash
git add package.json package-lock.json vite.config.ts tsconfig.json src/test/setup.ts src/test/smoke.test.tsx
git commit -m "test: add frontend test infrastructure"
```

---

### Task 2: Define the Rust Sensor Contract

**Files:**
- Create: `src-tauri/src/domain.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write serialization tests in `src-tauri/src/domain.rs`**

Start the file with the domain types and tests below. The tests intentionally reference the complete target contract.

```rust
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "kebab-case")]
pub enum SensorValue {
    Available { value: f64, unit: String },
    UnsupportedDevice,
    UnsupportedApp,
    Waiting,
    Error { message: String },
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IndicationLevel {
    Advisory,
    Warning,
    HighLoad,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Indication {
    pub level: IndicationLevel,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SensorReading {
    pub kind: String,
    pub label: String,
    pub value: SensorValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indication: Option<Indication>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DeviceKind {
    Cpu,
    Gpu,
    Memory,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceSnapshot {
    pub kind: DeviceKind,
    pub name: String,
    pub readings: Vec<SensorReading>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SensorSnapshot {
    pub collected_at: String,
    pub devices: Vec<DeviceSnapshot>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_available_value_with_tagged_status() {
        let json = serde_json::to_value(SensorValue::Available {
            value: 42.0,
            unit: "%".into(),
        })
        .unwrap();

        assert_eq!(json["status"], "available");
        assert_eq!(json["value"], 42.0);
        assert_eq!(json["unit"], "%");
    }

    #[test]
    fn serializes_wire_names_in_the_approved_format() {
        let snapshot = SensorSnapshot {
            collected_at: "2026-06-15T00:00:00Z".into(),
            devices: vec![DeviceSnapshot {
                kind: DeviceKind::Gpu,
                name: "Mock NVIDIA GeForce RTX 4070".into(),
                readings: vec![SensorReading {
                    kind: "gpu_temperature".into(),
                    label: "온도".into(),
                    value: SensorValue::UnsupportedApp,
                    indication: Some(Indication {
                        level: IndicationLevel::HighLoad,
                        message: "높은 부하".into(),
                    }),
                }],
            }],
        };

        let json = serde_json::to_value(snapshot).unwrap();
        assert_eq!(json["collectedAt"], "2026-06-15T00:00:00Z");
        assert_eq!(json["devices"][0]["kind"], "gpu");
        assert_eq!(json["devices"][0]["readings"][0]["value"]["status"], "unsupported-app");
        assert_eq!(json["devices"][0]["readings"][0]["indication"]["level"], "high-load");
    }
}
```

- [ ] **Step 2: Export the module from `src-tauri/src/lib.rs`**

Add at the top:

```rust
mod domain;
```

- [ ] **Step 3: Run the focused Rust tests**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml domain::tests
```

Expected: both serialization tests pass.

- [ ] **Step 4: Commit the contract**

```bash
git add src-tauri/src/domain.rs src-tauri/src/lib.rs
git commit -m "feat: define sensor snapshot contract"
```

---

### Task 3: Implement Deterministic Mock Scenarios

**Files:**
- Create: `src-tauri/src/collector.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write failing collector behavior tests**

Create `src-tauri/src/collector.rs` with the scenario API, trait, and tests first:

```rust
use serde::Deserialize;

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

pub trait SensorCollector {
    fn collect(&mut self, scenario: MockScenario, collected_at: String) -> SensorSnapshot;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_scenario_keeps_device_and_reading_order_stable() {
        for scenario in [
            MockScenario::Normal,
            MockScenario::Threshold,
            MockScenario::Unsupported,
            MockScenario::Waiting,
            MockScenario::Error,
        ] {
            let mut collector = MockCollector::new();
            let snapshot = collector.collect(scenario, "2026-06-15T00:00:00Z".into());
            assert_eq!(snapshot.devices.len(), 3);
            assert_eq!(snapshot.devices[0].kind, DeviceKind::Cpu);
            assert_eq!(snapshot.devices[1].kind, DeviceKind::Gpu);
            assert_eq!(snapshot.devices[2].kind, DeviceKind::Memory);
            assert_eq!(snapshot.devices[0].readings.len(), 4);
            assert_eq!(snapshot.devices[1].readings.len(), 6);
            assert_eq!(snapshot.devices[2].readings.len(), 2);
        }
    }

    #[test]
    fn same_scenario_and_sample_index_are_reproducible() {
        let first = MockCollector::new().collect(MockScenario::Normal, "same".into());
        let second = MockCollector::new().collect(MockScenario::Normal, "same".into());
        assert_eq!(first, second);
    }

    #[test]
    fn error_scenario_fails_only_selected_readings() {
        let snapshot = MockCollector::new().collect(MockScenario::Error, "now".into());
        assert!(matches!(snapshot.devices[0].readings[1].value, SensorValue::Error { .. }));
        assert!(matches!(snapshot.devices[0].readings[0].value, SensorValue::Available { .. }));
        assert!(matches!(snapshot.devices[1].readings[0].value, SensorValue::Available { .. }));
    }
}
```

- [ ] **Step 2: Run tests to verify the trait is not implemented**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml collector::tests
```

Expected: compilation fails because `MockCollector` does not implement `SensorCollector`.

- [ ] **Step 3: Implement the deterministic collector**

Add helpers and the implementation above the test module:

```rust
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

        let (cpu_temp, gpu_temp, memory_percent) = match scenario {
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
            available("gpu_usage", "사용률", if scenario == MockScenario::Threshold { 96.0 } else { 71.0 + wave }, "%"),
            available("gpu_temperature", "온도", gpu_temp, "C"),
            available("gpu_clock", "클럭", 2.5, "GHz"),
            available("gpu_power", "전력", 185.0 + wave, "W"),
            available("gpu_fan", "팬", 68.0, "%"),
            available("gpu_vram", "VRAM", 7.2, "GB"),
        ];
        let mut memory = vec![
            available("memory_usage", "사용률", memory_percent, "%"),
            available("memory_used", "사용 중", 17.3 + wave / 10.0, "GB"),
        ];

        match scenario {
            MockScenario::Unsupported => {
                gpu[1] = status("gpu_temperature", "온도", SensorValue::UnsupportedDevice);
                gpu[3] = status("gpu_power", "전력", SensorValue::UnsupportedApp);
                gpu[4] = status("gpu_fan", "팬", SensorValue::UnsupportedDevice);
            }
            MockScenario::Waiting => {
                for reading in cpu.iter_mut().chain(gpu.iter_mut()).chain(memory.iter_mut()) {
                    reading.value = SensorValue::Waiting;
                }
            }
            MockScenario::Error => {
                cpu[1] = status(
                    "cpu_temperature",
                    "온도",
                    SensorValue::Error { message: "온도 센서 응답이 없습니다.".into() },
                );
                gpu[4] = status(
                    "gpu_fan",
                    "팬",
                    SensorValue::Error { message: "팬 속도를 읽지 못했습니다.".into() },
                );
            }
            MockScenario::Normal | MockScenario::Threshold => {}
        }

        SensorSnapshot {
            collected_at,
            devices: vec![
                DeviceSnapshot { kind: DeviceKind::Cpu, name: "Mock AMD Ryzen 7 7800X3D".into(), readings: cpu },
                DeviceSnapshot { kind: DeviceKind::Gpu, name: "Mock NVIDIA GeForce RTX 4070".into(), readings: gpu },
                DeviceSnapshot { kind: DeviceKind::Memory, name: "Mock System Memory 32 GB".into(), readings: memory },
            ],
        }
    }
}
```

- [ ] **Step 4: Export the collector module**

Add to `src-tauri/src/lib.rs`:

```rust
mod collector;
```

- [ ] **Step 5: Run collector and all Rust tests**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml collector::tests
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: all tests pass.

- [ ] **Step 6: Commit the collector**

```bash
git add src-tauri/src/collector.rs src-tauri/src/lib.rs
git commit -m "feat: add deterministic mock sensor scenarios"
```

---

### Task 4: Add Stateful Guidance Evaluation

**Files:**
- Create: `src-tauri/src/warning.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write failing evaluator tests**

Create `src-tauri/src/warning.rs` with these tests and public shell:

```rust
use std::collections::HashMap;

use crate::domain::{Indication, IndicationLevel, SensorSnapshot, SensorValue};

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
    pub fn evaluate(&mut self, _snapshot: &mut SensorSnapshot, _now_seconds: i64) {}
}

#[cfg(test)]
mod tests {
    use crate::collector::{MockCollector, MockScenario, SensorCollector};
    use super::*;

    fn indication(snapshot: &SensorSnapshot, kind: &str) -> Option<IndicationLevel> {
        snapshot.devices.iter()
            .flat_map(|device| &device.readings)
            .find(|reading| reading.kind == kind)
            .and_then(|reading| reading.indication.as_ref())
            .map(|indication| indication.level.clone())
    }

    #[test]
    fn temperature_warning_requires_ten_continuous_seconds() {
        let mut evaluator = WarningEvaluator::default();
        let mut snapshot = MockCollector::new().collect(MockScenario::Threshold, "now".into());
        evaluator.evaluate(&mut snapshot, 100);
        assert_eq!(indication(&snapshot, "cpu_temperature"), None);

        evaluator.evaluate(&mut snapshot, 109);
        assert_eq!(indication(&snapshot, "cpu_temperature"), None);

        evaluator.evaluate(&mut snapshot, 110);
        assert_eq!(indication(&snapshot, "cpu_temperature"), Some(IndicationLevel::Warning));
    }

    #[test]
    fn unavailable_temperature_clears_the_timer() {
        let mut evaluator = WarningEvaluator::default();
        let mut hot = MockCollector::new().collect(MockScenario::Threshold, "now".into());
        evaluator.evaluate(&mut hot, 100);

        let mut missing = MockCollector::new().collect(MockScenario::Unsupported, "now".into());
        evaluator.evaluate(&mut missing, 105);

        let mut hot_again = MockCollector::new().collect(MockScenario::Threshold, "now".into());
        evaluator.evaluate(&mut hot_again, 110);
        assert_eq!(indication(&hot_again, "gpu_temperature"), None);
    }

    #[test]
    fn entering_warning_resets_the_sustained_timer_for_that_level() {
        let mut evaluator = WarningEvaluator::default();
        let mut advisory = MockCollector::new().collect(MockScenario::Threshold, "now".into());
        if let SensorValue::Available { value, .. } = &mut advisory.devices[0].readings[1].value {
            *value = 82.0;
        }
        evaluator.evaluate(&mut advisory, 100);
        evaluator.evaluate(&mut advisory, 110);
        assert_eq!(indication(&advisory, "cpu_temperature"), Some(IndicationLevel::Advisory));

        let mut warning = MockCollector::new().collect(MockScenario::Threshold, "now".into());
        evaluator.evaluate(&mut warning, 111);
        assert_eq!(indication(&warning, "cpu_temperature"), None);
        evaluator.evaluate(&mut warning, 121);
        assert_eq!(indication(&warning, "cpu_temperature"), Some(IndicationLevel::Warning));
    }

    #[test]
    fn memory_warning_is_immediate_and_utilization_is_neutral() {
        let mut evaluator = WarningEvaluator::default();
        let mut snapshot = MockCollector::new().collect(MockScenario::Threshold, "now".into());
        evaluator.evaluate(&mut snapshot, 100);

        assert_eq!(indication(&snapshot, "memory_usage"), Some(IndicationLevel::Warning));
        assert_eq!(indication(&snapshot, "gpu_usage"), Some(IndicationLevel::HighLoad));
    }
}
```

- [ ] **Step 2: Run tests to verify guidance is absent**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml warning::tests
```

Expected: tests fail because `evaluate` does not add indications.

- [ ] **Step 3: Implement minimal threshold evaluation**

Replace `evaluate` and add helpers:

```rust
impl WarningEvaluator {
    pub fn evaluate(&mut self, snapshot: &mut SensorSnapshot, now_seconds: i64) {
        for reading in snapshot.devices.iter_mut().flat_map(|device| &mut device.readings) {
            reading.indication = None;
            let value = match &reading.value {
                SensorValue::Available { value, .. } => *value,
                _ => {
                    self.crossed_at.remove(&reading.kind);
                    continue;
                }
            };

            match reading.kind.as_str() {
                "cpu_temperature" => self.temperature(reading, value, 80.0, 90.0, now_seconds),
                "gpu_temperature" => self.temperature(reading, value, 80.0, 85.0, now_seconds),
                "memory_usage" => reading.indication = immediate_guidance(value, 85.0, 95.0, "메모리 사용률"),
                "cpu_usage" | "gpu_usage" if value >= 90.0 => {
                    reading.indication = Some(Indication {
                        level: IndicationLevel::HighLoad,
                        message: "높은 부하".into(),
                    });
                }
                _ => {
                    self.crossed_at.remove(&reading.kind);
                }
            }
        }
    }

    fn temperature(
        &mut self,
        reading: &mut crate::domain::SensorReading,
        value: f64,
        advisory: f64,
        warning: f64,
        now_seconds: i64,
    ) {
        if value < advisory {
            self.crossed_at.remove(&reading.kind);
            return;
        }

        let thermal_level = if value >= warning {
            ThermalLevel::Warning
        } else {
            ThermalLevel::Advisory
        };
        let entry = self
            .crossed_at
            .entry(reading.kind.clone())
            .or_insert((thermal_level, now_seconds));
        if entry.0 != thermal_level {
            *entry = (thermal_level, now_seconds);
        }
        let crossed_at = entry.1;
        if now_seconds - crossed_at < SUSTAINED_SECONDS {
            return;
        }

        let level = match thermal_level {
            ThermalLevel::Advisory => IndicationLevel::Advisory,
            ThermalLevel::Warning => IndicationLevel::Warning,
        };
        reading.indication = Some(Indication {
            level,
            message: format!("{}가 일반적인 권장 범위보다 높습니다.", reading.label),
        });
    }
}

fn immediate_guidance(value: f64, advisory: f64, warning: f64, label: &str) -> Option<Indication> {
    let level = if value >= warning {
        IndicationLevel::Warning
    } else if value >= advisory {
        IndicationLevel::Advisory
    } else {
        return None;
    };

    Some(Indication {
        level,
        message: format!("{label}이 일반적인 권장 범위보다 높습니다."),
    })
}
```

- [ ] **Step 4: Export the warning module and run tests**

Add to `src-tauri/src/lib.rs`:

```rust
mod warning;
```

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml warning::tests
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: all tests pass.

- [ ] **Step 5: Commit guidance evaluation**

```bash
git add src-tauri/src/warning.rs src-tauri/src/lib.rs
git commit -m "feat: evaluate sensor guidance states"
```

---

### Task 5: Expose the Snapshot Service Through Tauri

**Files:**
- Create: `src-tauri/src/service.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: Add Chrono**

Add to `[dependencies]` in `src-tauri/Cargo.toml`:

```toml
chrono = { version = "0.4", default-features = false, features = ["clock"] }
```

- [ ] **Step 2: Write the service test first**

Create `src-tauri/src/service.rs`:

```rust
use crate::{
    collector::{MockCollector, MockScenario, SensorCollector},
    domain::SensorSnapshot,
    warning::WarningEvaluator,
};

#[derive(Default)]
pub struct SnapshotService {
    collector: MockCollector,
    warnings: WarningEvaluator,
}

impl SnapshotService {
    pub fn snapshot_at(
        &mut self,
        scenario: MockScenario,
        collected_at: String,
        _now_seconds: i64,
    ) -> SensorSnapshot {
        self.collector.collect(scenario, collected_at)
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
        let cpu_temperature = second.devices[0].readings.iter()
            .find(|reading| reading.kind == "cpu_temperature")
            .unwrap();
        assert_eq!(
            cpu_temperature.indication.as_ref().map(|item| item.level.clone()),
            Some(IndicationLevel::Warning),
        );
    }
}
```

- [ ] **Step 3: Run the focused test and confirm failure**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml service::tests
```

Expected: the test fails because the returned CPU temperature has no warning indication.

- [ ] **Step 4: Implement `snapshot_at`**

Replace the method body:

```rust
let mut snapshot = self.collector.collect(scenario, collected_at);
self.warnings.evaluate(&mut snapshot, now_seconds);
snapshot
```

- [ ] **Step 5: Replace the template command in `src-tauri/src/lib.rs`**

Use this complete application wiring:

```rust
mod collector;
mod domain;
mod service;
mod warning;

use std::sync::Mutex;

use chrono::Utc;
use collector::MockScenario;
use domain::SensorSnapshot;
use service::SnapshotService;
use tauri::State;

struct AppState {
    snapshots: Mutex<SnapshotService>,
}

#[tauri::command]
fn get_sensor_snapshot(
    scenario: MockScenario,
    state: State<'_, AppState>,
) -> Result<SensorSnapshot, String> {
    let now = Utc::now();
    let mut service = state
        .snapshots
        .lock()
        .map_err(|_| "센서 수집 상태를 잠그지 못했습니다.".to_string())?;

    Ok(service.snapshot_at(scenario, now.to_rfc3339(), now.timestamp()))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            snapshots: Mutex::new(SnapshotService::default()),
        })
        .invoke_handler(tauri::generate_handler![get_sensor_snapshot])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 6: Run Rust formatting, tests, and checks**

Run:

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: all commands succeed. If formatting fails, run `cargo fmt --manifest-path src-tauri/Cargo.toml`, then repeat all three commands.

- [ ] **Step 7: Commit the Tauri pipeline**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/lib.rs src-tauri/src/service.rs
git commit -m "feat: expose mock snapshots through tauri"
```

---

### Task 6: Add the TypeScript Contract and Tauri API Wrapper

**Files:**
- Create: `src/sensors/types.ts`
- Create: `src/sensors/api.ts`
- Create: `src/sensors/api.test.ts`

- [ ] **Step 1: Create the TypeScript wire types**

Create `src/sensors/types.ts`:

```ts
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
```

- [ ] **Step 2: Write a failing API wrapper test**

Create `src/sensors/api.test.ts`:

```ts
import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { getSensorSnapshot } from "./api";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

describe("getSensorSnapshot", () => {
  beforeEach(() => vi.mocked(invoke).mockReset());

  it("invokes the stable Tauri command with the selected scenario", async () => {
    vi.mocked(invoke).mockResolvedValue({ collectedAt: "now", devices: [] });
    await getSensorSnapshot("unsupported");
    expect(invoke).toHaveBeenCalledWith("get_sensor_snapshot", {
      scenario: "unsupported",
    });
  });
});
```

- [ ] **Step 3: Run the focused test and verify failure**

Run:

```bash
npx vitest run src/sensors/api.test.ts
```

Expected: compilation fails because `./api` does not exist.

- [ ] **Step 4: Implement the narrow API wrapper**

Create `src/sensors/api.ts`:

```ts
import { invoke } from "@tauri-apps/api/core";
import type { MockScenario, SensorSnapshot } from "./types";

export function getSensorSnapshot(scenario: MockScenario) {
  return invoke<SensorSnapshot>("get_sensor_snapshot", { scenario });
}
```

- [ ] **Step 5: Run tests and build**

Run:

```bash
npx vitest run src/sensors/api.test.ts
npm run build
```

Expected: both commands succeed.

- [ ] **Step 6: Commit the frontend contract**

```bash
git add src/sensors/types.ts src/sensors/api.ts src/sensors/api.test.ts
git commit -m "feat: add sensor snapshot client contract"
```

---

### Task 7: Build Non-Overlapping Polling

**Files:**
- Create: `src/sensors/useSensorSnapshot.ts`
- Create: `src/sensors/useSensorSnapshot.test.tsx`

- [ ] **Step 1: Write polling tests with fake timers**

Create `src/sensors/useSensorSnapshot.test.tsx`:

```tsx
import { act, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useSensorSnapshot } from "./useSensorSnapshot";
import type { MockScenario, SensorSnapshot } from "./types";

const snapshot: SensorSnapshot = { collectedAt: "now", devices: [] };

describe("useSensorSnapshot", () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it("loads immediately and schedules only after the request settles", async () => {
    let resolveRequest: (value: SensorSnapshot) => void = () => {};
    const load = vi.fn(() => new Promise<SensorSnapshot>((resolve) => {
      resolveRequest = resolve;
    }));

    renderHook(() => useSensorSnapshot("normal", load));
    expect(load).toHaveBeenCalledTimes(1);

    await act(async () => vi.advanceTimersByTimeAsync(5_000));
    expect(load).toHaveBeenCalledTimes(1);

    await act(async () => resolveRequest(snapshot));
    await act(async () => vi.advanceTimersByTimeAsync(999));
    expect(load).toHaveBeenCalledTimes(1);

    await act(async () => vi.advanceTimersByTimeAsync(1));
    expect(load).toHaveBeenCalledTimes(2);
  });

  it("preserves the last snapshot when a later request fails", async () => {
    const load = vi.fn()
      .mockResolvedValueOnce(snapshot)
      .mockRejectedValueOnce(new Error("offline"));

    const { result } = renderHook(() => useSensorSnapshot("normal", load));
    await act(async () => Promise.resolve());
    expect(result.current.snapshot).toEqual(snapshot);

    await act(async () => vi.advanceTimersByTimeAsync(1_000));
    expect(result.current.snapshot).toEqual(snapshot);
    expect(result.current.error).toBe("센서 데이터를 불러오지 못했습니다. 다시 시도합니다.");
  });

  it("loads immediately when the scenario changes", async () => {
    const load = vi.fn().mockResolvedValue(snapshot);
    const { rerender } = renderHook(
      ({ scenario }: { scenario: MockScenario }) => useSensorSnapshot(scenario, load),
      { initialProps: { scenario: "normal" as MockScenario } },
    );
    await act(async () => Promise.resolve());

    rerender({ scenario: "error" });
    expect(load).toHaveBeenLastCalledWith("error");
  });
});
```

- [ ] **Step 2: Run tests and verify the hook is missing**

Run:

```bash
npx vitest run src/sensors/useSensorSnapshot.test.tsx
```

Expected: compilation fails because the hook does not exist.

- [ ] **Step 3: Implement polling with timer cleanup and stale-request protection**

Create `src/sensors/useSensorSnapshot.ts`:

```ts
import { useEffect, useState } from "react";
import { getSensorSnapshot } from "./api";
import type { MockScenario, SensorSnapshot } from "./types";

type SnapshotLoader = (scenario: MockScenario) => Promise<SensorSnapshot>;

export function useSensorSnapshot(
  scenario: MockScenario,
  load: SnapshotLoader = getSensorSnapshot,
) {
  const [snapshot, setSnapshot] = useState<SensorSnapshot | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    let timer: number | undefined;

    const poll = async () => {
      try {
        const next = await load(scenario);
        if (!active) return;
        setSnapshot(next);
        setError(null);
      } catch {
        if (!active) return;
        setError("센서 데이터를 불러오지 못했습니다. 다시 시도합니다.");
      }

      if (active) timer = window.setTimeout(poll, 1_000);
    };

    void poll();

    return () => {
      active = false;
      if (timer !== undefined) window.clearTimeout(timer);
    };
  }, [scenario, load]);

  return { snapshot, error };
}
```

- [ ] **Step 4: Run the focused tests and adjust only test scheduling if React batching requires it**

Run:

```bash
npx vitest run src/sensors/useSensorSnapshot.test.tsx
```

Expected: all three tests pass. Use `await act(async () => {})` to flush promises if needed; do not add sleeps or overlapping intervals.

- [ ] **Step 5: Commit polling**

```bash
git add src/sensors/useSensorSnapshot.ts src/sensors/useSensorSnapshot.test.tsx
git commit -m "feat: poll sensor snapshots without overlap"
```

---

### Task 8: Render Stable Device Cards

**Files:**
- Create: `src/components/DeviceCard.tsx`
- Create: `src/components/DeviceCard.test.tsx`

- [ ] **Step 1: Write device-card rendering tests**

Create `src/components/DeviceCard.test.tsx`:

```tsx
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import DeviceCard from "./DeviceCard";
import type { DeviceSnapshot } from "../sensors/types";

const device: DeviceSnapshot = {
  kind: "gpu",
  name: "Mock RTX 4070",
  readings: [
    { kind: "gpu_usage", label: "사용률", value: { status: "available", value: 96, unit: "%" }, indication: { level: "high-load", message: "높은 부하" } },
    { kind: "gpu_temperature", label: "온도", value: { status: "unsupported-device" } },
    { kind: "gpu_power", label: "전력", value: { status: "unsupported-app" } },
    { kind: "gpu_fan", label: "팬", value: { status: "waiting" } },
    { kind: "gpu_vram", label: "VRAM", value: { status: "error", message: "읽기 실패" } },
  ],
};

describe("DeviceCard", () => {
  it("renders available values and every explicit status label", () => {
    render(<DeviceCard device={device} />);
    expect(screen.getByText("96%")).toBeInTheDocument();
    expect(screen.getByText("지원하지 않음")).toBeInTheDocument();
    expect(screen.getByText("현재 버전 미지원")).toBeInTheDocument();
    expect(screen.getByText("데이터 대기 중")).toBeInTheDocument();
    expect(screen.getByText("측정 실패")).toBeInTheDocument();
    expect(screen.getByText("높은 부하")).toBeInTheDocument();
  });

  it("uses the strongest indication as the card state", () => {
    const warningDevice: DeviceSnapshot = {
      ...device,
      readings: [{
        kind: "gpu_temperature",
        label: "온도",
        value: { status: "available", value: 87, unit: "C" },
        indication: { level: "warning", message: "온도가 일반적인 권장 범위보다 높습니다." },
      }],
    };
    render(<DeviceCard device={warningDevice} />);
    expect(screen.getByTestId("gpu-card")).toHaveAttribute("data-level", "warning");
  });
});
```

- [ ] **Step 2: Run tests and verify the component is missing**

Run:

```bash
npx vitest run src/components/DeviceCard.test.tsx
```

Expected: compilation fails because `DeviceCard` does not exist.

- [ ] **Step 3: Implement stable reading rows**

Create `src/components/DeviceCard.tsx`:

```tsx
import type { DeviceSnapshot, IndicationLevel, SensorValue } from "../sensors/types";

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
  return device.readings.reduce<IndicationLevel | undefined>((strongest, reading) => {
    const next = reading.indication?.level;
    if (!next) return strongest;
    if (!strongest || levelRank[next] > levelRank[strongest]) return next;
    return strongest;
  }, undefined);
}

export default function DeviceCard({ device }: { device: DeviceSnapshot }) {
  const level = cardLevel(device);

  return (
    <section className="device-card" data-level={level} data-testid={`${device.kind}-card`}>
      <header className="device-card__header">
        <span className="device-card__kind">{device.kind.toUpperCase()}</span>
        <h2>{device.name}</h2>
      </header>
      <dl className="reading-list">
        {device.readings.map((reading) => (
          <div className="reading" data-level={reading.indication?.level} key={reading.kind}>
            <dt>{reading.label}</dt>
            <dd>
              <span>{formatValue(reading.value)}</span>
              {reading.indication && <small>{reading.indication.message}</small>}
              {reading.value.status === "error" && <small>{reading.value.message}</small>}
            </dd>
          </div>
        ))}
      </dl>
    </section>
  );
}
```

- [ ] **Step 4: Run component tests**

Run:

```bash
npx vitest run src/components/DeviceCard.test.tsx
```

Expected: both tests pass.

- [ ] **Step 5: Commit the card component**

```bash
git add src/components/DeviceCard.tsx src/components/DeviceCard.test.tsx
git commit -m "feat: render stable sensor device cards"
```

---

### Task 9: Assemble the Dashboard and Scenario Selector

**Files:**
- Create: `src/components/Dashboard.tsx`
- Create: `src/components/Dashboard.test.tsx`
- Modify: `src/App.tsx`
- Delete: `src/test/smoke.test.tsx`

- [ ] **Step 1: Write dashboard interaction tests**

Create `src/components/Dashboard.test.tsx`:

```tsx
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import Dashboard from "./Dashboard";
import type { SensorSnapshot } from "../sensors/types";

const snapshot: SensorSnapshot = {
  collectedAt: "2026-06-15T12:00:00Z",
  devices: [
    { kind: "cpu", name: "Mock CPU", readings: [] },
    { kind: "gpu", name: "Mock GPU", readings: [] },
    { kind: "memory", name: "Mock Memory", readings: [] },
  ],
};

describe("Dashboard", () => {
  it("renders all device cards and changes scenarios", async () => {
    const onScenarioChange = vi.fn();
    render(
      <Dashboard
        snapshot={snapshot}
        error={null}
        scenario="normal"
        onScenarioChange={onScenarioChange}
      />,
    );

    expect(screen.getByTestId("cpu-card")).toBeInTheDocument();
    expect(screen.getByTestId("gpu-card")).toBeInTheDocument();
    expect(screen.getByTestId("memory-card")).toBeInTheDocument();

    await userEvent.selectOptions(screen.getByLabelText("개발 시나리오"), "error");
    expect(onScenarioChange).toHaveBeenCalledWith("error");
  });

  it("keeps cards visible while showing a command error", () => {
    render(
      <Dashboard
        snapshot={snapshot}
        error="센서 데이터를 불러오지 못했습니다. 다시 시도합니다."
        scenario="normal"
        onScenarioChange={() => {}}
      />,
    );
    expect(screen.getByRole("alert")).toHaveTextContent("다시 시도합니다");
    expect(screen.getByTestId("cpu-card")).toBeInTheDocument();
  });
});
```

- [ ] **Step 2: Run tests and verify the dashboard is missing**

Run:

```bash
npx vitest run src/components/Dashboard.test.tsx
```

Expected: compilation fails because `Dashboard` does not exist.

- [ ] **Step 3: Implement the presentation-only dashboard**

Create `src/components/Dashboard.tsx`:

```tsx
import DeviceCard from "./DeviceCard";
import type { MockScenario, SensorSnapshot } from "../sensors/types";

const scenarios: Array<{ value: MockScenario; label: string }> = [
  { value: "normal", label: "정상" },
  { value: "threshold", label: "임계값 초과" },
  { value: "unsupported", label: "미지원" },
  { value: "waiting", label: "대기" },
  { value: "error", label: "오류" },
];

type DashboardProps = {
  snapshot: SensorSnapshot | null;
  error: string | null;
  scenario: MockScenario;
  onScenarioChange: (scenario: MockScenario) => void;
};

export default function Dashboard({ snapshot, error, scenario, onScenarioChange }: DashboardProps) {
  return (
    <main className="app-shell">
      <header className="dashboard-header">
        <div>
          <p className="eyebrow">READ-ONLY PERFORMANCE MONITOR</p>
          <h1>PC Health</h1>
          <p className="updated-at">
            {snapshot ? `마지막 측정 ${new Date(snapshot.collectedAt).toLocaleTimeString("ko-KR")}` : "센서 데이터 연결 중"}
          </p>
        </div>
        <label className="scenario-control">
          <span>개발 시나리오</span>
          <select
            aria-label="개발 시나리오"
            value={scenario}
            onChange={(event) => onScenarioChange(event.target.value as MockScenario)}
          >
            {scenarios.map((item) => <option key={item.value} value={item.value}>{item.label}</option>)}
          </select>
        </label>
      </header>

      {error && <div className="dashboard-error" role="alert">{error}</div>}

      <div className="device-grid">
        {snapshot?.devices.map((device) => <DeviceCard device={device} key={device.kind} />)}
      </div>

      <p className="guidance-note">
        표시 기준은 일반적인 권장 범위이며 장치 제조사의 공식 한계값이 우선합니다.
      </p>
    </main>
  );
}
```

- [ ] **Step 4: Replace `src/App.tsx` with the stateful container**

```tsx
import { useState } from "react";
import "./App.css";
import Dashboard from "./components/Dashboard";
import type { MockScenario } from "./sensors/types";
import { useSensorSnapshot } from "./sensors/useSensorSnapshot";

function App() {
  const [scenario, setScenario] = useState<MockScenario>("normal");
  const { snapshot, error } = useSensorSnapshot(scenario);

  return (
    <Dashboard
      snapshot={snapshot}
      error={error}
      scenario={scenario}
      onScenarioChange={setScenario}
    />
  );
}

export default App;
```

- [ ] **Step 5: Remove the obsolete template smoke test**

Delete `src/test/smoke.test.tsx`. Its assertion targets the removed Tauri welcome screen, and dashboard coverage now replaces it.

- [ ] **Step 6: Run dashboard and full frontend tests**

Run:

```bash
npx vitest run src/components/Dashboard.test.tsx
npm test
```

Expected: all tests pass.

- [ ] **Step 7: Commit the dashboard behavior**

```bash
git add src/App.tsx src/components/Dashboard.tsx src/components/Dashboard.test.tsx src/test/smoke.test.tsx
git commit -m "feat: assemble realtime sensor dashboard"
```

---

### Task 10: Apply the Approved Three-Column Visual Design

**Files:**
- Modify: `src/App.css`
- Modify: `src-tauri/tauri.conf.json`

- [ ] **Step 1: Replace template CSS with the dashboard styles**

Use a dark neutral base, equal cards, and distinct non-color-only labels. Keep selectors limited to the classes already introduced:

```css
:root {
  font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  color: #e5edf8;
  background: #08101d;
  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
}

* { box-sizing: border-box; }
body { margin: 0; min-width: 320px; min-height: 100vh; background: radial-gradient(circle at top, #14243d 0, #08101d 52%); }
button, select { font: inherit; }

.app-shell { width: min(1180px, 100%); margin: 0 auto; padding: 40px 28px; }
.dashboard-header { display: flex; align-items: flex-end; justify-content: space-between; gap: 24px; margin-bottom: 24px; }
.eyebrow { margin: 0 0 6px; color: #79b8ff; font-size: 0.72rem; font-weight: 700; letter-spacing: 0.14em; }
h1 { margin: 0; font-size: clamp(2rem, 4vw, 3rem); line-height: 1; }
.updated-at { margin: 10px 0 0; color: #8fa3bf; }
.scenario-control { display: grid; gap: 7px; color: #9fb0c8; font-size: 0.8rem; }
.scenario-control select { min-width: 170px; border: 1px solid #304563; border-radius: 9px; padding: 10px 12px; color: #edf5ff; background: #101b2d; }
.dashboard-error { margin-bottom: 18px; border: 1px solid #9d3f48; border-radius: 10px; padding: 12px 14px; color: #ffd7da; background: #35181d; }
.device-grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 16px; }
.device-card { min-width: 0; border: 1px solid #273a55; border-radius: 16px; padding: 18px; background: rgba(15, 27, 45, 0.94); box-shadow: 0 14px 34px rgba(0, 0, 0, 0.22); }
.device-card[data-level="advisory"] { border-color: #b77a25; box-shadow: inset 0 3px #d99a3d; }
.device-card[data-level="warning"] { border-color: #b94b55; box-shadow: inset 0 3px #ef6b76; }
.device-card__header { min-height: 76px; border-bottom: 1px solid #263951; padding-bottom: 14px; }
.device-card__kind { color: #7fc2ff; font-size: 0.72rem; font-weight: 800; letter-spacing: 0.12em; }
.device-card h2 { margin: 7px 0 0; overflow-wrap: anywhere; color: #f4f8ff; font-size: 1rem; }
.reading-list { display: grid; margin: 0; }
.reading { display: grid; grid-template-columns: 0.8fr 1.2fr; gap: 12px; min-height: 66px; align-items: center; border-bottom: 1px solid #203149; padding: 12px 0; }
.reading:last-child { border-bottom: 0; }
.reading dt { color: #8fa3bf; }
.reading dd { display: grid; justify-items: end; gap: 4px; margin: 0; text-align: right; font-weight: 700; }
.reading small { color: #8fa3bf; font-size: 0.72rem; font-weight: 500; }
.reading[data-level="high-load"] small { color: #7fc2ff; }
.reading[data-level="advisory"] dd, .reading[data-level="advisory"] small { color: #f2b75d; }
.reading[data-level="warning"] dd, .reading[data-level="warning"] small { color: #ff7f89; }
.guidance-note { margin: 18px 0 0; color: #7186a3; font-size: 0.78rem; text-align: right; }

@media (max-width: 900px) {
  .device-grid { grid-template-columns: 1fr; }
  .device-card__header { min-height: auto; }
}

@media (max-width: 560px) {
  .app-shell { padding: 28px 18px; }
  .dashboard-header { align-items: stretch; flex-direction: column; }
  .scenario-control select { width: 100%; }
}
```

- [ ] **Step 2: Increase the initial Tauri window size**

Change the first window in `src-tauri/tauri.conf.json`:

```json
"width": 1200,
"height": 760,
"minWidth": 760,
"minHeight": 560
```

- [ ] **Step 3: Run frontend tests and build**

Run:

```bash
npm test
npm run build
```

Expected: tests and production build succeed.

- [ ] **Step 4: Start the app for visual verification**

Run:

```bash
npm run tauri dev
```

Expected: PC Health opens at the larger desktop size with three equal cards at desktop width. Confirm each selector option changes the visible state, error details do not collapse rows, and the warning scenario shows memory guidance immediately and temperature guidance after ten seconds.

- [ ] **Step 5: Commit the approved styling**

```bash
git add src/App.css src-tauri/tauri.conf.json
git commit -m "style: apply balanced dashboard layout"
```

---

### Task 11: Final Cross-Layer Verification

**Files:**
- Modify only files needed to fix failures caused by this implementation.

- [ ] **Step 1: Format Rust**

Run:

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

Expected: exits successfully. If it reports changes, run `cargo fmt --manifest-path src-tauri/Cargo.toml`, inspect the formatting-only diff, and rerun the check.

- [ ] **Step 2: Run all automated checks**

Run:

```bash
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: every command exits with code 0.

- [ ] **Step 3: Verify the diff is limited to the approved slice**

Run:

```bash
git status --short
git diff --stat HEAD~10..HEAD
git diff --check
```

Expected: no SQLite, recording, Windows collector, graph, export, or threshold-settings work appears in the diff; `git diff --check` is silent.

- [ ] **Step 4: Perform final browser-visible checks**

With the dev app running, verify:

1. `normal` shows changing numeric values once per second.
2. `threshold` immediately shows memory warning and high GPU load, then temperature warning after ten seconds.
3. `unsupported` shows both `지원하지 않음` and `현재 버전 미지원` without moving reading rows.
4. `waiting` shows `데이터 대기 중` for every planned reading.
5. `error` shows individual `측정 실패` details while other values continue updating.
6. A simulated command rejection in the React test preserves the last snapshot and displays the retry message.

- [ ] **Step 5: Commit any verification-only fixes**

If Step 1-4 required code fixes, stage only files from this feature that `git status --short` reports as modified. Use the explicit set below; Git ignores paths without changes:

```bash
git add package.json package-lock.json vite.config.ts tsconfig.json src/App.tsx src/App.css src/components/Dashboard.tsx src/components/Dashboard.test.tsx src/components/DeviceCard.tsx src/components/DeviceCard.test.tsx src/sensors/api.ts src/sensors/api.test.ts src/sensors/types.ts src/sensors/useSensorSnapshot.ts src/sensors/useSensorSnapshot.test.tsx src/test/setup.ts src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/tauri.conf.json src-tauri/src/collector.rs src-tauri/src/domain.rs src-tauri/src/lib.rs src-tauri/src/service.rs src-tauri/src/warning.rs
git commit -m "fix: complete realtime dashboard verification"
```

If no fixes were needed, do not create an empty commit.

---

## Plan Self-Review

- Spec coverage: every included item maps to Tasks 2-10; all excluded subsystems remain absent.
- Contract consistency: Rust kebab-case status/level values match `src/sensors/types.ts`; command and API wrapper both use `get_sensor_snapshot` with `{ scenario }`.
- Timing consistency: only the Rust evaluator owns the ten-second temperature duration; React owns request scheduling but no health logic.
- Failure consistency: individual sensor errors remain values inside a successful snapshot; command-wide failure is represented by a rejected Tauri invocation and dashboard alert.
- Testability: deterministic sample counters and injected epoch seconds avoid sleeping in Rust tests; Vitest fake timers avoid real one-second waits.
- Scope control: no persistence, Windows hardware integration, graph library, router, state-management library, or configurable threshold system is introduced.
