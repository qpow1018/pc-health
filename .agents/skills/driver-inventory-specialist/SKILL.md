---
name: driver-inventory-specialist
description: Use when planning pc-health driver versions, update availability, action history, installation, rollback, or privilege boundaries before implementation is approved.
---

# 드라이버 관리 Specialist

## 현재 Lifecycle
- 드라이버 관리는 `planning` 영역이다.
- approved design이 lifecycle을 바꾸기 전에는 inventory, update-check, action history, download, install, rollback code를 구현하지 않는다.

## 언제 사용할지
- 설치된 버전, 업데이트 가능 여부, 처리 기록의 제품 범위를 기획할 때 사용한다.
- vendor source, confidence, offline behavior, 설치와 rollback의 위험 단계를 나눌 때 사용한다.

## 필요한 입력
- 사용자가 지원하려는 device와 vendor 범위.
- 업데이트 정보의 신뢰도와 freshness 요구.
- download, install, reboot, rollback, action history에 대한 기대와 위험 허용도.
- `docs/harness/pc-health/team-spec.md`의 lifecycle과 안전 정책.

## 기획 질문
- 지원 device와 vendor 범위는 무엇인가?
- installed version과 available version의 authoritative source는 무엇인가?
- confidence, stale data, offline behavior를 어떻게 표현할 것인가?
- download, install, reboot, rollback, action history를 어떤 위험 phase로 나눌 것인가?
- 처리 기록에는 시도, 성공, 실패, 취소, reboot 요구 중 무엇을 보존할 것인가?

## 안전 경계
- 승인된 설계 없이 driver나 vendor tool을 download, install, rollback, 실행하지 않는다.
- 명시적 사용자 승인 없이 reboot, 자동 권한 상승, OS 변경을 구현하지 않는다.
- 새 버전이 있다는 이유만으로 현재 driver가 unsafe하다고 표현하지 않는다.

## 출력
- `_workspace/02_driver_inventory_findings.md`의 planning 질문, source 후보, 위험 단계.
- 구현 contract가 아닌 사용자 검토용 설계 제안.

## 검증
- authoritative source와 confidence가 아직 선택되지 않았음을 숨기지 않는가?
- inventory와 시스템 변경 단계를 섞지 않는가?
- approved design과 추가 안전 승인 전 구현 금지가 명확한가?
