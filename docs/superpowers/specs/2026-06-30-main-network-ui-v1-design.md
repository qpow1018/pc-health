# 메인 네트워크 UI v1 설계

## 목표

PC Health의 첫 화면을 인터넷 장애 진단 전용 대시보드로 정리한다. 사용자가 앱을 열었을 때 현재 상태, 의심 구간, 최신 근거, 최근 장애를 빠르게 확인할 수 있게 한다.

이 설계는 UI 정보 구조를 정하는 문서다. 새 probe, latency 판정, 자동 복구, packet capture, router 설정 변경, 외부 업로드는 포함하지 않는다.

## 제품 범위

- 제품 영역: 인터넷 장애 진단
- lifecycle: `active`
- 대상 환경: 개인용 Windows PC, 유선 LAN, 앱 실행 중 관찰 가능한 네트워크 상태
- 원칙: 읽기 전용, 근거 기반 판정, 근거 부족 시 `확인 불가`

성능 모니터링과 드라이버 관리는 이 화면에 다시 도입하지 않는다.

## 성공 기준

1. 첫 화면에서 현재 lifecycle, 추정 구간, 마지막 확인 시각을 바로 볼 수 있다.
2. 사용자가 `PC/어댑터 -> IPv4/Route -> 게이트웨이 -> DNS -> 외부 연결` 흐름에서 어디까지 확인됐는지 이해할 수 있다.
3. 최신 근거가 source별로 안정적인 행 구조에 표시된다.
4. 최근 장애와 원시 probe는 메인 판단을 방해하지 않는 우선순위로 배치된다.
5. `unknown`, unsupported, stale, error 상태가 정상처럼 보이지 않는다.

## 화면 우선순위

메인 화면은 다음 순서로 구성한다.

1. 현재 네트워크 진단 상태
2. 구간별 진단 경로
3. 최신 근거
4. 최근 장애
5. 원시 네트워크 확인

Raw JSON은 Windows 검증과 문제 제보에 필요하지만 사용자의 첫 판단 요소는 아니다. 따라서 접을 수 있는 하단 고급 영역으로 둔다.

## 현재 상태 요약

상단에는 다음 정보를 표시한다.

- lifecycle label: `정상`, `장애 의심`, `장애 확인`, `복구 확인 중`, `복구됨`, `확인 불가`
- 추정 구간: `로컬 연결 구간`, `공유기 또는 로컬 연결 구간`, `DNS`, `외부 연결 구간`, `확인 불가`, `해당 없음`
- 마지막 확인 시각: `observedAt`
- 진단 사용 가능 여부: `starting`, `unavailable`, `error`
- 짧은 설명: lifecycle과 suspected area를 과장 없이 설명하는 한 문장

상태 요약은 큰 hero나 경고 화면이 아니다. 조용한 desktop utility panel로 표시하고, 위험을 확정할 근거가 부족하면 `장애 확인` 대신 `장애 의심` 또는 `확인 불가`를 사용한다.

## 진단 경로

진단 경로는 메인 UI의 핵심 시각화다.

```text
PC/어댑터 -> IPv4/Route -> 게이트웨이 -> DNS -> 외부 연결
```

각 단계는 다음 상태 중 하나를 가진다.

- `성공`
- `실패`
- `시간 초과`
- `확인하지 않음`
- `확인 불가`

표현 규칙:

- 실패한 구간만 강하게 강조한다.
- 아직 검사하지 않은 구간은 실패처럼 보이지 않게 muted 처리한다.
- `DNS`는 `dns_microsoft`, `dns_google`의 요약 단계로 표시한다.
- `외부 연결`은 `http_microsoft`, `http_google`의 요약 단계로 표시한다.
- `ethernet`, `ipv4`, `default_route` evidence가 없으면 해당 단계는 `확인하지 않음` 또는 `확인 불가`로 남긴다.

이 경로는 원인을 확정하는 diagram이 아니라 현재 evidence의 readable summary다.

## 최신 근거

최신 근거는 카드 나열보다 밀도 있는 list/table에 가깝게 표현한다.

기본 행:

- PC/어댑터
- IPv4/Route
- 게이트웨이
- DNS
- Microsoft
- Google

각 행의 열:

- 구간
- 상태
- 지연 시간 또는 `-`
- 세부 정보
- 확인 시각

`durationMs`는 evidence로만 보여준다. 현재 state machine은 latency threshold로 lifecycle이나 suspected area를 바꾸지 않으므로 latency graph나 health score를 만들지 않는다.

## 최근 장애

최근 장애 영역은 SQLite incident persistence가 있을 때 표시한다. persistence가 아직 없으면 빈 panel을 만들지 않고 현재 상태와 최신 근거까지만 둔다.

표시 정보:

- 시작 시각
- 복구 또는 종료 시각
- lifecycle
- 추정 구간
- 대표 근거

v1에서는 timeline chart보다 incident list를 우선한다. incident가 충분히 쌓인 뒤에만 일별 안정성 막대나 추세 시각화를 검토한다.

## 원시 네트워크 확인

기존 manual raw probe는 유지한다.

- 기본 위치: 메인 화면 하단
- 기본 형태: 접을 수 있는 고급 영역
- 실행 동작: 사용자가 버튼을 눌렀을 때 한 번 수집
- 표현: raw JSON과 copy action

manual raw probe는 자동 진단 lifecycle을 바꾸지 않는다.

## 상태별 문구

### 정상

- 상태: `정상`
- 추정 구간: `해당 없음`
- 설명: `현재 확인된 구간에서 반복 이상이 없습니다.`

### 장애 의심

- 상태: `장애 의심`
- 추정 구간: 후보 구간 표시
- 설명: `이상이 감지됐지만 장애로 확정하려면 추가 근거가 필요합니다.`

### 장애 확인

- 상태: `장애 확인`
- 추정 구간: 확인된 구간 표시
- 설명: `호환되는 이상이 반복 확인되었습니다.`

### 복구 확인 중

- 상태: `복구 확인 중`
- 추정 구간: 이전 incident 구간 표시
- 설명: `정상 근거가 확인되어 추가 확인 중입니다.`

### 확인 불가

- 상태: `확인 불가`
- 추정 구간: `확인 불가`
- 설명: `진단 근거가 부족하거나 Windows 앱에서만 확인할 수 있습니다.`

## 텍스트 와이어프레임

```text
PC Health
인터넷 장애의 원인 구간을 근거와 함께 구분합니다.

[현재 네트워크 진단]
상태: 장애 의심
추정 구간: 공유기 또는 로컬 연결 구간
마지막 확인: 2026. 6. 30. 14:32:10
설명: 이상이 감지됐지만 장애로 확정하려면 추가 근거가 필요합니다.

[진단 경로]
PC/어댑터 성공 -> IPv4/Route 성공 -> 게이트웨이 시간 초과 -> DNS 확인하지 않음 -> 외부 연결 확인하지 않음

[최신 근거]
구간          상태        지연    세부 정보                 확인 시각
PC/어댑터     성공        -       Ethernet adapter 감지      14:32:10
IPv4/Route    성공        -       default route 선택됨       14:32:10
게이트웨이     시간 초과    1000ms  192.168.0.1 응답 없음     14:32:10
DNS           확인 안 함   -       집중 검사 전              확인 시각 없음
Microsoft     확인 안 함   -       집중 검사 전              확인 시각 없음
Google        확인 안 함   -       집중 검사 전              확인 시각 없음

[최근 장애]
최근 장애 기록이 있으면 list로 표시한다.

[원시 네트워크 확인]
네트워크 확인 실행
Raw JSON 접기/펼치기
```

## 제외 항목

- latency graph
- network health score
- 자동 복구 버튼
- router login 또는 설정 변경
- packet capture
- 방문 주소나 사용자 traffic 기록
- 외부 서버 업로드
- 성능 모니터링 dashboard
- 드라이버 관리 card

## 구현 메모

- 기존 `NetworkStatusPanel`을 확장하거나 route-local subcomponent로 분리한다.
- 새 generic design system을 만들지 않는다.
- CSS는 feature-local CSS Module을 사용한다.
- class 접근은 `styles["class-name"]` 형식을 따른다.
- 화면 폭과 tone은 `docs/ui-guidelines.md`의 dashboard shell 기준을 유지한다.
- unavailable, starting, error 상태에서도 layout이 크게 흔들리지 않아야 한다.

## 검증

- component test에서 `normal`, `suspected`, `incident`, `recovering`, `resolved`, `starting`, `unavailable`, `error` 렌더링을 확인한다.
- 진단 경로에서 failure, timeout, not checked, unavailable이 구분되는지 확인한다.
- latency가 graph나 판정 점수로 노출되지 않는지 확인한다.
- raw probe가 자동 진단 lifecycle과 분리되어 있는지 확인한다.
- Windows runtime 동작은 별도 artifact로 확인한다.
