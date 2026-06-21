# Repository Agents Guide

## What
- PC Health는 Windows PC의 성능 모니터링, 인터넷 장애 진단, 드라이버 관리를 다루는 Tauri + React + Rust 데스크톱 앱이다.
- 성능 모니터링은 현재 활성 개발 영역이며, 인터넷 장애 진단과 드라이버 관리는 기획 단계다.
- Windows 성능 모니터링의 기준 provider는 LibreHardwareMonitor다. CPU, GPU, 메모리, 저장장치, 메인보드, 팬, 전압을 포함한 지원 가능한 전체 하드웨어 센서 범위를 대상으로 한다.
- 개발은 macOS의 결정적인 mock 데이터로 진행할 수 있지만 실제 하드웨어 동작은 Windows에서 검증한다.
- `src-tauri/src/domain.rs`의 Rust 타입이 wire contract의 기준이다. frontend 타입, mock 데이터, UI 렌더링, 테스트 fixture는 이 계약과 함께 맞춰야 한다.

## Why
- 성능 모니터링은 LibreHardwareMonitor 위에 일관된 수집 경로를 만들고 누락되거나 잘못된 값을 정상 상태로 오인하지 않는 것을 우선한다.
- 인터넷 장애 진단과 드라이버 관리는 각각의 기획이 승인되기 전에는 구현 범위로 취급하지 않는다.
- 하드웨어 제어, 자동 권한 상승, 드라이버 설치, OS·네트워크 설정 변경은 별도 위험 단계다.

## How
- 구현 전 기존 문서와 코드를 먼저 읽는다:
  - `docs/superpowers/specs/2026-06-21-lhm-harness-realignment-design.md`
  - `docs/superpowers/specs/`
  - `docs/harness/pc-health/team-spec.md`
  - `.agents/skills/`
- 성능 모니터링 작업은 LibreHardwareMonitor helper, Rust collector, `SensorSnapshot`, frontend 소비자의 순서로 경계를 확인한다.
- Windows 검증은 provider 재선정이 아니라 helper 배포·생명주기·IPC, sensor mapping, invalid value 처리, sampling 부하를 확인한다.
- 인터넷 장애 진단과 드라이버 관리 요청은 승인된 설계가 생길 때까지 기획 또는 설계 단계에서 멈춘다.
- 현재 파일 배치와 스타일을 우선하고, 단일 사용 abstraction이나 미래 대비 구조를 만들지 않는다.
- UI를 추가하거나 바꿀 때는 `docs/ui-guidelines.md`를 먼저 확인하고, 기존 dashboard의 조용하고 밀도 있는 데스크톱 유틸리티 톤을 유지한다.
- 변경 범위에 맞춰 필요한 검증만 실행한다:
  - `npm test`
  - `npm run build`
  - `cargo test --manifest-path src-tauri/Cargo.toml`
  - `cargo check --manifest-path src-tauri/Cargo.toml`

## Maintenance
- 작업 중 반복될 repo 규칙, UI 패턴, 라우팅 구조, 초기 설정, 검증 명령이 새로 정해지면 바로 수정하지 말고 사용자에게 지침 갱신 후보로 제안한다.
- 짧은 repo-wide 규칙은 `AGENTS.md`, 역할별/절차별 지침은 `.agents/skills/` 또는 `docs/harness/`에 두도록 제안한다.
