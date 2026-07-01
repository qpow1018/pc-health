# Main Network UI v1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Upgrade the network diagnostics home UI so the first screen shows current status, diagnostic path, latest evidence, and a quieter raw probe area.

**Architecture:** Keep the work inside existing network feature components. Extend `NetworkStatusPanel` with small local helper functions for lifecycle copy, path stage summaries, and stable evidence rows; keep `NetworkProbePanel` as the manual raw JSON tool but make the JSON area collapsible after a successful run. Do not add persistence, new probes, latency scoring, router actions, or a generic design system.

**Tech Stack:** React 19, TypeScript, Vitest, Testing Library, CSS Modules with native nesting, Tauri command wrappers.

---

## File Structure

- Modify: `src/features/network-diagnostics/NetworkStatusPanel.tsx`
  - Owns current status summary, diagnostic path, latest evidence table, polling, and unavailable/error states.
- Modify: `src/features/network-diagnostics/NetworkStatusPanel.module.css`
  - Styles the status summary, path stages, and denser evidence table without changing global tokens.
- Modify: `src/features/network-diagnostics/NetworkStatusPanel.test.tsx`
  - Adds behavior checks for lifecycle copy, path stages, duration display, PC/route evidence rows, and muted unavailable states.
- Modify: `src/features/network-diagnostics/fixture.ts`
  - Adds ethernet, IPv4, and default route evidence to the normal fixture so the v1 path can render a complete success case.
- Modify: `src/features/network-probe/NetworkProbePanel.tsx`
  - Keeps the manual probe behavior but places raw JSON behind a disclosure after success.
- Modify: `src/features/network-probe/NetworkProbePanel.module.css`
  - Styles the disclosure area in the existing quiet utility tone.
- Modify: `src/features/network-probe/NetworkProbePanel.test.tsx`
  - Verifies raw JSON is hidden by default after success, can be expanded, can still be copied, and command errors remain visible.

No new route, app shell, persistence, Rust command, or backend contract is needed for this v1.

---

### Task 1: Status Summary Copy

**Files:**
- Modify: `src/features/network-diagnostics/NetworkStatusPanel.test.tsx`
- Modify: `src/features/network-diagnostics/NetworkStatusPanel.tsx`
- Modify: `src/features/network-diagnostics/NetworkStatusPanel.module.css`

- [ ] **Step 1: Add failing tests for lifecycle descriptions**

Add this test block inside `describe("NetworkStatusPanel", () => { ... })` after the lifecycle label test:

```tsx
  it.each([
    ["normal", null, "현재 확인된 구간에서 반복 이상이 없습니다."],
    [
      "suspected",
      "gateway_or_local",
      "이상이 감지됐지만 장애로 확정하려면 추가 근거가 필요합니다.",
    ],
    [
      "incident",
      "gateway_or_local",
      "호환되는 이상이 반복 확인되었습니다.",
    ],
    [
      "recovering",
      "gateway_or_local",
      "정상 근거가 확인되어 추가 확인 중입니다.",
    ],
    ["resolved", null, "현재 연결은 복구된 상태입니다."],
  ] as const)("shows direct summary copy for %s", async (
    lifecycle,
    suspectedArea,
    description,
  ) => {
    getStatusMock.mockResolvedValue({
      ...normalStatusFixture,
      lifecycle,
      suspectedArea,
    });

    render(<NetworkStatusPanel />);

    expect(await screen.findByText(description)).toBeInTheDocument();
  });

  it("shows quiet unavailable summary copy without claiming an outage", async () => {
    getStatusMock.mockResolvedValue(unavailableStatusFixture);

    render(<NetworkStatusPanel />);

    expect(
      await screen.findByText(
        "진단 근거가 부족하거나 Windows 앱에서만 확인할 수 있습니다.",
      ),
    ).toBeInTheDocument();
    expect(screen.queryByText("호환되는 이상이 반복 확인되었습니다.")).not.toBeInTheDocument();
  });
```

- [ ] **Step 2: Run the focused test and verify it fails**

Run:

```bash
npm test -- src/features/network-diagnostics/NetworkStatusPanel.test.tsx
```

Expected: FAIL because the description text is not rendered.

- [ ] **Step 3: Add summary description helper and markup**

In `src/features/network-diagnostics/NetworkStatusPanel.tsx`, add this helper below `areaLabelForStatus`:

```tsx
function descriptionForStatus(status: NetworkDiagnosticStatus) {
  if (status.availability !== "running" || !status.lifecycle) {
    return "진단 근거가 부족하거나 Windows 앱에서만 확인할 수 있습니다.";
  }

  if (status.lifecycle === "normal") {
    return "현재 확인된 구간에서 반복 이상이 없습니다.";
  }

  if (status.lifecycle === "suspected") {
    return "이상이 감지됐지만 장애로 확정하려면 추가 근거가 필요합니다.";
  }

  if (status.lifecycle === "incident") {
    return "호환되는 이상이 반복 확인되었습니다.";
  }

  if (status.lifecycle === "recovering") {
    return "정상 근거가 확인되어 추가 확인 중입니다.";
  }

  return "현재 연결은 복구된 상태입니다.";
}
```

Inside `NetworkStatusPanel`, add:

```tsx
  const statusDescription = descriptionForStatus(status);
```

Render it immediately after the availability/error messages and before the `<dl className={styles["summary"]}>`:

```tsx
      <p className={styles["status-description"]}>{statusDescription}</p>
```

- [ ] **Step 4: Add minimal styling**

In `src/features/network-diagnostics/NetworkStatusPanel.module.css`, add:

```css
.status-description {
  margin: 14px 0 0;
  max-width: 760px;
  color: #c3d0df;
  line-height: 1.55;
}
```

- [ ] **Step 5: Verify the focused test passes**

Run:

```bash
npm test -- src/features/network-diagnostics/NetworkStatusPanel.test.tsx
```

Expected: PASS.

- [ ] **Step 6: Commit Task 1**

```bash
git add src/features/network-diagnostics/NetworkStatusPanel.tsx src/features/network-diagnostics/NetworkStatusPanel.module.css src/features/network-diagnostics/NetworkStatusPanel.test.tsx
git commit -m "feat: add network status summary copy"
```

---

### Task 2: Diagnostic Path And Evidence Table

**Files:**
- Modify: `src/features/network-diagnostics/fixture.ts`
- Modify: `src/features/network-diagnostics/NetworkStatusPanel.test.tsx`
- Modify: `src/features/network-diagnostics/NetworkStatusPanel.tsx`
- Modify: `src/features/network-diagnostics/NetworkStatusPanel.module.css`

- [ ] **Step 1: Extend the normal fixture with PC and route evidence**

In `src/features/network-diagnostics/fixture.ts`, insert these three evidence entries at the start of `normalStatusFixture.evidence`:

```ts
    {
      source: "ethernet",
      status: "success",
      checkedAt: "2026-06-23T09:00:00Z",
      durationMs: null,
      detail: "Ethernet adapter 감지",
    },
    {
      source: "ipv4",
      status: "success",
      checkedAt: "2026-06-23T09:00:00Z",
      durationMs: null,
      detail: "192.168.0.23",
    },
    {
      source: "default_route",
      status: "success",
      checkedAt: "2026-06-23T09:00:00Z",
      durationMs: null,
      detail: "default route 선택됨",
    },
```

- [ ] **Step 2: Add failing tests for path stages and table rows**

In `src/features/network-diagnostics/NetworkStatusPanel.test.tsx`, add these tests after the existing timestamp/detail test:

```tsx
  it("shows the diagnostic path from PC to external connectivity", async () => {
    getStatusMock.mockResolvedValue(incidentStatusFixture);

    render(<NetworkStatusPanel />);

    expect(await screen.findByLabelText("구간별 진단 경로")).toBeInTheDocument();
    expect(screen.getByTestId("path-pc")).toHaveTextContent("PC/어댑터");
    expect(screen.getByTestId("path-pc")).toHaveTextContent("확인하지 않음");
    expect(screen.getByTestId("path-route")).toHaveTextContent("IPv4/Route");
    expect(screen.getByTestId("path-gateway")).toHaveTextContent("게이트웨이");
    expect(screen.getByTestId("path-gateway")).toHaveTextContent("시간 초과");
    expect(screen.getByTestId("path-dns")).toHaveTextContent("DNS");
    expect(screen.getByTestId("path-external")).toHaveTextContent("외부 연결");
  });

  it("renders latest evidence rows for PC, route, gateway, DNS, Microsoft and Google", async () => {
    getStatusMock.mockResolvedValue(normalStatusFixture);

    render(<NetworkStatusPanel />);

    await screen.findByText("정상");
    for (const testId of [
      "evidence-pc",
      "evidence-route",
      "evidence-gateway",
      "evidence-dns",
      "evidence-microsoft",
      "evidence-google",
    ]) {
      expect(screen.getByTestId(testId)).toBeInTheDocument();
    }
    expect(screen.getByTestId("evidence-pc")).toHaveTextContent("Ethernet adapter 감지");
    expect(screen.getByTestId("evidence-route")).toHaveTextContent("default route 선택됨");
    expect(screen.getByTestId("evidence-gateway")).toHaveTextContent("3ms");
    expect(screen.getByTestId("evidence-dns")).toHaveTextContent("5ms");
    expect(screen.queryByText("network health score")).not.toBeInTheDocument();
  });
```

- [ ] **Step 3: Run the focused test and verify it fails**

Run:

```bash
npm test -- src/features/network-diagnostics/NetworkStatusPanel.test.tsx
```

Expected: FAIL because path stages and new evidence rows are not rendered.

- [ ] **Step 4: Add local helpers for path and duration**

In `src/features/network-diagnostics/NetworkStatusPanel.tsx`, add this type and helpers below `latestEvidence`:

```tsx
type PathStage = {
  key: string;
  label: string;
  evidence: DiagnosticEvidence;
};

function formatDuration(value: number | null) {
  return value === null ? "-" : `${value}ms`;
}

function statusTone(status: EvidenceStatus) {
  if (status === "failure" || status === "timeout") return "danger";
  if (status === "unavailable" || status === "not_checked") return "unknown";
  return "normal";
}

function buildPathStages(evidence: DiagnosticEvidence[]): PathStage[] {
  return [
    {
      key: "pc",
      label: "PC/어댑터",
      evidence: latestEvidence(evidence, ["ethernet"]),
    },
    {
      key: "route",
      label: "IPv4/Route",
      evidence: latestEvidence(evidence, ["ipv4", "default_route"]),
    },
    {
      key: "gateway",
      label: "게이트웨이",
      evidence: latestEvidence(evidence, ["gateway"]),
    },
    {
      key: "dns",
      label: "DNS",
      evidence: latestEvidence(evidence, ["dns_microsoft", "dns_google"]),
    },
    {
      key: "external",
      label: "외부 연결",
      evidence: latestEvidence(evidence, ["http_microsoft", "http_google"]),
    },
  ];
}
```

- [ ] **Step 5: Replace the evidence row rendering with v1 rows**

In `NetworkStatusPanel`, replace the current `rows` constant with:

```tsx
  const pathStages = buildPathStages(status.evidence);
  const rows = [
    {
      label: "PC/어댑터",
      testId: "evidence-pc",
      evidence: latestEvidence(status.evidence, ["ethernet"]),
    },
    {
      label: "IPv4/Route",
      testId: "evidence-route",
      evidence: latestEvidence(status.evidence, ["ipv4", "default_route"]),
    },
    {
      label: "게이트웨이",
      testId: "evidence-gateway",
      evidence: latestEvidence(status.evidence, ["gateway"]),
    },
    {
      label: "DNS",
      testId: "evidence-dns",
      evidence: latestEvidence(status.evidence, ["dns_microsoft", "dns_google"]),
    },
    {
      label: "Microsoft",
      testId: "evidence-microsoft",
      evidence: latestEvidence(status.evidence, ["http_microsoft"]),
    },
    {
      label: "Google",
      testId: "evidence-google",
      evidence: latestEvidence(status.evidence, ["http_google"]),
    },
  ];
```

Update `EvidenceRow` so it renders duration as its own column:

```tsx
function EvidenceRow({
  evidence,
  label,
  testId,
}: {
  evidence: DiagnosticEvidence;
  label: string;
  testId: string;
}) {
  return (
    <li className={styles["evidence-row"]} data-testid={testId}>
      <span className={styles["evidence-name"]}>{label}</span>
      <strong data-status={evidence.status}>
        {evidenceLabels[evidence.status]}
      </strong>
      <span className={styles["evidence-duration"]}>
        {formatDuration(evidence.durationMs)}
      </span>
      <span className={styles["evidence-detail"]}>
        {evidence.detail ?? "세부 정보 없음"}
      </span>
      <span className={styles["evidence-time"]}>
        {evidence.checkedAt ? (
          <time dateTime={evidence.checkedAt}>{formatTime(evidence.checkedAt)}</time>
        ) : (
          "확인 시각 없음"
        )}
      </span>
    </li>
  );
}
```

- [ ] **Step 6: Render the diagnostic path**

In `NetworkStatusPanel.tsx`, render this block between the summary `<dl>` and the latest evidence block:

```tsx
      <div className={styles["path"]} aria-label="구간별 진단 경로">
        {pathStages.map((stage) => (
          <div
            className={styles["path-stage"]}
            data-tone={statusTone(stage.evidence.status)}
            data-testid={`path-${stage.key}`}
            key={stage.key}
          >
            <span className={styles["path-label"]}>{stage.label}</span>
            <strong>{evidenceLabels[stage.evidence.status]}</strong>
          </div>
        ))}
      </div>
```

- [ ] **Step 7: Update CSS for the path and wider evidence rows**

In `src/features/network-diagnostics/NetworkStatusPanel.module.css`, add:

```css
.path {
  display: grid;
  grid-template-columns: repeat(5, minmax(0, 1fr));
  gap: 8px;
  margin-top: 16px;
}

.path-stage {
  min-width: 0;
  border: 1px solid #263b57;
  border-radius: 8px;
  padding: 10px;
  background: #0d1522;

  strong {
    display: block;
    margin-top: 5px;
    color: #bac8d9;
    font-size: 0.78rem;
  }
}

.path-stage[data-tone="normal"] {
  border-color: #315d82;
}

.path-stage[data-tone="danger"] {
  border-color: #7e4148;
  background: #25161d;

  strong {
    color: #e8a3aa;
  }
}

.path-stage[data-tone="unknown"] {
  color: #8394aa;
}

.path-label {
  display: block;
  overflow-wrap: anywhere;
  color: #d7e1ee;
  font-size: 0.78rem;
  font-weight: 700;
}
```

Replace the `.evidence-row` grid template with:

```css
  grid-template-columns: 100px 86px 64px minmax(160px, 1fr) 160px;
```

Add `.evidence-duration` to the existing detail/time min-width rule:

```css
.evidence-duration,
.evidence-detail,
.evidence-time {
  min-width: 0;
}
```

Inside the existing `@media (max-width: 700px)` block, add:

```css
  .path {
    grid-template-columns: 1fr;
  }

  .evidence-duration,
  .evidence-detail,
  .evidence-time {
    grid-column: 1 / -1;
    text-align: left;
  }
```

- [ ] **Step 8: Verify the focused test passes**

Run:

```bash
npm test -- src/features/network-diagnostics/NetworkStatusPanel.test.tsx
```

Expected: PASS.

- [ ] **Step 9: Commit Task 2**

```bash
git add src/features/network-diagnostics/fixture.ts src/features/network-diagnostics/NetworkStatusPanel.tsx src/features/network-diagnostics/NetworkStatusPanel.module.css src/features/network-diagnostics/NetworkStatusPanel.test.tsx
git commit -m "feat: show network diagnostic path"
```

---

### Task 3: Collapsible Raw Probe

**Files:**
- Modify: `src/features/network-probe/NetworkProbePanel.test.tsx`
- Modify: `src/features/network-probe/NetworkProbePanel.tsx`
- Modify: `src/features/network-probe/NetworkProbePanel.module.css`

- [ ] **Step 1: Add failing tests for collapsed raw JSON**

In `src/features/network-probe/NetworkProbePanel.test.tsx`, replace the first test with:

```tsx
  it("runs once and keeps formatted raw JSON behind a disclosure", async () => {
    getSnapshotMock.mockResolvedValue(snapshot);
    const user = userEvent.setup();
    render(<NetworkProbePanel />);

    await user.click(screen.getByRole("button", { name: "네트워크 확인 실행" }));

    expect(getSnapshotMock).toHaveBeenCalledTimes(1);
    expect(screen.getByText("Raw JSON")).toBeInTheDocument();
    expect(screen.queryByText(/"collector": "windows-native"/)).not.toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Raw JSON 펼치기" }));

    expect(screen.getByText(/"collector": "windows-native"/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Raw JSON 접기" })).toBeInTheDocument();
  });
```

Update the copy test so it expands before clicking copy:

```tsx
  it("copies the displayed JSON", async () => {
    getSnapshotMock.mockResolvedValue(snapshot);
    const user = userEvent.setup();
    render(<NetworkProbePanel />);
    await user.click(screen.getByRole("button", { name: "네트워크 확인 실행" }));
    await user.click(screen.getByRole("button", { name: "Raw JSON 펼치기" }));

    await user.click(screen.getByRole("button", { name: "JSON 복사" }));

    await expect(navigator.clipboard.readText()).resolves.toBe(
      JSON.stringify(snapshot, null, 2),
    );
    expect(screen.getByText("복사했습니다.")).toBeInTheDocument();
  });
```

- [ ] **Step 2: Run the focused test and verify it fails**

Run:

```bash
npm test -- src/features/network-probe/NetworkProbePanel.test.tsx
```

Expected: FAIL because JSON is visible immediately and there is no disclosure button.

- [ ] **Step 3: Add disclosure state**

In `src/features/network-probe/NetworkProbePanel.tsx`, add state near the other `useState` calls:

```tsx
  const [isJsonExpanded, setIsJsonExpanded] = useState(false);
```

Inside `runProbe`, set the disclosure closed before the command starts:

```tsx
    setIsJsonExpanded(false);
```

- [ ] **Step 4: Render raw JSON only when expanded**

Replace the current `{json && (...)}` result block with:

```tsx
      {json && (
        <div className={styles["result"]}>
          <div className={styles["result-header"]}>
            <span>Raw JSON</span>
            <button
              type="button"
              onClick={() => setIsJsonExpanded((current) => !current)}
            >
              {isJsonExpanded ? "Raw JSON 접기" : "Raw JSON 펼치기"}
            </button>
          </div>
          {isJsonExpanded && (
            <>
              <pre>{json}</pre>
              <button type="button" onClick={copyJson}>
                JSON 복사
              </button>
              {copyMessage && (
                <p className={styles["copy-message"]}>{copyMessage}</p>
              )}
            </>
          )}
        </div>
      )}
```

- [ ] **Step 5: Adjust result CSS for the new button position**

In `src/features/network-probe/NetworkProbePanel.module.css`, add:

```css
.result > button {
  margin-top: 10px;
}
```

- [ ] **Step 6: Verify the focused test passes**

Run:

```bash
npm test -- src/features/network-probe/NetworkProbePanel.test.tsx
```

Expected: PASS.

- [ ] **Step 7: Commit Task 3**

```bash
git add src/features/network-probe/NetworkProbePanel.tsx src/features/network-probe/NetworkProbePanel.module.css src/features/network-probe/NetworkProbePanel.test.tsx
git commit -m "feat: collapse raw network probe output"
```

---

### Task 4: Final Verification

**Files:**
- No new files.
- Verify all changed frontend files and docs.

- [ ] **Step 1: Run all frontend tests**

Run:

```bash
npm test
```

Expected: PASS.

- [ ] **Step 2: Run the production build**

Run:

```bash
npm run build
```

Expected: PASS.

- [ ] **Step 3: Run Rust checks if frontend changes revealed contract drift**

Run this only if TypeScript changes require a Rust contract adjustment:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: PASS. If these commands are skipped because no Rust or Tauri contract changed, record that in the final handoff.

- [ ] **Step 4: Inspect the final diff**

Run:

```bash
git diff --stat HEAD~3..HEAD
git diff HEAD~3..HEAD -- src/features/network-diagnostics src/features/network-probe
```

Expected:

- `NetworkStatusPanel` contains status description, path stages, and evidence rows.
- `NetworkProbePanel` still invokes only `get_network_probe_snapshot` on button click.
- No latency graph, health score, router action, packet capture, external upload, driver UI, or performance monitoring UI was added.

- [ ] **Step 5: Commit verification notes only if files changed**

If no files changed during verification, do not create a commit. If a small test or style fix was required, commit it:

```bash
git add src/features/network-diagnostics src/features/network-probe
git commit -m "test: verify main network ui v1"
```
