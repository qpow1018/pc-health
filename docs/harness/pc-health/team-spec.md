# PC Health 하네스 팀 명세

## 목적

이 repo-local 하네스는 런타임 오케스트레이션을 추가하지 않고 pc-health 기능 작업을 안내한다. MVP의 기본 방향은 읽기 전용 Windows 진단, NVIDIA 우선 하드웨어 telemetry, 실용적인 네트워크 진단, 신중한 드라이버 inventory에 둔다.

기본 아키텍처는 Pipeline이며, 마지막에 Producer-Reviewer 방식의 QA 검토를 붙인다. 하드웨어, 네트워크, 드라이버, UI, repo convention 조사가 서로 독립적일 때만 제한적으로 병렬 조사를 허용한다.

## 역할

| 역할 | 스킬 | 담당 |
| --- | --- | --- |
| 오케스트레이터 | `.agents/skills/pc-health-orchestrator/SKILL.md` | 범위 정의, 순서 결정, specialist 선택, 종합, 검증 요약 |
| 하드웨어 telemetry | `.agents/skills/hardware-telemetry-specialist/SKILL.md` | CPU/GPU/메모리/저장장치 reading, Rust snapshot contract, mock collector, warning evaluation |
| 네트워크 진단 | `.agents/skills/network-diagnostics-specialist/SKILL.md` | NIC, gateway, DNS, public reachability 진단 |
| 드라이버 inventory | `.agents/skills/driver-inventory-specialist/SKILL.md` | 드라이버 버전, 설치일, 벤더, update-check 흐름, 설치 위험 분리 |
| UI 경험 | `.agents/skills/ui-experience-specialist/SKILL.md` | 데스크톱 유틸리티 layout, 상태 계층, dashboard 밀도, 사용자 흐름 |
| Repo convention | `.agents/skills/repo-conventions-specialist/SKILL.md` | 파일 배치, naming, 테스트 위치, Rust/TypeScript/Tauri 경계 |
| QA 안전 검토 | `.agents/skills/qa-safety-reviewer/SKILL.md` | 안전성 검토, 누락된 테스트, 위험 기능, contract drift |

## Handoff 파일

작업에 여러 단계가 있거나, 독립적인 specialist 조사 결과가 있거나, reviewer가 필요할 때만 `_workspace/`를 사용한다.

| 파일 | 담당 | 목적 |
| --- | --- | --- |
| `_workspace/01_request_summary.md` | 오케스트레이터 | 사용자 목표, 가정, 성공 기준, 선택된 specialist |
| `_workspace/02_hardware_telemetry_findings.md` | 하드웨어 telemetry | 센서 contract, collector 영향, warning/test 메모 |
| `_workspace/02_network_diagnostics_findings.md` | 네트워크 진단 | local-to-public 단계 진단 계획과 위험 |
| `_workspace/02_driver_inventory_findings.md` | 드라이버 inventory | inventory source, update-check 정책, 설치 phase gate |
| `_workspace/02_ui_experience_findings.md` | UI 경험 | 정보 구조와 상태 표시 메모 |
| `_workspace/02_repo_conventions_findings.md` | Repo convention | 파일 map, naming, 테스트, contract alignment |
| `_workspace/03_design_plan.md` | 오케스트레이터 | 구현 전 종합된 설계 또는 계획 |
| `_workspace/04_qa_review.md` | QA 안전 검토 | 판정, 발견 사항, 필수 수정, 검증 권장 사항 |

## 작업 흐름

1. 판단하기 전에 기존 문서와 코드를 먼저 읽는다.
2. 가정, 성공 기준, 가장 작은 유효 범위를 명시한다.
3. 필요한 specialist만 선택한다.
4. 독립 조사가 필요하면 같은 요청 snapshot을 기준으로 별도의 `_workspace/02_*_findings.md` 파일을 만든다.
5. 작업이 사소하지 않다면 `_workspace/03_design_plan.md`에 짧은 설계나 계획을 종합한다.
6. 기존 스타일과 파일 배치를 따라 필요한 부분만 수술적으로 구현한다.
7. 변경한 표면에 맞춰 집중 검증을 실행한다.
8. 위험하거나 경계를 넘나드는 작업은 최종 전달 전에 `qa-safety-reviewer`로 검토한다.

## 하드웨어 Provider 통합 순서

LibreHardwareMonitorLib, NVML, WMI provider, vendor SDK처럼 Windows 하드웨어 provider를 새로 도입하거나 교체할 때는 기능 구현이 아니라 provider 검증으로 시작한다.

1. provider를 앱 안에서 실행할 수 있는 최소 경로를 만든다.
2. 먼저 diagnostics/harness 출력에 raw hardware, sensor name, sensor type, identifier, raw value, update error, helper exit status를 노출한다.
3. Windows artifact에서 raw 센서 목록과 값이 실제 장비에서 신뢰 가능한지 확인한다.
4. 후보 sensor 선택 규칙과 invalid value 처리 기준을 정한다. `0`, `NaN`, 무한대, 물리적으로 말이 안 되는 값은 dashboard 값으로 승격하지 않는다.
5. 위 증거가 생긴 뒤에만 `SensorSnapshot`의 공식 reading에 연결한다.

이 순서를 건너뛰면 dashboard가 provider 통합 실패를 정상 telemetry처럼 보이게 만들 수 있으므로 완료로 보지 않는다.

## 검증 기준

- 생성된 모든 `SKILL.md`는 `name`과 `description`이 있는 YAML frontmatter를 포함한다.
- Rust `domain.rs`는 sensor wire contract의 기준으로 유지한다.
- wire contract가 바뀌면 TypeScript 타입, mock 데이터, UI 렌더링, 테스트를 함께 갱신한다.
- 새 하드웨어 provider는 raw diagnostics 증거와 Windows artifact 검증 전에는 authoritative dashboard reading으로 취급하지 않는다.
- 네트워크 진단은 local NIC에서 gateway, DNS, public reachability 순서로 진행한다.
- 별도 승인된 설치 phase가 없으면 드라이버 inventory는 읽기 전용으로 유지한다.
- UI 상태는 normal, caution, danger, unknown을 일관되게 구분한다.
- Frontend import는 같은 폴더의 `./`만 상대경로로 허용하고, 상위 폴더 접근 `../`는 피한다. 다른 영역은 `@/...` alias를 사용한다.
- Frontend 스타일은 `src/app/global.css`와 plain CSS Modules(`*.module.css`)를 사용한다. module CSS는 native nesting을 허용하고 JSX에서는 `styles['class-name']` 형태로 접근한다.
- AGENTS.md는 짧고 repo-wide하게 유지하고, 긴 역할 지침은 `.agents/skills/` 또는 `docs/harness/`에 둔다.

## 실패 정책

- 드라이버 설치, rollback, 관리자 권한 상승, OS 설정 변경, router 변경, packet capture, 원격 진단, 상시 백그라운드 모니터링을 구현하기 전에는 멈추고 사용자에게 확인한다.
- 하드웨어, 네트워크, 벤더 데이터를 확인할 수 없을 때는 잘못된 확신보다 `unknown`을 우선한다.
- 한쪽 contract만 바뀌었다면 Rust, TypeScript, mock 데이터, 테스트가 다시 맞춰질 때까지 완료로 보지 않는다.
- 요청된 기능이 하드웨어, 네트워크, 드라이버, UI 작업을 한 번에 묶는다면 설계 단계와 좁은 구현 slice로 나눈다.

## 시나리오 점검

### 정상 시나리오

요청: 읽기 전용 네트워크 진단 패널을 추가한다.

1. 오케스트레이터가 현재 문서, `Dashboard`, Tauri command 패턴을 읽는다.
2. `network-diagnostics-specialist`, `ui-experience-specialist`, `repo-conventions-specialist`로 라우팅한다.
3. Specialist들이 단계별 진단, compact UI 배치, 파일 map 메모를 만든다.
4. 오케스트레이터가 `_workspace/03_design_plan.md`에 내용을 종합한다.
5. 구현은 읽기 전용 command, typed frontend result, UI 상태를 추가한다.
6. QA는 네트워크 설정을 변경하지 않았고 실패가 단계별로 분리되는지 확인한다.

기대 결과: 앱은 문제 근거가 local device, gateway/router, DNS, public internet, unknown 중 어디를 가리키는지 설명할 수 있으며, 네트워크를 고치는 척하지 않는다.

### 실패 시나리오

요청: 최신 NVIDIA 드라이버를 자동 설치한다.

1. 오케스트레이터가 `driver-inventory-specialist`와 `qa-safety-reviewer`로 라우팅한다.
2. 드라이버 specialist가 inventory/update detection과 설치를 분리한다.
3. QA는 설치가 명시적 승인, rollback/failure policy, 관리자 권한 처리, 별도 설계를 필요로 하므로 blocked로 표시한다.

기대 결과: 하네스는 읽기 전용 inventory 또는 update 안내는 허용하지만, 초기 MVP 단계에서 자동 설치는 차단한다.
