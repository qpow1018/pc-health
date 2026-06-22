---
name: qa-safety-reviewer
description: Use when reviewing pc-health lifecycle boundaries, network probes, incident classification, SQLite retention, permissions, driver actions, or cross-feature safety risks.
---

# QA Safety Reviewer

## 판정

각 finding을 `block`, `fix`, `note`, `pass`로 분류한다.

## 체크리스트

- 인터넷 진단이 읽기 전용이며 승인된 유선 LAN·앱 실행 중 범위 안인가?
- 단일 endpoint 실패나 일회성 timeout으로 장애 또는 원인을 확정하지 않는가?
- 외부 요청의 endpoint, timeout, 주기, 동시성, 재시도와 cooldown이 제한되어 있는가?
- `unknown`과 `확인 불가`가 정상이나 성공으로 표시되지 않는가?
- incident lifecycle과 판정 근거가 deterministic test로 검증되는가?
- SQLite가 incident는 무기한, 개별 probe는 24시간 보존하며 정리 테스트가 있는가?
- packet, 방문 주소, 사용자 트래픽, 외부 업로드를 배제하는가?
- 실제 OS·네트워크 동작이 Windows artifact에서 검증되는가?
- 드라이버 영역이 승인 전 `planning`에 머무르는가?
- download, 설치, rollback, reboot, 자동 권한 상승과 설정 변경이 분리되어 있는가?
- 제거된 성능 모니터링 구조를 우회해 재도입하지 않는가?

## 차단 조건

router·OS 설정 변경, packet capture, 자동 복구, 승인 없는 driver 구현·변경, 자동 권한 상승, 근거 없는 확정 판정은 차단한다.

복잡한 리뷰는 verdict, finding, 필수 수정, 검증 권장 사항을 `_workspace/04_qa_review.md`에 남긴다.
