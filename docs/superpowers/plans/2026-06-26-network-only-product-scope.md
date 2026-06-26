# Network-Only Product Scope Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove driver management from the current product surface and durable guidance so PC Health is a network diagnostics-only app.

**Architecture:** Treat this as a scope cleanup, not a new feature. Keep the existing network diagnostic runtime, status panel, and raw probe intact; remove only driver planning UI and guidance. Historical plans may keep past driver mentions, but current routing docs and skills must not present driver management as active or planning work.

**Tech Stack:** React, TypeScript, CSS Modules, Vitest, Rust/Tauri test suite for regression checks

---

## File map

- Modify: `src/features/product-home/ProductHome.tsx` — remove the driver management product card and network/driver header copy.
- Modify: `src/features/product-home/ProductHome.test.tsx` — assert the home screen is network-only.
- Modify: `AGENTS.md` — define PC Health as a network diagnostics-only app.
- Modify: `README.md` — remove driver management status.
- Modify: `docs/harness/pc-health/team-spec.md` — remove driver role, planning pipeline, handoff and scenarios.
- Modify: `.agents/skills/pc-health-orchestrator/SKILL.md` — route only network diagnostics work.
- Modify: `.agents/skills/qa-safety-reviewer/SKILL.md` — remove driver action checks.
- Modify: `.agents/skills/repo-conventions-specialist/SKILL.md` — remove driver-planning wording.
- Modify: `.agents/skills/ui-experience-specialist/SKILL.md` — remove driver UI wording.
- Delete: `.agents/skills/driver-inventory-specialist/SKILL.md`

## Task 1: Remove driver management from the app surface

**Files:**
- Modify: `src/features/product-home/ProductHome.test.tsx`
- Modify: `src/features/product-home/ProductHome.tsx`

- [ ] **Step 1: Write the failing UI test**

Change the product home test to require only network diagnostics:

```tsx
it("shows network diagnostics as the only product area", () => {
  render(<ProductHome />);

  expect(
    screen.getByRole("heading", { name: "인터넷 장애 진단" }),
  ).toBeInTheDocument();
  expect(screen.getByText("활성")).toBeInTheDocument();
  expect(
    screen.queryByRole("heading", { name: "드라이버 관리" }),
  ).not.toBeInTheDocument();
  expect(screen.queryByText("기획 중")).not.toBeInTheDocument();
  expect(screen.getByText("현재 네트워크 진단 상태")).toBeInTheDocument();
  expect(screen.queryByText(/성능 모니터/)).not.toBeInTheDocument();
});
```

- [ ] **Step 2: Verify RED**

Run: `npm test -- ProductHome.test.tsx`

Expected: fail because the driver management heading and planning text still render.

- [ ] **Step 3: Remove the driver card and header copy**

In `ProductHome.tsx`, keep only the internet diagnostics area and replace the header copy with:

```tsx
<p>
  인터넷 장애의 원인 구간을 근거와 함께 구분하는 Windows
  유틸리티입니다.
</p>
```

- [ ] **Step 4: Verify GREEN**

Run: `npm test -- ProductHome.test.tsx`

Expected: pass.

## Task 2: Realign current product guidance

**Files:**
- Modify: `AGENTS.md`
- Modify: `README.md`
- Modify: `docs/harness/pc-health/team-spec.md`
- Modify: `.agents/skills/pc-health-orchestrator/SKILL.md`
- Modify: `.agents/skills/qa-safety-reviewer/SKILL.md`
- Modify: `.agents/skills/repo-conventions-specialist/SKILL.md`
- Modify: `.agents/skills/ui-experience-specialist/SKILL.md`
- Delete: `.agents/skills/driver-inventory-specialist/SKILL.md`

- [ ] **Step 1: Update durable guidance**

Rewrite the current guidance so it states:

- PC Health is a network diagnostics-only Windows desktop app.
- Network diagnostics is active.
- Driver management and performance monitoring are removed.
- The app remains read-only and does not mutate router, OS network settings, or collect user traffic.

- [ ] **Step 2: Delete the driver specialist skill**

Delete `.agents/skills/driver-inventory-specialist/SKILL.md`.

- [ ] **Step 3: Verify current guidance no longer routes driver work**

Run:

```bash
rg -n "드라이버 관리|driver-inventory|driver planning|driver-planning|Windows Update 후보|rollback|reboot|설치된 드라이버" AGENTS.md README.md docs/harness .agents/skills src
```

Expected: the only current-source match is the ProductHome test assertion that verifies the removed heading is absent. Current README, AGENTS, team spec and skills must not route driver work.

## Task 3: Full verification and commit

**Files:**
- All changed files from Tasks 1 and 2

- [ ] **Step 1: Run frontend verification**

Run:

```bash
npm test
npm run build
```

Expected: both pass.

- [ ] **Step 2: Run Rust verification**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: both pass.

- [ ] **Step 3: Commit**

Run:

```bash
git add AGENTS.md README.md docs/harness/pc-health/team-spec.md .agents/skills src/features/product-home docs/superpowers/specs/2026-06-26-network-only-product-scope-design.md docs/superpowers/plans/2026-06-26-network-only-product-scope.md
git commit -m "chore: narrow product scope to network diagnostics"
```
