# 네트워크 상시 진단 Runtime Foundation 설계

## 목표

PC Health가 실행 중인 동안 유선 LAN 상태를 자동으로 관찰하고 현재 상태, 추정 장애 구간, 최신 근거와 마지막 확인 시각을 앱 화면에 표시한다. 한 번 실행하는 기존 Windows raw probe를 검증 도구로 유지하면서, 낮은 부하의 평상시 관찰과 이상 감지 시 집중 검사를 분리한다.

이번 단계는 상시 진단 runtime과 메모리 기반 현재 상태까지 구현한다. 장애 이력과 SQLite 저장은 runtime 동작과 판정 근거를 Windows에서 검증한 다음 단계로 미룬다.

## 확정된 제품 조건

- 개인용 Windows PC 한 대와 공유기에 직접 연결된 유선 LAN을 대상으로 한다.
- 앱 프로세스가 실행 중인 동안만 자동 진단한다.
- 사용자가 시작 버튼을 누르지 않아도 앱 시작과 함께 진단을 시작한다.
- Windows service, Windows 알림, Wi-Fi, VPN, proxy는 다루지 않는다.
- packet capture, router 설정 변경, Windows 네트워크 설정 변경, 자동 복구, 외부 기록 업로드를 하지 않는다.
- 약 30초 안에 장애를 의심할 수 있는 낮은 부하의 단계형 관찰을 목표로 한다.
- 단일 endpoint 실패나 한 번의 timeout으로 장애를 확정하지 않는다.

## 범위

### 포함

- 앱 시작 시 전체 probe 1회
- 10초 주기의 adapter, IPv4, default route, gateway 관찰
- 시작 전체 probe 이후 40초부터 20초마다 Microsoft와 Google endpoint를 번갈아 확인하는 외부 경량 검사
- 이상 감지 시 gateway, system DNS 2곳과 HTTP endpoint 2곳을 확인하는 집중 검사
- 30초 집중 검사 cooldown과 중복 실행 방지
- `normal`, `suspected`, `incident`, `recovering`, `resolved` lifecycle
- 로컬 연결, 공유기 또는 로컬 연결, DNS, 외부 연결, 확인 불가의 초기 판정
- Rust 메모리에 보관하는 최신 진단 상태
- 현재 상태, 추정 구간, 최신 근거와 마지막 확인 시각 UI
- 기존 raw JSON 수동 probe 유지

### 제외

- SQLite schema, probe 보존과 정리
- 최근 장애와 장애 이력 UI
- 지연 임계값과 느린 인터넷 판정
- 사용자 polling 설정 UI
- Windows Network List Manager 상태 추가
- 자동 재시도, 자동 복구와 시스템 설정 변경
- 앱 종료 후 진단을 계속하는 Windows service

## 검토한 접근

### 선택: 단일 scheduler의 2단계 진단

전용 worker 하나가 경량 cycle과 집중 검사를 직렬 실행한다. 요청 수가 제한되고 실행 순서와 상태 전이를 결정적으로 테스트할 수 있으며, 느린 probe가 있어도 중복 실행을 만들지 않는다.

### 제외: 10초마다 전체 probe

구현은 단순하지만 DNS와 외부 endpoint 요청이 불필요하게 많다. 계속 켜두는 개인용 앱의 기본 동작으로 사용하지 않는다.

### 제외: probe별 독립 timer

각 주기를 세밀하게 조절할 수 있지만 동시 실행, 결과 병합, 종료와 cooldown 경계가 복잡해진다. 현재 범위에는 필요하지 않다.

## Backend 구조

### `NetworkProbeCoordinator`

기존 `NetworkProbeService`를 공유하는 직렬화 경계다. runtime 자동 probe와 raw JSON 수동 probe가 같은 coordinator를 사용하도록 해 두 경로의 Windows API와 외부 요청이 겹치지 않게 한다.

- 한 번에 probe 하나만 실행한다.
- manual raw probe가 진행 중이면 runtime cycle은 완료를 기다리고 catch-up 실행을 만들지 않는다.
- runtime probe가 진행 중이면 manual command는 해당 실행이 끝난 뒤 전체 probe를 수행한다.
- raw JSON contract와 `get_network_probe_snapshot` command 이름은 유지한다.
- manual raw probe 결과는 자동 진단 lifecycle을 변경하지 않는다.

### `NetworkObservationEngine`

한 회분의 probe를 실행하고 state machine 입력으로 변환한다.

- `startup_full`: inventory, gateway, DNS 2곳, HTTP 2곳
- `baseline`: inventory와 gateway, 외부 검사가 예정된 cycle이면 HTTP 한 곳
- `focused`: inventory, gateway, DNS 2곳, HTTP 2곳

외부 경량 대상은 Microsoft, Google 순서로 교대한다. 대상 교대는 요청을 시도한 뒤에만 진행하며 실패해도 같은 대상에 즉시 재시도하지 않는다.

### `NetworkDiagnosticStateMachine`

Windows API, thread와 wall clock에 의존하지 않는 순수 Rust 상태 전이기다. observation과 단조 증가하는 cycle 시각을 입력받아 lifecycle, 추정 구간과 판정 근거를 반환한다. UI용으로 보관한 과거 evidence를 다시 판정 입력으로 사용하지 않고 현재 observation과 명시적인 연속 이상 상태만 사용한다.

### `NetworkDiagnosticsRuntime`

앱당 하나의 worker와 최신 상태 저장소를 소유한다.

- Tauri setup에서 자동 시작한다.
- `Instant` 기반으로 주기를 계산한다.
- 다음 baseline은 이전 probe가 끝난 뒤 예약해 catch-up burst를 만들지 않는다.
- wait 중에는 종료 신호로 즉시 깨어난다.
- active probe는 기존 timeout 안에서 끝낸 뒤 종료한다.
- 종료 시 worker에 stop 신호를 보내고 join한다.
- worker 내부 실패나 lock 오류는 runtime 자체를 조용히 중단하지 않고 `error` availability와 마지막 오류 근거로 노출한다.

## 주기와 요청 제한

| 작업 | 주기 | 규칙 |
| --- | --- | --- |
| 시작 전체 probe | 앱 시작 시 1회 | retry 없음 |
| adapter, route, gateway | 10초 | 직렬 실행 |
| 외부 경량 HTTP | 시작 전체 probe 이후 40초부터 20초 | Microsoft와 Google 한 곳씩 교대 |
| 집중 검사 | 이상 감지 시 | DNS 2곳과 HTTP 2곳, retry 없음 |
| 집중 검사 cooldown | 완료 후 30초 | cooldown 중 baseline은 계속 수행 |

이 값은 이번 Windows 검증의 후보값이다. 코드에서는 runtime 전용 상수 한곳에 두되 사용자 설정과 범용 configuration abstraction을 만들지 않는다.

정상 상태의 자동 HTTP 요청 상한은 시작 전체 probe를 포함해 시간당 180회이며 각 endpoint는 시간당 90회다. 이를 위해 첫 외부 경량 HTTP는 시작 전체 probe 이후 40초에 실행하고 이후 20초마다 교대한다. 이상이 지속되어 cooldown마다 집중 검사가 필요한 최악의 경우에도 자동 HTTP 요청은 시간당 420회를 넘지 않는다. manual raw probe는 사용자가 명시적으로 실행한 별도 요청이다.

probe가 주기보다 오래 걸리면 해당 cycle이 끝난 뒤 다음 주기를 새로 계산한다. 같은 종류의 누락된 cycle을 몰아서 실행하지 않는다. 집중 검사 cooldown 중에도 반복된 baseline 이상은 lifecycle 전이 근거가 될 수 있지만 새 집중 검사는 시작하지 않는다.

## Lifecycle

runtime availability와 진단 lifecycle을 분리한다. availability는 `starting`, `running`, `unavailable`, `error`이며 첫 observation이 나오기 전에는 lifecycle을 제공하지 않는다.

### 전이 규칙

1. 앱 시작 직후 availability는 `starting`이다.
2. 시작 전체 probe가 정상이면 `normal`이다.
3. 시작 전체 probe에서 이상이 나와도 단 한 시점의 결과이므로 `suspected`로 시작한다.
4. `normal`에서 한 번의 이상을 감지하면 `suspected`가 되고 cooldown이 허용하면 집중 검사를 실행한다.
5. `suspected`에서 호환되는 이상이 다시 확인되면 `incident`가 된다.
6. `suspected`에서 정상 결과가 나오면 false alarm으로 보고 `normal`로 돌아간다.
7. `suspected`에서 근거가 서로 충돌하면 `suspected`를 유지하되 추정 구간은 `unknown`으로 둔다.
8. `incident`에서 첫 정상 observation이 나오면 `recovering`이다.
9. `recovering`에서 두 번째 정상 observation이 나오면 `resolved`다.
10. `resolved`에서 다음 정상 observation이 나오면 `normal`이다.
11. `recovering`에서 이상이 다시 나오면 기존 `incident`로 복귀한다.
12. `resolved`에서 이상이 나오면 새로운 `suspected`로 시작한다.

집중 검사는 단일 실패를 독립된 두 번째 observation으로 확인하는 역할을 한다. 한 endpoint 실패 후 집중 검사에서 나머지 근거가 정상이면 incident로 확정하지 않는다.

정상 observation이 incident 구간의 복구 근거가 되려면 해당 구간을 실제로 검사해야 한다. 로컬 연결과 gateway incident는 baseline 정상으로 복구를 확인할 수 있지만, DNS와 외부 연결 incident는 DNS·HTTP 전체를 포함한 focused 정상 결과가 있어야 `recovering`으로 이동한다. 해당 구간을 확인하지 않은 baseline은 기존 incident를 유지한다.

호환되는 반복 이상은 같은 비정상 구간이 연속으로 나온 경우다. 첫 observation이 단일 HTTP 실패라 `unknown`이었던 경우에는 focused observation에서 최초 실패 source가 계속 실패하고 추가 독립 실패가 확인되어 `dns` 또는 `external`로 구체화될 때만 호환되는 확인으로 본다. `unknown` 결과만 두 번 나온 것은 incident 확인 근거로 사용하지 않는다.

## 초기 장애 구간 판정

판정은 큰 구간만 제공하며 하드웨어 부품이나 통신사 원인을 확정하지 않는다.

### `local_connection`

adapter inventory가 정상적으로 수집됐지만 활성 Ethernet, IPv4 또는 선택 가능한 default route가 없다. inventory API 자체가 실패해 정보가 비어 있으면 로컬 연결 문제로 확정하지 않고 `unknown`으로 둔다.

### `gateway_or_local`

활성 Ethernet, IPv4와 default route는 있지만 gateway 실패 또는 timeout이 호환되는 두 observation에서 반복된다. 소프트웨어 증거만으로 공유기, LAN 케이블, port와 NIC를 나누지 않는다.

### `dns`

gateway가 성공하고 집중 검사에서 두 hostname의 system resolver가 모두 실패하며, 외부 HTTP 결과도 정상과 충돌하지 않을 때 사용한다. DNS 두 곳이 실패했는데 HTTP 두 곳이 모두 성공하면 stale cache나 probe 결함 가능성이 있으므로 `unknown`으로 둔다.

### `external`

gateway와 system DNS가 성공하고 Microsoft와 Google HTTP가 모두 실패할 때 사용한다. endpoint 한 곳만 실패하면 `suspected` 또는 `unknown` 근거일 뿐 incident의 외부 연결 구간으로 확정하지 않는다.

### `unknown`

inventory 수집 실패, 단일 endpoint 실패, 서로 충돌하는 결과, 집중 검사 차단과 지원하지 않는 조합에 사용한다. UI에서는 `확인 불가`로 표시한다.

지연 수치는 evidence로 표시할 수 있지만 이번 state machine은 latency threshold로 lifecycle이나 장애 구간을 바꾸지 않는다.

## Wire contract

Rust 타입이 기준이며 frontend 타입과 fixture를 함께 맞춘다.

### `NetworkDiagnosticStatus`

- `availability`: `starting | running | unavailable | error`
- `lifecycle`: `normal | suspected | incident | recovering | resolved | null`
- `suspectedArea`: `local_connection | gateway_or_local | dns | external | unknown | null`
- `observedAt`: 마지막 observation RFC 3339 시각 또는 `null`
- `lastFullProbeAt`: 마지막 시작 또는 집중 검사 시각 또는 `null`
- `evidence`: source별 최신 근거 목록
- `error`: runtime 자체 오류 또는 `null`

### `DiagnosticEvidence`

- `source`: `ethernet | ipv4 | default_route | gateway | dns_microsoft | dns_google | http_microsoft | http_google`
- `status`: `success | failure | timeout | unavailable | not_checked`
- `checkedAt`: 해당 source가 실제 확인된 RFC 3339 시각 또는 `null`
- `durationMs`: 측정 시간이 있으면 값, 없으면 `null`
- `detail`: raw error나 주소처럼 사용자에게 필요한 짧은 근거 또는 `null`

외부 endpoint를 교대로 확인하므로 evidence마다 `checkedAt`을 둔다. UI는 이전 endpoint 결과를 표시할 수 있지만 state machine은 오래된 UI evidence를 현재 판정 근거로 재사용하지 않는다.

## Tauri와 frontend

Tauri는 `NetworkDiagnosticsRuntime`을 managed state로 등록하고 `get_network_diagnostic_status` command로 최신 상태의 clone을 반환한다. command는 네트워크 요청을 실행하지 않는다.

frontend는 Tauri 환경에서 1초마다 이 command를 호출한다. 이는 메모리 상태 polling이며 네트워크 probe 주기와 무관하다. browser 개발 환경에서는 command를 반복 호출하지 않고 `Windows 앱에서 확인 가능` unavailable 상태를 표시한다.

새 현재 상태 패널은 raw JSON 패널 위에 배치한다.

- lifecycle의 한국어 상태
- 추정 구간
- 마지막 확인 시각
- gateway, DNS, Microsoft, Google 최신 근거
- `starting`이면 `첫 확인 중`
- `unavailable` 또는 `error`이면 과장된 장애 표시 없이 `확인 불가`

색상과 경고 강도는 `docs/ui-guidelines.md`의 조용한 데스크톱 유틸리티 톤을 따른다. Windows toast, 소리, modal을 사용하지 않는다. 기존 raw JSON 패널은 개발과 Windows 검증을 위해 유지한다.

## 오류와 종료 정책

- adapter나 route 일부 실패는 가능한 결과와 오류를 함께 보존한다.
- gateway, DNS와 HTTP timeout은 bounded 결과이며 runtime panic으로 취급하지 않는다.
- endpoint body mismatch와 redirect는 정상 연결로 간주하지 않는다.
- manual raw probe와 runtime probe는 coordinator를 통해 직렬화한다.
- lock poison, worker panic과 상태 저장 실패는 availability `error`로 노출한다.
- worker wait는 stop 신호로 깨우며 active Windows 호출은 기존 timeout 범위 안에서 완료한 뒤 join한다.
- retry와 catch-up burst를 사용하지 않는다.

## 테스트

### Rust state machine

- 시작 정상 결과가 `normal`이 된다.
- 시작 이상은 바로 incident가 아니라 `suspected`가 된다.
- 단일 endpoint 실패는 incident를 만들지 않는다.
- 호환되는 gateway 이상 반복은 `gateway_or_local` incident가 된다.
- gateway 성공과 두 DNS 실패가 DNS incident 후보가 된다.
- gateway와 DNS 성공, 두 HTTP 실패가 external incident 후보가 된다.
- DNS 실패와 HTTP 성공처럼 충돌하는 근거는 `unknown`이다.
- `incident -> recovering -> resolved -> normal`을 따른다.
- recovering 중 재실패는 incident로 복귀한다.
- resolved 중 재실패는 새로운 suspected가 된다.

### Rust runtime

- 시작 시 full probe를 한 번 실행한다.
- baseline은 10초, 외부 경량 검사는 시작 40초 뒤부터 20초 역할을 지킨다.
- Microsoft와 Google endpoint를 교대한다.
- 이상 감지 시 focused probe를 실행한다.
- focused probe 완료 후 30초 cooldown을 지킨다.
- 느린 probe가 있어도 실행 수가 중첩되지 않는다.
- manual raw probe와 runtime probe가 coordinator에서 직렬화된다.
- stop이 wait를 깨우고 worker를 정리한다.

runtime 테스트는 실제 sleep과 외부 네트워크에 의존하지 않도록 fake collector와 제어 가능한 scheduler clock/wait 경계를 사용한다. 범용 scheduler framework를 만들지 않고 이 runtime 테스트에 필요한 최소 경계만 둔다.

### Frontend

- browser 환경은 invoke 없이 unavailable 상태를 표시한다.
- starting, normal, suspected, incident, recovering, resolved, unavailable과 error를 렌더링한다.
- 추정 구간, 마지막 확인 시각과 최신 evidence를 표시한다.
- component unmount 시 1초 status polling timer를 정리한다.
- raw JSON 수동 panel 동작을 유지한다.

## Windows 검증

Windows Build artifact에서 다음을 확인한다.

1. 관리자 권한 없이 앱 실행과 함께 진단이 자동 시작된다.
2. 정상 유선 환경에서 10분 이상 실행해 UI가 멈추지 않고 상태가 normal을 유지한다.
3. Microsoft와 Google evidence의 확인 시각이 교대로 갱신된다.
4. LAN을 분리하면 약 30초 안에 suspected 또는 incident로 변한다.
5. LAN을 다시 연결하면 `recovering -> resolved -> normal` 순서를 확인한다.
6. raw JSON 수동 probe를 실행해도 background probe와 외부 요청이 겹치지 않는다.
7. 요청 폭증, 무제한 retry, catch-up burst와 종료 hang이 없다.

Windows 실제 검증 전에는 주기와 판정 정확도를 최종 확정했다고 주장하지 않는다. 결과가 다르면 회귀 테스트를 먼저 추가하고 좁은 수정만 수행한다.

## 완료 기준

- 앱 시작 시 상시 진단 runtime이 자동 시작된다.
- 경량 관찰과 집중 검사가 승인된 주기와 cooldown으로 직렬 실행된다.
- 최신 상태는 메모리에만 보관되고 읽기 전용 Tauri command로 제공된다.
- 현재 상태 UI가 lifecycle, 추정 구간, 시각과 근거를 표시한다.
- 기존 raw JSON 수동 probe와 wire contract가 유지된다.
- SQLite, 장애 목록, latency 판정과 시스템 변경 동작이 추가되지 않는다.
- frontend test와 build, Rust test와 check, strict Clippy가 통과한다.
- Windows artifact에서 자동 시작, 상태 전이와 종료 동작을 검증할 수 있다.

## 후속 단계

Windows runtime 검증이 끝난 뒤 별도 설계에서 SQLite를 추가한다. 개별 probe는 24시간, incident와 판정 근거는 무기한 보존하며 최근 장애와 장애 이력 UI를 구현한다.
