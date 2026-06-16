---
name: hardware-telemetry-specialist
description: pc-health CPU/GPU/메모리 센서 수집, Rust snapshot contract, mock collector 시나리오, warning evaluation 작업에 사용한다.
---

# 하드웨어 Telemetry Specialist

## 언제 사용할지
- CPU, NVIDIA GPU, 메모리, 저장장치, sensor reading, warning threshold, `SensorSnapshot`, `MockCollector`, Tauri sensor command 작업에 사용한다.
- Rust domain 타입, frontend sensor 타입, mock 시나리오, 테스트가 서로 어긋날 수 있을 때 사용한다.
- telemetry contract도 함께 바뀌는 경우가 아니라면 네트워크 진단이나 드라이버 update 정책에는 사용하지 않는다.

## 필요한 입력
- `docs/superpowers/specs/2026-06-15-windows-performance-monitor-design.md`
- `docs/superpowers/specs/2026-06-15-mock-realtime-dashboard-design.md`
- `src-tauri/src/domain.rs`
- `src-tauri/src/collector/`
- `src-tauri/src/service.rs`
- `src-tauri/src/warning.rs`
- `src/features/sensors/types.ts`

## 작업 흐름
1. Rust `domain.rs`를 authoritative wire contract로 취급한다.
2. 모든 contract 변경을 TypeScript 타입, mock 시나리오, UI 렌더링, 테스트와 대조한다.
3. telemetry는 읽기 전용으로 유지한다. 하드웨어 제어, overclocking, fan control, 드라이버 설치 동작을 추가하지 않는다.
4. Windows 지원은 NVIDIA 우선을 선호하되, 미지원 장치를 숨기지 말고 명시적으로 표현한다.
5. 사용자가 contract 변경을 요청하지 않았다면 안정적인 device와 reading 순서를 보존한다.
6. 사용할 수 없는 상태는 `unsupported-device`, `unsupported-app`, `waiting`, `error`로 보이게 유지하고, 절대 0으로 대체하지 않는다.
7. UI 동작에 기대기 전에 변경한 layer 가까이에 테스트를 추가한다.

## 출력
- 여러 단계 작업에서는 `_workspace/02_hardware_telemetry_findings.md`에 contract 메모를 남긴다.
- 작업이 요구할 때만 Rust/TypeScript contract 파일을 갱신한다.
- Rust 테스트와 frontend contract consumer를 포함하는 검증 근거.

## 검증
- Rust 타입이 바뀌면 serialization 이름과 enum tag를 확인한다.
- UI에 보이는 telemetry가 바뀌면 모든 `SensorValue` 상태의 frontend 렌더링을 확인한다.
- warning evaluation 위에 새 service를 쌓기 전에 `warning.rs` 동작을 다시 검토한다.
