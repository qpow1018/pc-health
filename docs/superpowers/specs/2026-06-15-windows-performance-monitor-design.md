# Windows Performance Monitor MVP Design

## 1. Product Goal

Build a personal, read-only Windows desktop application that shows the current performance and health of the user's computer. The MVP focuses on CPU, NVIDIA GPU, and system memory. It does not change clocks, fan speeds, power limits, drivers, or other hardware settings.

Development primarily happens on macOS with deterministic mock sensor data. Hardware integration and final validation happen on an accessible Windows 10 or Windows 11 PC.

## 2. MVP Scope

### Included

- Real-time dashboard refreshed once per second
- CPU usage, temperature, clock, and power
- NVIDIA GPU usage, temperature, clock, power, fan speed, and VRAM
- System memory usage percentage and used capacity
- Explicit UI states for unavailable sensor readings
- User-controlled recording start and stop
- In-app history for recorded sessions
- Time-series graphs plus average and maximum values
- Configurable thresholds with in-app visual warnings
- Monitoring and recording while the window is minimized
- Confirmation before closing while recording
- Recovery of a recording interrupted by an abnormal app exit
- Deletion of individual recorded sessions

### Excluded

- AMD and Intel GPU sensor support
- FPS and in-game overlay
- Hardware control or overclocking
- Windows notification center alerts
- System tray background operation
- Automatic startup
- CSV export
- Accounts, cloud sync, and remote monitoring
- Driver management and updates
- Internet connection diagnosis
- Automatic application updates

## 3. Technology Choice

- Desktop shell: Tauri
- Frontend: React, TypeScript, and Vite
- Native layer: Rust
- Local persistence: SQLite
- Windows sensor fallback: a narrowly scoped LibreHardwareMonitor-based helper if direct Rust and Windows APIs cannot provide the required readings reliably

Next.js is not used for the MVP because the application has no server-rendering, web routing, account, or cloud requirement. React remains suitable for the entire user interface, while Tauri and Rust provide the native system access unavailable to a browser.

## 4. Architecture

```text
React UI
  |-- real-time dashboard
  |-- recording controls
  |-- session history
  `-- threshold settings
          |
       Tauri IPC
          |
Rust application core
  |-- sampling coordinator
  |-- threshold evaluator
  |-- recording service
  |-- SQLite repositories
  `-- SensorCollector interface
        |-- MockCollector (macOS development and tests)
        `-- WindowsCollector (Windows hardware integration)
```

The React UI does not know how readings are collected. It consumes a stable sensor snapshot contract. This allows UI and persistence development on macOS without pretending that mock data validates Windows hardware behavior.

The sampling coordinator requests one snapshot every second. Collection must run away from the UI thread. Each completed snapshot is sent to the frontend; it is persisted only while a recording session is active.

## 5. Collector Boundary

The collector exposes device identity, sensor capabilities, and current readings. Each platform implementation returns the same domain model.

- `MockCollector` emits deterministic scenarios such as normal load, threshold crossing, unsupported sensors, temporary waiting, and collection failure.
- `WindowsCollector` reads CPU and memory from appropriate Windows facilities and NVIDIA GPU data from supported native APIs or libraries.
- Unsupported GPU vendors are detected and identified, but their detailed GPU sensor readings use the `unsupported-app` state in the MVP.

Sensor collection failures are isolated. Failure to read one sensor must not prevent other readings from reaching the UI or being recorded.

## 6. Sensor State Model

Every planned sensor remains visible even when no numeric value is available.

```ts
type SensorValue =
  | { status: "available"; value: number; unit: string }
  | { status: "unsupported-device" }
  | { status: "unsupported-app" }
  | { status: "waiting" }
  | { status: "error"; message: string };
```

UI labels:

| State | Display |
| --- | --- |
| `available` | Formatted value such as `65 C` or `42%` |
| `unsupported-device` | `지원하지 않음` |
| `unsupported-app` | `현재 버전 미지원` |
| `waiting` | `데이터 대기 중` |
| `error` | `측정 실패` with a concise detail or retry action |

Only `available` values participate in graphs, averages, maximums, and threshold evaluation. A status change is still retained in a recorded session so history does not misleadingly connect readings across missing intervals.

## 7. Dashboard

The main screen contains separate CPU, GPU, and memory cards. Each card includes current values, recent in-memory trends, and a clear device name. All planned fields remain in stable positions to avoid layout movement when a sensor becomes unavailable.

Threshold breaches change the affected reading and its parent card to a warning appearance and show an in-app explanation. The MVP does not emit operating-system notifications.

The dashboard continues sampling while minimized. Closing the window exits the application. If recording is active, closing first asks whether to stop and exit or remain in the app.

## 8. Recording And History

Recording is opt-in. Opening the app does not persist one-second sensor samples.

When the user starts recording:

1. A session is created with its start time and active status.
2. Each subsequent snapshot is stored once per second.
3. Stopping recording sets the end time and completed status.

If the app starts and finds an active session left by a previous process, it marks that session as `interrupted` using the last stored sample time as its effective end.

The history screen lists sessions by date, duration, and completion status. A session detail view shows time-series graphs and average and maximum values for available readings. Users can delete sessions individually. No export or automatic retention policy is included in the MVP.

## 9. Threshold Settings

The app provides sensible initial thresholds for supported temperature and utilization readings. Users can enable or disable each threshold and change its numeric value.

Threshold settings are stored locally. An unavailable sensor cannot trigger a warning. A threshold warning clears when a later available reading falls below the configured value; hysteresis and notification cooldowns are unnecessary because warnings remain inside the app.

## 10. Persistence Model

SQLite stores only durable user data:

- settings and thresholds
- recording session metadata
- recorded sensor samples and sensor states

Current dashboard values and the short recent trend shown before recording are held in memory. The database schema should represent sensor kind and device identity explicitly rather than creating one column for every possible future sensor.

## 11. Error Handling

- A failed sensor reports `error`; other sensors continue normally.
- A collector-wide failure produces a visible dashboard error and retries on the next one-second cycle.
- Database write failure stops the recording, preserves already committed data, and shows a blocking in-app message.
- Unsupported hardware is a normal capability state, not an application error.
- The UI never substitutes `0` for a missing or failed reading.

## 12. Testing And Verification

### macOS development

- Unit-test calculations, threshold evaluation, recording state, and interrupted-session recovery.
- Use deterministic mock scenarios for every `SensorValue` state.
- Test React rendering and interactions without requiring Windows hardware.
- Verify that only active recording sessions produce persisted samples.

### Windows validation

- Verify CPU, memory, and NVIDIA GPU identity and each required reading on the target PC.
- Compare values with Windows Task Manager and an established monitoring tool, allowing for sampling and vendor differences.
- Verify one-second updates during idle and sustained load.
- Verify minimize, close confirmation, recording, restart recovery, and session deletion.
- Verify behavior for unavailable sensors and, where practical, a non-NVIDIA or disabled GPU adapter.

CI may build and run non-hardware tests on Windows, but it does not replace physical-device validation.

## 13. Success Criteria

The MVP is complete when:

- It installs and runs on the target Windows 10 or Windows 11 PC.
- The dashboard updates once per second without visible UI blocking.
- Supported CPU, NVIDIA GPU, and memory readings are plausible when compared with established tools.
- Every unavailable reading displays the correct explicit state instead of disappearing or showing zero.
- Recording occurs only after user initiation and remains active while minimized.
- Completed and interrupted sessions can be reviewed with graphs, averages, and maximums.
- Configured threshold breaches are visibly identified inside the app.
- No MVP action changes hardware configuration.

## 14. Deferred Product Phases

After the MVP is stable on the target PC, separate design cycles may cover broader GPU support, internet diagnosis, driver inventory and update tracking, distribution to other users, system tray operation, exports, and overlays. These are independent expansions and are not architectural requirements for the initial implementation.
