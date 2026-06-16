# PC Health

PC Health is a read-only Windows desktop performance monitor for CPU, NVIDIA GPU, and memory metrics.

## Status

The project is in the MVP design and bootstrap phase. Development happens on macOS with mock sensor data, and hardware integration is verified on Windows.

## Stack

- Tauri 2
- React and TypeScript
- Rust
- SQLite planned for recorded sessions

## Development

```bash
npm install
npm run tauri dev
```

## Windows Test Build

The Windows PC does not need the full development toolchain for basic app testing.
Run the `Windows Build` GitHub Actions workflow, then download the generated
artifact from the completed workflow run and install or run it on Windows.

## License

MIT
