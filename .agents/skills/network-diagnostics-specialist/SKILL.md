---
name: network-diagnostics-specialist
description: pc-health에서 local NIC, gateway, DNS, public internet reachability를 단계적으로 진단하는 작업에 사용한다.
---

# 네트워크 진단 Specialist

## 언제 사용할지
- 인터넷 연결 진단, local adapter 상태, gateway/router reachability, DNS resolution, public reachability, network status UI contract 작업에 사용한다.
- 이후 phase에서 해당 범위를 명시적으로 승인하지 않았다면 지속적인 background monitoring에는 사용하지 않는다.

## 필요한 입력
- `src-tauri/src/lib.rs`의 현재 Tauri command 패턴.
- `src/features/dashboard/`와 `src/app/global.css`의 기존 frontend 상태와 status-display 패턴.
- 사용자가 승인한 진단 깊이와 외부 요청 허용 여부.

## 진단 순서
앱이 가능한 문제 위치를 설명할 수 있도록 가까운 대상에서 먼 대상으로 진단한다.

1. Local NIC와 IP configuration.
2. Default gateway 존재 여부와 reachability.
3. DNS server configuration과 DNS resolution.
4. 작고 명시적인 target set에 대한 public internet reachability.

## 작업 흐름
1. 읽기 전용 진단 설계와 status contract부터 시작한다.
2. Contract에서 `normal`, `caution`, `danger`, `unknown` 상태를 분리한다.
3. 한 단계의 실패가 앞선 증거를 지우지 않도록 모든 단계를 독립적으로 보고 가능하게 만든다.
4. 짧은 timeout과 명확한 실패 이유를 사용한다.
5. MVP 하네스에서는 원격 진단, packet capture, router login, 네트워크 변경을 피한다.

## 출력
- 단계별 진단 메모를 위한 `_workspace/02_network_diagnostics_findings.md`.
- 구현 전 제안된 Rust/Tauri result shape.
- 구현 시 local success, gateway failure, DNS failure, public reachability failure를 다루는 테스트나 fixture.

## 검증
- 먼 단계의 실패가 가까운 단계를 실패로 표시해서는 안 된다.
- 외부 요청에는 사용자에게 보이는 명시적 목적과 안전한 timeout 동작이 필요하다.
- Offline 또는 captive-network 조건은 오해를 부르는 확신이 아니라 `unknown` 또는 범위가 제한된 실패로 표현해야 한다.
