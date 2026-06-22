# PC Health 하네스 팀 명세

## 목적

이 하네스는 PC Health의 인터넷 장애 진단 구현과 드라이버 관리 기획을 안내한다. 기본 구조는 순차 pipeline이며 위험하거나 경계를 넘는 작업 끝에 QA 검토를 붙인다.

## 제품 Lifecycle

| 영역 | 상태 | 허용 범위 |
| --- | --- | --- |
| 인터넷 장애 진단 | `active` | 승인된 설계 안에서 계획, 구현, 검증한다. |
| 드라이버 관리 | `planning` | 읽기 전용 범위와 데이터 source를 설계한다. 코드는 작성하지 않는다. |
| 성능 모니터링 | `removed` | 관련 provider, helper, contract, UI를 복구하지 않는다. |

## 역할

| 역할 | 스킬 | 담당 |
| --- | --- | --- |
| 오케스트레이터 | `.agents/skills/pc-health-orchestrator/SKILL.md` | 제품 영역 분류, lifecycle routing, 계획, 종합 |
| 네트워크 진단 | `.agents/skills/network-diagnostics-specialist/SKILL.md` | probe, 판정 근거, incident 기록, Windows 검증 |
| 드라이버 관리 | `.agents/skills/driver-inventory-specialist/SKILL.md` | inventory, update 후보, history의 읽기 전용 설계 |
| UI 경험 | `.agents/skills/ui-experience-specialist/SKILL.md` | 현재 상태, 최근 장애, 이력 화면과 불확실성 표현 |
| Repo convention | `.agents/skills/repo-conventions-specialist/SKILL.md` | 파일 배치, Rust/TypeScript/Tauri 경계, 테스트 |
| QA 안전 검토 | `.agents/skills/qa-safety-reviewer/SKILL.md` | lifecycle, 외부 요청, 보존 정책, 권한과 변경 동작 검토 |

## Handoff 파일

여러 단계 작업이나 명시적 검토가 필요할 때만 `_workspace/`를 사용한다.

| 파일 | 목적 |
| --- | --- |
| `_workspace/01_request_summary.md` | 목표, 가정, 성공 기준, lifecycle |
| `_workspace/02_network_diagnostics_findings.md` | probe·판정·저장·Windows 증거 |
| `_workspace/02_driver_inventory_findings.md` | source 후보, 한계, 위험 단계 |
| `_workspace/02_ui_experience_findings.md` | 정보 구조와 상태 문구 |
| `_workspace/02_repo_conventions_findings.md` | 최소 file map과 contract 경계 |
| `_workspace/03_design_plan.md` | 승인된 설계를 구현 단위로 분해한 계획 |
| `_workspace/04_qa_review.md` | verdict, finding, 필수 수정, 검증 권장 사항 |

## 공통 흐름

1. 최신 spec, 관련 코드와 테스트를 읽는다.
2. 제품 영역, lifecycle, 가정, 성공 기준을 명시한다.
3. 필요한 specialist만 선택하고 가장 작은 변경 계획을 만든다.
4. 현재 구조에 맞춰 수술적으로 구현한다. `planning` 영역은 구현하지 않는다.
5. 변경한 표면을 집중 검증하고 OS 의존 동작은 Windows artifact로 확인한다.
6. 외부 요청, 로컬 기록, 권한 또는 시스템 변경이 관련되면 QA 검토를 수행한다.

## 인터넷 장애 진단 Pipeline

초기 대상은 공유기에 유선 LAN으로 연결된 개인용 Windows PC다. 앱 실행 중에만 동작하며 Wi-Fi, VPN, proxy, Windows service는 범위 밖이다.

1. 유선 adapter, IP 주소, default route와 gateway를 읽는다.
2. 낮은 빈도로 gateway와 Windows 연결 상태를 관찰한다.
3. Microsoft와 Google connectivity endpoint를 제한된 timeout·동시성·재시도로 확인한다.
4. 이상이 반복될 때만 DNS와 외부 연결 집중 검사를 짧게 수행하고 cooldown을 둔다.
5. LAN 설정 없음, gateway 실패, system DNS만 실패, 복수 외부 대상 실패를 서로 다른 근거로 보존한다.
6. 단일 timeout으로 장애를 확정하지 않고 근거 부족은 `unknown` 또는 `확인 불가`로 둔다.
7. `normal`, `suspected`, `incident`, `recovering`, `resolved` lifecycle을 UI와 기록에 일관되게 사용한다.
8. SQLite에는 incident를 무기한, 개별 probe를 24시간 보존한다. packet 내용, 방문 주소, 사용자 트래픽은 저장하지 않는다.

외부 endpoint, polling 주기, timeout, 집중 검사 조건은 구현 전에 Windows probe로 검증한다. router 설정 변경, packet capture, 자동 복구와 외부 서버 업로드는 하지 않는다.

## 드라이버 관리 Planning

- 설치 정보는 SetupAPI 또는 Configuration Manager API 후보를 검토한다.
- 업데이트 정보는 WUA에서 발견된 후보로 표현하며 vendor 전체의 최신 버전을 보장하지 않는다.
- WUA history는 Windows Update 처리 이력일 뿐 모든 수동·vendor 설치 기록이 아니다.
- download, install, rollback, reboot, 자동 권한 상승과 Windows Update 설정 변경은 별도 위험 단계다.
- 승인된 설계가 lifecycle을 바꾸기 전에는 source code, Tauri command, UI contract를 만들지 않는다.

## Repo와 UI 규칙

- 같은 frontend 폴더는 `./`, 다른 영역은 `@/...`를 사용하고 `../` 접근은 피한다.
- 전역 규칙은 `src/app/global.css`, feature 스타일은 plain CSS Modules와 native nesting을 사용한다.
- JSX의 CSS Module class는 `styles['class-name']` 형태로 접근한다.
- UI는 현재 상태, 근거, 마지막 확인 시각을 우선하고 과장된 경고나 근거 없는 확정을 피한다.
- `AGENTS.md`는 짧게 유지하고 긴 절차는 이 문서나 specialist skill에 둔다.

## 실패 정책

- `planning` 영역이 구현으로 넘어가면 중단한다.
- 외부 대상 하나의 실패만으로 인터넷 장애나 원인을 확정하면 중단한다.
- probe 제한, SQLite 보존·정리, 개인정보 제외가 누락되면 중단한다.
- driver 변경 동작이나 자동 권한 상승이 승인 없이 포함되면 중단한다.
- 성능 모니터링 구조를 재도입하는 변경은 새로운 제품 결정 없이는 중단한다.

## 시나리오 점검

### Active

요청: gateway는 정상인데 인터넷이 끊기는 사건을 기록한다.

기대: 복수 외부 근거와 DNS 결과를 수집하고 단일 timeout은 `suspected`로만 처리한다. incident 근거는 보존하고 정상 probe 원본은 24시간 뒤 정리한다.

### Planning

요청: 설치된 드라이버와 업데이트 가능 여부를 표시한다.

기대: SetupAPI·Configuration Manager·WUA의 source와 한계를 설계하고 구현은 승인 전까지 시작하지 않는다.

### 차단

요청: 장애 시 router를 재시작하거나 driver를 자동 설치한다.

기대: 읽기 전용 범위 밖의 시스템 변경으로 분류하고 별도 설계와 명시적 승인을 요구한다.
