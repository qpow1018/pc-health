---
name: driver-inventory-specialist
description: Use when planning pc-health driver inventory, Windows Update candidates, update history, source confidence, privilege boundaries, installation, or rollback.
---

# 드라이버 관리 Specialist

## Lifecycle

드라이버 관리는 `planning`이다. 승인된 설계가 lifecycle을 바꾸기 전에는 source code, Tauri command, UI contract를 구현하지 않는다.

## 설계 범위

- 설치 정보: SetupAPI 또는 Configuration Manager API의 device, provider, version, date, INF 범위를 검토한다.
- 업데이트 후보: WUA 검색 결과를 `Windows Update에서 발견된 후보`로 표현한다. vendor 전체의 최신 버전으로 단정하지 않는다.
- 처리 이력: WUA history의 범위를 명시하고 수동·vendor 설치의 완전한 이력으로 표현하지 않는다.
- offline, stale source, confidence와 관리자 권한 없는 읽기 동작을 정의한다.

## 차단

- driver·vendor utility download, 설치, rollback, reboot, 자동 권한 상승, Windows Update 설정 변경은 별도 설계와 승인 전까지 금지한다.
- 새 후보가 있다는 사실을 현재 driver의 위험 판정으로 바꾸지 않는다.

## 출력

`_workspace/02_driver_inventory_findings.md`에 source, 필드, 한계, 불확실성, 위험 단계를 정리한다. 구현 contract는 만들지 않는다.
