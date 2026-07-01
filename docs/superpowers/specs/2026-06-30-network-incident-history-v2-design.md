# 네트워크 장애 기록 v2 설계

## 목표

PC Health가 앱 실행 중 확인한 네트워크 장애를 로컬 SQLite에 저장하고, 메인 화면에 최근 장애 3개를 보여준다. 사용자는 현재 상태뿐 아니라 방금 지나간 장애가 언제 시작됐고 어떤 구간으로 의심됐는지 확인할 수 있어야 한다.

이번 단계는 저장소와 메인 화면 요약까지다. 전체 이력 페이지, 상세 timeline, chart, export, 사용자 설정은 포함하지 않는다.

## 제품 범위

- 제품 영역: 인터넷 장애 진단
- lifecycle: `active`
- 대상 환경: 개인용 Windows PC, 유선 LAN, 앱 실행 중 관찰 가능한 네트워크 상태
- 저장 위치: 사용자 PC의 Tauri app data directory 아래 로컬 SQLite 파일
- 원칙: 읽기 전용, 로컬 저장, 근거 기반 판정, 근거 부족 시 `unknown` 또는 `확인 불가`

성능 모니터링, 드라이버 관리, packet capture, router 설정 변경, Windows 네트워크 설정 자동 수정, 자동 복구, 외부 업로드는 포함하지 않는다.

## 성공 기준

1. `suspected`만으로는 장애 기록이 생성되지 않는다.
2. `incident`로 확정되는 순간 새 장애 기록이 생성되거나 열린 장애 기록이 갱신된다.
3. `recovering`과 `resolved`는 같은 장애 기록의 상태와 종료 정보를 갱신한다.
4. 개별 probe observation은 24시간 뒤 정리되고, incident 기록은 유지된다.
5. 메인 화면은 최근 장애 3개를 현재 진단 상태 아래에 표시한다.
6. SQLite 저장 실패가 현재 네트워크 진단 runtime을 중단시키지 않는다.

## 범위

### 포함

- SQLite schema와 migration
- 앱 시작 시 DB 초기화
- runtime lifecycle 전이에 따른 incident recorder
- 최근 장애 3개 조회 Tauri command
- 메인 화면 최근 장애 panel
- 저장 실패와 조회 실패의 사용자 표시
- Rust와 frontend focused test

### 제외

- 전체 장애 이력 페이지
- 날짜 검색, 필터, 정렬 UI
- incident 상세 page 또는 상세 drawer
- raw evidence timeline chart
- 장애 기록 export 또는 삭제 UI
- 사용자가 보존 기간을 바꾸는 설정
- 앱 종료 후 계속 기록하는 Windows service

## 저장 모델

SQLite는 앱 내부의 단순한 로컬 파일 DB로 사용한다. 네트워크 진단 결과를 서버로 보내지 않는다.

### `network_incidents`

장애로 확정된 사건 단위 기록이다.

| column | type | rule |
| --- | --- | --- |
| `id` | `INTEGER PRIMARY KEY AUTOINCREMENT` | UI와 command에서 쓰는 안정적인 local id |
| `status` | `TEXT NOT NULL` | `ongoing`, `recovering`, `resolved` |
| `area` | `TEXT NOT NULL` | `local_connection`, `gateway_or_local`, `dns`, `external`, `unknown` |
| `started_at` | `TEXT NOT NULL` | RFC 3339 |
| `last_observed_at` | `TEXT NOT NULL` | RFC 3339 |
| `resolved_at` | `TEXT NULL` | resolved가 된 시각 |
| `summary` | `TEXT NOT NULL` | 메인 list에 표시할 짧은 한국어 요약 |

### `network_incident_evidence`

incident에 연결된 대표 근거 snapshot이다. 모든 raw probe를 무기한 보존하지 않고, incident 판단과 화면 설명에 필요한 제한된 근거만 보존한다.

| column | type | rule |
| --- | --- | --- |
| `id` | `INTEGER PRIMARY KEY AUTOINCREMENT` | local id |
| `incident_id` | `INTEGER NOT NULL` | `network_incidents(id)` 참조 |
| `source` | `TEXT NOT NULL` | 기존 `DiagnosticEvidence.source` |
| `status` | `TEXT NOT NULL` | 기존 `DiagnosticEvidence.status` |
| `checked_at` | `TEXT NULL` | 실제 확인 시각 |
| `duration_ms` | `INTEGER NULL` | 측정 시간이 있을 때만 |
| `detail` | `TEXT NULL` | 짧은 근거 문구 |
| `observed_at` | `TEXT NOT NULL` | 이 evidence snapshot을 저장한 시각 |

### `network_probe_observations`

최근 probe 동작을 재현하고 retention을 검증하기 위한 24시간 보존 기록이다. 사용자의 traffic, 방문 주소, packet 내용은 저장하지 않는다.

| column | type | rule |
| --- | --- | --- |
| `id` | `INTEGER PRIMARY KEY AUTOINCREMENT` | local id |
| `observed_at` | `TEXT NOT NULL` | RFC 3339 |
| `lifecycle` | `TEXT NULL` | observation 이후 lifecycle |
| `suspected_area` | `TEXT NULL` | observation 이후 추정 구간 |
| `evidence_json` | `TEXT NOT NULL` | 승인된 evidence 목록만 JSON으로 저장 |

`evidence_json`에는 기존 `DiagnosticEvidence` 범위만 들어간다. packet payload, 사용자 방문 URL, 브라우저 traffic, router credential은 저장하지 않는다.

## 보존 정책

- `network_incidents`: 무기한 보존한다.
- `network_incident_evidence`: 연결된 incident와 함께 보존한다.
- `network_probe_observations`: 24시간 보존한다.

24시간 정리는 앱 시작 후 DB 초기화 시 1회 수행하고, 새 observation 저장 뒤에도 수행한다. 정리 실패는 현재 진단 status를 장애로 분류하지 않는다.

## Runtime 통합

새 recorder는 state machine 밖에 둔다. state machine은 계속 순수하게 lifecycle과 suspected area를 계산하고, recorder는 이전 public status와 현재 public status를 비교해 저장한다.

전이 규칙:

1. `normal -> suspected`: 저장하지 않는다.
2. `suspected -> normal`: 저장하지 않는다.
3. `suspected -> incident`: 새 incident를 생성한다.
4. `incident -> incident`: 열린 incident의 `last_observed_at`, `area`, 대표 evidence를 갱신한다.
5. `incident -> recovering`: 같은 incident를 `recovering`으로 갱신한다.
6. `recovering -> incident`: 같은 열린 incident로 되돌린다.
7. `recovering -> resolved`: 같은 incident를 `resolved`로 갱신하고 `resolved_at`을 설정한다.
8. `resolved -> normal`: 새 기록을 만들지 않는다.
9. `resolved -> suspected -> incident`: 새 incident를 생성한다.

`unknown` area도 저장할 수 있다. 다만 UI는 원인 확정처럼 보이지 않게 `확인 불가`로 표시한다.

## Backend 구조

구현은 현재 `src-tauri/src/network/` 경계 안에서 시작한다.

- `incident_store.rs`: SQLite 연결, migration, insert/update/query, retention
- `incident_recorder.rs`: lifecycle 전이 해석과 store 호출
- `commands.rs`: 최근 incident 조회 command 등록
- `network/mod.rs`: 새 module export

SQLite dependency는 `rusqlite`를 우선 검토한다. Windows artifact에서 별도 시스템 SQLite 설치에 기대지 않도록 `bundled` feature 사용을 기본 후보로 둔다. 이미 Tauri plugin이나 기존 crate가 SQLite를 제공한다면 중복 dependency 없이 그 경로를 따른다.

DB 파일 경로는 Tauri app data directory를 사용한다. 작업 디렉터리나 repository 안에 runtime DB를 만들지 않는다.

## Command contract

### `get_recent_network_incidents`

입력 없이 최신 3개 incident를 반환한다. v2에서는 pagination과 filter를 만들지 않는다.

반환 타입:

- `id`: number
- `status`: `ongoing | recovering | resolved`
- `area`: `local_connection | gateway_or_local | dns | external | unknown`
- `startedAt`: string
- `lastObservedAt`: string
- `resolvedAt`: string 또는 null
- `summary`: string
- `representativeEvidence`: 최신 대표 evidence 최대 4개

조회 실패 시 command는 오류를 반환한다. frontend는 이를 현재 네트워크 장애로 해석하지 않고 최근 기록 panel의 unavailable 상태로만 표시한다.

## UI

메인 화면 순서는 v1 구조를 유지하되 최근 장애 panel을 활성화한다.

1. 현재 네트워크 진단 상태
2. 구간별 진단 경로
3. 최신 근거
4. 최근 장애
5. 원시 네트워크 확인

최근 장애 panel은 최대 3개 행을 표시한다.

행 정보:

- 상태: `진행 중`, `복구 확인 중`, `복구됨`
- 추정 구간: 기존 area label
- 시작 시각
- 마지막 확인 또는 복구 시각
- 요약 문장

빈 상태 문구:

```text
저장된 장애 기록이 없습니다.
```

조회 실패 문구:

```text
장애 기록을 불러올 수 없습니다.
```

browser 개발 환경에서 Tauri command를 사용할 수 없으면 현재 진단처럼 과장된 오류를 만들지 않고 기록 unavailable 상태를 표시한다.

## Privacy와 안전 경계

저장 가능한 정보:

- adapter, IPv4, default route, gateway 확인 결과
- system DNS 확인 결과
- Microsoft와 Google connectivity endpoint 확인 결과
- duration, status, checked time, 짧은 detail

저장하지 않는 정보:

- packet payload
- 사용자가 접속한 URL 또는 domain
- 브라우저 traffic
- router credential
- Windows 네트워크 설정 값 변경 이력
- 외부 서버 업로드용 식별자

이 기능은 진단과 기록만 한다. router, Windows 설정, driver, firewall, DNS 설정을 자동으로 바꾸지 않는다.

## 오류 정책

- DB 초기화 실패: runtime은 현재 진단을 계속 수행한다. 최근 장애 조회는 unavailable 또는 error로 표시한다.
- 저장 실패: 현재 diagnostic status를 유지하고 다음 observation에서 다시 저장을 시도한다.
- migration 실패: command는 오류를 반환하고 UI는 기록 panel에만 실패를 표시한다.
- retention 실패: incident 저장을 막지 않는다. 테스트와 log 대상으로 남기되 사용자에게 네트워크 장애처럼 표시하지 않는다.

## 테스트

### Rust store

- migration을 빈 DB에 적용한다.
- migration을 여러 번 적용해도 schema가 유지된다.
- incident 생성, 갱신, resolved 처리가 저장된다.
- 최근 incident 3개만 최신순으로 조회된다.
- 24시간이 지난 `network_probe_observations`만 삭제된다.
- incident와 incident evidence는 retention으로 삭제되지 않는다.

### Rust recorder

- `suspected`는 incident를 만들지 않는다.
- `suspected -> incident`에서 incident가 생성된다.
- `incident -> recovering -> resolved`가 같은 incident를 갱신한다.
- `recovering -> incident`는 같은 incident를 유지한다.
- `resolved -> normal`은 새 incident를 만들지 않는다.
- 해결 이후 새 incident는 새 id를 가진다.
- store 실패가 runtime status 계산을 중단시키지 않는다.

### Frontend

- 최근 incident 0개, 1개, 3개를 렌더링한다.
- 4개 이상 fixture에서도 panel은 3개만 표시한다.
- ongoing, recovering, resolved label이 구분된다.
- unknown area는 `확인 불가`로 표시된다.
- command unavailable과 조회 실패가 현재 진단 장애처럼 보이지 않는다.
- 전체 이력 page나 상세 link가 v2에 나타나지 않는다.

### 검증 명령

- `cargo test --manifest-path src-tauri/Cargo.toml`
- `cargo check --manifest-path src-tauri/Cargo.toml`
- `npm test`
- `npm run build`

## Windows artifact 확인

Windows Build artifact에서 다음을 확인한다.

1. 앱 설치 후 app data directory에 DB가 생성된다.
2. 정상 네트워크에서는 최근 장애가 빈 상태로 유지된다.
3. 반복 이상으로 incident가 확정되면 최근 장애에 기록이 생긴다.
4. 네트워크가 복구되면 같은 기록이 `복구 확인 중` 이후 `복구됨`으로 갱신된다.
5. 앱을 재시작해도 최근 장애 기록이 남아 있다.
6. 24시간이 지난 probe observation 정리는 incident 기록을 삭제하지 않는다.
7. DB 오류가 있어도 현재 상태 진단 UI가 완전히 중단되지 않는다.

## v3와 v4로 미루는 항목

v3에서는 장애 이력 전용 페이지를 추가한다.

- 더 많은 incident 목록
- 날짜 범위
- 상태와 구간 filter
- 빈 상태와 loading/error state

v4에서는 incident 상세 화면을 추가한다.

- 사건별 evidence timeline
- lifecycle 전이 기록
- 대표 근거와 raw diagnostic snapshot 비교
- 사용자에게 복사 가능한 진단 요약
