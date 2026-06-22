---
name: pc-health-orchestrator
description: Use when pc-health work needs product-area classification, lifecycle routing, cross-boundary planning, implementation sequencing, or safety review.
---

# PC Health 오케스트레이터

## 기준

- 인터넷 장애 진단은 `active`다.
- 드라이버 관리는 `planning`이다.
- 성능 모니터링은 `removed`다.

## 흐름

1. `docs/superpowers/specs/2026-06-21-product-pivot-network-driver-design.md`, team spec, 관련 최신 spec·plan과 코드를 읽는다.
2. 요청을 제품 영역과 lifecycle로 분류하고 가정과 성공 기준을 적는다.
3. 네트워크 구현은 `network-diagnostics-specialist`, UI·배치·안전 경계는 필요한 specialist만 선택한다.
4. 드라이버 요청은 `driver-inventory-specialist`로 설계까지만 진행한다.
5. 가장 작은 변경을 구현하고 변경 표면에 맞춰 검증한다.
6. 외부 요청, 로컬 보존, 권한 또는 시스템 변경이 관련되면 `qa-safety-reviewer`로 검토한다.

## 차단

- 새 제품 결정 없이 성능 provider, helper, sensor contract나 dashboard를 복구하지 않는다.
- 승인 없이 드라이버 구현이나 시스템 변경을 시작하지 않는다.
- 근거 없는 네트워크 원인 확정, packet capture, router 변경, 외부 기록 업로드를 허용하지 않는다.

## 출력

복잡한 작업에서만 `_workspace/01_request_summary.md`, 필요한 specialist finding, `_workspace/03_design_plan.md`, `_workspace/04_qa_review.md`를 사용한다.
