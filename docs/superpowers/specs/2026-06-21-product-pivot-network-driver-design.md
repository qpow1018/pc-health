# PC Health 제품 전환 설계

## 목표

PC Health의 제품 범위를 인터넷 장애 진단과 드라이버 관리로 재편한다. 성능 모니터링은 Windows 센서 provider의 하드웨어별 제약과 유지 비용 때문에 제품에서 완전히 제거한다.

이번 전환은 다음 순서로 진행한다.

1. 기존 성능 모니터링 구현과 지침을 제거한다.
2. 승인된 인터넷 장애 진단 설계를 저장소의 다음 활성 개발 영역으로 전환한다.
3. 드라이버 관리는 읽기 전용 구현 가능성만 확인된 planning 영역으로 유지한다.

## 제품 영역과 Lifecycle

| 제품 영역 | Lifecycle | 방향 |
| --- | --- | --- |
| 인터넷 장애 진단 | `active` | 앱 실행 중 상시 관찰하고 PC·로컬 연결·공유기·DNS·외부 회선 문제를 근거와 함께 구분한다. |
| 드라이버 관리 | `planning` | 설치 버전, Windows Update 후보, 처리 이력의 읽기 전용 범위를 별도 설계한다. |
| 성능 모니터링 | `removed` | UI, domain, collector, helper, mock, warning, 테스트, 빌드 설정에서 제거한다. |

## 브랜치와 이력 처리

- `codex/cpu-persistent-sensor-helper`는 `dev`에 병합하지 않는다.
- 제품 전환은 `dev`의 `39b6f40`에서 분기한 `codex/product-pivot-cleanup`에서 수행한다.
- 제거되는 성능 모니터링 구현은 Git 이력으로만 보존한다.
- 제거한 코드를 비활성 상태로 남기거나 미래 사용을 위한 abstraction으로 유지하지 않는다.

## 인터넷 장애 진단

### 사용자 환경과 실행 범위

- 개인용 Windows PC 한 대를 대상으로 한다.
- PC는 공유기에 유선 LAN으로 연결된다.
- 초기 범위에서는 Wi-Fi, VPN, proxy를 다루지 않는다.
- 진단은 Windows 서비스가 아니라 PC Health가 실행 중인 동안만 동작한다.
- Windows toast나 소리 알림은 사용하지 않고 앱 화면에만 상태를 표시한다.

### 진단 방식

낮은 부하의 평상시 관찰과 장애 의심 시 집중 검사를 나누는 단계형 상시 진단을 사용한다.

평상시 관찰 대상은 다음과 같다.

- 유선 어댑터 연결 상태
- IP 주소와 기본 경로 변경
- default gateway 응답과 지연
- Windows 연결 상태
- Microsoft와 Google 연결 확인 endpoint의 가벼운 응답 검사

외부 확인 후보는 다음과 같다.

- `http://www.msftconnecttest.com/connecttest.txt`
- `https://connectivitycheck.gstatic.com/generate_204`

외부 요청은 개인용 범위에서도 보수적으로 운용한다. 동시 요청, 큰 응답 다운로드, 무제한 재시도를 금지하고 이상 감지 시 짧은 집중 검사와 cooldown을 적용한다. 최종 주기와 timeout은 구현 계획 전에 Windows probe로 검증한다.

### 초기 판정 범위

초기 판정은 큰 장애 구간만 구분하고 실제 기록을 바탕으로 점진적으로 고도화한다.

- LAN link, IP 주소 또는 기본 경로가 없으면 PC 또는 로컬 연결 문제로 본다.
- 기본 설정은 있지만 gateway가 응답하지 않으면 공유기·LAN 케이블·포트 구간 문제로 본다.
- gateway는 정상이지만 system DNS만 실패하면 DNS 문제로 본다.
- gateway는 정상이고 독립된 외부 대상이 함께 실패하면 외부 회선 문제로 본다.
- gateway부터 지연이 크면 로컬 네트워크 또는 공유기 구간으로 본다.
- gateway는 정상이고 외부 지연만 크면 통신사 또는 외부 구간으로 본다.
- 소프트웨어 증거만으로 케이블, 공유기 포트, NIC를 구분할 수 없으면 `로컬 연결 구간 문제`로 표시한다.
- 대상 하나의 실패나 일회성 timeout만으로 장애를 확정하지 않는다.
- 근거가 부족하거나 probe가 차단되면 `확인 불가`를 사용한다.

### 상태와 앱 화면

진단 상태는 다음 lifecycle을 사용한다.

- `normal`
- `suspected`
- `incident`
- `recovering`
- `resolved`

앱 화면은 다음 세 영역을 제공한다.

- 현재 상태: 상태, 추정 구간, 마지막 확인 시각
- 최근 장애: 발생·복구 시각, 지속 시간, 추정 위치
- 장애 이력: 날짜별 목록과 gateway·DNS·외부 probe 근거

앱이 보이지 않는 동안 별도 OS 알림을 띄우지 않는다. 사용자가 앱을 다시 열면 마지막 확인 이후 발생한 장애와 복구 이력을 확인할 수 있다.

### 로컬 기록

- 별도 DB 서버 없이 사용자 PC의 단일 SQLite 파일을 사용한다.
- 장애 사건은 발생·복구 시각, 추정 구간, confidence, 판정 근거와 당시 네트워크 상태를 무기한 보관한다.
- 개별 probe 결과는 최근 24시간만 보관한다.
- 오래된 정상 측정값은 삭제하며 장애 사건의 근거는 유지한다.
- packet 내용, 방문 주소, 사용자 트래픽을 저장하지 않는다.
- 프로그램 화면에서 기록을 확인하며 직접 DB 파일을 열어보는 기능은 요구하지 않는다.

### 안전 경계

다음 기능은 인터넷 장애 진단 범위에 포함하지 않는다.

- router login 또는 설정 변경
- packet capture와 사용자 트래픽 검사
- 방문 사이트 기록
- Windows 네트워크 설정 자동 수정
- VPN·proxy·Wi-Fi 진단
- 앱 종료 후 동작하는 Windows service
- 원격 진단과 외부 서버로의 기록 업로드

## 드라이버 관리 가능성

드라이버 관리는 이번 cleanup에서 구현하지 않는다. 다음 읽기 전용 범위는 별도 설계를 거치면 구현 가능하다.

### 설치된 드라이버

Windows SetupAPI 또는 Configuration Manager API를 사용해 장치명, 제조사, device instance ID, driver version, date, provider와 INF 정보를 읽을 수 있다. 관리자 권한 없는 읽기 전용 inventory를 목표로 한다.

### 업데이트 후보

Windows Update Agent 검색 결과에서 설치되지 않은 driver update 후보를 확인할 수 있다. 이 결과는 모든 vendor의 최신 버전을 보장하지 않으므로 `최신 버전`이 아니라 `Windows Update에서 발견된 후보`로 표시하고 source와 확인 시각을 함께 보존한다.

### 처리 이력

Windows Update Agent history에서 Windows Update가 처리한 설치 결과를 읽을 수 있다. vendor installer나 사용자가 직접 설치한 모든 기록을 완전하게 복원한다고 약속하지 않는다.

### 제외할 위험 단계

다음 기능은 별도 설계와 명시적 안전 승인이 있기 전까지 구현하지 않는다.

- driver 또는 vendor utility download
- 자동 설치와 관리자 권한 상승
- reboot 실행
- rollback
- Windows Update 설정 변경

## 성능 모니터링 제거 범위

cleanup 구현은 다음 항목을 제거한다.

- CPU, GPU, memory sensor dashboard와 diagnostics UI
- `SensorSnapshot`, sensor 상태, warning evaluator와 mock scenario
- Windows performance collector와 sensor helper 호출
- LibreHardwareMonitor .NET helper와 helper tests
- helper build·smoke scripts와 Tauri sidecar 설정
- sensor 전용 frontend hook, API, type, fixture와 테스트
- sensor helper를 위한 CI .NET setup, build, smoke 단계
- 성능 모니터링 전용 spec과 implementation plan
- hardware telemetry 전용 skill과 LHM 중심 지침

다음 항목은 유지한다.

- Tauri + React + Rust 애플리케이션 shell
- 공통 CSS와 UI guideline
- 아이콘과 패키징 기본 설정
- frontend test와 Rust test를 실행하는 Windows build workflow
- repo convention, QA, network, driver specialist 역할

cleanup 후 앱 화면은 인터넷 장애 진단과 드라이버 관리의 최소 진입 구조만 표시한다. 실제 probe, SQLite schema, driver inventory contract를 cleanup에 미리 구현하지 않는다.

## 지침 재정렬

cleanup은 코드와 함께 다음 durable guidance를 갱신한다.

- `AGENTS.md`: 두 제품 영역, lifecycle, 읽기 전용 원칙, Windows 검증 명령
- `docs/harness/pc-health/team-spec.md`: network active pipeline과 driver planning 경계
- `pc-health-orchestrator`: network와 driver 중심 routing
- `network-diagnostics-specialist`: 승인된 설계를 반영한 active 구현 workflow
- `driver-inventory-specialist`: planning과 읽기 전용 가능성 유지
- `qa-safety-reviewer`: 네트워크 외부 요청, SQLite retention, driver 변경 위험 검토
- `repo-conventions-specialist`와 `ui-experience-specialist`: sensor 전용 가정 제거

## 검증

cleanup은 다음 조건을 모두 만족해야 한다.

1. source, test, package script, Tauri config와 CI에 LibreHardwareMonitor 또는 sensor helper 참조가 없다.
2. 사용자 화면과 wire contract에 CPU, GPU, memory 성능 모니터링 surface가 없다.
3. 네트워크와 드라이버 구현을 cleanup 커밋에 선행 구현하지 않는다.
4. 앱은 macOS 개발 환경에서 frontend test, frontend build, Rust test와 Rust check를 통과한다.
5. Windows workflow는 helper 없이 Windows installer를 생성한다.
6. active guidance는 인터넷 장애 진단을 다음 구현 영역으로 가리킨다.
7. driver guidance는 download, install, reboot, rollback을 planning 위험 단계로 유지한다.
8. 현재 sensor feature 브랜치는 병합하거나 삭제하지 않는다.

## 후속 작업

1. 이 설계를 기준으로 cleanup implementation plan을 작성하고 수행한다.
2. cleanup을 `dev`에 반영한 뒤 새 세션에서 인터넷 장애 진단 implementation design을 검토한다.
3. 네트워크 MVP가 안정된 후 드라이버 inventory와 Windows Update 후보 조회를 별도 설계한다.
