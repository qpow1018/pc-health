# Windows Sensor Diagnostics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a manual live-mode diagnostics capture that exposes Windows sensor raw input, parsed telemetry, final snapshot, and collection duration without making normal dashboard polling heavier.

**Architecture:** Keep `SensorSnapshot` as the dashboard contract and add a separate diagnostics contract. Rust exposes `get_sensor_diagnostics`; the Windows collector creates diagnostics from the same live collection path, while non-Windows returns a structured unsupported-platform diagnostic. React adds a diagnostics API and a secondary live-mode panel invoked only by user action.

**Tech Stack:** Tauri 2, Rust, serde, chrono, React, TypeScript, Vitest, Testing Library.

---

### Task 1: Rust Diagnostics Contract And Collector Capture

**Files:**
- Modify: `src-tauri/src/domain.rs`
- Modify: `src-tauri/src/collector/windows.rs`

- [ ] **Step 1: Add Rust diagnostics types and serialization tests**

Add these types to `src-tauri/src/domain.rs` after `SensorSnapshot`:

```rust
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedTelemetry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_usage: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_clock_mhz: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_memory_kb: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub free_memory_kb: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SensorDiagnostics {
    pub collected_at: String,
    pub collector: String,
    pub duration_ms: u128,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_payload: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parsed_telemetry: Option<ParsedTelemetry>,
    pub snapshot: SensorSnapshot,
}
```

Add a test in `domain.rs`:

```rust
#[test]
fn diagnostics_serializes_with_camel_case_fields() {
    let diagnostics = SensorDiagnostics {
        collected_at: "2026-06-17T12:00:00Z".to_string(),
        collector: "windows-powershell".to_string(),
        duration_ms: 12,
        raw_payload: Some(r#"{"CpuUsage":37}"#.to_string()),
        raw_error: None,
        parsed_telemetry: Some(ParsedTelemetry {
            cpu_name: Some("AMD Ryzen".to_string()),
            cpu_usage: Some(37.0),
            cpu_clock_mhz: None,
            total_memory_kb: None,
            free_memory_kb: None,
        }),
        snapshot: SensorSnapshot {
            collected_at: "2026-06-17T12:00:00Z".to_string(),
            devices: vec![],
        },
    };

    assert_eq!(
        serde_json::to_value(diagnostics).unwrap(),
        json!({
            "collectedAt": "2026-06-17T12:00:00Z",
            "collector": "windows-powershell",
            "durationMs": 12,
            "rawPayload": "{\"CpuUsage\":37}",
            "parsedTelemetry": {
                "cpuName": "AMD Ryzen",
                "cpuUsage": 37.0
            },
            "snapshot": {
                "collectedAt": "2026-06-17T12:00:00Z",
                "devices": []
            }
        })
    );
}
```

- [ ] **Step 2: Run the Rust domain test and verify it fails before implementation**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml diagnostics_serializes_with_camel_case_fields
```

Expected before implementation: FAIL because `SensorDiagnostics` and `ParsedTelemetry` do not exist.

- [ ] **Step 3: Implement Windows diagnostic capture**

In `src-tauri/src/collector/windows.rs`, expose a method:

```rust
pub fn diagnose_live(&mut self, collected_at: String) -> SensorDiagnostics
```

The method should:

- start an `Instant`
- run the same hidden PowerShell command used by `collect_live`
- include `rawPayload` on successful stdout
- include `rawError` on process, stderr, or parse failure
- include `parsedTelemetry` when JSON parsing succeeds
- include a final `snapshot` using the same conversion path as live collection
- use `collector: "windows-powershell"`

Keep `collect_live` behavior intact by routing it through the same helper and returning only `snapshot`.

- [ ] **Step 4: Run focused Rust tests**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml windows_telemetry_json_maps_to_cpu_memory_snapshot diagnostics_serializes_with_camel_case_fields
```

Expected: PASS.

### Task 2: Tauri Diagnostics Command

**Files:**
- Modify: `src-tauri/src/collector/mod.rs`
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add a platform diagnostics service path**

In `src-tauri/src/collector/mod.rs`, export `SensorDiagnostics` from `domain.rs` where needed and add a platform diagnostics helper:

```rust
#[cfg(not(target_os = "windows"))]
pub fn unsupported_platform_diagnostics(collected_at: String) -> SensorDiagnostics
```

The unsupported diagnostic should use `collector: "unsupported-platform"`, `duration_ms: 0`, no raw payload, and a mock unsupported or waiting snapshot that keeps planned readings visible.

- [ ] **Step 2: Add command testable behavior through command code**

In `src-tauri/src/commands.rs`, add:

```rust
#[tauri::command]
pub async fn get_sensor_diagnostics(
    state: State<'_, AppState>,
) -> Result<SensorDiagnostics, String>
```

Use `spawn_blocking` like `get_sensor_snapshot`, lock the same `SnapshotService` only long enough to run the diagnostic capture, and return structured diagnostics.

- [ ] **Step 3: Register the command**

In `src-tauri/src/lib.rs`, add `get_sensor_diagnostics` to the imports and `tauri::generate_handler!`.

- [ ] **Step 4: Run Rust command checks**

Run:

```bash
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: both pass.

### Task 3: Frontend Diagnostics API And Types

**Files:**
- Modify: `src/features/sensors/types.ts`
- Modify: `src/features/sensors/api.ts`
- Modify: `src/features/sensors/api.test.ts`

- [ ] **Step 1: Write API tests**

Add tests to `api.test.ts`:

```ts
it("invokes the diagnostics command when Tauri is available", async () => {
  vi.mocked(invoke).mockResolvedValue({
    collectedAt: "now",
    collector: "windows-powershell",
    durationMs: 10,
    snapshot: { collectedAt: "now", devices: [] },
  });

  await getSensorDiagnostics();

  expect(invoke).toHaveBeenCalledWith("get_sensor_diagnostics");
});

it("rejects diagnostics clearly outside Tauri", async () => {
  Reflect.deleteProperty(window, "__TAURI_INTERNALS__");

  await expect(getSensorDiagnostics()).rejects.toBeInstanceOf(
    SensorRuntimeUnavailableError,
  );
  expect(invoke).not.toHaveBeenCalled();
});
```

- [ ] **Step 2: Run API tests and verify failure**

Run:

```bash
npm test -- src/features/sensors/api.test.ts
```

Expected before implementation: FAIL because `getSensorDiagnostics` and `SensorDiagnostics` do not exist.

- [ ] **Step 3: Implement frontend diagnostics types and API**

In `types.ts`, add:

```ts
export type ParsedTelemetry = {
  cpuName?: string;
  cpuUsage?: number;
  cpuClockMhz?: number;
  totalMemoryKb?: number;
  freeMemoryKb?: number;
};

export type SensorDiagnostics = {
  collectedAt: string;
  collector: string;
  durationMs: number;
  rawPayload?: string;
  rawError?: string;
  parsedTelemetry?: ParsedTelemetry;
  snapshot: SensorSnapshot;
};
```

In `api.ts`, add:

```ts
export function getSensorDiagnostics() {
  if (!("__TAURI_INTERNALS__" in window)) {
    return Promise.reject(new SensorRuntimeUnavailableError());
  }

  return invoke<SensorDiagnostics>("get_sensor_diagnostics");
}
```

- [ ] **Step 4: Run API tests**

Run:

```bash
npm test -- src/features/sensors/api.test.ts
```

Expected: PASS.

### Task 4: Dashboard Diagnostics Panel

**Files:**
- Create: `src/features/sensors/useSensorDiagnostics.ts`
- Create: `src/features/sensors/useSensorDiagnostics.test.tsx`
- Create: `src/features/dashboard/SensorDiagnosticsPanel.tsx`
- Create: `src/features/dashboard/SensorDiagnosticsPanel.test.tsx`
- Modify: `src/features/dashboard/Dashboard.tsx`
- Modify: `src/features/dashboard/Dashboard.module.css`
- Modify: `src/pages/DashboardPage.tsx`
- Modify: `src/features/dashboard/Dashboard.test.tsx`

- [ ] **Step 1: Write hook tests**

Create `useSensorDiagnostics.test.tsx` with tests that verify:

- no diagnostics request happens before user action
- `capture()` sets loading then stores diagnostics
- runtime-unavailable errors produce the Korean Tauri message
- generic errors produce a retryable diagnostics error message

- [ ] **Step 2: Implement the diagnostics hook**

Create `useSensorDiagnostics.ts`:

```ts
import { useState } from "react";
import { getSensorDiagnostics, SensorRuntimeUnavailableError } from "./api";
import type { SensorDiagnostics } from "./types";

type DiagnosticsLoader = () => Promise<SensorDiagnostics>;

export function useSensorDiagnostics(loader: DiagnosticsLoader = getSensorDiagnostics) {
  const [diagnostics, setDiagnostics] = useState<SensorDiagnostics | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  const capture = async () => {
    setIsLoading(true);
    setError(null);
    try {
      setDiagnostics(await loader());
    } catch (caughtError) {
      if (caughtError instanceof SensorRuntimeUnavailableError) {
        setError("Tauri 앱에서 실행해야 진단 정보를 불러올 수 있습니다.");
      } else {
        setError("진단 정보를 불러오지 못했습니다.");
      }
    } finally {
      setIsLoading(false);
    }
  };

  return { diagnostics, error, isLoading, capture };
}
```

- [ ] **Step 3: Write panel tests**

Create `SensorDiagnosticsPanel.test.tsx` with tests that verify:

- live-mode capture button calls `onCapture`
- loading disables the button
- raw payload, parsed telemetry, final snapshot, and raw error render when provided
- no panel is shown in development mode from `Dashboard`

- [ ] **Step 4: Implement panel and wire into Dashboard**

Add a compact secondary panel rendered only when `mode === "live"`. Pass diagnostics state from `DashboardPage` into `Dashboard`, and keep `Dashboard` presentational.

Use `navigator.clipboard.writeText(JSON.stringify(diagnostics, null, 2))` for copy, guarded by the presence of diagnostics.

- [ ] **Step 5: Run frontend tests**

Run:

```bash
npm test
```

Expected: PASS.

### Task 5: Full Verification And Commit

**Files:**
- All files changed by Tasks 1-4.

- [ ] **Step 1: Format and verify**

Run:

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
npm test
npm run build
```

Expected: all pass.

- [ ] **Step 2: Commit implementation**

Run:

```bash
git add src-tauri/src/domain.rs src-tauri/src/collector/mod.rs src-tauri/src/collector/windows.rs src-tauri/src/commands.rs src-tauri/src/lib.rs src/features/sensors/types.ts src/features/sensors/api.ts src/features/sensors/api.test.ts src/features/sensors/useSensorDiagnostics.ts src/features/sensors/useSensorDiagnostics.test.tsx src/features/dashboard/SensorDiagnosticsPanel.tsx src/features/dashboard/SensorDiagnosticsPanel.test.tsx src/features/dashboard/Dashboard.tsx src/features/dashboard/Dashboard.module.css src/features/dashboard/Dashboard.test.tsx src/pages/DashboardPage.tsx
git commit -m "feat: add Windows sensor diagnostics"
```

Expected: commit succeeds with only diagnostics-related files.

## Self-Review

- Spec coverage: manual diagnostics command, panel, raw/parsed/final snapshot visibility, non-Windows unsupported diagnostics, and no normal-path logging are covered.
- Scope check: PowerShell replacement, GPU support, persistent logging, and background sampling remain excluded.
- Type consistency: frontend `SensorDiagnostics` uses camelCase names matching Rust serde output.
