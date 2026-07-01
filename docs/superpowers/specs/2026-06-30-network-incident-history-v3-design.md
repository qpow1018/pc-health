# 네트워크 장애 이력 v3 설계

## 목표

PC Health에 별도 `장애 이력` 화면을 추가한다. v2의 메인 화면 `최근 장애` 3개는 빠른 요약으로 유지하고, v3는 앱이 실행 중 확정해 저장한 전체 incident 목록을 훑어보고 필터링하는 화면을 제공한다.

이번 단계는 전체 incident 목록과 기본 필터까지다. incident 상세 화면, timeline, raw evidence 전체 보기, chart, export, 삭제와 보존 기간 설정은 포함하지 않는다.

## 제품 범위

- 제품 영역: 인터넷 장애 진단
- lifecycle: `active`
- 대상 데이터: v2 SQLite에 저장된 `network_incidents`
- 조회 범위: 저장된 전체 incident
- 원칙: 읽기 전용, 로컬 조회, 근거 기반 문구, 근거 부족 시 `확인 불가`

성능 모니터링, 드라이버 관리, packet capture, router 설정 변경, Windows 네트워크 설정 자동 수정, 자동 복구, 외부 업로드는 포함하지 않는다.

## 성공 기준

1. 메인 화면의 최근 장애 panel에서 `전체 이력 보기`로 별도 이력 화면에 이동할 수 있다.
2. 이력 화면에서 `돌아가기`로 메인 화면에 복귀할 수 있다.
3. 이력 화면은 저장된 전체 incident를 최신순으로 표시한다.
4. 사용자는 기간, 상태, 구간 필터로 목록을 좁힐 수 있다.
5. 빈 기록, 필터 결과 없음, 조회 실패가 서로 구분된다.
6. 목록 행은 v3에서 클릭 동작이나 상세 진입을 제공하지 않는다.

## 범위

### 포함

- 앱 내부 view state 기반 화면 전환
- 메인 `최근 장애` panel의 `전체 이력 보기` action
- `NetworkIncidentHistoryPage` feature-local page component
- 전체 incident 조회 Tauri command
- frontend 기간, 상태, 구간 필터
- 이력 목록 table/list
- empty, filtered empty, loading, unavailable/error 상태
- Rust와 frontend focused test

### 제외

- React Router 같은 URL routing library
- incident 상세 page, drawer, modal
- 목록 행 click action
- disabled `상세 준비 중` button
- raw evidence 전체 보기
- evidence timeline
- chart, statistics, uptime summary
- export
- incident 삭제
- 보존 기간 설정
- background service

## 화면 전환

v3에서는 URL router를 도입하지 않는다. 현재 앱은 `App -> HomePage -> ProductHome`의 단일 화면 구조이므로, 가장 작은 내부 view state로 시작한다.

화면 상태:

- `home`
- `incident_history`

기본 흐름:

```text
ProductHome
  최근 장애 panel
    전체 이력 보기 -> incident_history

NetworkIncidentHistoryPage
  돌아가기 -> home
```

이 방식은 v3 범위를 작게 유지한다. v4에서 상세 화면이 필요해지면 같은 view state를 `incident_detail`까지 확장하거나 그 시점에 routing library 도입을 다시 검토한다.

## 데이터 조회

Backend는 전체 incident 목록을 최신순으로 반환한다. 기간, 상태, 구간 필터는 frontend에서 적용한다.

선택 이유:

- v3의 목표는 작은 전체 이력 페이지 완성이다.
- 개인 PC 앱에서 incident 개수는 초기에는 작을 가능성이 높다.
- query contract를 v3부터 복잡하게 만들지 않는다.
- 기록이 많아져 성능 문제가 실제로 보이면 이후 backend filtering으로 바꿀 수 있다.

### Command contract

새 command:

```text
get_network_incidents
```

입력:

- 없음

반환:

- `NetworkIncident[]`
- 최신순
- v2의 `NetworkIncident` contract를 그대로 사용한다.

조회 실패:

- command는 오류를 반환한다.
- frontend는 `장애 이력을 불러올 수 없습니다.`를 표시한다.
- 조회 실패를 현재 네트워크 장애로 해석하지 않는다.

## 필터

필터 기본값은 모두 `전체`다.

### 기간

- 전체
- 최근 24시간
- 7일
- 30일

기간 필터는 `startedAt` 기준으로 적용한다. `resolvedAt`이나 `lastObservedAt`이 기간 안에 있더라도 `startedAt`이 범위 밖이면 제외한다. 이 규칙은 사용자가 “이 기간에 시작된 장애”를 보는 모델과 일치한다.

### 상태

- 전체
- 진행 중: `ongoing`
- 복구 확인 중: `recovering`
- 복구됨: `resolved`

### 구간

- 전체
- 로컬 연결: `local_connection`
- 공유기 또는 로컬: `gateway_or_local`
- DNS: `dns`
- 외부 연결: `external`
- 확인 불가: `unknown`

`unknown`은 원인 확정처럼 보이지 않게 `확인 불가`로 표시한다.

## 목록 정보

목록은 card grid보다 table/list 형태를 우선한다. 이 화면은 반복 기록을 훑고 비교하는 용도이므로, 행 구조가 안정적이어야 한다.

각 행의 정보:

- 상태
- 추정 구간
- 시작 시각
- 복구 시각 또는 마지막 확인 시각
- 지속 시간
- 요약

지속 시간 규칙:

- `resolvedAt`이 있으면 `resolvedAt - startedAt`
- `resolvedAt`이 없으면 `lastObservedAt - startedAt`
- 계산할 수 없으면 `-`

행 동작:

- click 없음
- 상세 link 없음
- hover로 action을 암시하지 않음

## 화면 문구

상단:

```text
장애 이력
앱 실행 중 확정된 네트워크 장애 기록을 보여줍니다.
```

메인 이동 action:

```text
전체 이력 보기
```

돌아가기 action:

```text
돌아가기
```

빈 기록:

```text
저장된 장애 기록이 없습니다.
```

필터 결과 없음:

```text
조건에 맞는 장애 기록이 없습니다.
```

조회 실패:

```text
장애 이력을 불러올 수 없습니다.
```

loading:

```text
장애 이력 확인 중
```

## UI 방향

- 기존 dashboard shell 폭 `min(1180px, 100%)`를 유지한다.
- 큰 hero나 마케팅형 화면을 만들지 않는다.
- 필터는 compact native select 형태로 배치한다.
- 목록은 dense하지만 모바일 폭에서 텍스트가 겹치지 않게 한 열로 접는다.
- unavailable, loading, error 상태에서도 layout이 크게 흔들리지 않게 한다.
- 새 generic design system을 만들지 않는다.

## Backend 구조

기존 v2 구조를 확장한다.

- `NetworkIncidentStore`
  - `recent_incidents(3)`는 v2 메인 panel용으로 유지한다.
  - 새 `all_incidents()`를 추가해 전체 이력 command에서 사용한다.
- `commands.rs`
  - `get_recent_network_incidents` 유지
  - `get_network_incidents` 추가
- `NetworkIncidentHistoryState`
  - v2와 같은 unavailable error 방식을 사용한다.

DB schema는 변경하지 않는다. v3는 저장 구조를 바꾸지 않고 조회 표면과 UI를 추가한다.

## Frontend 구조

새 feature-local 구성:

- `src/features/network-incidents/NetworkIncidentHistoryPage.tsx`
- `src/features/network-incidents/NetworkIncidentHistoryPage.module.css`
- `src/features/network-incidents/filter.ts`
- 기존 `src/features/network-incidents/api.ts` 확장
- `src/features/network-incidents/NetworkIncidentHistoryPage.test.tsx`

App/view state:

- `HomePage`에서 `home | incident_history` state를 가진다.
- `ProductHome`은 `onOpenIncidentHistory` callback을 받는다.
- `RecentIncidentsPanel`은 `onOpenHistory` callback이 있을 때 `전체 이력 보기` button을 표시한다.
- `NetworkIncidentHistoryPage`는 `onBack` callback을 받는다.

## 오류 정책

- 이력 조회 실패는 이력 화면 안에서만 표시한다.
- 이력 조회 실패가 현재 진단 상태를 바꾸지 않는다.
- browser 개발 환경에서 Tauri command를 사용할 수 없으면 `장애 이력을 불러올 수 없습니다.`를 표시한다.
- 필터 결과 없음은 조회 실패와 구분한다.

## 테스트

### Rust

- 전체 incident 조회가 최신순으로 반환된다.
- `get_network_incidents`가 저장소 있음 상태에서 전체 목록을 반환한다.
- 저장소 없음 상태에서 `get_network_incidents`가 unavailable 오류를 반환한다.
- 기존 `get_recent_network_incidents`는 계속 3개만 반환한다.

### Frontend

- 메인 최근 장애 panel에 `전체 이력 보기` action이 표시된다.
- action을 누르면 이력 화면으로 전환된다.
- `돌아가기` action이 메인 화면으로 복귀한다.
- 이력 화면은 전체 목록을 표시한다.
- 기간 필터가 `startedAt` 기준으로 적용된다.
- 상태 필터가 ongoing, recovering, resolved를 구분한다.
- 구간 필터가 local, gateway/local, DNS, external, unknown을 구분한다.
- 빈 기록과 필터 결과 없음 문구가 구분된다.
- command unavailable/error가 현재 장애처럼 보이지 않는다.
- 목록 행 click action이나 상세 link가 없다.

### 검증 명령

- `cargo test --manifest-path src-tauri/Cargo.toml`
- `cargo check --manifest-path src-tauri/Cargo.toml`
- `npm test`
- `npm run build`

## Windows artifact 확인

v2 artifact 확인이 어려우면 v3 구현 전 로컬 검증까지만 진행할 수 있다. 다만 실제 수용 기준은 Windows artifact에서 다음을 확인한 뒤 완료로 본다.

1. 메인 화면에서 `전체 이력 보기`로 이력 화면에 들어간다.
2. 저장된 incident가 전체 목록에 최신순으로 표시된다.
3. 앱 재시작 후에도 이력 목록이 유지된다.
4. 기간, 상태, 구간 필터가 예상대로 적용된다.
5. 이력 조회 실패가 현재 진단 UI를 중단시키지 않는다.
6. 목록 행이 상세 화면으로 이동하지 않는다.

## v4로 미루는 항목

v4에서는 incident 상세 화면을 검토한다.

- 사건별 evidence timeline
- lifecycle 전이 기록
- 대표 근거와 raw diagnostic snapshot 비교
- 사용자에게 복사 가능한 진단 요약
- 상세 화면으로 이동하는 navigation
