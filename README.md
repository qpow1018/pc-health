# PC Health

PC Health는 인터넷 장애의 원인 구간과 Windows 드라이버 상태를 확인하는 개인용 읽기 전용 데스크톱 유틸리티다.

## Status

- 인터넷 장애 진단: 다음 활성 개발 영역. 앱 실행 중 PC, 로컬 연결, 공유기, DNS, 외부 회선 상태를 단계적으로 관찰한다.
- 드라이버 관리: 기획 단계. 설치 정보, Windows Update 후보와 처리 이력의 읽기 전용 범위를 검토한다.
- 성능 모니터링: 제품 범위에서 제거되었다.

## Stack

- Tauri 2
- React and TypeScript
- Rust
- SQLite planned for local network incident history

## Development

```bash
npm install
npm run tauri dev
```

변경 범위에 따라 `npm test`, `npm run build`, `cargo test --manifest-path src-tauri/Cargo.toml`, `cargo check --manifest-path src-tauri/Cargo.toml`을 실행한다.

## Windows Test Build

`Windows Build` GitHub Actions workflow를 실행하고 완료된 run의 artifact를 Windows PC에서 설치해 검증한다.

## License

MIT
