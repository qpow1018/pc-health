---
name: ui-experience-specialist
description: Use when approved pc-health surfaces need a quiet desktop-utility UI for variable hardware, sensor summaries, unavailable states, or warnings.
---

# UI Experience Specialist

## 언제 사용할지
- Dashboard layout, status hierarchy, warning/caution/normal/unknown 표시, 데스크톱 유틸리티 workflow 작업에 사용한다.
- 승인된 product surface에 variable hardware와 sensor summary를 추가할 때 사용한다.
- 필요 없이 marketing landing page, 과하게 큰 hero section, 장식적인 card 나열, 새 design system을 만들지 않는다.

## 필요한 입력
- `src/features/dashboard/Dashboard.tsx`
- `src/features/dashboard/DeviceCard.tsx`
- `src/features/dashboard/Dashboard.module.css`
- `src/features/dashboard/DeviceCard.module.css`
- `src/app/global.css`
- 현재 sensor state label과 warning copy.

## 작업 흐름
1. Layout 변경을 제안하기 전에 기존 UI component와 CSS를 확인한다.
2. 앱은 조용하고 실용적으로 유지한다. 빠르게 훑어볼 수 있을 만큼 밀도 있게 만들되, 시각적으로 과장하지 않는다.
3. 상태 의미를 일관되게 사용한다:
   - normal: 조치가 필요 없음
   - caution: 주의를 기울이면 좋음
   - danger: 사용자 조치나 즉시 확인이 필요할 가능성이 큼
   - unknown: 근거가 부족하거나 reading이 지원되지 않음
4. raw sensor inventory와 user-facing summary를 구분하고 모든 raw sensor를 dashboard에 자동 노출하지 않는다.
5. variable device count 때문에 모든 장치와 sensor를 하나의 복잡한 card에 밀어 넣지 않는다.
6. unsupported, permission-dependent, not-yet-mapped, waiting, error 상태를 성공과 구분해 보이게 한다.
7. storage, motherboard, fan, voltage view는 승인된 UI 설계 없이 자동 생성하지 않는다.
8. 새 global design system보다 현재 component를 최소 확장하는 방식을 선호한다.
9. 사용할 수 없는 데이터에도 안정적인 layout을 유지한다.
10. 스타일은 plain CSS Modules와 native CSS nesting을 사용한다. 전역 reset, body, token은 `src/app/global.css`에만 두고, feature/component 스타일은 가까운 `*.module.css`에 둔다.
11. JSX에서 CSS Module class는 `styles['class-name']` bracket access로 사용한다. 같은 폴더 import에는 `./`를 허용하고, 상위 폴더 접근 `../`는 피하며 다른 영역 접근은 `@/...` alias를 사용한다.

## 출력
- 정보 구조 메모를 위한 `_workspace/02_ui_experience_findings.md`.
- 제안된 component 경계와 state label.
- 시각 동작이 바뀔 때 screenshot 또는 test 근거.

## 검증
- 중요한 status text가 겹치거나 container 크기를 예측 불가능하게 바꾸면 안 된다.
- Unknown과 unsupported 상태가 성공처럼 보여서는 안 된다.
- Warning은 위험을 과장하지 않으면서 가능한 원인을 설명해야 한다.
