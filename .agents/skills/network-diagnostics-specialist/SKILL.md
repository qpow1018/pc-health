---
name: network-diagnostics-specialist
description: Use when planning pc-health internet diagnostics that must distinguish PC, router, DNS, and external-line problems before implementation is approved.
---

# 네트워크 진단 Specialist

## 현재 Lifecycle
- 인터넷 장애 진단은 `planning` 영역이다.
- approved design이 lifecycle을 바꾸기 전에는 Tauri command, external request, polling, UI contract를 구현하지 않는다.

## 언제 사용할지
- PC·공유기·외부 회선 문제를 어떤 근거로 구분할지 기획할 때 사용한다.
- local adapter, gateway, DNS, captive portal, VPN, IPv6, public reachability의 범위를 결정할 때 사용한다.

## 필요한 입력
- 사용자가 원하는 진단 결과와 설명 수준.
- 허용 가능한 외부 요청, 개인정보, timeout, 실행 빈도 제약.
- `docs/harness/pc-health/team-spec.md`의 lifecycle과 안전 정책.

## 기획 질문
- 어떤 근거로 PC, 공유기, DNS, 외부 회선 문제를 구분할 것인가?
- 어떤 외부 target과 timeout이 개인정보·가용성 측면에서 허용되는가?
- captive portal, VPN, IPv6, 무선 연결을 어느 phase에서 다룰 것인가?
- 진단을 수동 실행할지 지속 monitoring할지?
- 실패와 근거 부족을 어떤 상태와 문구로 구분할 것인가?

## 안전 경계
- 승인된 설계 없이 router login·설정 변경, packet capture, 원격 진단, background monitoring을 구현하지 않는다.
- 외부 target이나 요청 목적을 임의로 정하지 않는다.
- PC·공유기·외부 회선 중 하나를 근거 없이 원인으로 확정하지 않는다.

## 출력
- `_workspace/02_network_diagnostics_findings.md`의 planning 질문, 범위 후보, 안전 위험.
- 구현 contract가 아닌 사용자 검토용 설계 제안.

## 검증
- 결과가 구현 세부사항을 미리 고정하지 않는가?
- 외부 요청과 개인정보 경계가 질문으로 남아 있는가?
- approved design 전 구현 금지가 명확한가?
