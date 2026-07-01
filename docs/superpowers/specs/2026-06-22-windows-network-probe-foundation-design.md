# Windows 네트워크 Probe Foundation 설계

## 목표

상시 인터넷 장애 진단을 구현하기 전에 Windows PC의 실제 네트워크 상태를 한 번 수집하고 원시 JSON으로 확인하는 가장 작은 기능을 만든다. 이 단계의 목적은 장애를 판정하는 것이 아니라 adapter, route, gateway, DNS와 외부 endpoint의 관찰 가능성과 오류 형태를 Windows 증거로 확인하는 것이다.

## 범위

### 포함

- 개인용 Windows PC의 유선 IPv4 adapter와 주소
- 현재 IPv4 default route와 gateway 후보
- default gateway에 대한 ICMP 응답과 지연
- Windows system resolver를 통한 두 connectivity hostname 해석
- Microsoft와 Google connectivity endpoint의 제한된 HTTP 응답 검사
- 수집 시각, 단계별 소요 시간, 성공·timeout·unsupported·error 상태
- 앱 화면에서 일회성 실행, raw JSON 표시와 clipboard 복사
- Windows artifact에서 복사한 JSON을 이용한 후속 mapping 검토

### 제외

- 자동 polling과 background task
- `normal`, `suspected`, `incident`, `recovering`, `resolved` 판정
- SQLite schema와 history 보존
- Windows connectivity status API
- Wi-Fi, VPN, proxy, IPv6 전용 진단
- router login·설정 변경, packet capture, 네트워크 자동 복구
- Windows toast, 외부 서버 업로드

## 접근법

Rust에서 Windows API를 직접 호출한다. PowerShell과 `ipconfig`, `route`, `ping`, `nslookup` 출력 파싱은 locale과 출력 형식에 의존하므로 사용하지 않는다.

- adapter와 주소: [`GetAdaptersAddresses`](https://learn.microsoft.com/en-us/windows/win32/api/iphlpapi/nf-iphlpapi-getadaptersaddresses)
- IPv4 route: [`GetIpForwardTable2`](https://learn.microsoft.com/en-us/windows/win32/api/netioapi/nf-netioapi-getipforwardtable2)
- gateway ICMP: [`IcmpSendEcho2`](https://learn.microsoft.com/en-us/windows/win32/api/icmpapi/nf-icmpapi-icmpsendecho2)
- system name resolution: [`GetAddrInfoExW`](https://learn.microsoft.com/en-us/windows/win32/api/ws2tcpip/nf-ws2tcpip-getaddrinfoexw)

HTTP는 Rust client를 사용하되 redirect를 자동 추적하지 않고 response body 크기를 제한한다. 관리자 권한 상승을 요청하지 않는다.

## 아키텍처

```text
NetworkProbePanel
  -> get_network_probe_snapshot Tauri command
  -> NetworkProbeService
  -> WindowsNetworkCollector
       -> adapter and route
       -> gateway ICMP
       -> system DNS
       -> connectivity HTTP
  -> NetworkProbeSnapshot JSON
```

### Rust 경계

- `domain`: 직렬화되는 snapshot, adapter, route와 check result 타입만 소유한다.
- `collector/windows`: Windows API 호출과 raw error mapping을 소유한다.
- `collector/unsupported`: Windows가 아닌 환경에서 명시적 `unsupported` 결과를 반환한다.
- `service`: 단계 실행 순서와 snapshot 조립만 담당한다.
- `commands`: `get_network_probe_snapshot` Tauri command를 노출한다.

collector는 UI 문구나 장애 원인을 결정하지 않는다. command 전체를 실패시키기보다 각 단계의 결과를 독립적으로 보존한다. adapter 열거 자체가 실패해도 snapshot과 그 오류를 반환한다.

### Frontend 경계

`src/features/network-probe/`에 다음 책임을 둔다.

- Rust contract와 같은 TypeScript 타입
- Tauri command 호출 wrapper
- 실행 버튼, 진행 상태, raw JSON, 복사 버튼을 제공하는 panel
- browser test용 결정적 fixture

기존 제품 홈의 인터넷 장애 진단 영역 아래에 개발용 raw probe panel을 추가한다. 이 화면은 최종 진단 dashboard가 아니며 원인 판정이나 정상 배지를 표시하지 않는다.

## 수집 규칙

### Adapter

- `IfType`이 Ethernet 계열이고 operational status가 up인 adapter를 우선한다.
- adapter 이름, friendly name, interface index, operational status, MAC 주소와 IPv4 unicast 주소를 보존한다.
- 주소나 adapter가 없으면 빈 배열과 명시적 상태를 반환한다.
- MAC 주소는 로컬 화면에만 표시하며 외부 전송하지 않는다.

### Default route

- IPv4 route table에서 prefix length가 0인 route를 default route 후보로 보존한다.
- operational Ethernet adapter와 연결된 후보를 우선한다.
- 여러 후보가 있으면 route metric과 interface metric을 근거로 선택하되 모든 후보와 선택 이유를 JSON에 남긴다.
- 확실한 후보가 없으면 임의 gateway를 선택하지 않는다.

### Gateway ICMP

- 선택된 IPv4 gateway가 있을 때만 `IcmpSendEcho2`를 동기 호출한다.
- 성공 시 round-trip time과 reply address를 기록한다.
- reply 없음, timeout, ICMP 차단 가능성, API error를 구분 가능한 code와 message로 남긴다.
- ICMP 실패만으로 공유기 장애를 판정하지 않는다.

### DNS

Windows system resolver로 다음 hostname을 각각 해석한다.

- `www.msftconnecttest.com`
- `connectivitycheck.gstatic.com`

성공 시 중복 제거한 IPv4·IPv6 주소와 소요 시간을 기록한다. hosts file 등 system resolver가 사용하는 다른 namespace가 개입할 수 있으므로 결과 명칭은 `system_name_resolution`로 한다. DNS server 자체의 정상 여부로 단정하지 않는다.

### HTTP

- `http://www.msftconnecttest.com/connecttest.txt`: redirect 없이 status와 제한된 body가 예상 문자열과 맞는지 기록한다.
- `https://connectivitycheck.gstatic.com/generate_204`: redirect 없이 status가 204인지 기록한다.
- response body는 최대 4 KiB만 읽는다.
- 각 endpoint는 한 번만 요청하며 retry하지 않는다.
- timeout은 구현 상수로 노출하고 Windows evidence 전에는 상시 진단 정책으로 재사용하지 않는다.
- 단일 endpoint 실패는 인터넷 장애로 판정하지 않는다.

## Wire contract

최상위 결과는 다음 의미를 가진다.

```json
{
  "collectedAt": "2026-06-22T00:00:00Z",
  "collector": "windows-native",
  "durationMs": 0,
  "adapters": [],
  "defaultRoutes": [],
  "selectedRoute": null,
  "gatewayCheck": {
    "status": "not_run",
    "durationMs": 0,
    "detail": null,
    "error": null
  },
  "dnsChecks": [],
  "httpChecks": [],
  "errors": []
}
```

공통 check status는 `success`, `timeout`, `not_run`, `unsupported`, `error`다. `success`는 해당 probe가 기대한 응답을 관찰했다는 의미일 뿐 PC 전체 네트워크가 정상이라는 뜻이 아니다.

error는 최소한 다음 필드를 가진다.

- `stage`
- `code`
- `message`

Windows API 원본 numeric error code가 있으면 함께 보존한다. error message만 파싱해 상태를 결정하지 않는다.

## 실행과 오류 처리

- 한 번의 버튼 클릭은 한 snapshot만 실행한다.
- 실행 중에는 중복 실행 버튼을 비활성화한다.
- 각 probe는 bounded timeout을 가져야 하며 전체 실행은 무한 대기하지 않는다.
- 한 단계 실패가 뒤 단계 실행을 불가능하게 할 때는 `not_run`과 이유를 기록한다.
- clipboard 복사 실패는 probe 결과와 분리해 UI error로 표시한다.
- macOS Tauri 실행은 가짜 Windows 값을 만들지 않고 `unsupported` snapshot을 반환한다.

## 테스트

### Rust

- domain JSON serialization contract
- default route 후보 선택: 단일 후보, 복수 metric, Ethernet 불일치, 후보 없음
- 단계 실패가 snapshot 전체 실패로 바뀌지 않는 service 조립
- unsupported collector 결과
- Windows API error code mapping은 Windows 전용 unit test로 둔다.

### Frontend

- idle 상태에서 실행 버튼 표시
- 실행 중 중복 호출 방지
- snapshot을 format된 raw JSON으로 표시
- unsupported와 command error 표시
- clipboard 복사 동작

### Windows artifact

Windows Build artifact를 실행해 다음을 확인한다.

1. 관리자 권한 없이 실행된다.
2. 실제 Ethernet adapter, IPv4 주소와 default route가 올바른 장치에 연결된다.
3. gateway reply와 system name resolution 결과가 raw JSON에 보존된다.
4. 두 HTTP endpoint의 status, duration과 body match가 구분된다.
5. 네트워크 차단 또는 endpoint 실패 시 앱이 멈추지 않고 부분 결과를 반환한다.
6. raw JSON을 복사해 후속 설계 검토에 사용할 수 있다.

## 완료 기준

- `get_network_probe_snapshot`은 Windows에서 부분 실패를 포함한 구조화된 snapshot을 반환한다.
- UI에서 한 번 실행하고 raw JSON을 복사할 수 있다.
- source에 shell command 출력 파싱, polling, SQLite, 장애 판정이 없다.
- frontend test·build와 Rust test·check가 통과한다.
- Windows artifact에서 관리자 권한 없는 실제 raw JSON을 확보한다.

## 후속 단계

확보한 Windows JSON을 검토한 뒤 adapter·route 선택 규칙, timeout, endpoint 동작을 확정한다. 그 다음 별도 설계에서 polling cadence, 집중 검사, 장애 lifecycle과 SQLite persistence를 구현한다.
