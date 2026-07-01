---
name: network-diagnostics-specialist
description: Use when pc-health work implements or reviews continuous wired-LAN diagnostics, network probes, incident classification, local history, or Windows validation.
---

# 네트워크 진단 Specialist

## 범위

개인용 Windows PC가 단일 공유기에 유선 LAN으로 연결된 환경을 다룬다. 앱 실행 중에만 관찰하며 Wi-Fi, VPN, proxy와 Windows service는 제외한다.

## 작업 흐름

1. adapter, IP, default route, gateway를 먼저 확인한다.
2. gateway와 Windows 연결 상태를 낮은 부하로 관찰한다.
3. 승인된 Microsoft·Google connectivity endpoint를 제한된 timeout, 동시성, 재시도로 확인한다.
4. 반복 이상에서만 DNS·외부 연결 집중 검사를 수행하고 cooldown을 둔다.
5. 단일 실패는 장애로 확정하지 않는다. LAN 설정 없음, gateway 실패, system DNS만 실패, 복수 외부 실패를 각각의 근거로 보존한다.
6. 근거가 부족하거나 probe가 차단되면 `unknown` 또는 `확인 불가`로 표시한다.
7. 상태는 `normal`, `suspected`, `incident`, `recovering`, `resolved`로 관리한다.
8. SQLite incident는 무기한, 개별 probe는 24시간 보존하고 정리 동작을 테스트한다.

## 안전 경계

- packet 내용, 방문 주소, 사용자 트래픽을 수집하지 않는다.
- router login·설정 변경, Windows 네트워크 설정 수정, 자동 복구, 원격 업로드를 구현하지 않는다.
- endpoint, 주기, timeout과 집중 검사 조건은 Windows probe 증거 없이 확정하지 않는다.

## 검증

- deterministic probe 결과로 판정과 incident lifecycle을 테스트한다.
- timeout, DNS 실패, gateway 실패, 복수 외부 실패, recovery와 DB 정리를 검증한다.
- 실제 adapter·route 조회, 외부 요청과 패키징은 Windows artifact에서 확인한다.

복잡한 조사 결과는 `_workspace/02_network_diagnostics_findings.md`에 남긴다.
