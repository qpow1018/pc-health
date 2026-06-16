---
name: repo-conventions-specialist
description: pc-health React, TypeScript, Rust, Tauri 변경을 기존 파일 배치, naming, contract, 테스트와 맞춰야 할 때 사용한다.
---

# Repo Conventions Specialist

## 언제 사용할지
- 파일 배치, naming, 테스트, shared type, abstraction 도입 여부를 결정할 때 사용한다.
- Rust, Tauri IPC, frontend 타입, mock 데이터, fixture를 넘나드는 변경에 사용한다.
- 관련 없는 코드를 refactor하는 데 사용하지 않는다.

## 필요한 입력
- 현재 repository tree.
- `package.json`, `vite.config.ts`, 관련 `src/` 파일.
- `src-tauri/Cargo.toml`, 관련 `src-tauri/src/` 파일.
- 변경 지점 근처의 기존 테스트.

## 작업 흐름
1. 새 구조를 만들기 전에 기존 style과 파일 배치에 맞춘다.
2. 반복되는 실제 복잡성이 있을 때만 abstraction을 추가한다.
3. Rust wire contract, frontend TypeScript 타입, mock 데이터, 테스트를 동기화한다.
4. Frontend 테스트는 현재 repo 방식처럼 component나 module 근처에 둔다.
5. Frontend import는 같은 폴더의 `./`를 허용하고, 상위 폴더 접근 `../`는 피한다. 다른 app/page/feature 영역 접근은 `@/...` 단일 alias를 사용한다.
6. Frontend 스타일은 `src/app/global.css`에 reset, body, token 같은 전역 규칙만 두고, feature/component 스타일은 plain CSS Modules(`*.module.css`)와 native CSS nesting을 사용한다. JSX에서는 `styles['class-name']` bracket access를 사용한다.
7. 더 넓은 integration test가 정당화되지 않는 한 Rust 테스트는 검증 대상 module 안에 둔다.
8. AGENTS.md는 짧고 repo-wide하게 유지하고, 긴 절차는 `.agents/skills/` 또는 `docs/harness/`에 둔다.

## 지침 갱신 후보 감지
작업 중 다음이 새로 정해지면 최종 답변에 지침 갱신 후보로 제안한다:

- UI layout, 상태 표시, component 배치가 반복 패턴으로 굳어짐.
- Routing, page, feature folder 구조가 새 기준으로 정해짐.
- 초기 설정, dev command, test command, Windows 검증 절차가 바뀜.
- Rust wire contract와 frontend type 동기화 방식이 더 구체화됨.
- Frontend import alias, CSS Modules, global CSS 같은 코드 스타일 규칙이 바뀜.
- 같은 설명을 다음 작업에서도 반복해야 할 가능성이 큼.

바로 수정하지 말고 사용자 승인 후 반영한다. 짧은 repo-wide 규칙은 `AGENTS.md`, 역할별/절차별 내용은 `.agents/skills/` 또는 `docs/harness/`를 우선 제안한다.

## 출력
- 여러 파일 작업의 배치와 contract 메모를 위한 `_workspace/02_repo_conventions_findings.md`.
- 제안된 변경에 대한 최소 file map.
- 변경한 표면에 맞춘 verification command.

## 검증
- 변경된 모든 줄은 사용자 요청으로 추적 가능해야 한다.
- 명시적으로 바꾸지 않았다면 기존 no-data와 unsupported 상태는 호환성을 유지해야 한다.
- Generated artifact나 build artifact를 손으로 편집하지 않는다.
