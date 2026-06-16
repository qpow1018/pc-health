---
name: pc-health-orchestrator
description: pc-health 기능 작업에서 repo 조사, 좁은 설계, 구현 계획, 안전 검토가 필요할 때 사용한다.
---

# PC Health 오케스트레이터

## 언제 사용할지
- 새 pc-health 기능, 경계를 넘나드는 변경, Rust, Tauri IPC, React 타입, mock 데이터, 테스트를 건드리는 작업에 사용한다.
- 요청에 하드웨어 telemetry, 네트워크 진단, 드라이버 inventory, UI 구조, repo convention 판단이 필요할 때 사용한다.
- 완전히 한 파일 안에서 끝나고 검증 방법이 이미 명확한 한 줄 수정에는 사용하지 않는다.

## 필요한 입력
- 사용자 요청과 명시된 범위.
- 현재 저장소 파일. 특히 `docs/superpowers/specs/`, `src-tauri/src/`, `src/features/sensors/`, `src/features/dashboard/`, `src/pages/`.
- 작업 트리에 이미 존재하는 변경 파일.

## 아키텍처
기본은 Pipeline이며, 마지막에 제한된 Producer-Reviewer 검토를 붙인다.

1. 변경을 제안하기 전에 기존 문서와 코드를 확인한다.
2. 요청을 분류하고 필요한 specialist만 선택한다.
3. 짧은 구현 설계나 작업 계획을 만든다.
4. 기존 파일 배치에 맞춰 필요한 부분만 수술적으로 구현한다.
5. 집중 검증을 실행한다.
6. 위험하거나 경계를 넘나드는 작업은 `qa-safety-reviewer`로 보낸다.

병렬 worker는 하드웨어 telemetry, 네트워크 진단, 드라이버 inventory, UI 구조, repo convention처럼 서로 독립적인 조사에만 허용한다. 최종 종합은 오케스트레이터가 맡는다.

## 작업 흐름
1. 가정과 성공 기준을 명시한다.
2. 파일을 바꾸기 전에 기존 구현과 관련 문서를 읽는다.
3. Specialist를 선택한다:
   - `hardware-telemetry-specialist`: 센서 수집, snapshot, warning 작업.
   - `network-diagnostics-specialist`: NIC, gateway, DNS, internet reachability 작업.
   - `driver-inventory-specialist`: 드라이버 버전, 설치일, 벤더, update-check 흐름.
   - `ui-experience-specialist`: dashboard layout, 상태 표시, 사용자 흐름.
   - `repo-conventions-specialist`: 파일 배치, naming, contract, 테스트.
   - `qa-safety-reviewer`: 최종 안전성과 테스트 공백 검토.
4. 작업에 여러 단계나 reviewer가 있을 때만 `_workspace/` 아래에 handoff를 작성한다:
   - `_workspace/01_request_summary.md`
   - `_workspace/02_hardware_telemetry_findings.md`
   - `_workspace/02_network_diagnostics_findings.md`
   - `_workspace/02_driver_inventory_findings.md`
   - `_workspace/02_ui_experience_findings.md`
   - `_workspace/02_repo_conventions_findings.md`
   - `_workspace/03_design_plan.md`
   - `_workspace/04_qa_review.md`
5. AGENTS.md는 짧고 repo-wide하게 유지한다. 긴 역할 지침은 `.agents/skills/` 또는 `docs/harness/`에 둔다.

## 출력
- 범위가 사소하지 않을 때의 좁은 설계나 계획.
- 요청되었거나 명확히 필요한 경우에만 수행한 코드 변경.
- 집중 검증 결과.
- 위험 작업에 대한 QA 메모.

## 검증
- Rust wire contract, TypeScript 타입, mock 데이터, fixture는 서로 맞아야 한다.
- 변경한 표면이 필요로 할 때 `npm test`, `npm run build`, `cargo test --manifest-path src-tauri/Cargo.toml`, `cargo check --manifest-path src-tauri/Cargo.toml`를 우선 사용한다.
- macOS mock 결과를 Windows 하드웨어 검증으로 취급하지 않는다.
