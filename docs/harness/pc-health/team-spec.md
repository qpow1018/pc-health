# PC Health 하네스 팀 명세

## 목적

이 repo-local 하네스는 런타임 오케스트레이션을 추가하지 않고 pc-health 기능 작업을 안내한다. 현재 구현 pipeline은 LibreHardwareMonitor 기반 성능 모니터링에 집중한다. 인터넷 장애 진단과 드라이버 관리는 승인된 설계가 생길 때까지 planning 영역으로 유지한다.

기본 아키텍처는 Pipeline이며, 마지막에 Producer-Reviewer 방식의 QA 검토를 붙인다. 하드웨어, 네트워크, 드라이버, UI, repo convention 조사가 서로 독립적일 때만 제한적으로 병렬 조사를 허용한다.

## 제품 영역과 Lifecycle

| 영역 | 상태 | 하네스 동작 |
| --- | --- | --- |
| 성능 모니터링 | `active` | LibreHardwareMonitor 기반 설계·구현·검증 pipeline을 실행한다. |
| 인터넷 장애 진단 | `planning` | 요구사항과 안전 경계만 정리하며 승인된 설계 전에는 구현하지 않는다. |
| 드라이버 관리 | `planning` | 요구사항과 위험 단계를 분리하며 승인된 설계 전에는 구현하지 않는다. |

## 역할

| 역할 | 스킬 | 담당 |
| --- | --- | --- |
| 오케스트레이터 | `.agents/skills/pc-health-orchestrator/SKILL.md` | 범위 정의, 순서 결정, specialist 선택, 종합, 검증 요약 |
| 하드웨어 telemetry | `.agents/skills/hardware-telemetry-specialist/SKILL.md` | LHM helper, 전체 sensor inventory, Rust snapshot contract, mock, warning, Windows evidence |
| 네트워크 진단 | `.agents/skills/network-diagnostics-specialist/SKILL.md` | PC·공유기·외부 회선 문제 구분을 위한 기획 질문과 안전 경계 |
| 드라이버 관리 | `.agents/skills/driver-inventory-specialist/SKILL.md` | 버전·업데이트 가능 여부·처리 기록을 위한 기획 질문과 위험 경계 |
| UI 경험 | `.agents/skills/ui-experience-specialist/SKILL.md` | 데스크톱 유틸리티 layout, 상태 계층, dashboard 밀도, 사용자 흐름 |
| Repo convention | `.agents/skills/repo-conventions-specialist/SKILL.md` | 파일 배치, naming, 테스트 위치, Rust/TypeScript/Tauri 경계 |
| QA 안전 검토 | `.agents/skills/qa-safety-reviewer/SKILL.md` | 안전성 검토, 누락된 테스트, 위험 기능, contract drift |

## Handoff 파일

작업에 여러 단계가 있거나, 독립적인 specialist 조사 결과가 있거나, reviewer가 필요할 때만 `_workspace/`를 사용한다.

| 파일 | 담당 | 목적 |
| --- | --- | --- |
| `_workspace/01_request_summary.md` | 오케스트레이터 | 사용자 목표, 가정, 성공 기준, 선택된 specialist |
| `_workspace/02_hardware_telemetry_findings.md` | 하드웨어 telemetry | LHM helper contract, sensor mapping, invalid value, Windows evidence |
| `_workspace/02_network_diagnostics_findings.md` | 네트워크 진단 | planning 질문, 범위 후보, 외부 요청과 안전 위험 |
| `_workspace/02_driver_inventory_findings.md` | 드라이버 관리 | planning 질문, source 후보, 변경 작업의 위험 단계 |
| `_workspace/02_ui_experience_findings.md` | UI 경험 | 정보 구조와 상태 표시 메모 |
| `_workspace/02_repo_conventions_findings.md` | Repo convention | 파일 map, naming, 테스트, contract alignment |
| `_workspace/03_design_plan.md` | 오케스트레이터 | 구현 전 종합된 설계 또는 계획 |
| `_workspace/04_qa_review.md` | QA 안전 검토 | 판정, 발견 사항, 필수 수정, 검증 권장 사항 |

## 작업 흐름

1. 판단하기 전에 기존 문서와 코드를 먼저 읽는다.
2. 가정, 성공 기준, 가장 작은 유효 범위를 명시한다.
3. 제품 영역과 lifecycle을 분류하고 현재 단계에 필요한 specialist만 선택한다.
4. 독립 조사가 필요하면 같은 요청 snapshot을 기준으로 별도의 `_workspace/02_*_findings.md` 파일을 만든다.
5. 작업이 사소하지 않다면 `_workspace/03_design_plan.md`에 짧은 설계나 계획을 종합한다.
6. 기존 스타일과 파일 배치를 따라 필요한 부분만 수술적으로 구현한다.
7. 변경한 표면에 맞춰 집중 검증을 실행한다.
8. 위험하거나 경계를 넘나드는 작업은 최종 전달 전에 `qa-safety-reviewer`로 검토한다.

## 성능 모니터링 Pipeline

LibreHardwareMonitor는 Windows 성능 sensor의 기준 provider다. 목표 helper는 앱이 관리하는 persistent 프로세스이며 sample마다 새 프로세스를 실행하지 않는다. Rust는 helper 생명주기, IPC, 값 검증, domain mapping을 담당하고 별도 성능 provider를 병행하지 않는다.

1. LibreHardwareMonitor helper가 hardware, subhardware, sensor를 빠짐없이 열거한다.
2. helper contract가 type, name, identifier, parent device, unit, raw value, update error를 전달한다.
3. Rust collector가 helper 생명주기와 IPC를 관리하고 raw sensor를 domain 상태로 변환한다.
4. 승인된 대표 reading만 `SensorSnapshot`에 매핑하고 나머지는 raw inventory 또는 명시적 unavailable 상태로 유지한다.
5. frontend contract, mock, UI, warning 평가를 Rust contract와 함께 맞춘다.
6. Windows artifact에서 mapping 안정성, invalid value 처리, 권한, sampling 부하를 검증한다.

`null`, `0`, `NaN`, 무한대, 물리적으로 말이 안 되는 값은 `available`로 승격하지 않는다. 권한이 필요한 sensor 때문에 자동 권한 상승을 구현하지 않는다. raw sensor 수집 성공과 dashboard 표시 승인은 별개의 결정이다.

## 검증 기준

- 생성된 모든 `SKILL.md`는 `name`과 `description`이 있는 YAML frontmatter를 포함한다.
- Rust `domain.rs`는 sensor wire contract의 기준으로 유지한다.
- wire contract가 바뀌면 TypeScript 타입, mock 데이터, UI 렌더링, 테스트를 함께 갱신한다.
- LHM 통합은 helper contract와 Windows artifact 증거 없이 authoritative dashboard reading으로 승격하지 않는다.
- 인터넷 장애 진단과 드라이버 관리는 승인된 설계가 생기기 전에는 구현하지 않는다.
- UI 상태는 normal, caution, danger, unknown을 일관되게 구분한다.
- Frontend import는 같은 폴더의 `./`만 상대경로로 허용하고, 상위 폴더 접근 `../`는 피한다. 다른 영역은 `@/...` alias를 사용한다.
- Frontend 스타일은 `src/app/global.css`와 plain CSS Modules(`*.module.css`)를 사용한다. module CSS는 native nesting을 허용하고 JSX에서는 `styles['class-name']` 형태로 접근한다.
- AGENTS.md는 짧고 repo-wide하게 유지하고, 긴 역할 지침은 `.agents/skills/` 또는 `docs/harness/`에 둔다.

## 실패 정책

- 드라이버 설치, rollback, 관리자 권한 상승, OS 설정 변경, router 변경, packet capture, 원격 진단, 상시 백그라운드 모니터링을 구현하기 전에는 멈추고 사용자에게 확인한다.
- 하드웨어, 네트워크, 벤더 데이터를 확인할 수 없을 때는 잘못된 확신보다 `unknown`을 우선한다.
- 한쪽 contract만 바뀌었다면 Rust, TypeScript, mock 데이터, 테스트가 다시 맞춰질 때까지 완료로 보지 않는다.
- `planning` 영역의 요청이 구현을 요구하면 설계 단계로 되돌리고 lifecycle 변경 전에는 코드를 작성하지 않는다.
- 요청된 기능이 하드웨어, 네트워크, 드라이버, UI 작업을 한 번에 묶는다면 제품 영역별 설계와 좁은 구현 slice로 나눈다.

## 시나리오 점검

### Active 시나리오

요청: SSD 온도와 팬 속도를 성능 모니터링에 추가한다.

1. 오케스트레이터가 요청을 `active` 성능 모니터링으로 분류한다.
2. `hardware-telemetry-specialist`가 LHM raw inventory와 현재 `SensorSnapshot`을 확인한다.
3. raw sensor와 user-facing 대표 reading의 mapping을 설계한다.
4. Rust contract, TypeScript, mock, UI, warning 영향을 함께 검토한다.
5. Windows artifact에서 helper 전달, mapping, invalid value, sampling 부하를 검증한다.

기대 결과: 별도 provider를 추가하지 않고 LHM 경로 안에서 검증된 reading만 제품에 연결한다.

### Planning 시나리오

요청: 인터넷 장애 진단 패널을 추가한다.

1. 오케스트레이터가 요청을 `planning` 인터넷 장애 진단으로 분류한다.
2. `network-diagnostics-specialist`가 PC·공유기·외부 회선 구분 기준, 외부 target, timeout, 개인정보와 실행 방식을 질문으로 정리한다.
3. `_workspace/02_network_diagnostics_findings.md`와 설계 제안까지만 만든다.
4. 승인된 설계와 lifecycle 변경 전에는 Tauri command나 UI를 구현하지 않는다.

기대 결과: 아직 정해지지 않은 contract를 코드가 선점하지 않는다.

### 실패 시나리오

요청: 드라이버 버전 확인과 업데이트 설치를 추가한다.

1. 오케스트레이터가 `driver-inventory-specialist`와 `qa-safety-reviewer`로 라우팅한다.
2. 드라이버 specialist가 지원 device, authoritative source, confidence, offline behavior, action history와 변경 단계의 질문을 정리한다.
3. QA는 inventory와 update-check를 포함한 기능 전체가 아직 planning이며, download·설치·rollback·권한 상승은 추가 안전 승인이 필요하므로 구현을 차단한다.

기대 결과: 승인된 드라이버 관리 설계 전에는 contract나 구현을 만들지 않고, 시스템 변경 작업은 별도 위험 단계로 남긴다.
