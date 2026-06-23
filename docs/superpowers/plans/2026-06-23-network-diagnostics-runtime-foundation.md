# Network Diagnostics Runtime Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Start a bounded wired-LAN diagnostic runtime with the app, classify the latest evidence through a deterministic lifecycle, and show the current status without persistence.

**Architecture:** A shared `NetworkProbeCoordinator` serializes the existing manual raw probe and the new background worker. Focused Rust modules separate observation collection, pure classification/state transitions, and scheduling; Tauri exposes only a clone of the latest in-memory status. A feature-local React panel polls that memory command once per second while actual network probes remain on the Rust schedule.

**Tech Stack:** Rust, Windows APIs through `windows`, blocking `reqwest`, Tauri 2 managed state and commands, React 19, TypeScript, CSS Modules, Vitest, Testing Library

---

## File map

- `src-tauri/src/network/domain.rs`: wire enums and current-status contract.
- `src-tauri/src/network/collector.rs`: single-endpoint HTTP collector boundary.
- `src-tauri/src/network/service.rs`: full/baseline probe assembly and shared coordinator.
- `src-tauri/src/network/observation.rs`: snapshot-to-evidence classification.
- `src-tauri/src/network/state_machine.rs`: pure lifecycle transitions.
- `src-tauri/src/network/runtime.rs`: schedule, worker ownership, cooldown and latest state.
- `src-tauri/src/network/windows.rs`: one-endpoint Windows HTTP implementation.
- `src-tauri/src/network/unsupported.rs`: non-Windows single-endpoint result.
- `src-tauri/src/commands.rs`: raw probe through coordinator and status read command.
- `src-tauri/src/lib.rs`: Tauri managed state and automatic runtime startup.
- `src/features/network-diagnostics/`: frontend contract, fixture, API, current-status panel and tests.
- `src/features/product-home/ProductHome.tsx`: place current status above raw probe.

### Task 1: Add the current diagnostic wire contract

**Files:**
- Modify: `src-tauri/src/network/domain.rs`
- Create: `src/features/network-diagnostics/types.ts`

- [ ] **Step 1: Write failing Rust serialization tests**

Append tests that construct the status directly and require snake-case enum values plus camel-case fields:

```rust
#[test]
fn serializes_network_diagnostic_status_contract() {
    let status = NetworkDiagnosticStatus {
        availability: RuntimeAvailability::Running,
        lifecycle: Some(DiagnosticLifecycle::Suspected),
        suspected_area: Some(DiagnosticArea::GatewayOrLocal),
        observed_at: Some("2026-06-23T00:00:00Z".into()),
        last_full_probe_at: None,
        evidence: vec![DiagnosticEvidence {
            source: EvidenceSource::Gateway,
            status: EvidenceStatus::Timeout,
            checked_at: Some("2026-06-23T00:00:00Z".into()),
            duration_ms: Some(1500),
            detail: Some("icmp_timeout".into()),
        }],
        error: None,
    };

    let json = serde_json::to_value(status).unwrap();
    assert_eq!(json["availability"], "running");
    assert_eq!(json["lifecycle"], "suspected");
    assert_eq!(json["suspectedArea"], "gateway_or_local");
    assert_eq!(json["evidence"][0]["source"], "gateway");
}

#[test]
fn starting_status_has_no_invented_observation() {
    let status = NetworkDiagnosticStatus::starting();
    assert_eq!(status.availability, RuntimeAvailability::Starting);
    assert_eq!(status.lifecycle, None);
    assert_eq!(status.observed_at, None);
    assert!(status.evidence.is_empty());
}
```

- [ ] **Step 2: Run the focused test and verify RED**

Run: `cargo test --manifest-path src-tauri/Cargo.toml network::domain::tests -- --nocapture`

Expected: compilation fails because `NetworkDiagnosticStatus` and its enums do not exist.

- [ ] **Step 3: Add the Rust contract**

Add these public types in `domain.rs`; derive `Clone, Debug, PartialEq, Eq, Serialize` on every enum and struct:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeAvailability { Starting, Running, Unavailable, Error }

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticLifecycle { Normal, Suspected, Incident, Recovering, Resolved }

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticArea { LocalConnection, GatewayOrLocal, Dns, External, Unknown }

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSource {
    Ethernet, Ipv4, DefaultRoute, Gateway,
    DnsMicrosoft, DnsGoogle, HttpMicrosoft, HttpGoogle,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceStatus { Success, Failure, Timeout, Unavailable, NotChecked }

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticEvidence {
    pub source: EvidenceSource,
    pub status: EvidenceStatus,
    pub checked_at: Option<String>,
    pub duration_ms: Option<u64>,
    pub detail: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkDiagnosticStatus {
    pub availability: RuntimeAvailability,
    pub lifecycle: Option<DiagnosticLifecycle>,
    pub suspected_area: Option<DiagnosticArea>,
    pub observed_at: Option<String>,
    pub last_full_probe_at: Option<String>,
    pub evidence: Vec<DiagnosticEvidence>,
    pub error: Option<ProbeError>,
}

impl NetworkDiagnosticStatus {
    pub fn starting() -> Self {
        Self {
            availability: RuntimeAvailability::Starting,
            lifecycle: None,
            suspected_area: None,
            observed_at: None,
            last_full_probe_at: None,
            evidence: vec![],
            error: None,
        }
    }
}
```

- [ ] **Step 4: Mirror the exact contract in the final TypeScript feature**

Create `src/features/network-diagnostics/types.ts`. Import `ProbeError` from `@/features/network-probe/types` and add matching union types and interfaces without moving the raw snapshot contract:

```ts
export type RuntimeAvailability = "starting" | "running" | "unavailable" | "error";
export type DiagnosticLifecycle = "normal" | "suspected" | "incident" | "recovering" | "resolved";
export type DiagnosticArea = "local_connection" | "gateway_or_local" | "dns" | "external" | "unknown";
export type EvidenceSource = "ethernet" | "ipv4" | "default_route" | "gateway" | "dns_microsoft" | "dns_google" | "http_microsoft" | "http_google";
export type EvidenceStatus = "success" | "failure" | "timeout" | "unavailable" | "not_checked";

export interface DiagnosticEvidence {
  source: EvidenceSource;
  status: EvidenceStatus;
  checkedAt: string | null;
  durationMs: number | null;
  detail: string | null;
}

export interface NetworkDiagnosticStatus {
  availability: RuntimeAvailability;
  lifecycle: DiagnosticLifecycle | null;
  suspectedArea: DiagnosticArea | null;
  observedAt: string | null;
  lastFullProbeAt: string | null;
  evidence: DiagnosticEvidence[];
  error: ProbeError | null;
}
```

- [ ] **Step 5: Verify and commit**

Run: `cargo test --manifest-path src-tauri/Cargo.toml network::domain::tests && npm run build`

Expected: Rust domain tests and TypeScript build pass.

```bash
git add src-tauri/src/network/domain.rs src/features/network-diagnostics/types.ts
git commit -m "feat: define network diagnostic status contract"
```

### Task 2: Add scoped probes and serialize manual/background collection

**Files:**
- Modify: `src-tauri/src/network/collector.rs`
- Modify: `src-tauri/src/network/service.rs`
- Modify: `src-tauri/src/network/windows.rs`
- Modify: `src-tauri/src/network/unsupported.rs`

- [ ] **Step 1: Write service tests for full and baseline scopes**

Extend the existing fake collector with recorded HTTP URLs and add:

```rust
#[test]
fn baseline_collects_inventory_gateway_and_only_requested_http_endpoint() {
    let calls = Arc::new(Mutex::new(vec![]));
    let service = NetworkProbeService::new(Box::new(FakeCollector::recording(calls.clone())));

    let snapshot = service.collect_baseline(Some(MICROSOFT_URL));

    assert!(snapshot.dns_checks.is_empty());
    assert_eq!(snapshot.http_checks.len(), 1);
    assert_eq!(snapshot.http_checks[0].url, MICROSOFT_URL);
    assert_eq!(*calls.lock().unwrap(), vec![MICROSOFT_URL]);
}

#[test]
fn full_probe_keeps_both_dns_and_http_targets() {
    let service = NetworkProbeService::new(Box::new(FakeCollector::default()));
    let snapshot = service.collect_full();
    assert_eq!(snapshot.dns_checks.len(), 2);
    assert_eq!(snapshot.http_checks.len(), 2);
}
```

- [ ] **Step 2: Verify RED**

Run: `cargo test --manifest-path src-tauri/Cargo.toml network::service::tests -- --nocapture`

Expected: compilation fails because `collect_baseline`, `collect_full`, and the single-endpoint collector method are missing.

- [ ] **Step 3: Add the single-endpoint collector boundary**

Change the trait to require both paths:

```rust
pub trait NetworkCollector: Send + Sync {
    fn collector_name(&self) -> &'static str;
    fn collect_inventory(&self) -> NetworkInventory;
    fn check_gateway(&self, route: Option<&RouteSnapshot>) -> GatewayCheck;
    fn check_dns(&self) -> Vec<DnsCheck>;
    fn check_http_endpoint(&self, url: &str) -> HttpCheck;
    fn check_http(&self) -> Vec<HttpCheck>;
}
```

In both platform collectors, implement `check_http_endpoint`. The Windows implementation must build the same bounded client used today and pass one URL to the existing response mapper; `check_http` must continue to build one client and reuse it for both URLs. The unsupported implementation returns one `ProbeStatus::Unsupported` result for the supplied URL.

- [ ] **Step 4: Split service collection and add the coordinator**

Keep `collect()` as a compatibility alias for `collect_full()`, then implement:

```rust
pub fn collect_full(&self) -> NetworkProbeSnapshot {
    self.collect_with(None, true)
}

pub fn collect_baseline(&self, http_url: Option<&str>) -> NetworkProbeSnapshot {
    self.collect_with(http_url, false)
}

fn collect_with(&self, http_url: Option<&str>, full: bool) -> NetworkProbeSnapshot {
    let started = Instant::now();
    let inventory = self.collector.collect_inventory();
    let selected_route = select_default_route(&inventory.default_routes);
    let gateway_check = self.collector.check_gateway(selected_route.as_ref());
    let dns_checks = if full { self.collector.check_dns() } else { vec![] };
    let http_checks = if full {
        self.collector.check_http()
    } else {
        http_url.map(|url| vec![self.collector.check_http_endpoint(url)]).unwrap_or_default()
    };
    NetworkProbeSnapshot {
        collected_at: Utc::now().to_rfc3339(),
        collector: self.collector.collector_name().into(),
        duration_ms: duration_ms(started),
        adapters: inventory.adapters,
        default_routes: inventory.default_routes,
        selected_route,
        gateway_check,
        dns_checks,
        http_checks,
        errors: inventory.errors,
    }
}
```

Add a cloneable coordinator with no generic abstraction:

```rust
#[derive(Clone)]
pub struct NetworkProbeCoordinator(Arc<Mutex<NetworkProbeService>>);

impl NetworkProbeCoordinator {
    pub fn platform() -> Self { Self(Arc::new(Mutex::new(NetworkProbeService::platform()))) }
    pub fn collect_full(&self) -> Result<NetworkProbeSnapshot, String> {
        self.0.lock().map_err(|_| "network probe coordinator lock failed".to_string())
            .map(|service| service.collect_full())
    }
    pub fn collect_baseline(&self, url: Option<&str>) -> Result<NetworkProbeSnapshot, String> {
        self.0.lock().map_err(|_| "network probe coordinator lock failed".to_string())
            .map(|service| service.collect_baseline(url))
    }
}
```

- [ ] **Step 5: Verify and commit**

Run: `cargo test --manifest-path src-tauri/Cargo.toml network::service::tests && cargo check --manifest-path src-tauri/Cargo.toml`

Expected: service tests pass and the active macOS configuration type-checks; Windows compilation is verified in Task 7.

```bash
git add src-tauri/src/network/collector.rs src-tauri/src/network/service.rs src-tauri/src/network/windows.rs src-tauri/src/network/unsupported.rs
git commit -m "refactor: support scoped network probes"
```

### Task 3: Classify observations and implement lifecycle transitions

**Files:**
- Create: `src-tauri/src/network/observation.rs`
- Create: `src-tauri/src/network/state_machine.rs`
- Modify: `src-tauri/src/network/mod.rs`

- [ ] **Step 1: Write observation classification tests**

Use small snapshot builders and assert these exact outcomes:

```rust
#[test]
fn inventory_error_without_route_is_unknown_not_local_connection() {
    let assessment = assess(snapshot_with_inventory_error());
    assert_eq!(assessment.area, Some(DiagnosticArea::Unknown));
}

#[test]
fn repeated_gateway_failure_can_identify_gateway_or_local() {
    let assessment = assess(snapshot_with_gateway(ProbeStatus::Timeout));
    assert_eq!(assessment.area, Some(DiagnosticArea::GatewayOrLocal));
    assert!(assessment.failed_sources.contains(&EvidenceSource::Gateway));
}

#[test]
fn dns_failure_with_successful_http_is_conflicting_unknown() {
    let assessment = assess(snapshot_with_dns_failed_http_success()));
    assert_eq!(assessment.area, Some(DiagnosticArea::Unknown));
}

#[test]
fn two_http_failures_after_gateway_and_dns_success_are_external() {
    let assessment = assess(snapshot_with_external_failure());
    assert_eq!(assessment.area, Some(DiagnosticArea::External));
}
```

- [ ] **Step 2: Write lifecycle tests**

Construct `DiagnosticAssessment` directly and cover the approved sequence:

```rust
#[test]
fn abnormal_start_requires_confirmation_before_incident() {
    let mut machine = NetworkDiagnosticStateMachine::new();
    let first = machine.apply(gateway_failure(), "2026-06-23T00:00:00Z", true);
    let second = machine.apply(gateway_failure(), "2026-06-23T00:00:10Z", false);
    assert_eq!(first.lifecycle, Some(DiagnosticLifecycle::Suspected));
    assert_eq!(second.lifecycle, Some(DiagnosticLifecycle::Incident));
}

#[test]
fn incident_requires_two_normal_observations_to_resolve() {
    let mut machine = incident_machine(DiagnosticArea::External);
    assert_eq!(machine.apply(normal(), "t1", false).lifecycle, Some(DiagnosticLifecycle::Recovering));
    assert_eq!(machine.apply(normal(), "t2", false).lifecycle, Some(DiagnosticLifecycle::Resolved));
    assert_eq!(machine.apply(normal(), "t3", false).lifecycle, Some(DiagnosticLifecycle::Normal));
}

#[test]
fn unknown_only_never_confirms_incident() {
    let mut machine = NetworkDiagnosticStateMachine::new();
    machine.apply(unknown_failure(EvidenceSource::HttpMicrosoft), "t1", false);
    let status = machine.apply(unknown_failure(EvidenceSource::HttpMicrosoft), "t2", true);
    assert_eq!(status.lifecycle, Some(DiagnosticLifecycle::Suspected));
}
```

Also add focused-refinement, recovery relapse, and resolved-new-suspicion tests named exactly after those behaviors.

Add a coverage regression test: a normal baseline without DNS evidence must keep a DNS incident unchanged; a normal focused assessment containing both DNS sources may enter recovering.

- [ ] **Step 3: Verify RED**

Run: `cargo test --manifest-path src-tauri/Cargo.toml network::observation::tests network::state_machine::tests -- --nocapture`

Expected: compilation fails because both modules are absent.

- [ ] **Step 4: Implement `DiagnosticAssessment` and evidence mapping**

Use this internal contract:

```rust
#[derive(Clone, Debug)]
pub struct DiagnosticAssessment {
    pub area: Option<DiagnosticArea>,
    pub evidence: Vec<DiagnosticEvidence>,
    pub failed_sources: Vec<EvidenceSource>,
    pub checked_sources: Vec<EvidenceSource>,
}

impl DiagnosticAssessment {
    pub fn is_normal(&self) -> bool { self.area.is_none() }
    pub fn verifies_recovery_for(&self, area: &DiagnosticArea) -> bool {
        let required = match area {
            DiagnosticArea::LocalConnection => vec![EvidenceSource::Ethernet, EvidenceSource::Ipv4, EvidenceSource::DefaultRoute],
            DiagnosticArea::GatewayOrLocal => vec![EvidenceSource::Gateway],
            DiagnosticArea::Dns => vec![EvidenceSource::DnsMicrosoft, EvidenceSource::DnsGoogle],
            DiagnosticArea::External => vec![EvidenceSource::HttpMicrosoft, EvidenceSource::HttpGoogle],
            DiagnosticArea::Unknown => return false,
        };
        self.is_normal() && required.iter().all(|source| self.checked_sources.contains(source))
    }
}
```

Add `pub fn assess(snapshot: NetworkProbeSnapshot) -> DiagnosticAssessment`. Implement the precedence from the spec: inventory API error with missing topology → unknown; valid inventory without wired route → local; gateway failure → gateway/local; both DNS failures plus successful HTTP → unknown; gateway success + DNS success + both HTTP failures → external; one HTTP failure → unknown; no failed evidence → normal. Evidence timestamps use `snapshot.collected_at` and endpoint constants select Microsoft versus Google sources.

- [ ] **Step 5: Implement the pure state machine**

Store only the latest public status, the previous failed-source set, and the last confirmed incident area. Confirmation is true when a concrete current area repeats, or when a prior unknown shares a failed source and the focused result adds an independent failed source with a concrete area. Two unknown assessments never confirm.

```rust
pub struct NetworkDiagnosticStateMachine {
    status: NetworkDiagnosticStatus,
    previous_failed_sources: Vec<EvidenceSource>,
    incident_area: Option<DiagnosticArea>,
}

impl NetworkDiagnosticStateMachine {
    pub fn new() -> Self { Self { status: NetworkDiagnosticStatus::starting(), previous_failed_sources: vec![], incident_area: None } }
    pub fn status(&self) -> NetworkDiagnosticStatus { self.status.clone() }
}
```

Add `pub fn apply(&mut self, assessment: DiagnosticAssessment, observed_at: &str, full: bool) -> NetworkDiagnosticStatus`. Match `(current lifecycle, assessment.is_normal())` against the twelve transitions in the spec. Before moving incident to recovering, require `assessment.verifies_recovery_for(incident_area)`; a normal observation without the required sources keeps the incident unchanged. When `full` is true, update `last_full_probe_at`. Set availability to running after the first apply. Preserve the confirmed incident area through recovering; clear it after normal.

- [ ] **Step 6: Export modules, verify and commit**

Add `pub mod observation; pub mod state_machine;` to `network/mod.rs`.

Run: `cargo test --manifest-path src-tauri/Cargo.toml network::observation::tests && cargo test --manifest-path src-tauri/Cargo.toml network::state_machine::tests`

Expected: all classification and transition tests pass.

```bash
git add src-tauri/src/network/observation.rs src-tauri/src/network/state_machine.rs src-tauri/src/network/mod.rs
git commit -m "feat: classify network diagnostic lifecycle"
```

### Task 4: Build the bounded automatic runtime

**Files:**
- Create: `src-tauri/src/network/runtime.rs`
- Modify: `src-tauri/src/network/mod.rs`

- [ ] **Step 1: Write deterministic schedule tests**

Define tests against a fake wait driver, not wall-clock sleeps:

```rust
#[test]
fn startup_is_full_then_external_targets_alternate_every_two_baselines() {
    let run = scripted_runtime(vec![WaitOutcome::Elapsed; 5]);
    assert_eq!(run.probes, vec![
        ProbeRequest::Full,
        ProbeRequest::Baseline(None),
        ProbeRequest::Baseline(Some(MICROSOFT_URL)),
        ProbeRequest::Baseline(None),
        ProbeRequest::Baseline(Some(GOOGLE_URL)),
        ProbeRequest::Baseline(None),
    ]);
}

#[test]
fn focused_probe_respects_thirty_second_cooldown() {
    let run = scripted_abnormal_runtime_at_seconds(&[0, 10, 20, 30, 40]);
    assert_eq!(run.focused_at_seconds, vec![0, 30]);
}

#[test]
fn stop_interrupts_wait_without_another_probe() {
    let run = scripted_runtime(vec![WaitOutcome::Stopped]);
    assert_eq!(run.probes, vec![ProbeRequest::Full]);
}

#[test]
fn automatic_http_requests_stay_within_hourly_limits() {
    let normal = scheduled_requests_for_one_hour(false);
    let persistent_incident = scheduled_requests_for_one_hour(true);
    assert_eq!(normal.http_request_count(), 180);
    assert!(persistent_incident.http_request_count() <= 420);
}
```

Add a coordinator concurrency test using a fake collector with an atomic active count; run manual full and baseline calls on two threads and assert the maximum active count is one.

- [ ] **Step 2: Verify RED**

Run: `cargo test --manifest-path src-tauri/Cargo.toml network::runtime::tests -- --nocapture`

Expected: compilation fails because `runtime.rs` and schedule types do not exist.

- [ ] **Step 3: Implement schedule constants and driver boundary**

Use feature-local constants only:

```rust
const BASELINE_INTERVAL: Duration = Duration::from_secs(10);
const EXTERNAL_INTERVAL: Duration = Duration::from_secs(20);
const FOCUSED_COOLDOWN: Duration = Duration::from_secs(30);

#[derive(Clone, Debug, PartialEq, Eq)]
enum ProbeRequest { Full, Baseline(Option<&'static str>), Focused }

enum WaitOutcome { Elapsed, Stopped }

trait RuntimeWait: Send {
    fn elapsed(&self) -> Duration;
    fn wait(&mut self, duration: Duration) -> WaitOutcome;
}
```

Production wait uses `Instant` plus `Condvar::wait_timeout`; the cloned stop handle sets a boolean and notifies the condvar. Scripted tests advance a stored duration immediately.

- [ ] **Step 4: Implement runtime ownership and loop**

The runtime owns `Arc<RwLock<NetworkDiagnosticStatus>>`, a stop handle and one `JoinHandle`. Startup runs full immediately. Every elapsed 10 seconds runs baseline; every second baseline includes one alternating endpoint. A suspected abnormal assessment requests focused collection when cooldown permits. A confirmed DNS, external, or unknown incident also requests focused collection after cooldown because baseline cannot prove recovery for those areas. Focused collection uses `collect_full`, but is marked focused when passed to the state machine. No catch-up loop is allowed.

```rust
pub struct NetworkDiagnosticsRuntime {
    latest: Arc<RwLock<NetworkDiagnosticStatus>>,
    stop: StopHandle,
    worker: Option<JoinHandle<()>>,
}

impl NetworkDiagnosticsRuntime {
    #[cfg(target_os = "windows")]
    pub fn platform(coordinator: NetworkProbeCoordinator) -> Self { Self::start(coordinator, SystemWait::new()) }

    #[cfg(not(target_os = "windows"))]
    pub fn platform(_coordinator: NetworkProbeCoordinator) -> Self { Self::unavailable() }

    pub fn status(&self) -> Result<NetworkDiagnosticStatus, String> {
        self.latest.read().map(|status| status.clone()).map_err(|_| "network diagnostic status lock failed".into())
    }

    #[cfg(test)]
    pub fn unavailable_for_test() -> Self { Self::unavailable() }
}

impl Drop for NetworkDiagnosticsRuntime {
    fn drop(&mut self) {
        self.stop.stop();
        if let Some(worker) = self.worker.take() { let _ = worker.join(); }
    }
}
```

If collection or state storage fails, write availability `error` with stage `runtime`, code `runtime_failed`, the returned collection or lock error as its message, and no native code; then stop the worker. Implement `unavailable()` with an unavailable status, a no-op stop handle and `worker: None`; do not create a thread on non-Windows platforms.

- [ ] **Step 5: Export, verify and commit**

Add `pub mod runtime;` to `network/mod.rs`.

Run: `cargo test --manifest-path src-tauri/Cargo.toml network::runtime::tests && cargo test --manifest-path src-tauri/Cargo.toml`

Expected: schedule, serialization and all existing Rust tests pass without real sleeps.

```bash
git add src-tauri/src/network/runtime.rs src-tauri/src/network/mod.rs src-tauri/src/network/service.rs
git commit -m "feat: run bounded network diagnostics automatically"
```

### Task 5: Register runtime state and expose the read command

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add a command-level compile test contract**

Add a small helper in `commands.rs` and test it without a Tauri runtime:

```rust
fn clone_diagnostic_status(runtime: &NetworkDiagnosticsRuntime) -> Result<NetworkDiagnosticStatus, String> {
    runtime.status()
}

#[test]
fn status_command_reads_memory_without_collecting_a_probe() {
    let runtime = NetworkDiagnosticsRuntime::unavailable_for_test();
    let status = clone_diagnostic_status(&runtime).unwrap();
    assert_eq!(status.availability, RuntimeAvailability::Unavailable);
}
```

- [ ] **Step 2: Verify RED**

Run: `cargo test --manifest-path src-tauri/Cargo.toml commands::tests -- --nocapture`

Expected: compilation fails until the helper and test constructor exist.

- [ ] **Step 3: Route commands through managed state**

Change raw collection to clone the coordinator before `spawn_blocking`, and add the read command:

```rust
#[tauri::command]
pub async fn get_network_probe_snapshot(
    coordinator: tauri::State<'_, NetworkProbeCoordinator>,
) -> Result<NetworkProbeSnapshot, String> {
    let coordinator = coordinator.inner().clone();
    tauri::async_runtime::spawn_blocking(move || coordinator.collect_full())
        .await.map_err(|error| format!("network probe task failed: {error}"))?
}

#[tauri::command]
pub fn get_network_diagnostic_status(
    runtime: tauri::State<'_, NetworkDiagnosticsRuntime>,
) -> Result<NetworkDiagnosticStatus, String> {
    clone_diagnostic_status(runtime.inner())
}
```

In `run()`, create coordinator first, start runtime with its clone, manage both, and register both commands:

```rust
let coordinator = NetworkProbeCoordinator::platform();
let runtime = NetworkDiagnosticsRuntime::platform(coordinator.clone());
tauri::Builder::default()
    .manage(coordinator)
    .manage(runtime)
    .invoke_handler(tauri::generate_handler![
        commands::get_network_probe_snapshot,
        commands::get_network_diagnostic_status,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
```

- [ ] **Step 4: Verify and commit**

Run: `cargo test --manifest-path src-tauri/Cargo.toml && cargo check --manifest-path src-tauri/Cargo.toml`

Expected: commands and all Rust tests pass; macOS uses unavailable runtime and starts no worker.

```bash
git add src-tauri/src/commands.rs src-tauri/src/lib.rs src-tauri/src/network/runtime.rs
git commit -m "feat: expose current network diagnostic status"
```

### Task 6: Add the current-status panel

**Files:**
- Create: `src/features/network-diagnostics/types.ts`
- Create: `src/features/network-diagnostics/fixture.ts`
- Create: `src/features/network-diagnostics/api.ts`
- Create: `src/features/network-diagnostics/api.test.ts`
- Create: `src/features/network-diagnostics/NetworkStatusPanel.tsx`
- Create: `src/features/network-diagnostics/NetworkStatusPanel.module.css`
- Create: `src/features/network-diagnostics/NetworkStatusPanel.test.tsx`
- Modify: `src/features/product-home/ProductHome.tsx`
- Modify: `src/features/product-home/ProductHome.test.tsx`

- [ ] **Step 1: Add final fixtures for the status contract**

Use the `types.ts` created in Task 1. Create fixtures for `starting`, `normal`, `incident`, and `unavailable`. The normal fixture must include gateway and both HTTP evidence with distinct `checkedAt` values.

- [ ] **Step 2: Write API and panel tests first**

Mock `invoke` and `isTauri` in API tests:

```ts
it("reads the current in-memory status", async () => {
  invokeMock.mockResolvedValue(normalStatusFixture);
  await expect(getNetworkDiagnosticStatus()).resolves.toBe(normalStatusFixture);
  expect(invokeMock).toHaveBeenCalledWith("get_network_diagnostic_status");
});

it("reports browser mode without invoking Tauri", () => {
  isTauriMock.mockReturnValue(false);
  expect(canUseNetworkDiagnostics()).toBe(false);
  expect(invokeMock).not.toHaveBeenCalled();
});
```

Use fake timers in component tests:

```tsx
it("loads immediately and polls memory status every second", async () => {
  vi.useFakeTimers();
  getStatusMock.mockResolvedValue(normalStatusFixture);
  render(<NetworkStatusPanel />);
  await vi.runOnlyPendingTimersAsync();
  expect(getStatusMock).toHaveBeenCalled();
  vi.useRealTimers();
});

it("shows incident area and evidence without claiming an exact device cause", async () => {
  getStatusMock.mockResolvedValue(incidentStatusFixture);
  render(<NetworkStatusPanel />);
  expect(await screen.findByText("공유기 또는 로컬 연결 구간")).toBeInTheDocument();
  expect(screen.getByText("장애 확인")).toBeInTheDocument();
});

it("does not invoke in browser mode", () => {
  canUseMock.mockReturnValue(false);
  render(<NetworkStatusPanel />);
  expect(screen.getByText("Windows 앱에서 확인 가능")).toBeInTheDocument();
  expect(getStatusMock).not.toHaveBeenCalled();
});
```

Also test starting, suspected, recovering, resolved, command error, evidence timestamps, and interval cleanup on unmount.

- [ ] **Step 3: Verify RED**

Run: `npm test -- src/features/network-diagnostics/NetworkStatusPanel.test.tsx src/features/network-diagnostics/api.test.ts`

Expected: tests fail because the feature files do not exist.

- [ ] **Step 4: Implement API and polling component**

`api.ts` must contain only:

```ts
import { invoke, isTauri } from "@tauri-apps/api/core";
import type { NetworkDiagnosticStatus } from "./types";

export const canUseNetworkDiagnostics = () => isTauri();
export const getNetworkDiagnosticStatus = () =>
  invoke<NetworkDiagnosticStatus>("get_network_diagnostic_status");
```

`NetworkStatusPanel` initializes with `startingStatusFixture`, calls immediately, then every 1000 ms. Keep an `inFlight` closure flag so a slow memory command does not overlap, clear the interval on unmount, and ignore late results after cleanup. A command rejection creates local availability `error` with the message; it does not run the raw probe.

Use fixed label maps:

```ts
const lifecycleLabels = {
  normal: "정상", suspected: "장애 의심", incident: "장애 확인",
  recovering: "복구 확인 중", resolved: "복구됨",
} as const;

const areaLabels = {
  local_connection: "로컬 연결 구간",
  gateway_or_local: "공유기 또는 로컬 연결 구간",
  dns: "DNS",
  external: "외부 연결 구간",
  unknown: "확인 불가",
} as const;
```

Render stable rows for gateway, DNS, Microsoft and Google even when their evidence status is `not_checked`. Use `time` elements for `observedAt` and evidence timestamps. CSS uses neutral blue for normal, amber for suspected/recovering, muted for unavailable, and restrained red only for incident.

- [ ] **Step 5: Integrate with ProductHome**

Import `NetworkStatusPanel` through `@/features/network-diagnostics/NetworkStatusPanel`, render it before `NetworkProbePanel`, and change the network area badge from `다음 구현 영역` to `활성`. Update `ProductHome.test.tsx` to assert `활성` and mock the panel API so no timer escapes the test.

- [ ] **Step 6: Verify and commit**

Run: `npm test && npm run build`

Expected: all frontend tests pass, timers are cleaned, and TypeScript has one authoritative status contract.

```bash
git add src/features/network-diagnostics src/features/product-home/ProductHome.tsx src/features/product-home/ProductHome.test.tsx
git commit -m "feat: display current network diagnostic status"
```

### Task 7: Full QA and Windows handoff

**Files:**
- Modify only if verification exposes a scoped defect.

- [ ] **Step 1: Run repository verification**

```bash
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
git diff --check
```

Expected: 0 frontend failures, 0 Rust failures, successful build/check, and no Clippy or whitespace errors.

- [ ] **Step 2: Review scope against the spec**

Run:

```bash
rg -n "sqlite|rusqlite|notification|toast|packet capture|pcap|latency threshold|retry" src src-tauri
git diff --stat dev...HEAD
git status --short
```

Expected: no SQLite, notification, packet capture, latency classification or retry implementation; only runtime/current-status files and the preserved raw panel are changed; worktree is clean.

- [ ] **Step 3: Push and run Windows Build**

```bash
git push -u origin codex/network-diagnostics-runtime-foundation
gh workflow run windows-build.yml --ref codex/network-diagnostics-runtime-foundation
gh run watch "$(gh run list --workflow windows-build.yml --branch codex/network-diagnostics-runtime-foundation --limit 1 --json databaseId --jq '.[0].databaseId')" --exit-status
```

Expected: frontend tests, Windows Rust tests, installer packaging and `pc-health-windows-installers` upload all succeed.

- [ ] **Step 4: Perform actual Windows validation**

Install the artifact and record these observations:

1. Launch without administrator rights; status begins automatically.
2. Keep wired LAN connected for 10 minutes; status remains normal and evidence timestamps advance.
3. Confirm Microsoft and Google `checkedAt` values alternate rather than update together on baseline.
4. Run the raw JSON probe once; UI remains responsive and automatic evidence does not burst afterward.
5. Disconnect LAN; within about 30 seconds status becomes suspected or incident without naming a specific cable, port or NIC.
6. Reconnect LAN; observe recovering, resolved and normal across successive observations.
7. Exit the app; process terminates without a hanging worker.

- [ ] **Step 5: Fix only evidence-backed Windows defects**

For any defect, first add the narrowest failing Rust or frontend regression test, verify RED, implement one correction, and rerun Steps 1 and 3. Do not add persistence, history, latency thresholds or repair actions in this branch.

- [ ] **Step 6: Final handoff**

Report the exact Windows run URL, artifact name, test counts, actual transition timings, and any remaining unknown evidence. Keep the branch for review until the user chooses merge or PR.
