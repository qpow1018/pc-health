# LibreHardwareMonitor Harness Realignment Design

## Goal

Realign the repository guidance around the decision that LibreHardwareMonitor is the authoritative Windows provider for performance monitoring. This change updates planning and agent workflow documents only. Application code changes belong to a later implementation plan.

## Product Areas And Lifecycle

PC Health has three major product areas:

| Product area | Lifecycle | Current direction |
| --- | --- | --- |
| Performance monitoring | Active | Build on LibreHardwareMonitor |
| Internet outage diagnostics | Planning | Distinguish PC, router, and external-line failures after a dedicated design cycle |
| Driver management | Planning | Define inventory, update availability, and action history after a dedicated design cycle |

The planning areas retain lightweight specialist skills that describe purpose, unanswered planning questions, and prohibited high-risk actions. They must not contain implementation contracts that have not been approved through a design cycle.

## Documentation Hierarchy

### `AGENTS.md`

`AGENTS.md` remains short and repository-wide. It records:

- the three product areas and their lifecycle;
- LibreHardwareMonitor as the authoritative Windows performance provider;
- the read-only product boundary;
- macOS mock development and Windows hardware validation;
- pointers to deeper specifications, the harness team spec, and role skills.

It does not contain sensor-selection rules, helper protocol details, or feature-specific implementation playbooks.

### `docs/harness/pc-health/team-spec.md`

The team spec owns lifecycle routing, role topology, handoff artifacts, failure policy, and the active performance-monitoring pipeline. It distinguishes active implementation work from planning-only work.

### `.agents/skills/`

Each skill contains only role-specific judgment and workflow. Repository-wide facts stay in `AGENTS.md`; cross-role sequencing stays in the team spec. Skills refer to those sources instead of copying their rules.

### `docs/superpowers/specs/`

Existing specifications remain as historical records. This design supersedes the parts of older performance specifications that describe LibreHardwareMonitor as a fallback or limit the provider direction to CPU, NVIDIA GPU, and system memory.

## Performance Monitoring Architecture

The target Windows data path is:

```text
LibreHardwareMonitorLib
  -> .NET sensor helper
  -> Rust collector
  -> SensorSnapshot
  -> React dashboard
```

LibreHardwareMonitor is the authoritative provider for all supported performance hardware and sensors, including CPU, GPU, memory, storage, motherboard, fan, voltage, temperature, load, clock, and power readings.

The long-term helper is a persistent app-managed process. It opens LibreHardwareMonitor once, updates hardware on the sampling interval, and closes on application shutdown. The target architecture does not start a new helper process for every sample.

Rust owns helper lifecycle, IPC, input validation, domain mapping, and Tauri delivery. Rust does not maintain a parallel Windows performance provider for values that LibreHardwareMonitor supplies. macOS development continues to use deterministic mock snapshots.

The authoritative `SensorSnapshot` contract remains in `src-tauri/src/domain.rs`. Expanding the device and reading model is a later code-design task; this harness realignment does not decide the final UI inventory or alter the contract.

## Integration Validation Boundary

The repository treats LibreHardwareMonitor as the selected provider. Validation therefore evaluates the application integration rather than reopening provider selection.

Windows validation must establish that:

- the packaged helper starts and stops correctly;
- hardware, subhardware, sensor type, sensor name, identifier, unit, and raw value cross the helper boundary correctly;
- identifiers and device association remain stable across repeated samples;
- application selection rules map sensors to the intended device and reading;
- missing, null, zero, non-finite, or physically implausible values do not become available readings;
- unsupported or not-yet-mapped sensors remain explicit;
- one-second sampling does not overlap or create unreasonable load.

Some sensors can require administrator privileges. The application does not elevate automatically. Permission-dependent readings remain unavailable until a separate permission policy is designed and approved.

## Role Changes

### PC Health Orchestrator

The orchestrator first classifies work by product area and lifecycle. Performance-monitoring requests may enter the implementation pipeline. Internet-diagnostics and driver-management requests remain in planning or design until their specifications are approved.

### Hardware Telemetry Specialist

The hardware specialist owns:

- the LibreHardwareMonitor helper and Rust boundary;
- full hardware and sensor inventory;
- identifier, type, unit, value, and device association;
- invalid-value and unavailable-state policy;
- mapping into `SensorSnapshot`;
- deterministic mocks and Windows verification evidence.

The skill must not assume that only CPU, NVIDIA GPU, and memory exist. Whether every raw sensor appears in the product UI remains a product-design decision.

### Network Diagnostics And Driver Inventory Specialists

These skills remain discoverable but become planning-only. They retain:

- product purpose;
- questions that a future design must answer;
- read-only and safety boundaries;
- prohibited actions such as router mutation, packet capture without approval, automatic driver installation, or implicit privilege elevation.

They do not prescribe unapproved result contracts, implementation order, external targets, vendor sources, or UI behavior.

### QA Safety Reviewer

QA reviews LibreHardwareMonitor helper packaging, lifecycle, IPC failure, sensor misassociation, invalid-value promotion, unsupported states, permission requirements, and accidental implementation of planning-only features.

### Repository Conventions And UI Specialists

Existing code-layout, contract-alignment, import, CSS, and desktop-utility UI rules remain. CPU/GPU-card-specific assumptions are generalized to a variable device and sensor inventory where they appear in reusable guidance.

## Error And State Policy

Guidance must distinguish:

- unsupported device or missing sensor;
- helper startup, shutdown, or IPC failure;
- insufficient permission;
- null, zero, non-finite, or physically implausible values;
- a raw sensor that exists but has no approved product mapping;
- hardware behavior that has not yet been verified on Windows.

No missing or failed value is substituted with zero. Raw provider sensors and user-facing representative readings are separate concepts. A sensor can be collected successfully without being approved for dashboard presentation.

## Files To Realign

- `AGENTS.md`
- `docs/harness/pc-health/team-spec.md`
- `.agents/skills/pc-health-orchestrator/SKILL.md`
- `.agents/skills/hardware-telemetry-specialist/SKILL.md`
- `.agents/skills/network-diagnostics-specialist/SKILL.md`
- `.agents/skills/driver-inventory-specialist/SKILL.md`
- `.agents/skills/qa-safety-reviewer/SKILL.md`
- `.agents/skills/repo-conventions-specialist/SKILL.md`
- `.agents/skills/ui-experience-specialist/SKILL.md`

The repo-local `harness` meta-skill itself is not changed because its general harness-design rules remain valid.

## Validation

The documentation change is complete when:

1. `AGENTS.md`, the team spec, and role skills use the same lifecycle and provider terminology.
2. No active guidance treats LibreHardwareMonitor as a fallback, an unselected provider candidate, or a CPU-temperature-only integration.
3. No active reusable guidance limits the performance inventory to CPU, NVIDIA GPU, and memory.
4. A performance-monitoring scenario routes through the LibreHardwareMonitor hardware specialist workflow.
5. Network and driver implementation requests stop at planning or design until approved specifications exist.
6. Failure scenarios prevent automatic privilege elevation and invalid sensor promotion.
7. Every edited `SKILL.md` retains valid YAML frontmatter and real internal references.
8. No application source code changes as part of this documentation-only realignment.

## Follow-Up Boundary

After this harness realignment is approved and implemented, a separate implementation design and plan will cover the persistent helper protocol, raw sensor contract, Rust process management, `SensorSnapshot` evolution, UI inventory, and migration from the current one-shot CPU-temperature helper.
