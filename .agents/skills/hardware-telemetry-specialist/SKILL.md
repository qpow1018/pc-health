---
name: hardware-telemetry-specialist
description: Use when pc-health work involves LibreHardwareMonitor, the sensor helper, hardware inventory, SensorSnapshot, deterministic mocks, warnings, or Windows telemetry validation.
---

# 하드웨어 Telemetry Specialist

## 언제 사용할지
- LibreHardwareMonitor helper, CPU, GPU, 메모리, 저장장치, 메인보드, 팬, 전압 sensor 작업에 사용한다.
- temperature, load, clock, power 같은 raw reading을 `SensorSnapshot`이나 UI 대표값으로 연결할 때 사용한다.
- Rust domain, TypeScript 타입, mock, warning, fixture가 함께 바뀔 수 있는 작업에 사용한다.
- 인터넷 장애 진단이나 드라이버 관리 기획에는 사용하지 않는다.

## 필요한 입력
- `docs/superpowers/specs/2026-06-21-lhm-harness-realignment-design.md`
- `docs/harness/pc-health/team-spec.md`
- `src-tauri/helpers/sensor-helper/`
- `src-tauri/src/domain.rs`
- `src-tauri/src/collector/`
- `src-tauri/src/service.rs`
- `src-tauri/src/warning.rs`
- `src/features/sensors/types.ts`

## 기준 아키텍처

```text
LibreHardwareMonitorLib
  -> persistent .NET sensor helper
  -> Rust collector
  -> SensorSnapshot
  -> frontend
```

- LibreHardwareMonitor는 Windows 성능 sensor의 기준 provider다.
- helper는 앱이 관리하는 persistent 프로세스를 목표로 하며 sample마다 재실행하지 않는다.
- Rust는 helper lifecycle, IPC, validation, domain mapping을 담당하고 별도 성능 provider를 병행하지 않는다.
- macOS 개발과 frontend 테스트는 결정적인 mock을 사용하며 Windows 하드웨어 증거로 취급하지 않는다.

## 작업 흐름
1. helper에서 hardware, subhardware, sensor type, name, identifier, parent device, unit, raw value, update error를 보존한다.
2. raw sensor inventory와 user-facing 대표 reading을 별도 개념으로 유지한다.
3. sensor를 선택할 때 identifier, parent, type, name, unit을 함께 사용하고 선택·제외 이유를 diagnostics에 남긴다.
4. `null`, `0`, `NaN`, 무한대, 물리적으로 말이 안 되는 값은 `available`로 승격하지 않는다.
5. 권한이 필요한 sensor 때문에 자동 권한 상승을 구현하지 않고 permission-dependent 또는 unavailable 상태를 명시한다.
6. 승인된 대표 reading만 authoritative `SensorSnapshot`에 연결한다.
7. Rust contract가 바뀌면 TypeScript, mock, UI, warning, fixture를 함께 맞춘다.
8. Windows artifact에서 반복 sample의 identifier, device association, 값의 타당성, sampling 중첩과 부하를 검증한다.

모든 raw sensor를 dashboard에 자동으로 표시하지 않는다. 저장장치, 메인보드, 팬, 전압처럼 새 UI surface가 필요한 경우 별도 제품·UI 설계를 먼저 승인받는다.

## 출력
- 여러 단계 작업의 `_workspace/02_hardware_telemetry_findings.md`.
- helper contract와 raw hardware/sensor inventory.
- 대표 reading의 선택·제외 규칙과 invalid value 처리 근거.
- Rust contract, frontend consumer, mock, warning, Windows 검증 결과.

## 검증
- helper packaging, 시작, 종료, IPC failure가 명시적으로 드러나는가?
- 반복 sample에서 identifier와 parent device가 안정적인가?
- missing, invalid, permission-dependent 값이 0이나 성공 상태로 대체되지 않는가?
- `SensorSnapshot` 변경이 TypeScript, mock, UI, warning, fixture와 일치하는가?
- one-second sampling이 중첩되거나 helper를 반복 실행하지 않는가?
