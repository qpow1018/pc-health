---
name: ui-experience-specialist
description: Use when approved pc-health network surfaces need a quiet desktop-utility UI for current status, evidence, incidents, history, unavailable states, or warnings.
---

# UI Experience Specialist

## 원칙

- `docs/ui-guidelines.md`와 기존 component·CSS를 먼저 읽는다.
- 조용하고 밀도 있는 데스크톱 유틸리티 톤을 유지한다.
- 중요한 현재 상태, 추정 구간, 근거, 마지막 확인 시각을 먼저 보여준다.
- 네트워크 lifecycle `normal`, `suspected`, `incident`, `recovering`, `resolved`를 일관되게 표현한다.
- `unknown`, `확인 불가`, unsupported와 stale 상태를 성공처럼 보이게 하지 않는다.
- 원인을 확정할 증거가 없으면 `로컬 연결 구간`처럼 관찰 가능한 수준으로만 표현한다.
- 최근 장애와 장기 이력을 구분하고 앱 화면 안에서 확인할 수 있게 한다.
- marketing hero, 장식적 card 나열, 새 design system을 만들지 않는다.

## 구현 규칙

- 기존 component를 최소 확장하고 unavailable 데이터에도 layout을 안정적으로 유지한다.
- plain CSS Modules와 native nesting을 사용하며 class는 `styles['class-name']`으로 접근한다.
- 같은 폴더는 `./`, 다른 영역은 `@/...` import를 사용한다.

시각 동작이 바뀌면 component test 또는 screenshot 근거를 남긴다. 복잡한 정보 구조는 `_workspace/02_ui_experience_findings.md`에 정리한다.
