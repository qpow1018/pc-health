---
name: driver-inventory-specialist
description: pc-health 드라이버 inventory와 update-check 흐름을 설계하고, 설치 작업 전 명시적 safety gate가 필요할 때 사용한다.
---

# 드라이버 Inventory Specialist

## 언제 사용할지
- 드라이버 버전, 설치일, 벤더, device identity를 읽거나 새 드라이버 존재 여부를 확인하는 작업에 사용한다.
- NVIDIA 우선 driver inventory 계획에 사용한다.
- 별도로 승인된 phase가 명시적으로 요청하지 않았다면 이 스킬에서 드라이버 설치를 구현하지 않는다.

## 필요한 입력
- 사용자가 승인한 device 범위. 보통 Windows 우선, NVIDIA 우선이다.
- 기존 Rust/Tauri command 구조.
- normal, caution, danger, unknown 상태에 대한 현재 UI status 패턴.

## 작업 흐름
1. 작업을 inventory, update availability, installation phase로 나눈다.
2. MVP 작업에서는 읽기 전용 inventory와 안내성 update check만 구현하거나 설계한다.
3. Update availability에는 source와 confidence를 함께 보여준다. 벤더 데이터를 안전하게 확인할 수 없다면 unknown도 허용된다.
4. 별도의 명시적 승인 단계 없이 드라이버를 download, install, roll back, 변경하지 않는다.
5. 관리자 권한, installer, reboot, rollback, install failure는 고위험 후속 phase 관심사로 다룬다.

## 설치 Phase Gate
드라이버 설치는 별도의 위험 phase다. 구현 전 다음을 요구한다:

- 앱 안에서의 명시적 사용자 승인과 개발 요청에서의 명시적 승인
- rollback 또는 recovery policy
- partial install, reboot required, network failure, vendor installer error에 대한 실패 처리
- 코드 작성 전 QA review

## 출력
- source, contract, risk 메모를 위한 `_workspace/02_driver_inventory_findings.md`.
- version, install date, vendor, device, update status를 포함한 driver inventory contract.
- installation, rollback, vendor-specific automation에 대한 deferred phase 목록.

## 검증
- Inventory는 반드시 읽기 전용이어야 한다.
- Update check는 안전하고 눈에 보이게 실패해야 한다.
- 새 버전이 있다는 이유만으로 현재 드라이버가 unsafe하다고 암시하지 않는다.
