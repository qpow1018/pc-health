---
name: pc-health-orchestrator
description: Use when pc-health work needs product-area classification, lifecycle routing, cross-boundary design, implementation planning, or safety review.
---

# PC Health 오케스트레이터

## 언제 사용할지
- 새 기능이나 여러 파일·contract를 넘나드는 변경에 사용한다.
- 요청이 성능 모니터링, 인터넷 장애 진단, 드라이버 관리 중 어디에 속하는지 판단해야 할 때 사용한다.
- 한 파일 안의 명확한 사소한 수정에는 사용하지 않는다.

## 필요한 입력
- 사용자 요청과 명시된 범위.
- `docs/superpowers/specs/2026-06-21-lhm-harness-realignment-design.md`.
- `docs/harness/pc-health/team-spec.md`.
- 관련 spec, source, test와 현재 작업 트리.

## Lifecycle Routing
1. 요청을 제품 영역으로 분류한다.
2. team spec에서 현재 lifecycle을 확인한다.
3. `active` 성능 모니터링은 설계·계획·구현·검증으로 진행할 수 있다.
4. `planning` 인터넷 장애 진단과 드라이버 관리는 요구사항·설계까지만 진행한다.
5. planning 영역은 approved design이 lifecycle을 바꾸기 전까지 source code, Tauri command, UI contract를 구현하지 않는다.

## Specialist 선택
- `hardware-telemetry-specialist`: LHM helper, sensor inventory, `SensorSnapshot`, mock, warning, Windows evidence.
- `network-diagnostics-specialist`: PC·공유기·외부 회선 구분을 위한 planning.
- `driver-inventory-specialist`: 버전·업데이트 가능 여부·처리 기록과 위험 단계 planning.
- `ui-experience-specialist`: 승인된 product surface의 layout과 상태 표시.
- `repo-conventions-specialist`: 파일 배치, naming, contract, 테스트.
- `qa-safety-reviewer`: 안전성, lifecycle 위반, contract drift, 검증 공백.

제품 영역과 lifecycle을 판정한 뒤 필요한 specialist만 선택한다. 독립적인 조사만 제한적으로 병렬화하고 최종 종합은 오케스트레이터가 맡는다.

## 작업 흐름
1. 가정, 성공 기준, 제품 영역, lifecycle을 명시한다.
2. 관련 설계와 코드를 변경 전에 읽는다.
3. 범위가 작지 않으면 `_workspace/01_request_summary.md`와 필요한 `_workspace/02_*_findings.md`를 만든다.
4. 구현 가능한 요청만 `_workspace/03_design_plan.md` 또는 implementation plan으로 넘긴다.
5. 현재 파일 배치와 스타일에 맞춰 수술적으로 변경한다.
6. 변경한 표면에 맞춰 검증한다.
7. 위험하거나 경계를 넘는 변경은 `_workspace/04_qa_review.md`로 검토한다.

## 검증
- 성능 작업은 LibreHardwareMonitor 기준 경로를 유지하는가?
- planning 영역이 승인 없이 구현으로 넘어가지 않았는가?
- Rust wire contract, TypeScript, mock, UI, fixture가 맞는가?
- macOS mock을 Windows 하드웨어 증거로 취급하지 않았는가?
