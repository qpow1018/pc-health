---
name: repo-conventions-specialist
description: Use when pc-health React, TypeScript, Rust, Tauri, SQLite, network-diagnostics, or driver-planning changes must match existing placement, naming, contracts, and tests.
---

# Repo Conventions Specialist

## 작업 흐름

1. 현재 tree, 가까운 source와 test, `package.json`, `src-tauri/Cargo.toml`을 먼저 읽는다.
2. 기존 파일 배치와 naming을 따르고 실제 반복이 없으면 abstraction을 만들지 않는다.
3. OS raw data, Rust domain·command, frontend type·consumer와 SQLite record 경계를 구분한다.
4. wire contract가 바뀌면 Rust, TypeScript, fixture와 테스트를 함께 맞춘다.
5. 같은 frontend 폴더는 `./`, 다른 영역은 `@/...`를 사용하고 `../`는 피한다.
6. 전역 CSS는 `src/app/global.css`, feature 스타일은 plain CSS Modules와 native nesting을 사용한다.
7. JSX class는 `styles['class-name']` 형태로 접근한다.
8. 테스트는 검증 대상 module이나 component 가까이에 둔다.
9. 생성물과 build artifact를 직접 편집하지 않는다.

## 검증

- 모든 변경 줄이 요청과 연결되는가?
- unavailable·unknown·실패가 성공으로 변환되지 않는가?
- network probe와 판정은 deterministic test로 분리되는가?
- Windows 전용 동작의 검증 경계가 명시적인가?

반복될 새 규칙은 즉시 넓히지 말고 `AGENTS.md`, team spec 또는 specialist skill 갱신 후보로 제안한다.
