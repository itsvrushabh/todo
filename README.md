# Todo

[![CI](https://github.com/itsvrushabh/todo/actions/workflows/ci.yml/badge.svg)](https://github.com/itsvrushabh/todo/actions/workflows/ci.yml)
[![Pages](https://github.com/itsvrushabh/todo/actions/workflows/deploy-pages.yml/badge.svg)](https://itsvrushabh.github.io/todo/)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](#license)
[![Rust: 1.80+](https://img.shields.io/badge/rust-1.80%2B-orange.svg)](https://www.rust-lang.org)

> A high-performance, memory-safe, cross-platform Todo and Task Management application written in **Rust 2021**.

Todo runs seamlessly as an **interactive terminal UI (TUI)**, a **native desktop GUI window**, a **CLI tool**, a **standalone web server**, and a **WebAssembly web app**.

🌐 **Live Web Demo**: [https://itsvrushabh.github.io/todo/](https://itsvrushabh.github.io/todo/)  
📖 **Rustdoc API Docs**: [https://itsvrushabh.github.io/todo/docs/todo/](https://itsvrushabh.github.io/todo/docs/todo/)

---

## ✨ Features

- 🦀 **100% Pure Rust & Zero JavaScript**: Complete frontend and backend written in Rust 2021 with zero JavaScript code (`git ls-files '*.js'` = 0).
- 🖥️ **Native Desktop Window**: Standalone GPU-accelerated desktop GUI on Linux, Windows, or macOS with `cargo run -- gui` powered by `egui`/`eframe`.
- ⟲ **GUI Undo / Redo History**: 50-step circular snapshot history with `Ctrl+Z` / `Cmd+Z` (Undo) and `Ctrl+Y` / `Cmd+Shift+Z` (Redo) plus header controls.
- 🏷️ **Hierarchical Subtasks & #Tag Taxonomy**: Nested subtasks with progress tracking (`2/5 completed`), automatic `#hashtag` parsing from titles/descriptions, and multi-tag filtering chips.
- 🔄 **Two-Way Differential Sync & Tombstones**: Offline-resilient sync engine with deletion tombstones, deterministic Last-Write-Wins (LWW) conflict resolution, and `GET /api/sync?since=...` differential endpoints.
- 🗄️ **Pluggable Storage Backends**: Decoupled `TodoStorage` trait with crash-resilient JSON storage (atomic rename, backup restore, poisoned lock recovery) and high-performance indexed SQLite backend (`--features sqlite`).
- 📺 **Interactive TUI**: Rich terminal dashboard with live progress gauges, priority badges, detail pane, instant search (`/`), and filter tabs powered by `ratatui`.
- 💻 **Scriptable CLI & Shell Completions**: Direct CLI commands (`add`, `list`, `done`, `rm`, `subtask-done`, `stats`) plus shell autocompletions for Bash, Zsh, Fish, Elvish, and PowerShell (`todo completions <shell>`).
- 🌐 **WebAssembly Web App**: Compiles directly to `wasm32-unknown-unknown` for pure client-side execution on GitHub Pages or local Axum server.
- 🧪 **Offscreen Headless GUI Testing**: Comprehensive headless smoke tests (`tests/test_gui_headless.rs`) running layout, theme, action, and dialog lifecycles without requiring X11/Wayland.
- 🤖 **Automated CI/CD**: Release matrix automation producing Linux (x86_64, aarch64), macOS (Intel, Apple Silicon), Windows (x86_64), and WASM tarballs with SHA256 checksums, automated Gemini AI PR reviews, and Dependabot.

---

## 🚀 Quick Start

### Package Manager Installation

```bash
# Arch Linux (AUR)
yay -S todo-bin
# or: paru -S todo-bin

# macOS & Linux (Homebrew)
brew tap itsvrushabh/tap && brew install todo
```

### From Source (Cargo)

### 1. Launch the Native Desktop GUI
```bash
cargo run --release -- gui
```

### 2. Run the Web Server (serves WebAssembly client)
```bash
cargo run --release -- serve
```
* **Local**: `http://localhost:8000`
* **Mobile / LAN**: Scan the ASCII QR code printed in your terminal from your phone or tablet camera.

### 3. Launch the Terminal UI (TUI)
```bash
cargo run --release -- tui
```

### 4. Use the CLI
```bash
# Add a task with priority, tags, and subtasks
cargo run -- add "Ship Rust App #v0.2 #release" --priority high --subtasks "Write docs,Run headless tests,Tag release"

# List tasks (optionally filtered by tag)
cargo run -- list
cargo run -- list --tag v0.2

# Complete subtask #1 on task #1
cargo run -- subtask-done 1 1

# Mark task #1 as complete
cargo run -- done 1

# Generate shell completions (bash, zsh, fish, elvish, powershell)
cargo run -- completions zsh > ~/.zfunc/_todo
```

### 5. Run with SQLite Storage Backend
```bash
# Launch with high-performance SQLite engine
cargo run --features sqlite -- --storage todos.db gui
```

---

## 📚 Documentation Index

For detailed guides, please refer to the specialized documentation:

| Document | Description |
|---|---|
| **[User & Platform Guide](docs/usage.md)** | TUI keybindings cheat sheet, full CLI flags, and Android/iOS/Desktop PWA installation. |
| **[Deployment Guide](docs/deployment.md)** | Docker, Docker Compose, Fly.io, Render, Railway, and Linux Systemd VPS setups. |
| **[REST API Reference](docs/api.md)** | HTTP endpoints, query filters, and JSON request/response schemas. |
| **[Release & Architecture](docs/releases.md)** | Weekly/monthly/yearly LTS schedules, Gemini AI code reviewer, and CI bot architecture. |
| **[Contributing Guidelines](CONTRIBUTING.md)** | Development environment setup, branch naming, formatting, and linting rules. |

---

## 🤝 Contributing

Contributions are welcome! Todo enforces an issue-first workflow where all pull requests link to an issue and pass automated CI and Gemini AI code reviews. Please read **[CONTRIBUTING.md](CONTRIBUTING.md)** before opening a PR.

---

## 📜 License

Dual-licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
