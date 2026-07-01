# Repository Agents Guide

## 제품 범위

- Net Checker는 개인용 Windows PC의 인터넷 장애 진단을 다루는 Tauri + React + Rust 데스크톱 앱이다.
- 인터넷 장애 진단은 승인된 설계를 가진 유일한 `active` 제품 영역이다.
- 성능 모니터링은 제품에서 제거되었다. LibreHardwareMonitor, sensor helper, hardware telemetry 구조를 되살리지 않는다.
- 기본 원칙은 읽기 전용과 근거 기반 판정이다. 근거가 부족하면 성공이나 원인 확정 대신 `unknown` 또는 `확인 불가`를 사용한다.

## 작업 순서

1. `docs/superpowers/specs/2026-06-26-network-only-product-scope-design.md`, 관련 최신 network spec·plan, `docs/harness/pc-health/team-spec.md`, 필요한 `.agents/skills/`를 먼저 읽는다.
2. 제품 영역과 lifecycle, 가정, 가장 작은 성공 기준을 명시한다.
3. 현재 파일 배치와 스타일을 따라 요청된 부분만 변경한다. 미래 기능을 위한 abstraction을 만들지 않는다.
4. UI 변경 전 `docs/ui-guidelines.md`를 확인하고 조용하고 밀도 있는 데스크톱 유틸리티 톤을 유지한다.
5. 변경 범위에 맞춰 `npm test`, `npm run build`, `cargo test --manifest-path src-tauri/Cargo.toml`, `cargo check --manifest-path src-tauri/Cargo.toml`을 선택해 실행한다.
6. OS API, 실제 네트워크, 패키징 동작은 Windows artifact에서 검증한다.

## 안전 경계

- router 설정, packet capture, 사용자 트래픽 수집, Windows 네트워크 설정 자동 수정, 원격 업로드를 구현하지 않는다.
- 제거된 제품 영역과 성능 모니터링 구조를 새 제품 결정 없이 복구하지 않는다.

## 지침 유지

- 반복될 repo-wide 규칙은 `AGENTS.md`, 역할·절차 지침은 `.agents/skills/` 또는 `docs/harness/` 갱신 후보로 사용자에게 제안한다.
