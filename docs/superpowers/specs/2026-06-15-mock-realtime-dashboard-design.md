# Mock Real-Time Dashboard Design

## 1. Goal

Implement the first vertical slice of the PC Health MVP: a deterministic Rust mock collector exposed through Tauri IPC and rendered as a real-time React dashboard. The dashboard refreshes once per second and demonstrates normal, warning, unsupported, waiting, and error states without requiring Windows hardware.

This slice establishes the sensor contract and dashboard behavior that the later Windows collector, recording service, and history views will consume.

## 2. Scope

### Included

- Rust domain model for devices, sensor readings, sensor states, and warnings
- Deterministic `MockCollector`
- Development scenario selector for `normal`, `threshold`, `unsupported`, `waiting`, and `error`
- Tauri command returning one sensor snapshot
- React polling once per second without overlapping requests
- CPU, NVIDIA GPU, and memory cards in a balanced three-column layout
- Explicit display for every sensor state
- General-guidance temperature and memory warnings
- Neutral high-load indication for CPU and GPU utilization
- Unit and UI tests for the behaviors introduced by this slice

### Excluded

- Windows hardware collection
- SQLite and recorded sessions
- Trend graphs and history
- User-editable or persisted thresholds
- Background Rust sampling
- Minimize and close-recording behavior

## 3. Architecture

```text
React dashboard
  |-- scenario selector
  |-- one-second non-overlapping poll
  `-- CPU / GPU / memory cards
              |
         Tauri command
         get_sensor_snapshot
              |
       Snapshot service
       |-- MockCollector
       `-- warning evaluator
```

React calls `get_sensor_snapshot(scenario)` after the previous request completes, then schedules the next call one second later. Changing the scenario triggers an immediate request and resets any pending timer.

This polling design is intentionally limited to the first vertical slice. Recording will require sampling to continue independently from the visible React view, so the recording design will replace polling with a Rust-owned background sampler rather than extending this temporary loop.

## 4. Domain Contract

Each planned sensor remains present even when it has no numeric value.

```ts
type SensorValue =
  | { status: "available"; value: number; unit: string }
  | { status: "unsupported-device" }
  | { status: "unsupported-app" }
  | { status: "waiting" }
  | { status: "error"; message: string };

type WarningLevel = "advisory" | "warning";

type SensorReading = {
  kind: string;
  label: string;
  value: SensorValue;
  indication?: {
    level: WarningLevel | "high-load";
    message: string;
  };
};

type DeviceSnapshot = {
  kind: "cpu" | "gpu" | "memory";
  name: string;
  readings: SensorReading[];
};

type SensorSnapshot = {
  collectedAt: string;
  devices: DeviceSnapshot[];
};
```

The Rust types are authoritative and serialized with stable kebab-case status values. Only `available` values participate in warning evaluation.

## 5. Mock Scenarios

The development selector changes only the mock collector input.

- `normal`: all supported values remain in ordinary ranges.
- `threshold`: temperature and memory values remain above their configured guidance thresholds long enough to demonstrate advisory and warning states.
- `unsupported`: GPU fields exercise both device and application support states.
- `waiting`: planned readings remain visible with `waiting` status.
- `error`: selected individual readings return errors while unaffected readings continue normally.

Values change slightly according to a deterministic sample counter. The same scenario and counter always produce the same snapshot, which keeps tests reproducible.

## 6. Guidance Thresholds

The dashboard distinguishes general operating guidance from manufacturer safety limits.

| Reading | Advisory | Warning | Behavior |
| --- | ---: | ---: | --- |
| CPU temperature | 80 C | 90 C | Requires 10 continuous seconds above the level |
| GPU core temperature | 80 C | 85 C | Requires 10 continuous seconds above the level |
| Memory usage | 85% | 95% | Applies immediately |
| CPU/GPU utilization | 90% | none | Shows neutral `high-load`, not a health warning |

These defaults are product heuristics commonly used to draw attention to sustained heat or memory pressure. They are not universal hardware safety specifications. UI copy must say that a value is above a general recommended range, never that hardware is unsafe or damaged.

The future Windows collector should prefer a device-provided thermal limit when one is reliably available. Intel documents that maximum processor temperature varies by product and system design. NVIDIA NVML exposes device temperature thresholds. A device-specific limit overrides the general temperature guidance for that device.

References:

- [Intel processor temperature guidance](https://www.intel.com/content/www/us/en/support/articles/000005597/processors.html)
- [NVIDIA NVML device temperature threshold API](https://docs.nvidia.com/deploy/nvml-api/group__nvmlDeviceQueries.html)

## 7. Sustained Temperature Evaluation

Sustained duration belongs to the Rust snapshot service rather than React. The service tracks when each available temperature reading first crossed its current level.

- Below advisory: clear the timer and indication.
- At or above advisory but below warning: emit advisory only after 10 continuous seconds.
- At or above warning: emit warning only after 10 continuous seconds at or above the warning threshold.
- A missing, waiting, unsupported, or error reading clears the timer.
- Dropping below a level clears that level immediately; no hysteresis is included in this slice.

The mock threshold scenario starts with deterministic elapsed-state support in tests so the ten-second behavior can be verified without sleeping.

## 8. Dashboard UI

The selected layout is a balanced three-column dashboard with equal CPU, GPU, and memory cards.

- The header contains the product name, latest collection time, and development scenario selector.
- Each card shows the device name and every planned reading in a stable position.
- Advisory state uses an amber accent on the reading and parent card.
- Warning state uses a stronger red accent and a concise explanation below the cards.
- High utilization uses a neutral blue `높은 부하` label.
- Unsupported, waiting, and failed readings use the approved Korean state labels from the main MVP design.
- A failed Tauri command leaves the last successful snapshot visible, adds a dashboard-level connection error, and retries on the next cycle.
- The UI never substitutes zero for an unavailable value.

The first slice does not include trend graphs. Card structure should leave room for them later without adding placeholder charts now.

## 9. Testing

### Rust

- Every mock scenario returns all three device groups and stable sensor positions.
- Deterministic counters produce reproducible values.
- One sensor error does not suppress other readings.
- Only available readings enter evaluation.
- Temperature indications require ten continuous seconds and clear correctly.
- Memory advisory and warning levels apply immediately.
- High utilization remains distinct from health warnings.

### React

- Available values and all explicit status labels render correctly.
- CPU, GPU, and memory cards retain stable structure across scenarios.
- Advisory, warning, and high-load treatments render distinctly.
- Scenario changes request an immediate snapshot.
- Polling waits for completion before scheduling the next request.
- Command failure preserves the last snapshot and shows the retrying error state.

### Completion Checks

```bash
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
```

The implementation plan must add the minimum test tooling necessary because the bootstrap project does not currently define `npm test`.

## 10. Success Criteria

- The app displays deterministic CPU, GPU, and memory snapshots from Rust.
- Values refresh once per second without overlapping requests or visible UI blocking.
- All five sensor statuses can be demonstrated from the scenario selector.
- Sustained temperature, immediate memory, and neutral high-load indications follow this design.
- Command-wide and per-sensor failures remain distinguishable.
- The frontend build and Rust checks pass, and the new Rust and React tests pass.
