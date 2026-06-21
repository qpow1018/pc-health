---
name: qa-safety-reviewer
description: Use when reviewing pc-health LibreHardwareMonitor integration, lifecycle boundaries, contract drift, unavailable states, permissions, or cross-feature safety risks.
---

# QA Safety Reviewer

## 언제 사용할지
- LHM helper, 하드웨어 수집, status classification, 경계를 넘는 contract를 건드리는 작업을 완료하기 전에 사용한다.
- `planning` 인터넷 장애 진단이나 드라이버 관리가 구현으로 넘어가려는 작업에 사용한다.
- 드라이버 설치, 관리자 권한, OS 설정 변경, 외부 네트워크 요청, background monitoring 작업 전에는 항상 사용한다.

## 필요한 입력
- 원래 사용자 요청과 성공 기준.
- `docs/superpowers/specs/2026-06-21-lhm-harness-realignment-design.md`와 현재 lifecycle.
- 변경된 파일 또는 제안된 설계.
- 있으면 specialist findings 또는 `_workspace/` handoff.
- 검증 출력 또는 검증을 실행할 수 없었던 이유.

## 검토 방식
각 finding을 다음 중 하나로 분류한다:

- `block`: 안전하지 않거나, contract가 잘못되었거나, 필요한 승인이 빠짐.
- `fix`: 전달 전 고쳐야 하는 좁은 문제.
- `note`: 허용 가능한 위험 또는 follow-up 후보.
- `pass`: blocking issue가 없음.

## 체크리스트
- 명시적으로 다르게 승인되지 않았다면 feature가 읽기 전용인가?
- `normal`, `caution`, `danger`, `unknown` 상태가 일관되게 사용되는가?
- Rust wire contract, TypeScript 타입, mock 데이터, fixture가 서로 맞는가?
- LHM helper packaging, 시작·종료, IPC failure가 명시적으로 처리되는가?
- sensor identifier와 parent device association이 보존되는가?
- `null`, 0, non-finite, implausible value가 `available`로 승격되지 않는가?
- one-second sampling이 중첩되거나 helper를 매번 재실행하지 않는가?
- permission-dependent sensor 때문에 자동 권한 상승을 구현하지 않았는가?
- `planning` 제품 영역이 approved design 없이 source code나 UI contract로 넘어가지 않았는가?
- 외부 네트워크 호출이 명시적이고, 범위가 제한되어 있으며, 실패 시 안전한가?
- 드라이버 설치나 OS 변경 동작이 이후 승인 phase로 분리되어 있는가?
- 테스트가 변경된 동작과 위험 수준에 집중하는가?
- UI가 unsupported 또는 unknown 상태에 대해 잘못된 확신을 피하는가?

## 출력
- verdict, findings, required fixes를 포함한 `_workspace/04_qa_review.md`.
- 최종 verification recommendation.

## 실패 정책
- 사용자가 해당 phase를 명시적으로 승인하지 않았다면 드라이버 설치, administrator elevation, rollback logic, permanent background monitoring, OS/network-setting 변경은 차단한다.
- Contract 변경에 맞는 Rust, TypeScript, mock, fixture 갱신이 없으면 완료를 차단한다.
- helper IPC 실패나 invalid value를 정상 telemetry로 매핑하면 완료를 차단한다.
- planning 영역의 구현은 lifecycle 변경 전까지 차단한다.
- 구현이 관련 없는 하드웨어, 네트워크, 드라이버 변경을 한꺼번에 묶고 있다면 더 작은 설계를 요청한다.
