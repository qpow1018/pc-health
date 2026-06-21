# Persistent CPU Sensor Helper Design

## Goal

Replace the one-shot CPU-temperature helper interaction with a persistent LibreHardwareMonitor helper that exposes complete raw CPU sensor diagnostics over stdio NDJSON. This slice establishes the process and contract boundary before any raw sensor is promoted into `SensorSnapshot` or the dashboard.

## Scope

Included:

- Keep one app-managed helper process alive across diagnostic samples.
- Use stdin/stdout NDJSON for request-response IPC.
- Enable CPU hardware only in this first slice.
- Return raw CPU hardware, subhardware, and sensor identity and values.
- Expose helper PID, sample index, collection errors, and the raw response through existing diagnostics.
- Test protocol, lifecycle, timeout, restart, and shutdown behavior.
- Extend Windows CI with a helper protocol smoke test.
- Validate the packaged result on a physical Windows PC through the existing diagnostics panel.

Excluded:

- Mapping new LibreHardwareMonitor values into `SensorSnapshot`.
- Changing dashboard cards or warning evaluation.
- Enabling GPU, memory, storage, motherboard, fan, or voltage hardware.
- Selecting authoritative CPU temperature, load, clock, or power sensors.
- Automatic administrator elevation.
- Named pipes, local HTTP, Windows services, or multi-client access.

## Architecture

```text
Existing diagnostics action
  -> Tauri get_sensor_diagnostics
  -> SnapshotService
  -> WindowsCollector
  -> SensorHelperClient
       -> persistent pc-health-sensor-helper.exe
            stdin: NDJSON request
            stdout: NDJSON response
            stderr: helper logs
  -> SensorDiagnostics.rawPayload
  -> existing diagnostics panel and copy action
```

`SensorHelperClient` owns the child process, writable stdin, stdout response reader, stderr drain, request ID counter, and restart state. It starts lazily on the first live diagnostic request and remains attached to the app-owned collector until shutdown or failure.

The current `SnapshotService` mutex serializes access, so this slice supports one in-flight helper request. It does not add concurrent request multiplexing.

## NDJSON Protocol

Every request and response occupies exactly one UTF-8 line. JSON responses are written only to stdout. Human-readable logs are written only to stderr.

Supported requests:

```json
{"id":1,"command":"sample"}
{"id":2,"command":"shutdown"}
```

Successful sample response:

```json
{
  "id": 1,
  "ok": true,
  "helperPid": 1234,
  "sampleIndex": 2,
  "hardware": [
    {
      "name": "AMD Ryzen 7",
      "hardwareType": "Cpu",
      "identifier": "/amdcpu/0",
      "parentIdentifier": null,
      "updateError": null,
      "sensors": [
        {
          "name": "CPU Package",
          "sensorType": "Temperature",
          "identifier": "/amdcpu/0/temperature/0",
          "parentIdentifier": "/amdcpu/0",
          "unit": "C",
          "value": 61.5
        }
      ]
    }
  ],
  "errors": []
}
```

Error response:

```json
{
  "id": 1,
  "ok": false,
  "helperPid": 1234,
  "sampleIndex": 0,
  "hardware": [],
  "errors": ["CPU hardware update failed: ..."]
}
```

Rules:

- The response repeats the request ID.
- `sampleIndex` increases within one helper process and resets after restart.
- Hardware and sensor identifiers come directly from LibreHardwareMonitor.
- Subhardware is represented as a hardware entry with `parentIdentifier` set.
- A missing sensor value is serialized as `null`; it is not changed to zero.
- Non-finite values cannot be represented as JSON numbers and are serialized as `null` with an error entry.
- Units are derived from the LibreHardwareMonitor sensor type using an explicit helper-side mapping.
- Unknown sensor types retain their type name and use a null unit rather than being dropped.

## Helper Behavior

At startup the helper creates one `Computer` with CPU enabled and calls `Open()` once. It then reads stdin line by line:

- `sample`: update CPU hardware recursively, collect hardware and sensor records, increment the sample index, and write one response line.
- `shutdown`: write a successful acknowledgment, close the computer, and exit with code 0.
- malformed JSON or unknown command: write an error response when an ID can be recovered; otherwise write a protocol error with a null ID and continue.
- stdin EOF: close the computer and exit cleanly.

One hardware update failure must not erase successfully collected hardware. The response records the failure close to the affected hardware or in the top-level error list.

## Rust Client Lifecycle

The Windows-only `SensorHelperClient`:

1. Finds and spawns the packaged helper with hidden-window flags.
2. Pipes stdin, stdout, and stderr.
3. Uses a dedicated stdout reader to parse complete response lines.
4. Continuously drains stderr so the child cannot block on a full buffer.
5. Sends one request at a time with a monotonic request ID.
6. Waits up to two seconds for the matching response.
7. Treats timeout, EOF, broken pipe, invalid JSON, or mismatched ID as a transport failure.
8. On transport failure, kills and reaps the old child, starts one replacement, and retries the diagnostic sample once.
9. If the retry fails, returns the error through `SensorDiagnostics.rawError`.
10. On drop, sends `shutdown` best-effort, waits briefly, then kills and reaps a helper that does not exit.

The client never requests administrator elevation. Permission-dependent or absent readings remain visible in raw diagnostics as missing values or errors.

## Diagnostics Integration

This slice uses the existing manual `get_sensor_diagnostics` path and diagnostics panel. The complete helper response is serialized into `SensorDiagnostics.rawPayload` so it can be viewed and copied on Windows.

The existing `SensorSnapshot` remains unchanged. Native dashboard readings continue to behave as they do before this slice. The raw CPU inventory is evidence for the next design decision, not an authoritative dashboard source yet.

## Testing

### macOS and cross-platform tests

- Serialize sample and shutdown requests as one-line NDJSON.
- Parse successful, partial, and error responses.
- Reject malformed JSON and mismatched request IDs.
- Verify missing and non-finite values do not become zero.
- Use a fake transport to verify one in-flight request, timeout, one restart attempt, and final failure.
- Verify shutdown is attempted and an unresponsive child reaches the kill/reap fallback.
- Preserve all existing `SensorSnapshot` serialization and frontend tests.

The tests do not claim that LibreHardwareMonitor accessed real sensors on macOS.

### Windows CI

The existing `windows-build.yml` adds a protocol smoke test before packaging:

1. Build the self-contained helper.
2. Start one helper process with redirected stdin/stdout.
3. Send two sample requests and verify valid JSON, matching IDs, the same helper PID, and increasing sample index.
4. Do not require a non-empty sensor list on the GitHub-hosted VM.
5. Send shutdown and verify exit code 0.
6. Run Rust and frontend tests before producing the installer artifact.

### Physical Windows validation

After the branch is pushed to `dev` and the `pc-health-windows-installers` artifact is produced:

1. Install and run the application on the target Windows PC.
2. Capture diagnostics twice while idle and once under CPU load.
3. Copy each raw JSON response and return it for analysis.
4. Verify the helper PID stays the same and sample index increases.
5. Verify CPU hardware and sensor identifiers remain stable.
6. Check whether temperature, load, clock, and power values respond plausibly to load.
7. Confirm only one helper process exists and it exits with the app.

## Success Criteria

- Repeated manual diagnostics reuse one helper process.
- Raw CPU hardware and sensor identity cross the helper boundary without lossy selection.
- Missing, invalid, and permission-dependent values are not presented as zero or available readings.
- Transport failures are visible and receive at most one automatic restart attempt.
- The existing dashboard and `SensorSnapshot` contract do not change.
- Windows CI verifies the protocol without pretending to validate physical sensors.
- The physical Windows handoff produces copyable raw JSON for the next CPU mapping design.
