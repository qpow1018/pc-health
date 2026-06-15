# PC Health Project Bootstrap Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create a public MIT-licensed `pc-health` GitHub repository and a matching local Tauri 2, React, TypeScript, and Vite project.

**Architecture:** The repository starts with the standard Tauri React template so the frontend and Rust native shell build together from day one. Product design documents live in the repository, while generated build files and local measurement databases remain ignored.

**Tech Stack:** Tauri 2, React, TypeScript, Vite, Rust, npm, Git, GitHub

---

### Task 1: Scaffold The Desktop Application

**Files:**
- Create: `/Users/choewonjin/Desktop/workspace/pc-health/package.json`
- Create: `/Users/choewonjin/Desktop/workspace/pc-health/src/`
- Create: `/Users/choewonjin/Desktop/workspace/pc-health/src-tauri/`

- [ ] **Step 1: Generate the project from the official Tauri template**

```bash
cd /Users/choewonjin/Desktop/workspace
npm create tauri-app@latest pc-health -- --manager npm --template react-ts --tauri-version 2 --yes
```

Expected: a `pc-health` directory containing React source and `src-tauri/Cargo.toml`.

- [ ] **Step 2: Install JavaScript dependencies**

```bash
cd /Users/choewonjin/Desktop/workspace/pc-health
npm install
```

Expected: dependencies install successfully and `package-lock.json` exists.

- [ ] **Step 3: Verify both toolchains**

```bash
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: both commands exit successfully.

### Task 2: Add Repository Metadata And Approved Design

**Files:**
- Create: `/Users/choewonjin/Desktop/workspace/pc-health/LICENSE`
- Create: `/Users/choewonjin/Desktop/workspace/pc-health/README.md`
- Create: `/Users/choewonjin/Desktop/workspace/pc-health/docs/superpowers/specs/2026-06-15-windows-performance-monitor-design.md`
- Create: `/Users/choewonjin/Desktop/workspace/pc-health/docs/superpowers/plans/2026-06-15-pc-health-project-bootstrap.md`
- Modify: `/Users/choewonjin/Desktop/workspace/pc-health/.gitignore`

- [ ] **Step 1: Add the MIT license**

Use the standard MIT License text with copyright holder `choewonjin` and year `2026`.

- [ ] **Step 2: Replace the template README**

```markdown
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

## License

MIT
```

- [ ] **Step 3: Protect local runtime data**

Append these entries to `.gitignore` if they are not already covered:

```gitignore
*.db
*.db-shm
*.db-wal
*.sqlite
*.sqlite3
logs/
recordings/
```

- [ ] **Step 4: Copy the approved design and this bootstrap plan into the repository**

```bash
mkdir -p docs/superpowers/specs docs/superpowers/plans
cp /Users/choewonjin/Documents/Codex/2026-06-15/cpu-afterburner-https-kr-msi-com/docs/superpowers/specs/2026-06-15-windows-performance-monitor-design.md docs/superpowers/specs/
cp /Users/choewonjin/Documents/Codex/2026-06-15/cpu-afterburner-https-kr-msi-com/docs/superpowers/plans/2026-06-15-pc-health-project-bootstrap.md docs/superpowers/plans/
```

- [ ] **Step 5: Verify repository hygiene**

```bash
git status --short
git check-ignore node_modules src-tauri/target sample.sqlite
```

Expected: source and documentation files are visible to Git; generated dependencies, Rust build output, and `sample.sqlite` are ignored.

### Task 3: Initialize And Publish The Repository

**Files:**
- Create: `/Users/choewonjin/Desktop/workspace/pc-health/.git/`

- [ ] **Step 1: Initialize Git with the `main` branch**

```bash
git init -b main
git add .
git commit -m "chore: bootstrap PC Health"
```

Expected: the working tree is clean after the initial commit.

- [ ] **Step 2: Create the GitHub repository**

Create a public repository named `pc-health` in the user's currently logged-in GitHub account. Do not initialize it with a README, license, or `.gitignore` because those files already exist locally.

- [ ] **Step 3: Connect and push**

```bash
git remote add origin https://github.com/<github-owner>/pc-health.git
git push -u origin main
```

Expected: GitHub shows the initial commit, README, MIT license, source tree, design, and bootstrap plan on `main`.

- [ ] **Step 4: Final verification**

```bash
git status --short --branch
git remote -v
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: `main` tracks `origin/main`, the working tree is clean, and both frontend and Rust checks pass.
