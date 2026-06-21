# LibreHardwareMonitor Harness Realignment Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Realign repository guidance so LibreHardwareMonitor is the authoritative provider for active performance-monitoring work while internet diagnostics and driver management remain planning-only.

**Architecture:** Keep `AGENTS.md` short and repository-wide, put lifecycle routing and cross-role sequencing in `docs/harness/pc-health/team-spec.md`, and keep role-specific judgment in `.agents/skills/`. Preserve existing contract, UI, and safety conventions while removing stale fallback-provider and CPU/GPU-only assumptions from active guidance.

**Tech Stack:** Markdown repository guidance, repo-local Codex skills, shell validation with `rg`, Git.

---

## File Map

| File | Responsibility after this change |
| --- | --- |
| `AGENTS.md` | Product areas, lifecycle, authoritative provider, repo-wide safety and verification rules |
| `docs/harness/pc-health/team-spec.md` | Lifecycle routing, active LHM pipeline, handoffs, failure policy |
| `.agents/skills/pc-health-orchestrator/SKILL.md` | Product-area and lifecycle classification |
| `.agents/skills/hardware-telemetry-specialist/SKILL.md` | LHM helper boundary, complete inventory, mapping, states, mocks, Windows evidence |
| `.agents/skills/network-diagnostics-specialist/SKILL.md` | Planning questions and safety boundary only |
| `.agents/skills/driver-inventory-specialist/SKILL.md` | Planning questions and safety boundary only |
| `.agents/skills/qa-safety-reviewer/SKILL.md` | LHM integration review and planning-only leakage checks |
| `.agents/skills/repo-conventions-specialist/SKILL.md` | File, contract, test, import, and CSS conventions |
| `.agents/skills/ui-experience-specialist/SKILL.md` | Variable device/sensor inventory and explicit unavailable states |

### Task 1: Establish Repository-Wide Product And Provider Rules

**Files:**
- Modify: `AGENTS.md`
- Reference: `docs/superpowers/specs/2026-06-21-lhm-harness-realignment-design.md`

- [ ] **Step 1: Capture the stale assumptions**

Run:

```bash
rg -n "provider|CPU|GPU|메모리|네트워크|드라이버" AGENTS.md
```

Expected: the file contains a generic provider-validation rule and does not identify performance monitoring as the only active product area.

- [ ] **Step 2: Replace the product summary**

Keep the existing `What / Why / How / Maintenance` shape. The revised `What` and `Why` must state:

```markdown
## What
- PC Health는 Windows PC의 성능 모니터링, 인터넷 장애 진단, 드라이버 관리를 다루는 Tauri + React + Rust 데스크톱 앱이다.
- 성능 모니터링은 현재 활성 개발 영역이며, 인터넷 장애 진단과 드라이버 관리는 기획 단계다.
- Windows 성능 모니터링의 기준 provider는 LibreHardwareMonitor다. CPU, GPU, 메모리, 저장장치, 메인보드, 팬, 전압을 포함한 지원 가능한 전체 하드웨어 센서 범위를 대상으로 한다.
- 개발은 macOS의 결정적인 mock 데이터로 진행할 수 있지만 실제 하드웨어 동작은 Windows에서 검증한다.
- `src-tauri/src/domain.rs`의 Rust 타입이 wire contract의 기준이다.

## Why
- 성능 모니터링은 LibreHardwareMonitor 위에 일관된 수집 경로를 만들고 누락되거나 잘못된 값을 정상 상태로 오인하지 않는 것을 우선한다.
- 인터넷 장애 진단과 드라이버 관리는 각각의 기획이 승인되기 전에는 구현 범위로 취급하지 않는다.
- 하드웨어 제어, 자동 권한 상승, 드라이버 설치, OS·네트워크 설정 변경은 별도 위험 단계다.
```

Under `How`, point to the approved design, team spec, and skills. Replace the generic new-provider gate with:

```markdown
- 성능 모니터링 작업은 LibreHardwareMonitor helper, Rust collector, `SensorSnapshot`, frontend 소비자의 순서로 경계를 확인한다.
- Windows 검증은 provider 재선정이 아니라 helper 배포·생명주기·IPC, sensor mapping, invalid value 처리, sampling 부하를 확인한다.
- 인터넷 장애 진단과 드라이버 관리 요청은 승인된 설계가 생길 때까지 기획 또는 설계 단계에서 멈춘다.
```

Preserve the UI-guideline pointer and the four verification commands.

- [ ] **Step 3: Verify the root guide**

Run:

```bash
wc -l AGENTS.md
rg -n "LibreHardwareMonitor|활성 개발|기획 단계|src-tauri/src/domain.rs|npm test|cargo test" AGENTS.md
```

Expected: `AGENTS.md` remains under 60 lines and every searched rule is present.

- [ ] **Step 4: Commit**

```bash
git add AGENTS.md
git commit -m "docs: define LHM performance direction"
```

### Task 2: Rebuild Team Routing Around Product Lifecycle

**Files:**
- Modify: `docs/harness/pc-health/team-spec.md`

- [ ] **Step 1: Record the old routing**

Run:

```bash
rg -n "NVIDIA 우선|Provider 통합|LibreHardwareMonitorLib|네트워크 진단|드라이버 inventory" docs/harness/pc-health/team-spec.md
```

Expected: LHM appears as one provider candidate and network/driver implementation workflows are active.

- [ ] **Step 2: Add the lifecycle table**

```markdown
## 제품 영역과 Lifecycle

| 영역 | 상태 | 하네스 동작 |
| --- | --- | --- |
| 성능 모니터링 | `active` | LibreHardwareMonitor 기반 설계·구현·검증 pipeline을 실행한다. |
| 인터넷 장애 진단 | `planning` | 요구사항과 안전 경계만 정리하며 승인된 설계 전에는 구현하지 않는다. |
| 드라이버 관리 | `planning` | 요구사항과 위험 단계를 분리하며 승인된 설계 전에는 구현하지 않는다. |
```

- [ ] **Step 3: Replace the provider gate with the active pipeline**

```markdown
## 성능 모니터링 Pipeline

1. LibreHardwareMonitor helper가 hardware, subhardware, sensor를 빠짐없이 열거한다.
2. helper contract가 type, name, identifier, parent device, unit, raw value, update error를 전달한다.
3. Rust collector가 helper 생명주기와 IPC를 관리하고 raw sensor를 domain 상태로 변환한다.
4. 승인된 대표 reading만 `SensorSnapshot`에 매핑하고 나머지는 raw inventory 또는 명시적 unavailable 상태로 유지한다.
5. frontend contract, mock, UI, warning 평가를 Rust contract와 함께 맞춘다.
6. Windows artifact에서 mapping 안정성, invalid value 처리, 권한, sampling 부하를 검증한다.
```

Also state that the target helper is persistent and app-managed, Rust does not maintain a parallel performance provider, and automatic elevation is prohibited.

- [ ] **Step 4: Restrict planning-only routing**

Replace the network success scenario with a planning scenario that produces questions and a design proposal but no Tauri command. State that driver inventory, update-check, action history, download, installation, and rollback all require an approved driver-management design; mutation and elevation additionally require explicit safety approval.

- [ ] **Step 5: Align handoffs and validate**

Keep the existing deterministic `_workspace/` filenames. Require the hardware finding to include the helper contract, sensor mapping, invalid values, and Windows evidence. Mark network and driver findings as planning artifacts.

Run:

```bash
rg -n "active|planning|성능 모니터링 Pipeline|persistent|SensorSnapshot|자동 권한" docs/harness/pc-health/team-spec.md
rg -n "새 하드웨어 provider|NVIDIA 우선" docs/harness/pc-health/team-spec.md
```

Expected: the first command finds all new concepts; the second returns no matches.

- [ ] **Step 6: Commit**

```bash
git add docs/harness/pc-health/team-spec.md
git commit -m "docs: align harness with LHM pipeline"
```

### Task 3: Make Hardware Telemetry The Active LHM Specialist

**Files:**
- Modify: `.agents/skills/hardware-telemetry-specialist/SKILL.md`

- [ ] **Step 1: Record three baseline scenarios**

```text
Scenario A: "GPU와 SSD 센서를 추가해줘" -> inventory LHM sensors first, then design mappings; do not create a second provider.
Scenario B: "온도 값이 0이니 그대로 표시해줘" -> reject available mapping and preserve unavailable/error.
Scenario C: "관리자 권한으로 자동 재실행해줘" -> stop for a separate permission-policy design.
```

Read the current skill and record which decision is ambiguous. Expected: A remains biased toward CPU/NVIDIA GPU/memory and the candidate-provider gate can reopen provider selection.

- [ ] **Step 2: Rewrite selection and inputs**

Make the frontmatter description start with `Use when` and name LHM, helper, sensor inventory, `SensorSnapshot`, mock, and warning work. Required inputs must include the 2026-06-21 design, team spec, helper project, Rust domain/collector/service/warning files, and frontend sensor types.

- [ ] **Step 3: Replace the provider gate**

The workflow must explicitly:

1. Keep LHM as the authoritative Windows performance provider.
2. Preserve hardware, subhardware, sensor type, name, identifier, parent, unit, and raw value.
3. Target an app-managed persistent helper instead of per-sample execution.
4. Assign lifecycle, IPC, validation, and domain mapping to Rust without a parallel provider.
5. Separate raw inventory from representative UI readings.
6. Reject `null`, zero, `NaN`, infinite, and implausible values.
7. Avoid automatic elevation and expose permission-dependent sensors as unavailable.
8. Synchronize Rust contract changes with TypeScript, mock, UI, warning, and fixtures.

Include CPU, GPU, memory, storage, motherboard, fan, voltage, temperature, load, clock, and power without promising that every raw sensor is displayed.

- [ ] **Step 4: Validate the specialist**

Run:

```bash
rg -n "LibreHardwareMonitor|persistent|subhardware|identifier|storage|motherboard|fan|voltage|SensorSnapshot|권한" .agents/skills/hardware-telemetry-specialist/SKILL.md
rg -n "새 provider|OpenHardwareMonitor|NVML|NVIDIA 우선" .agents/skills/hardware-telemetry-specialist/SKILL.md
```

Expected: the first command finds the complete LHM boundary; the second returns no matches.

- [ ] **Step 5: Commit**

```bash
git add .agents/skills/hardware-telemetry-specialist/SKILL.md
git commit -m "docs: focus telemetry skill on LHM"
```

### Task 4: Align Orchestration And Planning Specialists

**Files:**
- Modify: `.agents/skills/pc-health-orchestrator/SKILL.md`
- Modify: `.agents/skills/network-diagnostics-specialist/SKILL.md`
- Modify: `.agents/skills/driver-inventory-specialist/SKILL.md`

- [ ] **Step 1: Update orchestrator routing**

Classify product area and lifecycle before specialist selection. Route active performance work to `hardware-telemetry-specialist`; allow planning/design work for internet and driver areas; block their implementation until an approved design changes lifecycle. Preserve the single-agent default and limited reviewer behavior.

- [ ] **Step 2: Reduce network guidance to planning questions**

Keep discovery keywords for PC/router/external-line diagnosis. Replace fixed implementation order and result contract with:

```markdown
- 어떤 근거로 PC, 공유기, DNS, 외부 회선 문제를 구분할 것인가?
- 어떤 외부 target과 timeout이 개인정보·가용성 측면에서 허용되는가?
- captive portal, VPN, IPv6, 무선 연결을 어느 phase에서 다룰 것인가?
- 진단을 수동 실행할지 지속 monitoring할지?
```

Retain prohibitions on router mutation, packet capture, remote diagnostics, background monitoring, and external calls without an approved design.

- [ ] **Step 3: Reduce driver guidance to planning questions**

Replace NVIDIA-first and predefined contract details with:

```markdown
- 지원 device와 vendor 범위는 무엇인가?
- installed version과 available version의 authoritative source는 무엇인가?
- confidence, stale data, offline behavior를 어떻게 표현할 것인가?
- download, install, reboot, rollback, action history를 어떤 위험 phase로 나눌 것인가?
```

Retain prohibitions on download, installation, rollback, reboot, automatic elevation, and OS mutation without approved designs and explicit authorization.

- [ ] **Step 4: Validate routing**

Run:

```bash
rg -n "active|planning|approved design|hardware-telemetry-specialist" .agents/skills/pc-health-orchestrator/SKILL.md
rg -n "기획|공유기|외부 회선|승인된 설계" .agents/skills/network-diagnostics-specialist/SKILL.md
rg -n "기획|authoritative source|action history|승인된 설계" .agents/skills/driver-inventory-specialist/SKILL.md
rg -n "진단 순서|NVIDIA 우선|inventory contract" .agents/skills/network-diagnostics-specialist/SKILL.md .agents/skills/driver-inventory-specialist/SKILL.md
```

Expected: the first three commands find lifecycle rules; the final command returns no matches.

- [ ] **Step 5: Commit**

```bash
git add .agents/skills/pc-health-orchestrator/SKILL.md .agents/skills/network-diagnostics-specialist/SKILL.md .agents/skills/driver-inventory-specialist/SKILL.md
git commit -m "docs: route planning-only product areas"
```

### Task 5: Align QA, Repository, And UI Rules

**Files:**
- Modify: `.agents/skills/qa-safety-reviewer/SKILL.md`
- Modify: `.agents/skills/repo-conventions-specialist/SKILL.md`
- Modify: `.agents/skills/ui-experience-specialist/SKILL.md`

- [ ] **Step 1: Extend QA**

Require review of helper packaging and lifecycle, IPC failure, sensor/device association, invalid-value promotion, permission requirements, non-overlapping sampling, contract drift, and accidental implementation of planning-only areas. Preserve `block / fix / note / pass`.

- [ ] **Step 2: Generalize repository conventions**

Preserve file placement, Rust/TypeScript contract alignment, nearby tests, import aliases, CSS Modules, and instruction-update rules. Add the helper project and helper-to-Rust JSON contract to telemetry inputs. Replace fixed device assumptions with variable hardware/reading inventories.

- [ ] **Step 3: Generalize UI guidance**

Preserve the quiet desktop-utility tone and explicit states. Add:

```markdown
- raw sensor inventory and user-facing summaries are separate;
- variable device counts must not force every sensor into one dashboard;
- unsupported, permission-dependent, not-yet-mapped, waiting, and error states remain visible;
- storage, motherboard, fan, or voltage views require an approved UI design rather than automatic card generation.
```

- [ ] **Step 4: Validate cross-cutting rules**

Run:

```bash
rg -n "helper|IPC|invalid|permission|planning" .agents/skills/qa-safety-reviewer/SKILL.md
rg -n "helper|JSON|variable|contract" .agents/skills/repo-conventions-specialist/SKILL.md
rg -n "raw sensor|variable|storage|motherboard|permission" .agents/skills/ui-experience-specialist/SKILL.md
```

Expected: every command finds its new boundary.

- [ ] **Step 5: Commit**

```bash
git add .agents/skills/qa-safety-reviewer/SKILL.md .agents/skills/repo-conventions-specialist/SKILL.md .agents/skills/ui-experience-specialist/SKILL.md
git commit -m "docs: align LHM review boundaries"
```

### Task 6: Validate The Whole Harness

**Files:**
- Verify: `AGENTS.md`
- Verify: `docs/harness/pc-health/team-spec.md`
- Verify: `.agents/skills/*/SKILL.md`

- [ ] **Step 1: Scan active guidance for stale assumptions**

```bash
rg -n "LHM fallback|Windows sensor fallback|새 Windows 하드웨어 provider|NVIDIA 우선|CPU, NVIDIA GPU, and system memory" AGENTS.md docs/harness/pc-health .agents/skills
```

Expected: no matches. Historical specifications may retain superseded wording.

- [ ] **Step 2: Verify required lifecycle language**

```bash
rg -l "LibreHardwareMonitor" AGENTS.md docs/harness/pc-health/team-spec.md .agents/skills/hardware-telemetry-specialist/SKILL.md .agents/skills/qa-safety-reviewer/SKILL.md
rg -l "planning" docs/harness/pc-health/team-spec.md .agents/skills/pc-health-orchestrator/SKILL.md .agents/skills/network-diagnostics-specialist/SKILL.md .agents/skills/driver-inventory-specialist/SKILL.md
```

Expected: each command lists every supplied path.

- [ ] **Step 3: Verify skill frontmatter**

```bash
for file in .agents/skills/*/SKILL.md; do sed -n '1,5p' "$file"; done
```

Expected: every file begins with `---` and contains non-empty `name` and `description` fields.

- [ ] **Step 4: Verify references and diff hygiene**

```bash
test -f docs/superpowers/specs/2026-06-21-lhm-harness-realignment-design.md
test -f docs/harness/pc-health/team-spec.md
test -f src-tauri/helpers/sensor-helper/PcHealth.SensorHelper.csproj
test -f src-tauri/src/domain.rs
test -f src/features/sensors/types.ts
git diff --check
```

Expected: every command exits with status 0.

- [ ] **Step 5: Confirm source code was untouched**

```bash
git diff --name-only HEAD~5..HEAD | rg '^(src/|src-tauri/src/|src-tauri/helpers/)'
```

Expected: no matches; the implementation commits contain no application or helper source changes.

- [ ] **Step 6: Record final status**

```bash
git status --short
git log -6 --oneline
```

Expected: the worktree is clean and the documentation implementation commits are visible after the design commit.
