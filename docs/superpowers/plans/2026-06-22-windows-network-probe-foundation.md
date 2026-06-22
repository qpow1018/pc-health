# Windows Network Probe Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Windows에서 유선 adapter, IPv4 default route, gateway ICMP, system name resolution과 두 connectivity endpoint를 한 번 측정하고 앱에서 raw JSON을 복사한다.

**Architecture:** Rust의 `NetworkCollector`가 동기 probe를 수행하고 `NetworkProbeService`가 부분 실패를 하나의 `NetworkProbeSnapshot`으로 조립한다. Tauri command는 이 동기 작업을 blocking task로 실행하며 frontend panel은 수동 실행, JSON 표시와 clipboard 복사만 담당한다.

**Tech Stack:** Tauri 2, Rust 2021, `windows` 0.61, `reqwest` 0.13 blocking client, Serde, Chrono, React 19, TypeScript, Vitest, Testing Library

---

## File map

- `src-tauri/src/network/domain.rs`: JSON wire contract와 pure default-route 선택 규칙.
- `src-tauri/src/network/collector.rs`: collector trait와 platform module routing.
- `src-tauri/src/network/unsupported.rs`: non-Windows `unsupported` raw 결과.
- `src-tauri/src/network/windows.rs`: Windows adapter, route, ICMP, DNS와 HTTP probe.
- `src-tauri/src/network/service.rs`: 단계 결과를 snapshot으로 조립.
- `src-tauri/src/network/mod.rs`: network module exports.
- `src-tauri/src/commands.rs`: Tauri blocking command.
- `src-tauri/src/lib.rs`: command registration.
- `src/features/network-probe/types.ts`: Rust contract의 frontend mirror.
- `src/features/network-probe/api.ts`: Tauri invoke wrapper.
- `src/features/network-probe/NetworkProbePanel.tsx`: 일회성 실행과 raw JSON 복사 UI.
- `src/features/network-probe/NetworkProbePanel.module.css`: panel layout.
- `src/features/network-probe/NetworkProbePanel.test.tsx`: UI state와 clipboard tests.
- `src/features/product-home/ProductHome.tsx`: probe panel 배치.

### Task 1: Wire contract와 default-route 선택

**Files:**
- Create: `src-tauri/src/network/domain.rs`
- Create: `src-tauri/src/network/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: failing serialization과 route-selection tests 작성**

`domain.rs`에 tests부터 작성한다. `ProbeStatus`가 snake_case로 직렬화되고 Ethernet/up 후보가 metric보다 우선하며 같은 우선순위에서는 `combined_metric`이 가장 낮은 route를 선택해야 한다.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn route(index: u32, ethernet: bool, up: bool, metric: u32) -> RouteSnapshot {
        RouteSnapshot {
            interface_index: index,
            gateway: format!("192.168.0.{index}"),
            route_metric: metric,
            interface_metric: 0,
            combined_metric: metric,
            adapter_is_ethernet: ethernet,
            adapter_is_up: up,
        }
    }

    #[test]
    fn serializes_probe_status_as_snake_case() {
        assert_eq!(serde_json::to_string(&ProbeStatus::NotRun).unwrap(), "\"not_run\"");
    }

    #[test]
    fn selects_up_ethernet_before_lower_metric_non_ethernet() {
        let routes = vec![route(1, false, true, 1), route(2, true, true, 20)];
        assert_eq!(select_default_route(&routes).unwrap().interface_index, 2);
    }

    #[test]
    fn selects_lowest_metric_within_same_adapter_priority() {
        let routes = vec![route(1, true, true, 30), route(2, true, true, 10)];
        assert_eq!(select_default_route(&routes).unwrap().interface_index, 2);
    }

    #[test]
    fn returns_none_without_candidates() {
        assert!(select_default_route(&[]).is_none());
    }
}
```

- [ ] **Step 2: RED 확인**

Run: `cargo test --manifest-path src-tauri/Cargo.toml network::domain::tests`

Expected: FAIL because the network module and types do not exist.

- [ ] **Step 3: dependencies와 최소 contract 구현**

`src-tauri/Cargo.toml`에 다음 direct dependencies를 추가한다.

```toml
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[target.'cfg(windows)'.dependencies]
reqwest = { version = "0.13", features = ["blocking"] }
windows = { version = "0.61", features = [
  "Win32_Foundation",
  "Win32_NetworkManagement_IpHelper",
  "Win32_NetworkManagement_Ndis",
  "Win32_Networking_WinSock",
  "Win32_System_IO",
] }
```

`domain.rs`에 다음 public contract를 구현한다. 모든 field는 `#[serde(rename_all = "camelCase")]`, enum은 `#[serde(rename_all = "snake_case")]`를 사용한다.

```rust
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProbeStatus { Success, Timeout, NotRun, Unsupported, Error }

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeError {
    pub stage: String,
    pub code: String,
    pub message: String,
    pub native_code: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdapterSnapshot {
    pub name: String,
    pub friendly_name: String,
    pub interface_index: u32,
    pub if_type: u32,
    pub operational_status: String,
    pub mac_address: Option<String>,
    pub ipv4_addresses: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteSnapshot {
    pub interface_index: u32,
    pub gateway: String,
    pub route_metric: u32,
    pub interface_metric: u32,
    pub combined_metric: u32,
    pub adapter_is_ethernet: bool,
    pub adapter_is_up: bool,
}
```

나머지 check와 snapshot contract는 다음 field로 정의한다.

```rust
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GatewayCheck {
    pub status: ProbeStatus,
    pub duration_ms: u64,
    pub reply_address: Option<String>,
    pub round_trip_ms: Option<u32>,
    pub error: Option<ProbeError>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DnsCheck {
    pub hostname: String,
    pub status: ProbeStatus,
    pub duration_ms: u64,
    pub addresses: Vec<String>,
    pub error: Option<ProbeError>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpCheck {
    pub url: String,
    pub status: ProbeStatus,
    pub duration_ms: u64,
    pub status_code: Option<u16>,
    pub body_matches: Option<bool>,
    pub error: Option<ProbeError>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkProbeSnapshot {
    pub collected_at: String,
    pub collector: String,
    pub duration_ms: u64,
    pub adapters: Vec<AdapterSnapshot>,
    pub default_routes: Vec<RouteSnapshot>,
    pub selected_route: Option<RouteSnapshot>,
    pub gateway_check: GatewayCheck,
    pub dns_checks: Vec<DnsCheck>,
    pub http_checks: Vec<HttpCheck>,
    pub errors: Vec<ProbeError>,
}
```

`select_default_route`는 `min_by_key`에 `(!(up && ethernet), !up, combined_metric, interface_index)` tuple을 사용해 결정적인 clone을 반환한다.

- [ ] **Step 4: GREEN 확인**

Run: `cargo test --manifest-path src-tauri/Cargo.toml network::domain::tests`

Expected: 4 passed, 0 failed.

- [ ] **Step 5: commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/lib.rs src-tauri/src/network
git commit -m "feat: define network probe contract"
```

### Task 2: Service와 non-Windows unsupported collector

**Files:**
- Create: `src-tauri/src/network/collector.rs`
- Create: `src-tauri/src/network/unsupported.rs`
- Create: `src-tauri/src/network/service.rs`
- Modify: `src-tauri/src/network/mod.rs`

- [ ] **Step 1: failing service test 작성**

collector가 adapter 단계 오류와 `unsupported` checks를 반환해도 service가 `NetworkProbeSnapshot`을 반환하고 top-level `errors`를 보존하는 test를 작성한다. Fake collector는 각 method 호출 횟수를 기록해 inventory가 route 선택보다 먼저 실행되는 것도 검증한다.

```rust
#[test]
fn preserves_partial_results_in_snapshot() {
    let service = NetworkProbeService::new(FakeCollector::partial_failure());
    let snapshot = service.collect();
    assert_eq!(snapshot.collector, "fake");
    assert_eq!(snapshot.errors[0].stage, "adapters");
    assert_eq!(snapshot.gateway_check.status, ProbeStatus::NotRun);
}
```

- [ ] **Step 2: RED 확인**

Run: `cargo test --manifest-path src-tauri/Cargo.toml network::service::tests`

Expected: FAIL because `NetworkProbeService` and `NetworkCollector` do not exist.

- [ ] **Step 3: collector contract와 service 구현**

service가 inventory를 받은 뒤 route를 선택하고 gateway check에 전달할 수 있도록 단계별 trait를 사용한다.

```rust
pub trait NetworkCollector: Send + Sync {
    fn collector_name(&self) -> &'static str;
    fn collect_inventory(&self) -> NetworkInventory;
    fn check_gateway(&self, route: Option<&RouteSnapshot>) -> GatewayCheck;
    fn check_dns(&self) -> Vec<DnsCheck>;
    fn check_http(&self) -> Vec<HttpCheck>;
}

pub struct NetworkInventory {
    pub adapters: Vec<AdapterSnapshot>,
    pub default_routes: Vec<RouteSnapshot>,
    pub errors: Vec<ProbeError>,
}
```

`NetworkProbeService`는 `Box<dyn NetworkCollector>`를 소유한다. `collect()`은 `Instant`로 전체 duration을 재고 `Utc::now().to_rfc3339()`를 기록하며 다음 순서를 지킨다.

1. `collect_inventory()`
2. `select_default_route(&inventory.default_routes)`
3. `check_gateway(selected_route.as_ref())`
4. `check_dns()`
5. `check_http()`
6. 모든 결과를 `NetworkProbeSnapshot`으로 조립

`UnsupportedCollector`는 빈 inventory, `unsupported` gateway check, 두 hostname의 `unsupported` DNS checks, 두 URL의 `unsupported` HTTP checks를 반환한다. 가짜 Windows 데이터는 만들지 않는다.

- [ ] **Step 4: GREEN과 전체 Rust test 확인**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`

Expected: all tests pass.

- [ ] **Step 5: commit**

```bash
git add src-tauri/src/network
git commit -m "feat: assemble partial network probe snapshots"
```

### Task 3: Tauri command와 browser-safe frontend API

**Files:**
- Create: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `package.json`
- Modify: `package-lock.json`
- Create: `src/features/network-probe/types.ts`
- Create: `src/features/network-probe/api.ts`
- Create: `src/features/network-probe/api.test.ts`

- [ ] **Step 1: failing frontend API tests 작성**

`invoke`가 `get_network_probe_snapshot`으로 호출되고 browser 환경에서는 명시적 error를 반환하는 tests를 작성한다. `vi.mock("@tauri-apps/api/core")`로 invoke를 제어한다.

```ts
it("invokes the one-shot network command", async () => {
  invokeMock.mockResolvedValue(snapshotFixture);
  await getNetworkProbeSnapshot();
  expect(invokeMock).toHaveBeenCalledWith("get_network_probe_snapshot");
});
```

- [ ] **Step 2: RED 확인**

Run: `npm test -- src/features/network-probe/api.test.ts`

Expected: FAIL because the module does not exist.

- [ ] **Step 3: command와 API 구현**

`@tauri-apps/api`를 dependency로 추가한다. TypeScript에는 Rust contract와 같은 camelCase field를 정의한다.

Rust command는 platform collector를 선택하고 UI thread 밖에서 실행한다.

```rust
#[tauri::command]
pub async fn get_network_probe_snapshot() -> Result<NetworkProbeSnapshot, String> {
    tauri::async_runtime::spawn_blocking(|| NetworkProbeService::platform().collect())
        .await
        .map_err(|error| format!("network probe task failed: {error}"))
}
```

`lib.rs`는 `mod commands; mod network;`와 `invoke_handler(tauri::generate_handler![commands::get_network_probe_snapshot])`를 등록한다.

Frontend API는 `invoke<NetworkProbeSnapshot>("get_network_probe_snapshot")`만 감싼다. browser fixture를 production API에서 자동 반환하지 않는다.

- [ ] **Step 4: GREEN 확인**

Run: `npm test -- src/features/network-probe/api.test.ts && cargo test --manifest-path src-tauri/Cargo.toml`

Expected: frontend API tests and Rust tests pass.

- [ ] **Step 5: commit**

```bash
git add package.json package-lock.json src-tauri/src src/features/network-probe
git commit -m "feat: expose one-shot network probe command"
```

### Task 4: Windows adapter와 default-route inventory

**Files:**
- Create: `src-tauri/src/network/windows.rs`
- Modify: `src-tauri/src/network/collector.rs`
- Modify: `src-tauri/src/network/service.rs`

- [ ] **Step 1: Windows raw mapping의 failing pure tests 작성**

FFI와 분리된 `RawAdapter`와 `RawRoute`를 domain 값으로 연결하는 tests를 작성한다. operational Ethernet interface가 `adapter_is_ethernet = true`, `adapter_is_up = true`로 route에 결합되고 interface metric이 combined metric에 포함되어야 한다.

```rust
#[test]
fn joins_route_with_adapter_and_interface_metric() {
    let mapped = map_route(raw_route(7, 5), &[raw_ethernet_adapter(7)], 20);
    assert_eq!(mapped.combined_metric, 25);
    assert!(mapped.adapter_is_ethernet);
    assert!(mapped.adapter_is_up);
}
```

- [ ] **Step 2: RED 확인**

Run on Windows CI: `cargo test --manifest-path src-tauri/Cargo.toml network::windows::tests`

Expected: FAIL because `windows.rs` does not exist.

- [ ] **Step 3: Windows inventory 구현**

`GetAdaptersAddresses(AF_INET, ...)`는 15 KiB initial buffer를 사용하고 `ERROR_BUFFER_OVERFLOW`일 때 반환된 크기로 한 번 재할당한다. linked list를 순회해 Ethernet `IF_TYPE_ETHERNET_CSMACD`, interface index, `OperStatus`, physical address와 IPv4 unicast addresses를 구조화한다.

`GetIpForwardTable2(AF_INET, ...)` 결과는 RAII wrapper의 `Drop`에서 `FreeMibTable`로 해제한다. prefix length가 0이고 next hop이 `0.0.0.0`이 아닌 row만 보존한다. 각 row의 `InterfaceLuid`와 `InterfaceIndex`로 `GetIpInterfaceEntry`를 호출해 interface metric을 읽는다. overflow는 `saturating_add`로 combined metric을 계산한다.

unsafe block마다 다음 invariant를 바로 주석으로 남긴다.

- buffer lifetime 동안 linked pointer가 유효함
- `NumEntries` 범위만 table slice로 읽음
- Windows가 할당한 table을 정확히 한 번 해제함
- sockaddr family를 확인한 뒤 IPv4 bytes를 변환함

API 실패는 `ProbeError { stage: "adapters" | "routes", code, message, native_code }`로 보존하고 다른 inventory를 계속 시도한다.

- [ ] **Step 4: Windows compile과 tests 확인**

Run: `cargo check --manifest-path src-tauri/Cargo.toml --target x86_64-pc-windows-msvc`

Run in Windows workflow: `cargo test --manifest-path src-tauri/Cargo.toml`

Expected: compile succeeds; mapping tests pass.

- [ ] **Step 5: commit**

```bash
git add src-tauri/src/network
git commit -m "feat: collect Windows network inventory"
```

### Task 5: Gateway, system resolver와 bounded HTTP checks

**Files:**
- Modify: `src-tauri/src/network/windows.rs`

- [ ] **Step 1: failing result-mapping tests 작성**

ICMP no-reply, DNS timeout, Microsoft body mismatch, Google 204 success를 각각 `ProbeStatus`와 typed field로 mapping하는 pure tests를 작성한다.

```rust
#[test]
fn google_204_is_success() {
    let check = map_http_response(GOOGLE_URL, 204, &[], 12);
    assert_eq!(check.status, ProbeStatus::Success);
    assert_eq!(check.status_code, Some(204));
    assert_eq!(check.body_matches, Some(true));
}
```

- [ ] **Step 2: RED 확인**

Run in Windows workflow: `cargo test --manifest-path src-tauri/Cargo.toml network::windows::tests`

Expected: FAIL because check functions do not exist.

- [ ] **Step 3: gateway ICMP 구현**

선택 route가 있을 때 `IcmpCreateFile`, `IcmpSendEcho2`, `IcmpCloseHandle`을 RAII handle로 감싼다. request payload는 고정된 작은 byte slice를 사용하고 reply buffer는 `ICMP_ECHO_REPLY + payload + 8` 이상으로 할당한다. provisional timeout은 1500 ms다. reply 0이면 `GetLastError`를 numeric code와 함께 mapping하며 timeout과 generic error를 구분한다.

- [ ] **Step 4: system name resolution 구현**

각 hostname에 `GetAddrInfoExW`를 사용하고 `TIMEVAL { tv_sec: 3, tv_usec: 0 }` timeout을 전달한다. 반환 linked list는 `FreeAddrInfoExW`로 정확히 한 번 해제하고 AF_INET·AF_INET6 주소를 문자열로 변환해 정렬·중복 제거한다. 두 hostname은 각각 한 번만 조회한다.

- [ ] **Step 5: HTTP checks 구현**

blocking `reqwest::Client`를 다음 설정으로 한 번 생성해 두 요청에 재사용한다.

```rust
let client = reqwest::blocking::Client::builder()
    .redirect(reqwest::redirect::Policy::none())
    .connect_timeout(Duration::from_secs(3))
    .timeout(Duration::from_secs(5))
    .build()?;
```

각 response는 `Read::take(4097)`로 읽고 4096 bytes를 넘으면 `body_too_large` error로 기록한다. Microsoft check는 status 200과 body `Microsoft Connect Test`, Google check는 status 204와 empty body를 기대한다. request retry는 하지 않는다.

- [ ] **Step 6: GREEN과 Windows compile 확인**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`

Run: `cargo check --manifest-path src-tauri/Cargo.toml --target x86_64-pc-windows-msvc`

Expected: all native-independent tests pass and Windows target compiles.

- [ ] **Step 7: commit**

```bash
git add src-tauri/src/network/windows.rs src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "feat: add bounded Windows connectivity checks"
```

### Task 6: Raw JSON panel과 clipboard 복사

**Files:**
- Create: `src/features/network-probe/NetworkProbePanel.tsx`
- Create: `src/features/network-probe/NetworkProbePanel.module.css`
- Create: `src/features/network-probe/NetworkProbePanel.test.tsx`
- Create: `src/features/network-probe/fixture.ts`
- Modify: `src/features/product-home/ProductHome.tsx`
- Modify: `src/features/product-home/ProductHome.module.css`
- Modify: `src/features/product-home/ProductHome.test.tsx`

- [ ] **Step 1: failing component tests 작성**

API module과 `navigator.clipboard.writeText`를 mock해 다음을 검증한다.

- 실행 버튼이 한 번 command를 호출함
- pending 동안 버튼이 disabled임
- 완료 후 formatted JSON이 `<pre>`에 표시됨
- copy 버튼이 표시된 JSON과 같은 문자열을 clipboard에 씀
- command rejection은 오류 문구를 표시함

```tsx
it("prevents duplicate runs while the probe is pending", async () => {
  getSnapshotMock.mockReturnValue(new Promise(() => {}));
  render(<NetworkProbePanel />);
  await userEvent.click(screen.getByRole("button", { name: "네트워크 확인 실행" }));
  expect(screen.getByRole("button", { name: "확인 중" })).toBeDisabled();
  expect(getSnapshotMock).toHaveBeenCalledTimes(1);
});
```

- [ ] **Step 2: RED 확인**

Run: `npm test -- src/features/network-probe/NetworkProbePanel.test.tsx`

Expected: FAIL because the component does not exist.

- [ ] **Step 3: 최소 panel 구현**

local state는 `idle | loading | success | error`만 사용한다. 실행 성공 시 `JSON.stringify(snapshot, null, 2)`를 저장한다. 상태 판정 badge를 만들지 않는다. clipboard 실패는 probe error state를 덮지 않고 별도 copy message로 표시한다.

ProductHome의 인터넷 진단 card 다음에 panel을 배치한다. 스타일은 `docs/ui-guidelines.md`와 기존 CSS Module 규칙을 따르며 raw JSON 영역은 monospace, scroll, selectable text를 제공한다.

- [ ] **Step 4: GREEN과 frontend build 확인**

Run: `npm test && npm run build`

Expected: all tests pass and Vite production build succeeds.

- [ ] **Step 5: commit**

```bash
git add src/features/network-probe src/features/product-home
git commit -m "feat: display copyable network probe JSON"
```

### Task 7: 전체 QA와 Windows artifact handoff

**Files:**
- Modify only if verification exposes a defect.

- [ ] **Step 1: forbidden-scope scan**

Run:

```bash
rg -n 'rusqlite|CREATE TABLE|setInterval|packet capture|driver install|router login' src src-tauri
```

Expected: no matches from implementation source.

- [ ] **Step 2: complete local verification**

Run:

```bash
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml --target x86_64-pc-windows-msvc
git diff --check
```

Expected: every command exits 0.

- [ ] **Step 3: QA safety review**

Use `qa-safety-reviewer` and verify:

- no polling, classification or SQLite was added;
- no shell command output is parsed;
- all network requests have target, timeout, body limit and zero retry;
- `success` is probe-local and not a global health claim;
- partial failures remain visible;
- no elevation or network mutation exists.

- [ ] **Step 4: Windows artifact 실행**

Push the feature branch only after user approval, run `Windows Build`, install the artifact without elevation, execute the probe once, and copy the raw JSON. Validate adapter/route association, ICMP result, DNS addresses, HTTP status/body match and bounded completion.

- [ ] **Step 5: Windows evidence follow-up**

If the raw JSON exposes mapping or API defects, add a failing regression test, implement the narrow fix, rerun Task 7 Step 2, and commit with `fix: correct Windows network probe mapping`. Do not add polling, classification or persistence in this branch.
