# PC Health UI Guidelines

이 문서는 기능이 늘어도 PC Health UI가 같은 제품처럼 보이도록 돕는 최소 규칙이다. 새 디자인 시스템을 만들기보다 현재 대시보드의 방향을 기준으로 확장한다.

## Product Feel

- 조용하고 실용적인 데스크톱 유틸리티처럼 보여야 한다.
- 마케팅 페이지, 큰 hero, 장식적인 card 나열은 피한다.
- 사용자가 빠르게 훑어볼 수 있도록 정보 밀도를 유지한다.
- 위험을 과장하지 않는다. 일반 권장 범위, 확인 필요, 측정 실패를 구분해서 말한다.

## Layout

- 기본 화면 폭은 현재 dashboard shell처럼 `min(1180px, 100%)` 안쪽에 둔다.
- 하드웨어, 네트워크, 드라이버 정보는 같은 화면에 넣더라도 모든 세부 정보를 하나의 복잡한 card에 밀어 넣지 않는다.
- 반복 정보는 grid나 list로 안정적인 위치를 유지한다.
- unavailable 상태에서도 행이나 card가 사라져 layout이 크게 흔들리지 않게 한다.
- 모바일 폭에서는 한 열로 접되, status text와 값이 겹치지 않아야 한다.

## Status Levels

UI에서 상태 의미는 일관되게 사용한다.

| Level | Meaning | Treatment |
| --- | --- | --- |
| normal | 조치가 필요 없음 | 기본 foreground와 border |
| caution | 주의를 기울이면 좋음 | amber 계열 accent |
| danger | 사용자 조치나 즉시 확인이 필요할 가능성이 큼 | red 계열 accent |
| unknown | 근거가 부족하거나 reading이 지원되지 않음 | muted text, 성공처럼 보이지 않게 표시 |

- `unsupported-device`, `unsupported-app`, `waiting`, `error`는 숨기지 않는다.
- high load는 health warning과 구분한다. 현재처럼 neutral blue 계열을 사용한다.
- warning copy는 가능한 원인이나 기준을 설명하되, 하드웨어 손상이나 안전 한계 초과를 단정하지 않는다.

## Cards And Panels

- card는 개별 device, 진단 group, 반복 item을 담을 때만 사용한다.
- page section 전체를 떠 있는 card처럼 만들지 않는다.
- card radius, border, spacing은 기존 dashboard card에 맞춘다.
- card header에는 category와 이름을 먼저 보여주고, reading/value list는 안정적인 행 구조를 유지한다.
- 새 panel이 기존 `DeviceCard`와 의미가 다르면 generic abstraction보다 feature-local component를 먼저 만든다.

## Copy Tone

- 문장은 짧고 직접적으로 쓴다.
- 앱이 시스템을 고치거나 조작하는 것처럼 말하지 않는다.
- 측정 불가, 미지원, 대기, 실패는 사용자에게 보이는 상태로 남긴다.
- 권장 범위 안내는 제조사 공식 한계값보다 낮은 일반 기준임을 명확히 한다.

## CSS And Component Rules

- 전역 reset, body, shared token은 `src/app/global.css`에만 둔다.
- feature/component 스타일은 가까운 `*.module.css`에 둔다.
- CSS Modules class는 JSX에서 `styles["class-name"]` 형태로 접근한다.
- 같은 폴더 import는 `./`를 허용하고, 상위 폴더 접근 `../`는 피한다. 다른 영역 접근은 `@/...` alias를 사용한다.
- lucide 같은 icon library가 도입되기 전에는 icon 사용을 기능상 필요한 곳으로 제한한다.
- 새 색상은 기존 foreground, border, blue, amber, red, muted scale과 어울리는지 먼저 확인한다.

## New Feature Checklist

새 UI 기능을 추가하기 전에 확인한다.

- 이 기능이 읽기 전용 진단이라는 제품 원칙을 유지하는가?
- unavailable, waiting, error 상태가 사용자에게 보이는가?
- normal, caution, danger, unknown 중 어떤 의미를 쓰는지 명확한가?
- 기존 dashboard layout이나 card/list 패턴으로 충분한가?
- 새 abstraction이 단일 사용을 넘어서 실제 중복을 줄이는가?
- 변경한 화면에 맞는 focused test나 screenshot 확인이 가능한가?
