# 네트워크 전용 제품 범위 설계

## 목표

PC Health의 제품 범위를 인터넷 장애 진단 전용으로 재정의한다. 드라이버 관리는 제품 범위, 앱 UI, durable guidance와 repo-local skill에서 완전히 제거한다. 성능 모니터링은 기존 결정대로 제거 상태를 유지한다.

## 제품 범위

| 영역 | 상태 | 방향 |
| --- | --- | --- |
| 인터넷 장애 진단 | `active` | 개인용 Windows PC가 유선 LAN으로 공유기에 연결된 환경에서 앱 실행 중 PC, 로컬 연결, 공유기, DNS, 외부 연결 상태를 근거와 함께 구분한다. |
| 드라이버 관리 | `removed` | 현재 제품에서 제외한다. inventory, Windows Update 후보, history, 설치·rollback·reboot 관련 지침과 UI를 유지하지 않는다. |
| 성능 모니터링 | `removed` | LibreHardwareMonitor, sensor helper, hardware telemetry 구조를 복구하지 않는다. |

## UI 방향

홈 화면은 네트워크 진단을 단일 제품 영역으로 보여준다.

- 드라이버 관리 planning card를 제거한다.
- 헤더 문구는 인터넷 장애 진단 전용 Windows 유틸리티로 설명한다.
- 현재 네트워크 진단 상태와 raw JSON 수동 probe는 유지한다.
- 새 기능이나 장식적 product card를 추가하지 않는다.

## Durable guidance 정리

다음 지침은 네트워크 전용으로 갱신한다.

- `AGENTS.md`: 제품 범위, 작업 순서, 안전 경계를 네트워크 진단 중심으로 재작성한다.
- `README.md`: 드라이버 상태 확인과 planning status를 제거한다.
- `docs/harness/pc-health/team-spec.md`: 드라이버 역할, handoff, planning pipeline과 차단 시나리오를 제거한다.
- `.agents/skills/pc-health-orchestrator/SKILL.md`: 제품 routing을 네트워크 active와 성능 monitoring removed로 축소한다.
- `.agents/skills/qa-safety-reviewer/SKILL.md`: driver action 검토 항목을 제거하고 네트워크 읽기 전용 안전 경계를 유지한다.
- `.agents/skills/repo-conventions-specialist/SKILL.md`: driver-planning 언급을 제거한다.
- `.agents/skills/ui-experience-specialist/SKILL.md`: network surface 전용 UI 지침으로 정리한다.
- `.agents/skills/driver-inventory-specialist/`: 삭제한다.

## Historical docs 처리

이미 완료된 과거 implementation plan은 작업 당시의 기록으로 유지한다. 다만 현재 제품 결정을 안내하는 spec, team spec, AGENTS, README, skills에는 드라이버 관리가 활성 또는 planning 영역처럼 보이지 않아야 한다.

## 안전 경계

제품이 네트워크 전용이 되어도 다음은 계속 금지한다.

- router login 또는 설정 변경
- packet capture와 사용자 트래픽 수집
- 방문 주소 기록
- Windows 네트워크 설정 자동 수정
- 자동 복구
- 원격 업로드
- 제거된 성능 모니터링 구조 복구

## 검증

1. 앱 UI에 드라이버 관리 카드나 상태 문구가 없다.
2. 현재 제품 지침에서 드라이버 관리가 `planning` 영역으로 남아 있지 않다.
3. `.agents/skills/driver-inventory-specialist/`가 제거된다.
4. 네트워크 진단 runtime, status panel, raw JSON manual probe는 유지된다.
5. `npm test`, `npm run build`, `cargo test --manifest-path src-tauri/Cargo.toml`, `cargo check --manifest-path src-tauri/Cargo.toml`을 통과한다.
