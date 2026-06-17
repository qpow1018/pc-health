# Repository Agents Guide

## What
- PC Health는 Windows 하드웨어와 네트워크 상태를 읽기 전용으로 진단하는 Tauri + React + Rust 데스크톱 앱이다.
- 개발은 macOS에서 결정적인 mock 데이터로 진행할 수 있지만, 실제 하드웨어 동작은 Windows에서 검증해야 한다.
- `src-tauri/src/domain.rs`의 Rust 타입이 wire contract의 기준이다. frontend 타입, mock 데이터, UI 렌더링, 테스트 fixture는 이 계약과 함께 맞춰야 한다.

## Why
- MVP는 자동화나 시스템 변경보다 안정적인 읽기 전용 상태 표시를 우선한다.
- 미지원, 대기, 오류 상태는 숨기지 말고 UI에 명시적으로 보여줘야 한다.
- 드라이버 설치, 관리자 권한 상승, OS 설정 변경, 상시 백그라운드 모니터링은 별도 위험 단계로 분리한다.

## How
- 구현 전 기존 문서와 코드를 먼저 읽는다:
  - `docs/superpowers/specs/`
  - `docs/harness/pc-health/team-spec.md`
  - `.agents/skills/`
- 새 Windows 하드웨어 provider나 센서 라이브러리를 도입할 때는 먼저 diagnostics/harness로 raw 센서 목록과 값의 신뢰성을 검증하고, 그 증거가 생긴 뒤 dashboard `SensorSnapshot` 값에 연결한다.
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
