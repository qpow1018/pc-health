# Net Checker 브랜딩 설계

## 목표

PC Health의 현재 제품 표시명과 패키징 metadata를 `Net Checker`로 바꾼다. 제품 범위는 계속 인터넷 연결 문제를 근거와 함께 확인하는 읽기 전용 Windows 유틸리티다.

## 범위

- 앱 표시 이름: `Net Checker`
- npm package name: `net-checker`
- Tauri product name과 window title: `Net Checker`
- Tauri identifier: `com.choewonjin.net-checker`
- 설명: `인터넷 연결 문제를 근거와 함께 확인하는 Windows 유틸리티`
- 현재 README, repo-wide guidance, UI guideline, harness team spec의 제품명 표기를 갱신한다.
- Windows build artifact 이름은 새 제품명에 맞춰 갱신한다.
- 기존 Tauri 아이콘 세트를 새 로고에서 다시 생성한다.

## 비범위

- 과거 spec과 plan의 기록성 본문은 대량 수정하지 않는다.
- 네트워크 진단 동작, incident 저장, probe, lifecycle 문구는 바꾸지 않는다.
- 제거된 성능 모니터링 구조를 복구하지 않는다.

## 로고 방향

텍스트 없는 앱 아이콘으로 만든다. 작은 크기에서도 읽히도록 네트워크 연결 노드와 체크 표시를 결합한 단순한 형태를 사용한다. 제품 UI의 조용한 데스크톱 유틸리티 톤에 맞춰 과한 장식이나 경고 색상은 피한다.

## 검증

- `npm test`
- `npm run build`
- 아이콘 파일이 `src-tauri/icons/`에 다시 생성됐는지 확인
- `rg`로 현재 표면의 `PC Health`와 `pc-health` 잔여 표기를 확인
