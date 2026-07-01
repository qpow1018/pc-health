# 네트워크 장애 상세 v4 설계

## 목표

PC Health의 `장애 이력` 화면에서 저장된 incident 하나를 선택해 상세 근거를 확인할 수 있게 한다. v3의 전체 이력 목록과 필터는 유지하고, v4는 사용자가 “이 장애가 왜 이 구간으로 판단됐는지”를 대표 근거 중심으로 읽을 수 있게 하는 단계다.

이번 단계는 이력 화면 안의 상세 패널까지다. 별도 URL route, raw evidence 전체 timeline, chart, export, 삭제, 보존 기간 설정은 포함하지 않는다.

## 제품 범위

- 제품 영역: 인터넷 장애 진단
- lifecycle: `active`
- 대상 데이터: v2 SQLite에 저장된 `network_incidents`와 연결된 대표 근거
- 조회 범위: v3 `get_network_incidents`가 이미 반환하는 incident 목록
- 원칙: 읽기 전용, 로컬 조회, 근거 기반 문구, 근거 부족 시 `확인 불가`

성능 모니터링, 드라이버 관리, packet capture, router 설정 변경, Windows 네트워크 설정 자동 수정, 자동 복구, 외부 업로드는 포함하지 않는다.

## 성공 기준

1. 장애 이력 목록에서 incident 하나를 선택할 수 있다.
2. 선택된 incident의 상세 패널이 같은 화면 안에 표시된다.
3. 상세 패널은 상태, 추정 구간, 시작 시각, 마지막 확인 시각, 복구 시각, 지속 시간, 요약을 보여준다.
4. 상세 패널은 대표 근거 목록을 source, status, duration, detail, checked time, observed time으로 보여준다.
5. `unknown` area와 근거 부족 상태는 `확인 불가`로 표시하고 원인 확정처럼 보이지 않는다.
6. 필터 변경으로 선택된 incident가 목록에서 사라지면 선택 상태를 해제한다.
7. 행 선택은 상세 확인 동작만 제공한다. 상세 화면에서 수정, 삭제, export, 자동 복구 동작은 제공하지 않는다.

## 범위

### 포함

- 이력 목록 행 선택 상태
- 선택된 incident 상세 패널
- 상세 패널 닫기 또는 선택 해제 action
- 대표 근거 dense list/table
- 선택된 행의 시각적 상태
- empty, filtered empty, loading, error 상태와 선택 상태의 충돌 방지
- frontend focused test

### 제외

- React Router 같은 URL routing library
- 별도 incident 상세 page
- modal 또는 drawer
- raw evidence 전체 timeline
- chart, statistics, uptime summary
- export
- incident 삭제
- 보존 기간 설정
- 자동 복구 또는 설정 변경 action
- packet capture 또는 사용자 traffic 기록
- backend schema 변경
- 새 Tauri command

## 화면 구조

v4는 v3의 `NetworkIncidentHistoryPage` 안에서 확장한다.

```text
NetworkIncidentHistoryPage
  header
  filters
  content
    incident list
    selected incident detail panel
```

데스크톱 폭에서는 목록과 상세를 두 열로 배치한다. 목록은 반복 기록을 훑는 역할이고, 상세 패널은 선택한 기록의 근거를 읽는 역할이다. 모바일 폭에서는 목록 아래에 상세 패널을 배치해 텍스트가 겹치지 않게 한다.

기록이 선택되지 않은 기본 상태에서는 상세 패널에 빈 placeholder를 크게 만들지 않는다. 목록 하단 또는 오른쪽에 조용한 문구로 다음 행동을 안내한다.

```text
기록을 선택하면 대표 근거를 확인할 수 있습니다.
```

이 문구는 기능 설명용 마케팅 문구가 아니라 비어 있는 상세 영역의 상태 표시로만 사용한다.

## 목록 행 동작

v3에서는 행에 click 동작이 없었다. v4에서는 행 선택을 명시적인 상세 확인 동작으로 바꾼다.

규칙:

- 행은 `button` 역할을 가진 요소로 구현하거나, 행 안에 `상세 보기` 버튼을 둔다.
- 추천 구현은 행 전체를 button처럼 보이게 만들기보다 목록 행 안쪽에 작고 명확한 `상세 보기` button을 두는 방식이다.
- 선택된 행은 border 또는 background로 약하게 표시한다.
- hover가 자동 복구나 수정 action처럼 보이면 안 된다.
- 선택된 incident가 필터 결과에서 사라지면 선택을 해제한다.

## 상세 패널 정보

상단 요약:

- 상태: `진행 중`, `복구 확인 중`, `복구됨`
- 추정 구간: 기존 area label
- 시작 시각
- 마지막 확인 시각
- 복구 시각 또는 `아직 복구 기록 없음`
- 지속 시간
- 요약 문장

대표 근거:

- source label
- status label
- 지연 시간 또는 `-`
- 세부 정보 또는 `세부 정보 없음`
- 실제 확인 시각 또는 `확인 시각 없음`
- 저장 관찰 시각

대표 근거가 없을 때:

```text
저장된 대표 근거가 없습니다.
```

이 상태는 장애가 없다는 뜻이 아니라 저장된 근거 snapshot이 없다는 뜻으로만 표시한다.

## Source와 Status 라벨

source는 기존 `DiagnosticEvidence.source` 범위를 그대로 사용한다.

- `ethernet`: PC/어댑터
- `ipv4`: IPv4
- `default_route`: 기본 경로
- `gateway`: 게이트웨이
- `dns_microsoft`: DNS Microsoft
- `dns_google`: DNS Google
- `http_microsoft`: Microsoft 연결
- `http_google`: Google 연결

status는 기존 evidence status를 사용자 문구로 변환한다.

- `success`: 성공
- `failure`: 실패
- `timeout`: 시간 초과
- `unavailable`: 확인 불가
- `not_checked`: 확인하지 않음

`unavailable`, `not_checked`, missing time은 성공처럼 보이지 않게 muted 처리한다.

## 데이터 흐름

새 backend command를 만들지 않는다. v3의 `get_network_incidents`는 이미 `NetworkIncident[]`와 각 incident의 `representativeEvidence`를 반환한다.

프론트엔드 상태:

- `incidents`: command 결과 전체
- `filters`: v3 필터 상태
- `selectedIncidentId`: 선택된 incident id 또는 `null`
- `filtered`: 필터 적용 결과
- `selectedIncident`: `filtered` 안에서 `selectedIncidentId`와 일치하는 incident

필터가 바뀌거나 새 조회 결과가 들어와서 `selectedIncident`가 없어지면 `selectedIncidentId`를 `null`로 정리한다.

## 오류 정책

- 이력 조회 실패 시 상세 패널도 표시하지 않는다.
- 빈 기록 상태에서는 상세 선택 영역을 표시하지 않는다.
- 필터 결과 없음 상태에서는 기존 선택을 해제하고 상세 패널을 표시하지 않는다.
- 대표 근거가 비어 있어도 조회 실패로 보지 않는다.
- 상세 패널은 현재 네트워크 상태를 바꾸지 않는다.

## UI 방향

- 기존 dashboard shell 폭 `min(1180px, 100%)`를 유지한다.
- 큰 hero, modal, drawer, 새 design system을 만들지 않는다.
- 목록과 상세는 dense하지만 안정적인 행 구조를 유지한다.
- 상세 패널은 반복 item card가 아니라 선택된 incident의 정보 panel로 둔다.
- unknown, unavailable, missing time은 muted tone으로 표시한다.
- 모바일 폭에서는 목록과 상세가 한 열로 접히며 텍스트가 겹치지 않아야 한다.

## 테스트

### Frontend

- incident 행의 `상세 보기` action으로 상세 패널이 열린다.
- 상세 패널에 상태, 구간, 시작/마지막 확인/복구 시각, 지속 시간이 표시된다.
- 대표 근거 source, status, duration, detail, checked time, observed time이 표시된다.
- 대표 근거가 없으면 `저장된 대표 근거가 없습니다.`가 표시된다.
- `unknown` area는 `확인 불가`로 표시된다.
- 필터 변경으로 선택된 incident가 사라지면 상세 패널이 닫힌다.
- loading, empty, filtered empty, error 상태에서 상세 패널이 잘못 표시되지 않는다.
- 상세 패널에 export, 삭제, 자동 복구, 설정 변경 action이 없다.

### 검증 명령

- `npm test -- src/features/network-incidents/NetworkIncidentHistoryPage.test.tsx`
- `npm test -- src/features/network-incidents`
- `npm run build`

Rust schema와 command contract는 바꾸지 않으므로 v4 구현 자체에는 Rust focused test가 필요하지 않다. 전체 릴리스 전에는 기존 전체 검증인 `cargo test --manifest-path src-tauri/Cargo.toml`, `cargo check --manifest-path src-tauri/Cargo.toml`, `npm test`, `npm run build`를 다시 수행한다.

## Windows artifact 확인

Windows Build artifact에서는 다음을 확인한다.

1. 실제 저장된 incident가 이력 목록에 표시된다.
2. `상세 보기`를 누르면 대표 근거가 표시된다.
3. 조회 실패나 Tauri command unavailable 상태가 현재 네트워크 장애처럼 보이지 않는다.
4. 상세 패널에 시스템 변경, 자동 복구, export, 삭제 동작이 없다.
