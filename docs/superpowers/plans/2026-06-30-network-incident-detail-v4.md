# Network Incident Detail v4 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a read-only detail panel to the existing network incident history screen so users can inspect why a stored incident was classified.

**Architecture:** Keep the v3 `NetworkIncidentHistoryPage` as the only screen and add local selection state plus a feature-local detail panel inside that component. Reuse the existing `get_network_incidents` response because each `NetworkIncident` already includes `representativeEvidence`; do not change Rust, SQLite schema, Tauri commands, or app routing.

**Tech Stack:** React 19, TypeScript, Vitest, Testing Library, CSS Modules.

---

## File Structure

- Modify: `src/features/network-incidents/NetworkIncidentHistoryPage.test.tsx`
  - Add tests for selecting an incident, rendering detail fields, empty evidence, filter-driven selection clearing, and excluded actions.
- Modify: `src/features/network-incidents/NetworkIncidentHistoryPage.tsx`
  - Add evidence source/status labels, selected incident state, selected incident cleanup, row detail action, and read-only detail panel rendering.
- Modify: `src/features/network-incidents/NetworkIncidentHistoryPage.module.css`
  - Add two-column content layout, row action styles, selected row state, detail panel summary/evidence styles, and mobile stacking.
- Modify: `src/features/network-incidents/fixture.ts`
  - Add a fixture with no representative evidence if needed by tests.

No backend files change in v4.

---

### Task 1: Detail Panel Tests

**Files:**
- Modify: `src/features/network-incidents/NetworkIncidentHistoryPage.test.tsx`
- Modify: `src/features/network-incidents/fixture.ts`

- [ ] **Step 1: Add an incident fixture without representative evidence**

In `src/features/network-incidents/fixture.ts`, append:

```ts
export const incidentWithoutEvidenceFixture: NetworkIncident = {
  ...resolvedIncidentFixture,
  id: 5,
  representativeEvidence: [],
  summary: "대표 근거가 없는 저장 기록입니다.",
};
```

- [ ] **Step 2: Add failing detail panel tests**

In `src/features/network-incidents/NetworkIncidentHistoryPage.test.tsx`, import the new fixture:

```ts
import {
  incidentWithoutEvidenceFixture,
  ongoingIncidentFixture,
  recoveringIncidentFixture,
  resolvedIncidentFixture,
  unknownIncidentFixture,
} from "./fixture";
```

Append these tests inside the existing `describe("NetworkIncidentHistoryPage", () => { ... })` block:

```tsx
  it("opens a read-only detail panel for a selected incident", async () => {
    getIncidentsMock.mockResolvedValue([ongoingIncidentFixture]);

    render(<NetworkIncidentHistoryPage onBack={vi.fn()} />);
    fireEvent.click(await screen.findByRole("button", { name: "상세 보기" }));

    const detail = screen.getByRole("region", { name: "선택한 장애 상세" });
    expect(within(detail).getByText("진행 중")).toBeInTheDocument();
    expect(
      within(detail).getByText("공유기 또는 로컬 연결 구간"),
    ).toBeInTheDocument();
    expect(within(detail).getByText("아직 복구 기록 없음")).toBeInTheDocument();
    expect(within(detail).getByText("게이트웨이")).toBeInTheDocument();
    expect(within(detail).getByText("시간 초과")).toBeInTheDocument();
    expect(within(detail).getByText("1000ms")).toBeInTheDocument();
    expect(within(detail).getByText("gateway timeout")).toBeInTheDocument();
  });

  it("closes the detail panel when the close action is pressed", async () => {
    getIncidentsMock.mockResolvedValue([ongoingIncidentFixture]);

    render(<NetworkIncidentHistoryPage onBack={vi.fn()} />);
    fireEvent.click(await screen.findByRole("button", { name: "상세 보기" }));
    fireEvent.click(screen.getByRole("button", { name: "상세 닫기" }));

    expect(
      screen.queryByRole("region", { name: "선택한 장애 상세" }),
    ).not.toBeInTheDocument();
  });

  it("shows an empty evidence message when the selected incident has no representative evidence", async () => {
    getIncidentsMock.mockResolvedValue([incidentWithoutEvidenceFixture]);

    render(<NetworkIncidentHistoryPage onBack={vi.fn()} />);
    fireEvent.click(await screen.findByRole("button", { name: "상세 보기" }));

    expect(
      screen.getByText("저장된 대표 근거가 없습니다."),
    ).toBeInTheDocument();
  });

  it("clears the selected incident when filters remove it from the list", async () => {
    getIncidentsMock.mockResolvedValue([ongoingIncidentFixture]);

    render(<NetworkIncidentHistoryPage onBack={vi.fn()} />);
    fireEvent.click(await screen.findByRole("button", { name: "상세 보기" }));
    expect(
      screen.getByRole("region", { name: "선택한 장애 상세" }),
    ).toBeInTheDocument();

    fireEvent.change(screen.getByLabelText("상태"), {
      target: { value: "resolved" },
    });

    expect(
      await screen.findByText("조건에 맞는 장애 기록이 없습니다."),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("region", { name: "선택한 장애 상세" }),
    ).not.toBeInTheDocument();
  });

  it("does not expose destructive or repair actions in the detail panel", async () => {
    getIncidentsMock.mockResolvedValue([ongoingIncidentFixture]);

    render(<NetworkIncidentHistoryPage onBack={vi.fn()} />);
    fireEvent.click(await screen.findByRole("button", { name: "상세 보기" }));

    expect(screen.queryByRole("button", { name: /삭제/ })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /복구/ })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /내보내기|export/i })).not.toBeInTheDocument();
  });
```

- [ ] **Step 3: Run the focused test and verify failure**

Run:

```bash
npm test -- src/features/network-incidents/NetworkIncidentHistoryPage.test.tsx
```

Expected: FAIL because `상세 보기`, `선택한 장애 상세`, and `상세 닫기` do not exist yet.

- [ ] **Step 4: Commit the failing test only if your workflow allows red commits**

Default for this repo: do not commit failing tests separately. Keep the failing tests in the working tree and continue to Task 2.

---

### Task 2: Detail Panel Rendering

**Files:**
- Modify: `src/features/network-incidents/NetworkIncidentHistoryPage.tsx`

- [ ] **Step 1: Extend imports and labels**

Replace the current React import:

```ts
import { useEffect, useMemo, useState } from "react";
```

with:

```ts
import { useEffect, useMemo, useState } from "react";
import type {
  DiagnosticArea,
  EvidenceSource,
  EvidenceStatus,
} from "@/features/network-diagnostics/types";
```

Replace the current `areaLabels` declaration with this explicit diagnostic area map:

```ts
const areaLabels: Record<DiagnosticArea, string> = {
  local_connection: "로컬 연결 구간",
  gateway_or_local: "공유기 또는 로컬 연결 구간",
  dns: "DNS",
  external: "외부 연결 구간",
  unknown: "확인 불가",
};
```

Add these label maps below `areaLabels`:

```ts
const sourceLabels: Record<EvidenceSource, string> = {
  ethernet: "PC/어댑터",
  ipv4: "IPv4",
  default_route: "기본 경로",
  gateway: "게이트웨이",
  dns_microsoft: "DNS Microsoft",
  dns_google: "DNS Google",
  http_microsoft: "Microsoft 연결",
  http_google: "Google 연결",
};

const evidenceStatusLabels: Record<EvidenceStatus, string> = {
  success: "성공",
  failure: "실패",
  timeout: "시간 초과",
  not_checked: "확인하지 않음",
  unsupported: "확인 불가",
};
```

- [ ] **Step 2: Add formatting helpers**

Add these helpers below `formatTime`:

```ts
function formatOptionalTime(value: string | null | undefined, fallback: string) {
  return value ? formatTime(value) : fallback;
}

function formatDurationMs(value: number | null | undefined) {
  return typeof value === "number" ? `${value}ms` : "-";
}
```

- [ ] **Step 3: Add selected incident state and cleanup**

Inside `NetworkIncidentHistoryPage`, after the existing `filters` state, add:

```ts
  const [selectedIncidentId, setSelectedIncidentId] = useState<number | null>(null);
```

After the existing `filtered` memo, add:

```ts
  const selectedIncident = useMemo(
    () => filtered.find((incident) => incident.id === selectedIncidentId) ?? null,
    [filtered, selectedIncidentId],
  );

  useEffect(() => {
    if (selectedIncidentId !== null && !selectedIncident) {
      setSelectedIncidentId(null);
    }
  }, [selectedIncident, selectedIncidentId]);
```

- [ ] **Step 4: Replace the list section rendering with content and detail layout**

Inside `<section className={styles["panel"]} aria-label="장애 이력 목록">`, replace the successful `filtered.length > 0` block with:

```tsx
        {!loading && !failed && filtered.length > 0 ? (
          <div className={styles["content"]}>
            <ul className={styles["list"]}>
              {filtered.map((incident) => {
                const endAt = incident.resolvedAt ?? incident.lastObservedAt;
                const selected = incident.id === selectedIncidentId;
                return (
                  <li
                    className={`${styles["row"]} ${
                      selected ? styles["row-selected"] : ""
                    }`}
                    key={incident.id}
                  >
                    <strong>{statusLabels[incident.status]}</strong>
                    <span>{areaLabels[incident.area]}</span>
                    <time dateTime={incident.startedAt}>
                      {formatTime(incident.startedAt)}
                    </time>
                    <time dateTime={endAt}>{formatTime(endAt)}</time>
                    <span>{formatIncidentDuration(incident)}</span>
                    <p>{incident.summary}</p>
                    <button
                      className={styles["detail-button"]}
                      type="button"
                      aria-pressed={selected}
                      onClick={() => setSelectedIncidentId(incident.id)}
                    >
                      상세 보기
                    </button>
                  </li>
                );
              })}
            </ul>
            {selectedIncident ? (
              <aside
                className={styles["detail"]}
                aria-label="선택한 장애 상세"
                role="region"
              >
                <div className={styles["detail-header"]}>
                  <div>
                    <p className={styles["eyebrow"]}>INCIDENT DETAIL</p>
                    <h2>선택한 장애 상세</h2>
                  </div>
                  <button
                    className={styles["close-button"]}
                    type="button"
                    onClick={() => setSelectedIncidentId(null)}
                  >
                    상세 닫기
                  </button>
                </div>

                <dl className={styles["detail-summary"]}>
                  <div>
                    <dt>상태</dt>
                    <dd>{statusLabels[selectedIncident.status]}</dd>
                  </div>
                  <div>
                    <dt>추정 구간</dt>
                    <dd>{areaLabels[selectedIncident.area]}</dd>
                  </div>
                  <div>
                    <dt>시작</dt>
                    <dd>{formatTime(selectedIncident.startedAt)}</dd>
                  </div>
                  <div>
                    <dt>마지막 확인</dt>
                    <dd>{formatTime(selectedIncident.lastObservedAt)}</dd>
                  </div>
                  <div>
                    <dt>복구</dt>
                    <dd>
                      {formatOptionalTime(
                        selectedIncident.resolvedAt,
                        "아직 복구 기록 없음",
                      )}
                    </dd>
                  </div>
                  <div>
                    <dt>지속 시간</dt>
                    <dd>{formatIncidentDuration(selectedIncident)}</dd>
                  </div>
                </dl>

                <p className={styles["detail-summary-text"]}>
                  {selectedIncident.summary}
                </p>

                <div className={styles["evidence-section"]}>
                  <h3>대표 근거</h3>
                  {selectedIncident.representativeEvidence.length === 0 ? (
                    <p className={styles["muted"]}>
                      저장된 대표 근거가 없습니다.
                    </p>
                  ) : (
                    <ul className={styles["evidence-list"]}>
                      {selectedIncident.representativeEvidence.map((evidence) => (
                        <li
                          className={styles["evidence-row"]}
                          key={`${evidence.source}-${evidence.observedAt}-${evidence.detail ?? ""}`}
                        >
                          <strong>{sourceLabels[evidence.source]}</strong>
                          <span>{evidenceStatusLabels[evidence.status]}</span>
                          <span>{formatDurationMs(evidence.durationMs)}</span>
                          <p>{evidence.detail ?? "세부 정보 없음"}</p>
                          <time dateTime={evidence.checkedAt ?? undefined}>
                            {formatOptionalTime(evidence.checkedAt, "확인 시각 없음")}
                          </time>
                          <time dateTime={evidence.observedAt}>
                            {formatTime(evidence.observedAt)}
                          </time>
                        </li>
                      ))}
                    </ul>
                  )}
                </div>
              </aside>
            ) : (
              <p className={styles["selection-hint"]}>
                기록을 선택하면 대표 근거를 확인할 수 있습니다.
              </p>
            )}
          </div>
        ) : null}
```

- [ ] **Step 5: Run the focused test and verify behavior**

Run:

```bash
npm test -- src/features/network-incidents/NetworkIncidentHistoryPage.test.tsx
```

Expected: FAIL only for styling-independent query mistakes or PASS if markup matches the tests.

---

### Task 3: Detail Panel Styling

**Files:**
- Modify: `src/features/network-incidents/NetworkIncidentHistoryPage.module.css`

- [ ] **Step 1: Update row grid for the detail action**

Replace the `.row` grid columns:

```css
  grid-template-columns: 100px 150px 160px 160px 90px minmax(220px, 1fr);
```

with:

```css
  grid-template-columns: 86px 132px 150px 150px 82px minmax(180px, 1fr) 84px;
```

- [ ] **Step 2: Add content, selection, buttons, and detail styles**

Append these styles before the existing media query:

```css
.content {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(300px, 360px);
  gap: 18px;
  align-items: start;
}

.row-selected {
  background: #142033;
}

.detail-button,
.close-button {
  border: 1px solid #304563;
  border-radius: 8px;
  padding: 7px 9px;
  color: #d7e1ee;
  background: #0d1522;
  cursor: pointer;
}

.detail-button[aria-pressed="true"] {
  border-color: #79b8ff;
  color: #f4f8ff;
}

.detail {
  border: 1px solid #304563;
  border-radius: 10px;
  padding: 16px;
  background: #0d1522;
}

.detail-header {
  display: flex;
  gap: 12px;
  align-items: start;
  justify-content: space-between;
  margin-bottom: 14px;
}

.detail-header h2 {
  margin: 0;
  font-size: 1rem;
}

.detail-summary {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
  margin: 0;
}

.detail-summary div {
  display: grid;
  gap: 3px;
}

.detail-summary dt {
  color: #8394aa;
  font-size: 0.72rem;
}

.detail-summary dd {
  margin: 0;
  color: #d7e1ee;
  font-size: 0.82rem;
}

.detail-summary-text {
  margin: 14px 0 0;
  color: #c3d0df;
  font-size: 0.84rem;
  line-height: 1.55;
}

.evidence-section {
  margin-top: 16px;
}

.evidence-section h3 {
  margin: 0 0 10px;
  font-size: 0.9rem;
}

.evidence-list {
  display: grid;
  gap: 10px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.evidence-row {
  display: grid;
  gap: 5px;
  border-top: 1px solid #263b57;
  padding-top: 10px;
  color: #9fb0c8;
  font-size: 0.76rem;
}

.evidence-row:first-child {
  border-top: 0;
  padding-top: 0;
}

.evidence-row strong {
  color: #f4f8ff;
}

.evidence-row p {
  margin: 0;
  color: #c3d0df;
}

.selection-hint {
  margin: 0;
  border: 1px dashed #304563;
  border-radius: 10px;
  padding: 16px;
  color: #9fb0c8;
  background: #0d1522;
}
```

- [ ] **Step 3: Update mobile layout**

Inside the existing `@media (max-width: 820px)` block, replace:

```css
  .filters,
  .row {
    grid-template-columns: 1fr;
  }
```

with:

```css
  .filters,
  .content,
  .row,
  .detail-summary {
    grid-template-columns: 1fr;
  }
```

- [ ] **Step 4: Run focused frontend verification**

Run:

```bash
npm test -- src/features/network-incidents/NetworkIncidentHistoryPage.test.tsx
npm test -- src/features/network-incidents
npm run build
```

Expected: all commands pass.

- [ ] **Step 5: Commit the v4 implementation**

Run:

```bash
git add src/features/network-incidents/fixture.ts src/features/network-incidents/NetworkIncidentHistoryPage.test.tsx src/features/network-incidents/NetworkIncidentHistoryPage.tsx src/features/network-incidents/NetworkIncidentHistoryPage.module.css
git commit -m "feat: show network incident details"
```

---

### Task 4: Full Verification and Scope Check

**Files:**
- No edits expected.

- [ ] **Step 1: Run full verification**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml --quiet
cargo check --manifest-path src-tauri/Cargo.toml --quiet
npm test
npm run build
```

Expected:

- Rust tests pass with 68 tests.
- `cargo check` exits 0.
- Vitest exits 0.
- Vite production build exits 0.

- [ ] **Step 2: Check for forbidden v4 scope**

Run:

```bash
rg -n "delete|삭제|timeline|packet|capture|자동 복구|라우터|Router|Routes|BrowserRouter|export" src/features/network-incidents src/pages src/features/product-home
```

Expected: no matches except TypeScript `export` declarations if the command scans source files broadly. If matches appear for delete, packet capture, automatic repair, router routing, route libraries, chart timeline, or user-facing export actions, remove them before completing v4.

- [ ] **Step 3: Check git status**

Run:

```bash
git status --short --branch
```

Expected: clean working tree, `dev` ahead of `origin/dev` by the v4 design, plan, and implementation commits until the user pushes.
