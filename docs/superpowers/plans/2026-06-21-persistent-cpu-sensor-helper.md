# Persistent CPU Sensor Helper Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace per-sample helper execution with one persistent LibreHardwareMonitor CPU helper and expose complete raw CPU sensor responses through existing diagnostics.

**Architecture:** A testable C# `HelperHost` owns the NDJSON command loop and delegates CPU collection to a LibreHardwareMonitor sampler. A Windows-only Rust `SensorHelperClient` owns the child process, stdio reader threads, timeout, one-restart policy, and shutdown; `WindowsCollector` reuses that client for both live CPU temperature and manual diagnostics while keeping `SensorSnapshot` unchanged.

**Tech Stack:** .NET 8, LibreHardwareMonitorLib 0.9.6, xUnit, Rust std process/mpsc, serde/serde_json, Tauri 2, GitHub Actions PowerShell.

---

## File Map

| File | Responsibility |
| --- | --- |
| `src-tauri/helpers/sensor-helper/Protocol.cs` | NDJSON request/response DTOs, unit mapping, finite-value normalization |
| `src-tauri/helpers/sensor-helper/HelperHost.cs` | Persistent command loop independent of console and LHM |
| `src-tauri/helpers/sensor-helper/LibreHardwareCpuSampler.cs` | Open/update/close LHM CPU hardware and build raw records |
| `src-tauri/helpers/sensor-helper/Program.cs` | Wire console streams, process ID, sampler, and host |
| `src-tauri/helpers/sensor-helper-tests/PcHealth.SensorHelper.Tests.csproj` | xUnit project referencing the helper |
| `src-tauri/helpers/sensor-helper-tests/HelperHostTests.cs` | Protocol loop, repeated sample, shutdown, malformed command tests |
| `src-tauri/src/collector/sensor_helper.rs` | Rust protocol parsing and Windows child-process client |
| `src-tauri/src/collector/windows.rs` | Reuse persistent sample for existing CPU temperature and diagnostics raw payload |
| `src-tauri/src/collector/mod.rs` | Register the helper module |
| `src-tauri/Cargo.toml` | Remove unused legacy WMI dependency after fallback deletion |
| `scripts/build-sensor-helper.mjs` | Replace an existing sidecar binary safely on repeated builds |
| `scripts/test-sensor-helper.ps1` | Windows persistent-process smoke test |
| `.github/workflows/windows-build.yml` | Run frontend, Rust, .NET, and helper smoke tests before packaging |

### Task 1: Define And Test The NDJSON Host

**Files:**
- Create: `src-tauri/helpers/sensor-helper/Protocol.cs`
- Create: `src-tauri/helpers/sensor-helper/HelperHost.cs`
- Create: `src-tauri/helpers/sensor-helper-tests/PcHealth.SensorHelper.Tests.csproj`
- Create: `src-tauri/helpers/sensor-helper-tests/HelperHostTests.cs`
- Modify: `src-tauri/helpers/sensor-helper/PcHealth.SensorHelper.csproj`

- [ ] **Step 1: Create the failing xUnit project and host tests**

`PcHealth.SensorHelper.Tests.csproj`:

```xml
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net8.0-windows</TargetFramework>
    <ImplicitUsings>enable</ImplicitUsings>
    <Nullable>enable</Nullable>
    <IsPackable>false</IsPackable>
  </PropertyGroup>
  <ItemGroup>
    <PackageReference Include="Microsoft.NET.Test.Sdk" Version="18.6.0" />
    <PackageReference Include="xunit.v3" Version="3.2.2" />
    <ProjectReference Include="../sensor-helper/PcHealth.SensorHelper.csproj" />
  </ItemGroup>
</Project>
```

Write tests using a `FakeSampler : ICpuSensorSampler` and `StringReader/StringWriter`:

```csharp
[Fact]
public void Run_reuses_one_sampler_for_two_samples_and_shuts_down()
{
    var input = new StringReader(
        "{\"id\":1,\"command\":\"sample\"}\n" +
        "{\"id\":2,\"command\":\"sample\"}\n" +
        "{\"id\":3,\"command\":\"shutdown\"}\n");
    var output = new StringWriter();
    var sampler = new FakeSampler();

    new HelperHost(sampler, helperPid: 42).Run(input, output, TextWriter.Null);

    var lines = output.ToString().Split(Environment.NewLine, StringSplitOptions.RemoveEmptyEntries);
    Assert.Equal(3, lines.Length);
    Assert.Contains("\"helperPid\":42", lines[0]);
    Assert.Contains("\"sampleIndex\":1", lines[0]);
    Assert.Contains("\"sampleIndex\":2", lines[1]);
    Assert.Contains("\"command\":\"shutdown\"", lines[2]);
    Assert.Equal(2, sampler.SampleCount);
}
```

Add tests asserting malformed JSON emits `ok:false` and continues, unknown commands emit an error response, missing values serialize as `null`, and non-finite values normalize to `null` plus an error.

- [ ] **Step 2: Run the tests and verify RED**

Run on a machine with .NET 8:

```bash
dotnet test src-tauri/helpers/sensor-helper-tests/PcHealth.SensorHelper.Tests.csproj
```

Expected: FAIL because `HelperHost`, `ICpuSensorSampler`, and protocol DTOs do not exist.

- [ ] **Step 3: Implement protocol DTOs**

Create these public records in `Protocol.cs` with camelCase JSON configuration:

```csharp
public sealed record HelperRequest(long? Id, string? Command);
public sealed record SensorRecord(
    string Name,
    string SensorType,
    string Identifier,
    string ParentIdentifier,
    string? Unit,
    double? Value);
public sealed record HardwareRecord(
    string Name,
    string HardwareType,
    string Identifier,
    string? ParentIdentifier,
    string? UpdateError,
    IReadOnlyList<SensorRecord> Sensors);
public sealed record HelperResponse(
    long? Id,
    bool Ok,
    int HelperPid,
    long SampleIndex,
    string Command,
    IReadOnlyList<HardwareRecord> Hardware,
    IReadOnlyList<string> Errors);
```

Provide `ProtocolJson.Options`, `SensorUnits.For(SensorType)`, and `SensorValues.Normalize(float?, errors, identifier)`. Map Temperature→`C`, Load/Control/Level→`%`, Clock→`MHz`, Power→`W`, Fan→`RPM`, Voltage→`V`, Data→`GB`, Throughput→`B/s`, and unknown types→null.

- [ ] **Step 4: Implement the host loop**

`ICpuSensorSampler` exposes:

```csharp
IReadOnlyList<HardwareRecord> Sample(ICollection<string> errors);
```

`HelperHost.Run(TextReader input, TextWriter output, TextWriter error)` must read one line at a time, increment sample index only for `sample`, write and flush exactly one JSON line per command, acknowledge `shutdown`, continue after malformed input, and never write logs to stdout.

- [ ] **Step 5: Run GREEN and commit**

Run:

```bash
dotnet test src-tauri/helpers/sensor-helper-tests/PcHealth.SensorHelper.Tests.csproj
```

Expected: all helper host tests pass.

```bash
git add src-tauri/helpers/sensor-helper src-tauri/helpers/sensor-helper-tests
git commit -m "test: define sensor helper protocol"
```

### Task 2: Implement LibreHardwareMonitor CPU Inventory

**Files:**
- Create: `src-tauri/helpers/sensor-helper/LibreHardwareCpuSampler.cs`
- Modify: `src-tauri/helpers/sensor-helper/Program.cs`
- Test: `src-tauri/helpers/sensor-helper-tests/HelperHostTests.cs`

- [ ] **Step 1: Add failing sampler-shape tests**

Test a pure `HardwareRecordFactory.Create` method with fake input records so that a child hardware item retains its parent identifier, sensor identifiers remain unchanged, null values stay null, and unknown units stay null. Keep LHM interfaces out of the fake test.

- [ ] **Step 2: Run and verify RED**

```bash
dotnet test src-tauri/helpers/sensor-helper-tests/PcHealth.SensorHelper.Tests.csproj
```

Expected: FAIL because `HardwareRecordFactory` does not exist.

- [ ] **Step 3: Implement the LHM sampler**

`LibreHardwareCpuSampler` owns one `Computer { IsCpuEnabled = true }`, calls `Open()` in its constructor, recursively updates CPU hardware on every sample, and calls `Close()` from `Dispose()`.

For each hardware/subhardware record preserve:

- `Name`
- `HardwareType.ToString()`
- `Identifier.ToString()`
- parent identifier
- update exception text
- all sensors with name, type, identifier, parent identifier, explicit unit, and normalized raw value

Catch update failures per hardware, append errors, and continue collecting other hardware. Do not select CPU package temperature or any other representative reading.

- [ ] **Step 4: Replace the one-shot Program**

`Program.cs` becomes only:

```csharp
using PcHealth.SensorHelper;

using var sampler = new LibreHardwareCpuSampler();
var host = new HelperHost(sampler, Environment.ProcessId);
host.Run(Console.In, Console.Out, Console.Error);
```

Set the helper project's root namespace consistently and remove the old `SensorHelperOutput`, temperature priority, and one-shot logic.

- [ ] **Step 5: Run tests and commit**

```bash
dotnet test src-tauri/helpers/sensor-helper-tests/PcHealth.SensorHelper.Tests.csproj
```

Expected: all tests pass.

```bash
git add src-tauri/helpers/sensor-helper src-tauri/helpers/sensor-helper-tests
git commit -m "feat: expose persistent CPU sensor inventory"
```

### Task 3: Add The Rust Protocol And Persistent Client

**Files:**
- Create: `src-tauri/src/collector/sensor_helper.rs`
- Modify: `src-tauri/src/collector/mod.rs`

- [ ] **Step 1: Write failing cross-platform protocol tests**

Define tests before implementation:

```rust
#[test]
fn parses_raw_cpu_sensor_response_without_changing_null_to_zero() {
    let response = parse_response(
        r#"{"id":7,"ok":true,"helperPid":42,"sampleIndex":2,"command":"sample","hardware":[{"name":"CPU","hardwareType":"Cpu","identifier":"/cpu/0","parentIdentifier":null,"updateError":null,"sensors":[{"name":"Package","sensorType":"Temperature","identifier":"/cpu/0/temp/0","parentIdentifier":"/cpu/0","unit":"C","value":null}]}],"errors":[]}"#,
        7,
    ).unwrap();

    assert_eq!(response.helper_pid, 42);
    assert_eq!(response.hardware[0].sensors[0].value, None);
}
```

Also test mismatched IDs, `ok:false`, malformed JSON, sample-request serialization with newline, and selection of the existing CPU temperature from valid Temperature records only.

- [ ] **Step 2: Verify RED**

```bash
cargo test --manifest-path src-tauri/Cargo.toml sensor_helper
```

Expected: FAIL because the module and protocol functions do not exist.

- [ ] **Step 3: Implement cross-platform protocol types**

Add serde camelCase structs matching the C# records exactly. Implement:

```rust
fn sample_request(id: u64) -> Result<String, String>;
fn shutdown_request(id: u64) -> Result<String, String>;
fn parse_response(line: &str, expected_id: u64) -> Result<SensorHelperResponse, String>;
fn select_cpu_temperature(response: &SensorHelperResponse) -> Option<f64>;
```

Temperature selection preserves current behavior: finite values in `(0, 130]`, prefer names containing package/tctl/tdie, and never treat null or zero as available.

- [ ] **Step 4: Implement the Windows child process**

Under `cfg(target_os = "windows")`, `SensorHelperClient` owns `Child`, `ChildStdin`, an mpsc stdout receiver, a drained stderr tail, and next request ID. Spawn with piped streams and `CREATE_NO_WINDOW`.

`sample()` writes and flushes a request, waits with `recv_timeout(Duration::from_secs(2))`, validates the ID, and returns both parsed response and original line. On EOF, timeout, broken pipe, invalid JSON, or ID mismatch, reset the child and retry once.

`Drop` sends shutdown best-effort, polls `try_wait()` for 500 ms, then calls `kill()` and `wait()`.

Extract the one-retry policy into a small closure-based function and test: first failure/second success, two failures, and no third attempt.

- [ ] **Step 5: Run Rust tests and commit**

```bash
cargo test --manifest-path src-tauri/Cargo.toml sensor_helper
```

Expected: all sensor-helper protocol and retry tests pass on macOS; Windows-only process code compiles later in Windows CI.

```bash
git add src-tauri/src/collector/mod.rs src-tauri/src/collector/sensor_helper.rs
git commit -m "feat: add persistent sensor helper client"
```

### Task 4: Integrate The Client Into Windows Diagnostics

**Files:**
- Modify: `src-tauri/src/collector/windows.rs`
- Modify: `src-tauri/Cargo.toml`
- Test: `src-tauri/src/collector/windows.rs`

- [ ] **Step 1: Write failing integration tests**

Replace `native_diagnostics_have_no_raw_payload` with a test that builds diagnostics from telemetry plus a helper raw line and asserts:

```rust
assert_eq!(diagnostics.collector, "windows-native+librehardwaremonitor");
assert_eq!(diagnostics.raw_payload.as_deref(), Some(raw_line));
assert!(diagnostics.raw_error.is_none());
assert_eq!(cpu_temperature(&diagnostics.snapshot), Some(64.5));
```

Add a test where helper sampling fails: CPU temperature remains unsupported, CPU/memory native readings remain available, and `raw_error` contains the helper failure.

- [ ] **Step 2: Verify RED**

```bash
cargo test --manifest-path src-tauri/Cargo.toml collector::windows
```

Expected: FAIL because diagnostics currently has no helper raw payload and `WindowsCollector` has no persistent client.

- [ ] **Step 3: Integrate one persistent client**

Add `sensor_helper: SensorHelperClient` to the Windows-only collector state and implement `Default` manually. During native collection:

1. collect existing `GetSystemTimes`, memory, registry name, and registry clock values;
2. call the persistent helper once;
3. derive the existing CPU temperature from the parsed raw response;
4. preserve the original response line for diagnostics;
5. if helper sampling fails, keep native CPU/memory telemetry and record a helper error instead of failing the whole snapshot.

Set diagnostics collector to `windows-native+librehardwaremonitor`. Keep the existing `SensorSnapshot` devices/readings and warning behavior unchanged.

- [ ] **Step 4: Remove legacy parallel paths**

Delete:

- the one-shot `Command::output()` CPU helper path;
- `SensorHelperOutput`;
- LibreHardwareMonitor/OpenHardwareMonitor WMI namespace fallback;
- COM/WMI structs and selection helpers used only by that fallback;
- the now-unused `windows` crate dependency and WMI feature list from `Cargo.toml`.

Keep `windows-sys` registry, memory, and CPU-time dependencies because those existing native readings remain in this slice.

- [ ] **Step 5: Run Rust tests and commit**

```bash
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: all Rust tests pass and macOS check succeeds.

```bash
git add src-tauri/Cargo.toml src-tauri/src/collector
git commit -m "feat: reuse LHM helper for CPU diagnostics"
```

### Task 5: Add Windows Protocol Smoke Coverage

**Files:**
- Modify: `scripts/build-sensor-helper.mjs`
- Create: `scripts/test-sensor-helper.ps1`
- Modify: `.github/workflows/windows-build.yml`

- [ ] **Step 1: Write the smoke script before wiring CI**

`test-sensor-helper.ps1` starts the published helper with redirected stdin/stdout/stderr, sends sample IDs 1 and 2, parses each response with `ConvertFrom-Json`, and asserts:

```powershell
if (-not $first.ok) { throw "first sample failed: $($first.errors -join '; ')" }
if ($first.id -ne 1 -or $second.id -ne 2) { throw "request id mismatch" }
if ($first.helperPid -ne $second.helperPid) { throw "helper process was not reused" }
if ($second.sampleIndex -ne ($first.sampleIndex + 1)) { throw "sample index did not increase" }
```

Do not assert that `hardware` is non-empty. Send shutdown ID 3, close stdin, wait up to five seconds, kill on timeout, and require exit code 0.

- [ ] **Step 2: Make repeated helper builds safe**

Import `rm` in `build-sensor-helper.mjs` and run:

```javascript
await rm(target, { force: true });
await rename(source, target);
```

This allows the explicit CI helper build and Tauri's later `beforeBuildCommand` to run in the same job.

- [ ] **Step 3: Add CI verification before packaging**

After `npm ci`, add:

```yaml
      - name: Run frontend tests
        run: npm test

      - name: Run Rust tests
        run: cargo test --manifest-path src-tauri/Cargo.toml

      - name: Run sensor helper tests
        run: dotnet test src-tauri/helpers/sensor-helper-tests/PcHealth.SensorHelper.Tests.csproj

      - name: Build sensor helper
        run: npm run build:sensor-helper

      - name: Smoke test persistent sensor helper
        shell: pwsh
        run: ./scripts/test-sensor-helper.ps1
```

Keep the existing Tauri build and installer upload steps.

- [ ] **Step 4: Validate scripts and commit**

Run locally:

```bash
npm test
cargo test --manifest-path src-tauri/Cargo.toml
npm run build
```

Expected: frontend tests, Rust tests, and frontend build pass. The PowerShell smoke test is marked Windows-only and is verified by Actions.

```bash
git add scripts .github/workflows/windows-build.yml
git commit -m "ci: verify persistent sensor helper"
```

### Task 6: Final Verification And Windows Handoff

**Files:**
- Verify: `src-tauri/helpers/sensor-helper/`
- Verify: `src-tauri/src/collector/`
- Verify: `.github/workflows/windows-build.yml`
- Create when needed: `_workspace/02_hardware_telemetry_findings.md`

- [ ] **Step 1: Run all local checks**

```bash
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
git diff --check
```

Expected: all commands exit 0.

- [ ] **Step 2: Verify scope**

```bash
git diff --name-only dev...HEAD
rg -n "OpenHardwareMonitor|ROOT\\\\LibreHardwareMonitor|Command::new.*sensor" src-tauri/src
```

Expected: changes are limited to the design/plan, helper, collector integration, scripts, Cargo dependency cleanup, and workflow; the stale parallel provider patterns return no matches.

- [ ] **Step 3: Push only after user approval**

Report local verification and ask before:

```bash
git push -u origin codex/cpu-persistent-sensor-helper
```

- [ ] **Step 4: Merge or push to dev only after user choice**

The Windows workflow runs on `dev`, so a feature-branch push alone does not produce installers. After review, merge the feature branch into `dev` and push `dev` only with user approval.

- [ ] **Step 5: Record physical Windows evidence**

After the user returns idle/idle/load raw JSON captures, write `_workspace/02_hardware_telemetry_findings.md` with:

- helper PID and sample-index continuity;
- CPU hardware and sensor identifiers;
- missing, invalid, permission-dependent, and error values;
- temperature/load/clock/power candidate families;
- whether the evidence is sufficient for a separate `SensorSnapshot` mapping design.
