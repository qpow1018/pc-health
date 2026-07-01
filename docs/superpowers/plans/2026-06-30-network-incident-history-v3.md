# Network Incident History v3 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a separate incident history screen that lists all stored network incidents with period, status, and area filters.

**Architecture:** Extend the existing v2 incident store and Tauri history state with a full-list command, then add a feature-local React history page that filters in the frontend. Use `HomePage` view state for `home | incident_history`; do not add URL routing, detail navigation, raw evidence views, charts, export, deletion, or settings.

**Tech Stack:** Rust 2021, Tauri 2 commands, rusqlite, React 19, TypeScript, Vitest, Testing Library, CSS Modules.

---

## File Structure

- Modify: `src-tauri/src/network/incident_store.rs`
  - Add `all_incidents()` returning all incidents latest-first while preserving `recent_incidents(3)`.
- Modify: `src-tauri/src/commands.rs`
  - Add `all_network_incidents(...)` helper and `get_network_incidents` Tauri command with the same unavailable-state policy as `get_recent_network_incidents`.
- Modify: `src-tauri/src/lib.rs`
  - Register `get_network_incidents` in the invoke handler.
- Modify: `src/features/network-incidents/api.ts`
  - Add `getNetworkIncidents()` invoking `get_network_incidents`.
- Modify: `src/features/network-incidents/api.test.ts`
  - Verify the new command name.
- Create: `src/features/network-incidents/filter.ts`
  - Define filter types/options and pure `filterNetworkIncidents` / duration helpers.
- Create: `src/features/network-incidents/filter.test.ts`
  - Verify period, status, area, and duration behavior.
- Create: `src/features/network-incidents/NetworkIncidentHistoryPage.tsx`
  - Fetch full incident list, render filters and dense list/table, and expose `돌아가기`.
- Create: `src/features/network-incidents/NetworkIncidentHistoryPage.module.css`
  - Quiet desktop utility styling for the page, filters, and stable rows.
- Create: `src/features/network-incidents/NetworkIncidentHistoryPage.test.tsx`
  - Verify loading, empty, filtered empty, error, filtering, back action, and no row link/click affordance.
- Modify: `src/features/network-incidents/RecentIncidentsPanel.tsx`
  - Accept optional `onOpenHistory` callback and show `전체 이력 보기` action only when provided.
- Modify: `src/features/network-incidents/RecentIncidentsPanel.test.tsx`
  - Verify the optional action is hidden by default and calls the callback when present.
- Modify: `src/features/network-incidents/RecentIncidentsPanel.module.css`
  - Add header/action layout styles without changing existing panel behavior.
- Modify: `src/pages/HomePage.tsx`
  - Own `home | incident_history` view state and switch between `ProductHome` and `NetworkIncidentHistoryPage`.
- Modify: `src/features/product-home/ProductHome.tsx`
  - Accept `onOpenIncidentHistory` and pass it to `RecentIncidentsPanel`.
- Modify: `src/features/product-home/ProductHome.test.tsx`
  - Verify the callback is connected.

---

### Task 1: Backend Full Incident List Command

**Files:**
- Modify: `src-tauri/src/network/incident_store.rs`
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add failing store and command tests**

In `src-tauri/src/network/incident_store.rs`, add this test inside the existing `tests` module:

```rust
    #[test]
    fn reads_all_incidents_latest_first() {
        let store = NetworkIncidentStore::open_in_memory().unwrap();
        for index in 0..5 {
            store
                .create_incident(
                    DiagnosticArea::GatewayOrLocal,
                    &format!("2026-06-30T00:00:0{index}Z"),
                    "공유기 또는 로컬 연결 구간에서 이상 근거가 반복 확인되었습니다.",
                    &[evidence(EvidenceSource::Gateway, EvidenceStatus::Timeout)],
                )
                .unwrap();
        }

        let incidents = store.all_incidents().unwrap();

        assert_eq!(incidents.len(), 5);
        assert_eq!(incidents[0].started_at, "2026-06-30T00:00:04Z");
        assert_eq!(incidents[4].started_at, "2026-06-30T00:00:00Z");
    }
```

In `src-tauri/src/commands.rs`, add this test inside `mod tests`:

```rust
    #[test]
    fn network_incidents_command_returns_all_incidents() {
        let store = NetworkIncidentStore::open_in_memory().unwrap();
        for index in 0..4 {
            store
                .create_incident(
                    DiagnosticArea::External,
                    &format!("2026-06-30T00:00:0{index}Z"),
                    "외부 연결 구간에서 이상 근거가 반복 확인되었습니다.",
                    &[DiagnosticEvidence {
                        source: EvidenceSource::HttpGoogle,
                        status: EvidenceStatus::Timeout,
                        checked_at: Some("2026-06-30T00:00:00Z".into()),
                        duration_ms: Some(10),
                        detail: Some("test".into()),
                    }],
                )
                .unwrap();
        }

        let state = NetworkIncidentHistoryState::new(Some(store));

        assert_eq!(all_network_incidents(&state).unwrap().len(), 4);
    }

    #[test]
    fn network_incidents_command_returns_unavailable_when_store_is_absent() {
        let state = NetworkIncidentHistoryState::new(None);

        assert_eq!(
            all_network_incidents(&state).unwrap_err(),
            "network incident history unavailable"
        );
    }
```

- [ ] **Step 2: Run backend focused tests and verify failure**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml incident_store::tests::reads_all_incidents_latest_first
cargo test --manifest-path src-tauri/Cargo.toml commands::tests::network_incidents_command
```

Expected: FAIL because `all_incidents` and `all_network_incidents` do not exist.

- [ ] **Step 3: Implement store full-list method**

In `src-tauri/src/network/incident_store.rs`, add this method inside `impl NetworkIncidentStore` after `recent_incidents`:

```rust
    pub fn all_incidents(&self) -> Result<Vec<NetworkIncident>, String> {
        self.query_incidents(None)
    }
```

Refactor `recent_incidents` to delegate to a private helper:

```rust
    pub fn recent_incidents(&self, limit: usize) -> Result<Vec<NetworkIncident>, String> {
        let limit = i64::try_from(limit).map_err(|_| "incident limit is too large".to_string())?;
        self.query_incidents(Some(limit))
    }

    fn query_incidents(&self, limit: Option<i64>) -> Result<Vec<NetworkIncident>, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "incident store lock failed".to_string())?;
        let sql = match limit {
            Some(_) => {
                "SELECT id, status, area, started_at, last_observed_at, resolved_at, summary
                 FROM network_incidents
                 ORDER BY started_at DESC, id DESC
                 LIMIT ?1"
            }
            None => {
                "SELECT id, status, area, started_at, last_observed_at, resolved_at, summary
                 FROM network_incidents
                 ORDER BY started_at DESC, id DESC"
            }
        };
        let mut statement = connection.prepare(sql).map_err(|error| error.to_string())?;
        let rows = match limit {
            Some(limit) => statement
                .query_map(params![limit], incident_row)
                .map_err(|error| error.to_string())?,
            None => statement
                .query_map([], incident_row)
                .map_err(|error| error.to_string())?,
        };

        let mut incidents = Vec::new();
        for row in rows {
            let (id, status, area, started_at, last_observed_at, resolved_at, summary) =
                row.map_err(|error| error.to_string())?;
            incidents.push(NetworkIncident {
                id,
                status: NetworkIncidentStatus::from_str(&status)?,
                area: DiagnosticArea::from_str(&area)?,
                started_at,
                last_observed_at,
                resolved_at,
                summary,
                representative_evidence: incident_evidence(&connection, id)?,
            });
        }
        Ok(incidents)
    }
```

Add this helper near the other private functions:

```rust
fn incident_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<(
    i64,
    String,
    String,
    String,
    String,
    Option<String>,
    String,
)> {
    Ok((
        row.get::<_, i64>(0)?,
        row.get::<_, String>(1)?,
        row.get::<_, String>(2)?,
        row.get::<_, String>(3)?,
        row.get::<_, String>(4)?,
        row.get::<_, Option<String>>(5)?,
        row.get::<_, String>(6)?,
    ))
}
```

- [ ] **Step 4: Implement command and register it**

In `src-tauri/src/commands.rs`, add:

```rust
fn all_network_incidents(
    state: &NetworkIncidentHistoryState,
) -> Result<Vec<NetworkIncident>, String> {
    let Some(store) = &state.store else {
        return Err("network incident history unavailable".into());
    };
    store.all_incidents()
}

#[tauri::command]
pub fn get_network_incidents(
    state: tauri::State<'_, NetworkIncidentHistoryState>,
) -> Result<Vec<NetworkIncident>, String> {
    all_network_incidents(state.inner())
}
```

In `src-tauri/src/lib.rs`, add `commands::get_network_incidents` to `tauri::generate_handler!`.

- [ ] **Step 5: Verify backend tests**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml incident_store::tests::reads_all_incidents_latest_first
cargo test --manifest-path src-tauri/Cargo.toml commands::tests::network_incidents_command
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: PASS.

- [ ] **Step 6: Commit Task 1**

```bash
git add src-tauri/src/network/incident_store.rs src-tauri/src/commands.rs src-tauri/src/lib.rs
git commit -m "feat: expose network incident history list"
```

---

### Task 2: Frontend History Filtering And Page

**Files:**
- Modify: `src/features/network-incidents/api.ts`
- Modify: `src/features/network-incidents/api.test.ts`
- Create: `src/features/network-incidents/filter.ts`
- Create: `src/features/network-incidents/filter.test.ts`
- Create: `src/features/network-incidents/NetworkIncidentHistoryPage.tsx`
- Create: `src/features/network-incidents/NetworkIncidentHistoryPage.module.css`
- Create: `src/features/network-incidents/NetworkIncidentHistoryPage.test.tsx`

- [ ] **Step 1: Add API failing test**

In `src/features/network-incidents/api.test.ts`, add:

```ts
  it("reads full incident history from Tauri", async () => {
    invokeMock.mockResolvedValue([resolvedIncidentFixture]);

    await expect(getNetworkIncidents()).resolves.toEqual([
      resolvedIncidentFixture,
    ]);
    expect(invokeMock).toHaveBeenCalledWith("get_network_incidents");
  });
```

Update the import to include `getNetworkIncidents`.

- [ ] **Step 2: Implement API wrapper**

In `src/features/network-incidents/api.ts`, add:

```ts
export const getNetworkIncidents = () =>
  invoke<NetworkIncident[]>("get_network_incidents");
```

- [ ] **Step 3: Add filter tests**

Create `src/features/network-incidents/filter.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import {
  ongoingIncidentFixture,
  recoveringIncidentFixture,
  resolvedIncidentFixture,
  unknownIncidentFixture,
} from "./fixture";
import {
  filterNetworkIncidents,
  formatIncidentDuration,
  type IncidentHistoryFilters,
} from "./filter";

const baseFilters: IncidentHistoryFilters = {
  period: "all",
  status: "all",
  area: "all",
};

describe("network incident history filters", () => {
  it("keeps all incidents when every filter is all", () => {
    expect(
      filterNetworkIncidents(
        [ongoingIncidentFixture, recoveringIncidentFixture, resolvedIncidentFixture],
        baseFilters,
        new Date("2026-06-30T12:00:00Z"),
      ),
    ).toHaveLength(3);
  });

  it("filters by status and area", () => {
    const incidents = [
      ongoingIncidentFixture,
      recoveringIncidentFixture,
      resolvedIncidentFixture,
      unknownIncidentFixture,
    ];

    expect(
      filterNetworkIncidents(
        incidents,
        { ...baseFilters, status: "resolved" },
        new Date("2026-06-30T12:00:00Z"),
      ),
    ).toEqual([resolvedIncidentFixture]);
    expect(
      filterNetworkIncidents(
        incidents,
        { ...baseFilters, area: "unknown" },
        new Date("2026-06-30T12:00:00Z"),
      ),
    ).toEqual([unknownIncidentFixture]);
  });

  it("filters period by startedAt", () => {
    const recent = {
      ...ongoingIncidentFixture,
      startedAt: "2026-06-30T11:30:00Z",
      lastObservedAt: "2026-06-30T11:40:00Z",
    };
    const old = {
      ...resolvedIncidentFixture,
      startedAt: "2026-06-28T11:30:00Z",
      lastObservedAt: "2026-06-30T11:40:00Z",
      resolvedAt: "2026-06-30T11:40:00Z",
    };

    expect(
      filterNetworkIncidents(
        [recent, old],
        { ...baseFilters, period: "24h" },
        new Date("2026-06-30T12:00:00Z"),
      ),
    ).toEqual([recent]);
  });

  it("formats duration from resolvedAt or lastObservedAt", () => {
    expect(
      formatIncidentDuration({
        ...resolvedIncidentFixture,
        startedAt: "2026-06-30T00:00:00Z",
        resolvedAt: "2026-06-30T01:05:00Z",
      }),
    ).toBe("1시간 5분");
    expect(
      formatIncidentDuration({
        ...ongoingIncidentFixture,
        startedAt: "2026-06-30T00:00:00Z",
        lastObservedAt: "2026-06-30T00:05:00Z",
      }),
    ).toBe("5분");
  });
});
```

- [ ] **Step 4: Implement filter helpers**

Create `src/features/network-incidents/filter.ts`:

```ts
import type { DiagnosticArea } from "@/features/network-diagnostics/types";
import type { NetworkIncident, NetworkIncidentStatus } from "./types";

export type IncidentHistoryPeriod = "all" | "24h" | "7d" | "30d";
export type IncidentHistoryStatus = "all" | NetworkIncidentStatus;
export type IncidentHistoryArea = "all" | DiagnosticArea;

export interface IncidentHistoryFilters {
  period: IncidentHistoryPeriod;
  status: IncidentHistoryStatus;
  area: IncidentHistoryArea;
}

const periodDurations: Record<Exclude<IncidentHistoryPeriod, "all">, number> = {
  "24h": 24 * 60 * 60 * 1000,
  "7d": 7 * 24 * 60 * 60 * 1000,
  "30d": 30 * 24 * 60 * 60 * 1000,
};

export const defaultIncidentHistoryFilters: IncidentHistoryFilters = {
  period: "all",
  status: "all",
  area: "all",
};

export function filterNetworkIncidents(
  incidents: NetworkIncident[],
  filters: IncidentHistoryFilters,
  now = new Date(),
) {
  return incidents.filter((incident) => {
    if (filters.status !== "all" && incident.status !== filters.status) {
      return false;
    }
    if (filters.area !== "all" && incident.area !== filters.area) {
      return false;
    }
    if (filters.period === "all") return true;

    const startedAt = new Date(incident.startedAt).getTime();
    if (Number.isNaN(startedAt)) return false;
    return startedAt >= now.getTime() - periodDurations[filters.period];
  });
}

export function formatIncidentDuration(incident: NetworkIncident) {
  const startedAt = new Date(incident.startedAt).getTime();
  const endedAt = new Date(incident.resolvedAt ?? incident.lastObservedAt).getTime();
  if (Number.isNaN(startedAt) || Number.isNaN(endedAt) || endedAt < startedAt) {
    return "-";
  }

  const totalMinutes = Math.max(1, Math.round((endedAt - startedAt) / 60000));
  const hours = Math.floor(totalMinutes / 60);
  const minutes = totalMinutes % 60;
  if (hours > 0 && minutes > 0) return `${hours}시간 ${minutes}분`;
  if (hours > 0) return `${hours}시간`;
  return `${minutes}분`;
}
```

- [ ] **Step 5: Add history page tests**

Create `src/features/network-incidents/NetworkIncidentHistoryPage.test.tsx`:

```tsx
import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  ongoingIncidentFixture,
  recoveringIncidentFixture,
  resolvedIncidentFixture,
  unknownIncidentFixture,
} from "./fixture";
import NetworkIncidentHistoryPage from "./NetworkIncidentHistoryPage";

const { canUseMock, getIncidentsMock } = vi.hoisted(() => ({
  canUseMock: vi.fn(),
  getIncidentsMock: vi.fn(),
}));

vi.mock("./api", () => ({
  canUseNetworkIncidents: canUseMock,
  getNetworkIncidents: getIncidentsMock,
}));

describe("NetworkIncidentHistoryPage", () => {
  beforeEach(() => {
    canUseMock.mockReturnValue(true);
    getIncidentsMock.mockReset();
  });

  it("renders the full incident history", async () => {
    getIncidentsMock.mockResolvedValue([
      ongoingIncidentFixture,
      recoveringIncidentFixture,
      resolvedIncidentFixture,
    ]);

    render(<NetworkIncidentHistoryPage onBack={vi.fn()} />);

    expect(await screen.findByText("진행 중")).toBeInTheDocument();
    expect(screen.getByText("복구 확인 중")).toBeInTheDocument();
    expect(screen.getByText("복구됨")).toBeInTheDocument();
    expect(screen.getAllByRole("listitem")).toHaveLength(3);
  });

  it("calls back when the back button is pressed", () => {
    getIncidentsMock.mockResolvedValue([]);
    const onBack = vi.fn();

    render(<NetworkIncidentHistoryPage onBack={onBack} />);
    fireEvent.click(screen.getByRole("button", { name: "돌아가기" }));

    expect(onBack).toHaveBeenCalledTimes(1);
  });

  it("shows an empty state when there are no saved incidents", async () => {
    getIncidentsMock.mockResolvedValue([]);

    render(<NetworkIncidentHistoryPage onBack={vi.fn()} />);

    expect(
      await screen.findByText("저장된 장애 기록이 없습니다."),
    ).toBeInTheDocument();
  });

  it("shows a filtered empty state when filters hide all incidents", async () => {
    getIncidentsMock.mockResolvedValue([ongoingIncidentFixture]);

    render(<NetworkIncidentHistoryPage onBack={vi.fn()} />);
    await screen.findByText("진행 중");
    fireEvent.change(await screen.findByLabelText("상태"), {
      target: { value: "resolved" },
    });

    expect(
      await screen.findByText("조건에 맞는 장애 기록이 없습니다."),
    ).toBeInTheDocument();
  });

  it("filters by status and area", async () => {
    getIncidentsMock.mockResolvedValue([
      ongoingIncidentFixture,
      resolvedIncidentFixture,
      unknownIncidentFixture,
    ]);

    render(<NetworkIncidentHistoryPage onBack={vi.fn()} />);
    await screen.findByText("진행 중");

    fireEvent.change(screen.getByLabelText("상태"), {
      target: { value: "resolved" },
    });
    expect(screen.queryByText("진행 중")).not.toBeInTheDocument();
    expect(screen.getByText("복구됨")).toBeInTheDocument();

    fireEvent.change(screen.getByLabelText("상태"), {
      target: { value: "all" },
    });
    fireEvent.change(screen.getByLabelText("구간"), {
      target: { value: "unknown" },
    });
    expect(screen.getByText("확인 불가")).toBeInTheDocument();
    expect(screen.queryByText("공유기 또는 로컬 연결 구간")).not.toBeInTheDocument();
  });

  it("shows unavailable state without claiming a current outage", async () => {
    getIncidentsMock.mockRejectedValue(new Error("history unavailable"));

    render(<NetworkIncidentHistoryPage onBack={vi.fn()} />);

    expect(
      await screen.findByText("장애 이력을 불러올 수 없습니다."),
    ).toBeInTheDocument();
    expect(screen.queryByText("장애 확인")).not.toBeInTheDocument();
  });

  it("does not render row links or detail actions", async () => {
    getIncidentsMock.mockResolvedValue([ongoingIncidentFixture]);

    render(<NetworkIncidentHistoryPage onBack={vi.fn()} />);

    expect(await screen.findByText("진행 중")).toBeInTheDocument();
    expect(screen.queryByRole("link")).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /상세/ })).not.toBeInTheDocument();
  });
});
```

- [ ] **Step 6: Implement history page and CSS**

Create `src/features/network-incidents/NetworkIncidentHistoryPage.tsx`:

```tsx
import { useEffect, useMemo, useState } from "react";
import { canUseNetworkIncidents, getNetworkIncidents } from "./api";
import {
  defaultIncidentHistoryFilters,
  filterNetworkIncidents,
  formatIncidentDuration,
  type IncidentHistoryFilters,
} from "./filter";
import type { NetworkIncident, NetworkIncidentStatus } from "./types";
import styles from "./NetworkIncidentHistoryPage.module.css";

const statusLabels: Record<NetworkIncidentStatus, string> = {
  ongoing: "진행 중",
  recovering: "복구 확인 중",
  resolved: "복구됨",
};

const areaLabels: Record<NetworkIncident["area"], string> = {
  local_connection: "로컬 연결 구간",
  gateway_or_local: "공유기 또는 로컬 연결 구간",
  dns: "DNS",
  external: "외부 연결 구간",
  unknown: "확인 불가",
};

function formatTime(value: string) {
  return new Intl.DateTimeFormat("ko-KR", {
    dateStyle: "short",
    timeStyle: "medium",
  }).format(new Date(value));
}

export default function NetworkIncidentHistoryPage({
  onBack,
}: {
  onBack: () => void;
}) {
  const [available] = useState(canUseNetworkIncidents);
  const [incidents, setIncidents] = useState<NetworkIncident[]>([]);
  const [filters, setFilters] = useState<IncidentHistoryFilters>(
    defaultIncidentHistoryFilters,
  );
  const [failed, setFailed] = useState(!available);
  const [loading, setLoading] = useState(available);

  useEffect(() => {
    if (!available) return;
    let cancelled = false;
    getNetworkIncidents()
      .then((items) => {
        if (!cancelled) setIncidents(items);
      })
      .catch(() => {
        if (!cancelled) setFailed(true);
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [available]);

  const filtered = useMemo(
    () => filterNetworkIncidents(incidents, filters),
    [incidents, filters],
  );

  const setFilter = <K extends keyof IncidentHistoryFilters>(
    key: K,
    value: IncidentHistoryFilters[K],
  ) => {
    setFilters((current) => ({ ...current, [key]: value }));
  };

  return (
    <main className={styles["shell"]}>
      <header className={styles["page-header"]}>
        <button className={styles["back-button"]} type="button" onClick={onBack}>
          돌아가기
        </button>
        <div>
          <p className={styles["eyebrow"]}>INCIDENT HISTORY</p>
          <h1>장애 이력</h1>
          <p>앱 실행 중 확정된 네트워크 장애 기록을 보여줍니다.</p>
        </div>
      </header>

      <section className={styles["filters"]} aria-label="장애 이력 필터">
        <label>
          기간
          <select
            value={filters.period}
            onChange={(event) =>
              setFilter("period", event.target.value as IncidentHistoryFilters["period"])
            }
          >
            <option value="all">전체</option>
            <option value="24h">최근 24시간</option>
            <option value="7d">7일</option>
            <option value="30d">30일</option>
          </select>
        </label>
        <label>
          상태
          <select
            value={filters.status}
            onChange={(event) =>
              setFilter("status", event.target.value as IncidentHistoryFilters["status"])
            }
          >
            <option value="all">전체</option>
            <option value="ongoing">진행 중</option>
            <option value="recovering">복구 확인 중</option>
            <option value="resolved">복구됨</option>
          </select>
        </label>
        <label>
          구간
          <select
            value={filters.area}
            onChange={(event) =>
              setFilter("area", event.target.value as IncidentHistoryFilters["area"])
            }
          >
            <option value="all">전체</option>
            <option value="local_connection">로컬 연결</option>
            <option value="gateway_or_local">공유기 또는 로컬</option>
            <option value="dns">DNS</option>
            <option value="external">외부 연결</option>
            <option value="unknown">확인 불가</option>
          </select>
        </label>
      </section>

      <section className={styles["panel"]} aria-label="장애 이력 목록">
        {loading ? <p className={styles["muted"]}>장애 이력 확인 중</p> : null}
        {!loading && failed ? (
          <p className={styles["muted"]}>장애 이력을 불러올 수 없습니다.</p>
        ) : null}
        {!loading && !failed && incidents.length === 0 ? (
          <p className={styles["muted"]}>저장된 장애 기록이 없습니다.</p>
        ) : null}
        {!loading && !failed && incidents.length > 0 && filtered.length === 0 ? (
          <p className={styles["muted"]}>조건에 맞는 장애 기록이 없습니다.</p>
        ) : null}
        {!loading && !failed && filtered.length > 0 ? (
          <ul className={styles["list"]}>
            {filtered.map((incident) => {
              const endAt = incident.resolvedAt ?? incident.lastObservedAt;
              return (
                <li className={styles["row"]} key={incident.id}>
                  <strong>{statusLabels[incident.status]}</strong>
                  <span>{areaLabels[incident.area]}</span>
                  <time dateTime={incident.startedAt}>
                    {formatTime(incident.startedAt)}
                  </time>
                  <time dateTime={endAt}>{formatTime(endAt)}</time>
                  <span>{formatIncidentDuration(incident)}</span>
                  <p>{incident.summary}</p>
                </li>
              );
            })}
          </ul>
        ) : null}
      </section>
    </main>
  );
}
```

Create CSS:

```css
.shell {
  width: min(1180px, 100%);
  margin: 0 auto;
  padding: 32px 20px 48px;
}

.page-header {
  display: grid;
  gap: 16px;
  margin-bottom: 18px;
}

.back-button {
  justify-self: start;
  border: 1px solid #304563;
  border-radius: 8px;
  padding: 8px 11px;
  color: #d7e1ee;
  background: #101827;
  cursor: pointer;
}

.eyebrow {
  margin: 0 0 8px;
  color: #79b8ff;
  font-size: 0.72rem;
  font-weight: 700;
  letter-spacing: 0.12em;
}

.page-header h1 {
  margin: 0;
  font-size: 1.55rem;
}

.page-header p:last-child {
  margin: 8px 0 0;
  color: #9fb0c8;
}

.filters {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
  margin-bottom: 16px;
}

.filters label {
  display: grid;
  gap: 6px;
  color: #8394aa;
  font-size: 0.78rem;
}

.filters select {
  min-width: 0;
  border: 1px solid #304563;
  border-radius: 8px;
  padding: 8px 10px;
  color: #d7e1ee;
  background: #101827;
}

.panel {
  border: 1px solid #304563;
  border-radius: 10px;
  padding: 18px;
  background: #101827;
}

.muted {
  margin: 0;
  color: #9fb0c8;
}

.list {
  display: grid;
  gap: 0;
  margin: 0;
  padding: 0;
  list-style: none;
}

.row {
  display: grid;
  grid-template-columns: 100px 150px 160px 160px 90px minmax(220px, 1fr);
  gap: 12px;
  align-items: center;
  min-height: 42px;
  border-top: 1px solid #263b57;
  color: #9fb0c8;
  font-size: 0.78rem;
}

.row:first-child {
  border-top: 0;
}

.row strong {
  color: #f4f8ff;
}

.row p {
  margin: 0;
  color: #c3d0df;
}

@media (max-width: 820px) {
  .filters,
  .row {
    grid-template-columns: 1fr;
  }

  .row {
    gap: 6px;
    padding: 12px 0;
  }
}
```

- [ ] **Step 7: Verify frontend history tests**

Run:

```bash
npm test -- src/features/network-incidents
```

Expected: PASS.

- [ ] **Step 8: Commit Task 2**

```bash
git add src/features/network-incidents
git commit -m "feat: add network incident history page"
```

---

### Task 3: Home Navigation And Recent Panel Action

**Files:**
- Modify: `src/features/network-incidents/RecentIncidentsPanel.tsx`
- Modify: `src/features/network-incidents/RecentIncidentsPanel.module.css`
- Modify: `src/features/network-incidents/RecentIncidentsPanel.test.tsx`
- Modify: `src/features/product-home/ProductHome.tsx`
- Modify: `src/features/product-home/ProductHome.test.tsx`
- Modify: `src/pages/HomePage.tsx`

- [ ] **Step 1: Add recent panel action tests**

In `src/features/network-incidents/RecentIncidentsPanel.test.tsx`, add:

```tsx
  it("hides the full history action when no callback is provided", async () => {
    getRecentMock.mockResolvedValue([]);

    render(<RecentIncidentsPanel />);

    expect(
      screen.queryByRole("button", { name: "전체 이력 보기" }),
    ).not.toBeInTheDocument();
  });

  it("calls the full history action when callback is provided", async () => {
    getRecentMock.mockResolvedValue([]);
    const onOpenHistory = vi.fn();

    render(<RecentIncidentsPanel onOpenHistory={onOpenHistory} />);
    fireEvent.click(screen.getByRole("button", { name: "전체 이력 보기" }));

    expect(onOpenHistory).toHaveBeenCalledTimes(1);
  });
```

Update imports to include `fireEvent`.

- [ ] **Step 2: Implement recent panel action**

In `RecentIncidentsPanel.tsx`, change the signature:

```tsx
export default function RecentIncidentsPanel({
  onOpenHistory,
}: {
  onOpenHistory?: () => void;
}) {
```

Inside the header after `<h2>` render:

```tsx
        {onOpenHistory ? (
          <button type="button" onClick={onOpenHistory}>
            전체 이력 보기
          </button>
        ) : null}
```

Adjust CSS header to support the action:

```css
.header {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 8px 16px;
  align-items: start;
}

.header p,
.header h2 {
  grid-column: 1;
}

.header button {
  grid-column: 2;
  grid-row: 1 / span 2;
  border: 1px solid #304563;
  border-radius: 8px;
  padding: 7px 10px;
  color: #d7e1ee;
  background: #0d1522;
  cursor: pointer;
}
```

- [ ] **Step 3: Add ProductHome and HomePage tests**

In `src/features/product-home/ProductHome.test.tsx`, change the incident panel mock:

```tsx
const { openHistoryMock } = vi.hoisted(() => ({
  openHistoryMock: vi.fn(),
}));

vi.mock("@/features/network-incidents/RecentIncidentsPanel", () => ({
  default: ({ onOpenHistory }: { onOpenHistory?: () => void }) => {
    openHistoryMock.mockImplementation(onOpenHistory ?? vi.fn());
    return <section>최근 장애</section>;
  },
}));
```

Add this test:

```tsx
  it("passes the incident history action to the recent incidents panel", () => {
    const onOpenIncidentHistory = vi.fn();

    render(<ProductHome onOpenIncidentHistory={onOpenIncidentHistory} />);
    openHistoryMock();

    expect(onOpenIncidentHistory).toHaveBeenCalledTimes(1);
  });
```

Create or update `src/pages/HomePage.test.tsx`:

```tsx
import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import HomePage from "./HomePage";

vi.mock("@/features/product-home/ProductHome", () => ({
  default: ({ onOpenIncidentHistory }: { onOpenIncidentHistory: () => void }) => (
    <button type="button" onClick={onOpenIncidentHistory}>
      open history
    </button>
  ),
}));

vi.mock("@/features/network-incidents/NetworkIncidentHistoryPage", () => ({
  default: ({ onBack }: { onBack: () => void }) => (
    <section>
      <h1>장애 이력</h1>
      <button type="button" onClick={onBack}>
        돌아가기
      </button>
    </section>
  ),
}));

describe("HomePage", () => {
  it("switches between home and incident history views", () => {
    render(<HomePage />);

    fireEvent.click(screen.getByRole("button", { name: "open history" }));
    expect(screen.getByRole("heading", { name: "장애 이력" })).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "돌아가기" }));
    expect(screen.getByRole("button", { name: "open history" })).toBeInTheDocument();
  });
});
```

- [ ] **Step 4: Implement HomePage view state and ProductHome prop**

In `ProductHome.tsx`, change:

```tsx
export default function ProductHome({
  onOpenIncidentHistory,
}: {
  onOpenIncidentHistory: () => void;
}) {
```

Pass:

```tsx
<RecentIncidentsPanel onOpenHistory={onOpenIncidentHistory} />
```

In `HomePage.tsx`, replace the file with:

```tsx
import { useState } from "react";
import ProductHome from "@/features/product-home/ProductHome";
import NetworkIncidentHistoryPage from "@/features/network-incidents/NetworkIncidentHistoryPage";

type HomeView = "home" | "incident_history";

export default function HomePage() {
  const [view, setView] = useState<HomeView>("home");

  if (view === "incident_history") {
    return <NetworkIncidentHistoryPage onBack={() => setView("home")} />;
  }

  return <ProductHome onOpenIncidentHistory={() => setView("incident_history")} />;
}
```

- [ ] **Step 5: Verify focused navigation tests**

Run:

```bash
npm test -- src/features/network-incidents/RecentIncidentsPanel.test.tsx src/features/product-home/ProductHome.test.tsx src/pages/HomePage.test.tsx
```

Expected: PASS.

- [ ] **Step 6: Commit Task 3**

```bash
git add src/features/network-incidents/RecentIncidentsPanel.tsx src/features/network-incidents/RecentIncidentsPanel.module.css src/features/network-incidents/RecentIncidentsPanel.test.tsx src/features/product-home/ProductHome.tsx src/features/product-home/ProductHome.test.tsx src/pages/HomePage.tsx src/pages/HomePage.test.tsx
git commit -m "feat: navigate to network incident history"
```

---

### Task 4: Full Verification

**Files:**
- No planned code changes unless verification finds a defect.

- [ ] **Step 1: Run full Rust verification**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: PASS.

- [ ] **Step 2: Run full frontend verification**

Run:

```bash
npm test
npm run build
```

Expected: PASS.

- [ ] **Step 3: Check scope boundaries**

Run:

```bash
rg -n "router|Router|Route|export|delete|삭제|timeline|packet|capture|라우터|자동 복구|상세 준비" src docs/superpowers/specs/2026-06-30-network-incident-history-v3-design.md
```

Expected: no new v3 implementation that adds URL routing, export/delete/timeline/packet capture/router actions/auto repair/disabled detail affordance. Matches in docs exclusions are acceptable.

- [ ] **Step 4: Final commit only if fixes were needed**

If verification required code fixes:

```bash
git add <changed-files>
git commit -m "fix: stabilize network incident history"
```

If no fixes were needed, do not create an empty commit.

---

## Self-Review

- Spec coverage: Task 1 covers the full-list backend command and unavailable error behavior. Task 2 covers the history page, frontend filtering, loading/empty/error states, duration display, and no row action. Task 3 covers view-state navigation and the recent panel `전체 이력 보기` action. Task 4 covers final verification and scope checks.
- Exclusions preserved: No React Router, detail page/drawer/modal, row click action, disabled detail button, raw evidence timeline, chart/statistics, export, delete, retention settings, background service, packet capture, router action, Windows setting change, automatic recovery, or upload is added.
- Type consistency: Rust continues returning v2 `NetworkIncident`; frontend uses existing `NetworkIncident` types. New command is `get_network_incidents`; existing recent command remains `get_recent_network_incidents`.
- Verification: Final gate is `cargo test --manifest-path src-tauri/Cargo.toml`, `cargo check --manifest-path src-tauri/Cargo.toml`, `npm test`, and `npm run build`. Windows artifact validation remains required for real app data persistence and restart behavior.
