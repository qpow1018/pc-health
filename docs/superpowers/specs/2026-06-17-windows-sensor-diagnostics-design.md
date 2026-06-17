# Windows Sensor Diagnostics Design

## Goal

Make Windows-only sensor problems observable before replacing the current live collector. The app should let a tester capture one diagnostic sample that explains what the native layer collected, how it was parsed, what snapshot reached the UI, and how long collection took.

This is a debugging and validation aid for the MVP. It must not make the normal dashboard heavier.

## Context

The current live path is:

```text
React useSensorSnapshot
  -> Tauri get_sensor_snapshot
  -> SnapshotService
  -> WindowsCollector
  -> powershell.exe
  -> Win32_Processor / Win32_OperatingSystem JSON
  -> WindowsTelemetry
  -> SensorSnapshot
  -> DeviceCard
```

The recent Windows test showed three risks:

- The app felt choppy while running beside other applications.
- A console window appeared briefly during live collection.
- Some displayed data was missing or unclear.

The console-window issue has a short-term mitigation, but the deeper concern remains: one-second polling should not repeatedly start heavy external processes. Before changing the collector backend, we need a way to see whether a missing value was absent at the source, failed during parsing, intentionally mapped to an unsupported state, or lost in the UI.

## Scope

Included:

- Add a manual diagnostics command for one live Windows sensor capture.
- Show the latest diagnostics in the app only when the user requests it.
- Include raw source payload when available, parsed telemetry, final snapshot, collection duration, and failure reason.
- Keep all diagnostic data read-only and local to the running app.
- Preserve the normal dashboard polling path and current `SensorSnapshot` contract.

Excluded:

- Replacing the PowerShell collector in this step.
- NVIDIA GPU support changes.
- Persistent logging or writing diagnostic files by default.
- Background sampling changes.
- Windows notification, tray, or performance overlay work.

## UX

The dashboard gets a small diagnostics control visible in live mode. It should be secondary to the main readings, not a new workflow. Pressing the control runs a single diagnostic capture and shows a compact panel with:

- collection status
- collection duration in milliseconds
- collector source name
- raw payload or raw error
- parsed CPU and memory telemetry
- final snapshot JSON

The panel should also provide a copy action so Windows test results can be pasted into an issue or chat without using browser devtools. The copy action copies only the latest diagnostic object.

When diagnostics are not opened, the normal dashboard should look and behave as it does today.

## Native Contract

Add a separate command rather than changing `get_sensor_snapshot`.

```ts
type SensorDiagnostics = {
  collectedAt: string;
  collector: string;
  durationMs: number;
  rawPayload?: string;
  rawError?: string;
  parsedTelemetry?: {
    cpuName?: string;
    cpuUsage?: number;
    cpuClockMhz?: number;
    totalMemoryKb?: number;
    freeMemoryKb?: number;
  };
  snapshot: SensorSnapshot;
};
```

The Rust type remains internal to diagnostics and must not replace the authoritative `SensorSnapshot` wire contract. `SensorSnapshot` remains the data consumed by the dashboard cards and warning evaluator.

## Data Flow

Normal polling:

```text
Dashboard -> get_sensor_snapshot -> SensorSnapshot
```

Manual diagnostics:

```text
Diagnostics button
  -> get_sensor_diagnostics
  -> WindowsCollector diagnostic capture
  -> raw payload + parsed telemetry + final snapshot + duration
  -> diagnostics panel
```

Diagnostics should share the same live collection logic as much as practical so the captured result explains the real dashboard behavior. The normal path should not allocate or serialize raw diagnostics payloads.

On non-Windows platforms, the diagnostics command should return a structured unsupported-platform diagnostic instead of trying to run live hardware collection. This keeps frontend tests and macOS development deterministic while making it clear that hardware evidence still requires Windows.

## Performance Rules

- Do not run diagnostics automatically.
- Do not write diagnostic logs on every sample.
- Keep one-second dashboard polling non-overlapping.
- Measure collection duration in Rust, close to the collector.
- Treat diagnostics as a temporary visibility layer, not the final performance fix.

The next collector step should remove repeated PowerShell process creation for CPU and memory. The diagnostics panel gives before/after evidence for that change.

## Error Handling

Diagnostics should return a structured object even when collection fails. The final `snapshot` may contain per-reading `error` values, matching the dashboard contract.

Examples:

- PowerShell process failed: include `rawError`, error readings for CPU and memory, and duration.
- JSON parse failed: include `rawPayload`, parse error in `rawError`, error readings for affected devices, and duration.
- Missing source field: include parsed telemetry with missing field absent, final snapshot showing the current error or unsupported mapping, and duration.
- Non-Windows runtime: include `collector: "unsupported-platform"`, no raw payload, and a snapshot that keeps planned readings visible with unsupported or waiting states.

Unsupported hardware or not-yet-supported readings should remain normal `unsupported-app` or `unsupported-device` states, not diagnostics failures.

## Testing

Rust:

- Unit-test conversion from successful raw telemetry to diagnostics.
- Unit-test command failure diagnostics with an error snapshot.
- Preserve existing `SensorSnapshot` serialization tests.

Frontend:

- Test diagnostics button invokes the diagnostics API only on user action.
- Test loading, success, and failure panel states.
- Test that normal dashboard rendering still works without diagnostics.

Manual Windows validation:

- Run a Windows artifact and confirm diagnostics capture shows duration.
- Confirm no console window appears during normal polling.
- Compare CPU and memory displayed values with the diagnostics parsed telemetry.
- Use the raw payload to explain any missing visible data.

## Rollout

1. Commit this design on `dev`.
2. Implement the diagnostics command and panel as a small dev-branch change.
3. Build a Windows artifact and capture evidence from the target PC.
4. Use that evidence to replace the PowerShell CPU/memory collector in a separate change.
