# Product Pivot Cleanup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove the performance-monitoring product surface and leave a clean, tested Tauri + React shell for the approved network-diagnostics direction and the planning-only driver area.

**Architecture:** Work from `dev` on `codex/product-pivot-cleanup`; do not merge or rewrite `codex/cpu-persistent-sensor-helper`. Replace the sensor dashboard with a static two-area product overview, reduce Rust to the Tauri shell, remove helper and sensor packaging, and realign durable guidance around network-active/driver-planning lifecycles.

**Tech Stack:** Tauri 2, Rust, React 19, TypeScript, CSS Modules, Vitest, GitHub Actions

---

## File map

### Create

- `src/features/product-home/ProductHome.tsx`: minimal two-area product overview.
- `src/features/product-home/ProductHome.module.css`: quiet desktop-utility layout for the overview.
- `src/features/product-home/ProductHome.test.tsx`: verifies the new product scope and status labels.
- `src/pages/HomePage.tsx`: page boundary for the product overview.

### Modify

- `src/app/App.tsx`: render `HomePage` instead of the sensor dashboard.
- `package.json` and `package-lock.json`: remove sensor-helper and unused Tauri frontend scripts/dependencies.
- `src-tauri/src/lib.rs`: keep only the Tauri application shell.
- `src-tauri/Cargo.toml` and `src-tauri/Cargo.lock`: remove telemetry-only and unused opener dependencies.
- `src-tauri/capabilities/default.json`: retain only `core:default`.
- `.github/workflows/windows-build.yml`: run frontend and Rust tests before packaging; remove no-longer-relevant helper assumptions.
- `README.md`: describe network diagnostics as active and driver management as planning.
- `AGENTS.md`: record the two-area product scope and lifecycle.
- `docs/harness/pc-health/team-spec.md`: replace the LHM pipeline with the approved network pipeline and driver planning boundary.
- `.agents/skills/pc-health-orchestrator/SKILL.md`: route network implementation and driver planning.
- `.agents/skills/network-diagnostics-specialist/SKILL.md`: change from planning-only to the approved active scope.
- `.agents/skills/driver-inventory-specialist/SKILL.md`: preserve planning-only, read-only feasibility, and risk boundaries.
- `.agents/skills/qa-safety-reviewer/SKILL.md`: review external probes, retention, and driver mutation boundaries.
- `.agents/skills/repo-conventions-specialist/SKILL.md`: remove telemetry contract assumptions.
- `.agents/skills/ui-experience-specialist/SKILL.md`: replace sensor-specific UI guidance with network/driver state guidance.

### Delete

- `src/pages/DashboardPage.tsx`
- `src/features/dashboard/`
- `src/features/sensors/`
- `src-tauri/src/collector/`
- `src-tauri/src/commands.rs`
- `src-tauri/src/domain.rs`
- `src-tauri/src/service.rs`
- `src-tauri/src/warning.rs`
- `src-tauri/helpers/`
- `src-tauri/binaries/`
- `scripts/build-sensor-helper.mjs`
- `src-tauri/tauri.windows.conf.json`
- `.agents/skills/hardware-telemetry-specialist/`
- performance-monitoring specs and plans superseded by `docs/superpowers/specs/2026-06-21-product-pivot-network-driver-design.md`

## Task 1: Replace the sensor dashboard with the product overview

**Files:**
- Create: `src/features/product-home/ProductHome.test.tsx`
- Create: `src/features/product-home/ProductHome.tsx`
- Create: `src/features/product-home/ProductHome.module.css`
- Create: `src/pages/HomePage.tsx`
- Modify: `src/app/App.tsx`
- Delete: `src/pages/DashboardPage.tsx`
- Delete: `src/features/dashboard/`
- Delete: `src/features/sensors/`

- [ ] **Step 1: Write the failing product-scope test**

```tsx
import { render, screen } from "@testing-library/react";
import ProductHome from "./ProductHome";

describe("ProductHome", () => {
  it("shows network diagnostics as active and driver management as planning", () => {
    render(<ProductHome />);

    expect(screen.getByRole("heading", { name: "인터넷 장애 진단" })).toBeInTheDocument();
    expect(screen.getByText("다음 구현 영역")).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "드라이버 관리" })).toBeInTheDocument();
    expect(screen.getByText("기획 중")).toBeInTheDocument();
    expect(screen.queryByText(/성능 모니터/)).not.toBeInTheDocument();
  });
});
```

- [ ] **Step 2: Run the focused test and verify RED**

Run: `npm test -- src/features/product-home/ProductHome.test.tsx`

Expected: FAIL because `./ProductHome` does not exist.

- [ ] **Step 3: Implement the minimal product overview**

`src/features/product-home/ProductHome.tsx`:

```tsx
import styles from "./ProductHome.module.css";

const areas = [
  {
    title: "인터넷 장애 진단",
    status: "다음 구현 영역",
    description: "PC, 로컬 연결, 공유기, DNS와 외부 회선 상태를 구분합니다.",
  },
  {
    title: "드라이버 관리",
    status: "기획 중",
    description: "설치 버전, Windows Update 후보와 처리 이력을 읽기 전용으로 확인합니다.",
  },
];

export default function ProductHome() {
  return (
    <main className={styles["shell"]}>
      <header className={styles["header"]}>
        <p className={styles["eyebrow"]}>READ-ONLY PC DIAGNOSTICS</p>
        <h1>PC Health</h1>
        <p>인터넷 장애 진단과 드라이버 상태 확인을 위한 Windows 유틸리티입니다.</p>
      </header>
      <section className={styles["areas"]} aria-label="제품 영역">
        {areas.map((area) => (
          <article className={styles["area"]} key={area.title}>
            <span>{area.status}</span>
            <h2>{area.title}</h2>
            <p>{area.description}</p>
          </article>
        ))}
      </section>
    </main>
  );
}
```

`src/pages/HomePage.tsx`:

```tsx
import ProductHome from "@/features/product-home/ProductHome";

export default function HomePage() {
  return <ProductHome />;
}
```

`src/app/App.tsx`:

```tsx
import "./global.css";
import HomePage from "@/pages/HomePage";

function App() {
  return <HomePage />;
}

export default App;
```

`src/features/product-home/ProductHome.module.css`:

```css
.shell {
  width: min(920px, 100%);
  margin: 0 auto;
  padding: 40px 28px;
}

.header {
  margin-bottom: 24px;

  h1 {
    margin: 0;
    font-size: clamp(2rem, 4vw, 3rem);
  }

  > p:last-child {
    margin: 10px 0 0;
    color: #8fa3bf;
  }
}

.eyebrow {
  margin: 0 0 6px;
  color: #79b8ff;
  font-size: 0.72rem;
  font-weight: 700;
  letter-spacing: 0.14em;
}

.areas {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 14px;
}

.area {
  border: 1px solid #304563;
  border-radius: 10px;
  padding: 18px;
  background: #101827;

  span {
    color: #79b8ff;
    font-size: 0.76rem;
    font-weight: 700;
  }

  h2 {
    margin: 8px 0;
    font-size: 1.1rem;
  }

  p {
    margin: 0;
    color: #9fb0c8;
    line-height: 1.6;
  }
}

@media (max-width: 700px) {
  .shell {
    padding: 28px 18px;
  }

  .areas {
    grid-template-columns: 1fr;
  }
}
```

- [ ] **Step 4: Run the focused test and verify GREEN**

Run: `npm test -- src/features/product-home/ProductHome.test.tsx`

Expected: 1 test passes.

- [ ] **Step 5: Remove the sensor UI and run the frontend suite**

Delete `src/pages/DashboardPage.tsx`, `src/features/dashboard/`, and `src/features/sensors/`.

Run: `npm test`

Expected: only the new product-home test remains and passes; no sensor test is collected.

- [ ] **Step 6: Build the frontend**

Run: `npm run build`

Expected: TypeScript and Vite build succeed without sensor imports.

- [ ] **Step 7: Commit the frontend cleanup**

```bash
git add src/app/App.tsx src/pages src/features
git commit -m "refactor: replace sensor dashboard with product overview"
```

## Task 2: Reduce Rust to the Tauri shell

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Delete: `src-tauri/src/collector/`
- Delete: `src-tauri/src/commands.rs`
- Delete: `src-tauri/src/domain.rs`
- Delete: `src-tauri/src/service.rs`
- Delete: `src-tauri/src/warning.rs`

- [ ] **Step 1: Verify the pre-cleanup guard fails**

Run:

```bash
test ! -e src-tauri/src/domain.rs && test ! -d src-tauri/src/collector
```

Expected: non-zero exit because telemetry files still exist.

- [ ] **Step 2: Replace `src-tauri/src/lib.rs` with the minimal shell**

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Delete telemetry modules**

Delete the six paths listed for this task. Do not create replacement domain or command abstractions before the network contract is designed.

- [ ] **Step 4: Verify the cleanup guard passes**

Run:

```bash
test ! -e src-tauri/src/domain.rs && test ! -d src-tauri/src/collector
```

Expected: exit 0.

- [ ] **Step 5: Run Rust verification**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`

Expected: build succeeds with zero Rust unit tests.

Run: `cargo check --manifest-path src-tauri/Cargo.toml`

Expected: exit 0.

- [ ] **Step 6: Commit the Rust cleanup**

```bash
git add src-tauri/src
git commit -m "refactor: remove performance telemetry backend"
```

## Task 3: Remove helper packaging and unused dependencies

**Files:**
- Modify: `package.json`
- Modify: `package-lock.json`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock`
- Modify: `src-tauri/capabilities/default.json`
- Modify: `.github/workflows/windows-build.yml`
- Delete: `src-tauri/helpers/`
- Delete: `src-tauri/binaries/`
- Delete: `scripts/build-sensor-helper.mjs`
- Delete: `src-tauri/tauri.windows.conf.json`

- [ ] **Step 1: Verify helper references exist before cleanup**

Run:

```bash
rg -n "sensor-helper|LibreHardwareMonitor|build:sensor-helper|externalBin" package.json scripts src-tauri .github/workflows/windows-build.yml
```

Expected: matches in package scripts, helper sources, build script, and Windows Tauri config.

- [ ] **Step 2: Remove helper files and Windows sidecar config**

Delete `src-tauri/helpers/`, `src-tauri/binaries/`, `scripts/build-sensor-helper.mjs`, and `src-tauri/tauri.windows.conf.json`.

- [ ] **Step 3: Remove unused npm dependencies and scripts**

Remove `build:sensor-helper` from `package.json`. Remove `@tauri-apps/api` and `@tauri-apps/plugin-opener` because the cleanup shell does not import them. Run `npm install` to update `package-lock.json` mechanically.

Expected: npm completes without changing the application version.

- [ ] **Step 4: Reduce Rust dependencies**

Keep only:

```toml
[dependencies]
tauri = { version = "2", features = [] }
```

Remove `tauri-plugin-opener`, `serde`, `serde_json`, `chrono`, `windows`, and `windows-sys`. Run `cargo check --manifest-path src-tauri/Cargo.toml` to update and validate `Cargo.lock`.

- [ ] **Step 5: Remove opener capability**

Set `permissions` in `src-tauri/capabilities/default.json` to:

```json
"permissions": ["core:default"]
```

- [ ] **Step 6: Make Windows CI verify before packaging**

Insert after `npm ci`:

```yaml
      - name: Run frontend tests
        run: npm test

      - name: Run Rust tests
        run: cargo test --manifest-path src-tauri/Cargo.toml
```

Do not add .NET, helper build, or smoke-test steps.

- [ ] **Step 7: Verify packaging cleanup**

Run:

```bash
rg -n "sensor-helper|LibreHardwareMonitor|build:sensor-helper|externalBin|tauri_plugin_opener" package.json src-tauri .github/workflows/windows-build.yml
```

Expected: no matches and `rg` exits 1.

Run: `npm test && npm run build`

Expected: frontend test and build pass.

Run: `cargo test --manifest-path src-tauri/Cargo.toml && cargo check --manifest-path src-tauri/Cargo.toml`

Expected: both Rust commands exit 0.

- [ ] **Step 8: Commit packaging cleanup**

```bash
git add package.json package-lock.json scripts src-tauri .github/workflows/windows-build.yml
git commit -m "build: remove sensor helper packaging"
```

## Task 4: Realign repository guidance

**Files:**
- Modify: `README.md`
- Modify: `AGENTS.md`
- Modify: `docs/harness/pc-health/team-spec.md`
- Modify: `.agents/skills/pc-health-orchestrator/SKILL.md`
- Modify: `.agents/skills/network-diagnostics-specialist/SKILL.md`
- Modify: `.agents/skills/driver-inventory-specialist/SKILL.md`
- Modify: `.agents/skills/qa-safety-reviewer/SKILL.md`
- Modify: `.agents/skills/repo-conventions-specialist/SKILL.md`
- Modify: `.agents/skills/ui-experience-specialist/SKILL.md`
- Delete: `.agents/skills/hardware-telemetry-specialist/`
- Delete: superseded performance specs and plans

- [ ] **Step 1: Rewrite `README.md` around the two product areas**

Use this status summary:

```markdown
PC Health is a personal, read-only Windows desktop utility for diagnosing internet outages and reviewing driver state.

## Status

- Internet diagnostics: active, approved design; implementation follows cleanup.
- Driver management: planning; installed versions, Windows Update candidates, and history are feasible read-only slices.
- Performance monitoring: removed.
```

Keep stack, development, Windows artifact, and MIT sections. State that SQLite is planned for local network incident history.

- [ ] **Step 2: Rewrite `AGENTS.md` as compact operational guidance**

Include these exact repo-wide rules:

```markdown
- PC Health has two product areas: active internet diagnostics and planning-only driver management.
- The app is read-only; router mutation, packet capture, automatic network repair, driver download/install/rollback, privilege elevation, and reboot require separate approval.
- The approved network design is `docs/superpowers/specs/2026-06-21-product-pivot-network-driver-design.md`.
- macOS development uses deterministic tests; live network and Windows driver behavior require Windows validation.
- Do not recreate performance-monitoring, LibreHardwareMonitor, sensor-helper, or `SensorSnapshot` paths.
```

Keep the existing verification commands and UI-guideline pointer.

- [ ] **Step 3: Rewrite the team spec**

Define:

- network diagnostics as `active` with a staged observation/probe/classification/persistence/UI pipeline;
- driver management as `planning` with read-only feasibility but no implementation contract;
- network evidence states `normal`, `suspected`, `incident`, `recovering`, `resolved`, and `unknown`/`확인 불가` when evidence is insufficient;
- SQLite incident retention: incidents indefinite, individual probes 24 hours;
- explicit safety blocks for router mutation, packet capture, OS repair, driver mutation, elevation, and reboot.

Remove all LHM, sensor, helper, hardware inventory, and `SensorSnapshot` text.

- [ ] **Step 4: Update specialist skills**

Apply these boundaries:

- orchestrator routes network implementation and driver planning only;
- network specialist reads the approved product-pivot spec and may design/implement the approved wired-LAN personal scope;
- driver specialist stays planning-only and names SetupAPI/Configuration Manager and Windows Update Agent only as source candidates;
- QA reviews external request cadence, retry/cooldown, uncertain classification, SQLite retention, privacy, and driver mutation boundaries;
- repo conventions remove helper/wire-contract rules but retain aliases, CSS Modules, adjacent tests, and minimal abstractions;
- UI specialist covers current network state, recent incident, history, evidence confidence, and driver planning surfaces without sensor assumptions.

Delete `.agents/skills/hardware-telemetry-specialist/`.

- [ ] **Step 5: Remove superseded performance documents**

Delete:

```text
docs/superpowers/plans/2026-06-15-mock-realtime-dashboard.md
docs/superpowers/plans/2026-06-15-pc-health-project-bootstrap.md
docs/superpowers/plans/2026-06-17-windows-sensor-diagnostics.md
docs/superpowers/plans/2026-06-21-lhm-harness-realignment.md
docs/superpowers/specs/2026-06-15-mock-realtime-dashboard-design.md
docs/superpowers/specs/2026-06-15-windows-performance-monitor-design.md
docs/superpowers/specs/2026-06-17-windows-sensor-diagnostics-design.md
docs/superpowers/specs/2026-06-21-lhm-harness-realignment-design.md
```

Keep the product-pivot spec and this cleanup plan.

- [ ] **Step 6: Verify guidance consistency**

Run:

```bash
rg -n "LibreHardwareMonitor|SensorSnapshot|sensor helper|sensor-helper|성능 모니터링.*active|performance monitor" AGENTS.md README.md docs/harness .agents/skills docs/superpowers
```

Expected: no active guidance match. A historical explanation inside the product-pivot spec may mention removal; inspect and allow only those explicit removal statements.

Run:

```bash
for file in .agents/skills/*/SKILL.md; do sed -n '1,5p' "$file"; done
```

Expected: every skill starts with YAML frontmatter containing `name` and `description`.

- [ ] **Step 7: Commit guidance cleanup**

```bash
git add README.md AGENTS.md docs .agents/skills
git commit -m "docs: realign guidance around network and drivers"
```

## Task 5: Final cleanup verification and QA review

**Files:**
- Review only; modify only files directly responsible for a discovered cleanup defect.

- [ ] **Step 1: Scan the complete tracked tree for removed product references**

Run:

```bash
rg -n "LibreHardwareMonitor|SensorSnapshot|sensor-helper|cpu_temperature|gpu_temperature|memory_usage" --glob '!docs/superpowers/specs/2026-06-21-product-pivot-network-driver-design.md' --glob '!docs/superpowers/plans/2026-06-21-product-pivot-cleanup.md'
```

Expected: no matches.

- [ ] **Step 2: Run complete frontend verification**

Run: `npm test`

Expected: all collected tests pass with zero failures.

Run: `npm run build`

Expected: TypeScript and Vite build exit 0.

- [ ] **Step 3: Run complete Rust verification**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`

Expected: exit 0.

Run: `cargo check --manifest-path src-tauri/Cargo.toml`

Expected: exit 0.

- [ ] **Step 4: Review scope and safety**

Confirm from `git diff dev...HEAD`:

- no network probe, SQLite schema, driver inventory, download, install, rollback, elevation, or reboot code was added;
- only the product overview remains in the UI;
- the sensor feature branch was neither merged nor deleted;
- Windows CI still packages installers after tests.

- [ ] **Step 5: Commit only if verification required a narrow correction**

```bash
git add -u
git commit -m "fix: complete product pivot cleanup"
```

If no correction was required, do not create an empty commit.
