# TaskMaster

[![CI](https://github.com/itsvrushabh/todo/actions/workflows/ci.yml/badge.svg)](https://github.com/itsvrushabh/todo/actions/workflows/ci.yml)
[![Pages](https://github.com/itsvrushabh/todo/actions/workflows/deploy-pages.yml/badge.svg)](https://itsvrushabh.github.io/todo/)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](#license)
[![Rust: 1.80+](https://img.shields.io/badge/rust-1.80%2B-orange.svg)](https://www.rust-lang.org)

> A high-performance, memory-safe, cross-platform Todo and Task Management application written in **Rust 2021**.

TaskMaster runs seamlessly as an **interactive terminal UI (TUI)**, a **CLI tool**, a **standalone web server**, and an **installable Progressive Web App (PWA)** across Android, iOS, Windows, macOS, and Linux.

🌐 **Live Web Demo**: [https://itsvrushabh.github.io/todo/](https://itsvrushabh.github.io/todo/)  
📖 **Rustdoc API Docs**: [https://itsvrushabh.github.io/todo/docs/todo/](https://itsvrushabh.github.io/todo/docs/todo/)

---

## ✨ Features

- 🦀 **Single Binary**: Built with Axum and Ratatui; all web assets, stylesheets, scripts, and PWA manifests are compiled directly into the binary via `rust-embed` with zero runtime dependencies.
- 📺 **Interactive TUI**: Rich terminal dashboard with live progress gauges, priority badges, detail pane, instant search (`/`), and filter tabs.
- 💻 **Scriptable CLI**: Direct terminal subcommands (`add`, `list`, `done`, `rm`, `clear`, `stats`) for shell scripts and shortcuts.
- 📱 **Progressive Web App (PWA)**: Auto-detects local LAN IP and prints an in-terminal ASCII QR code for one-tap mobile camera pairing (Android, iPhone, iPad).
- 💾 **Crash-Resilient Storage**: Thread-safe (`RwLock`) atomic JSON writes with automatic backup restoration on corruption.
- 🤖 **Automated CI/CD**: Weekly minor releases, monthly major releases, yearly LTS maintenance, Gemini AI PR reviews, and Dependabot automation.

---

## 🚀 Quick Start

### 1. Run the Web Server & PWA
```bash
cargo run --release -- serve
```
* **Local**: `http://localhost:8000`
* **Mobile / LAN**: Scan the ASCII QR code printed in your terminal from your phone or tablet camera.

### 2. Launch the Terminal UI (TUI)
```bash
cargo run --release -- tui
```

### 3. Use the CLI
```bash
# Add a task
cargo run -- add "Ship Rust App" --priority high

# List tasks
cargo run -- list

# Mark task #1 as complete
cargo run -- done 1
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

Contributions are welcome! TaskMaster enforces an issue-first workflow where all pull requests link to an issue and pass automated CI and Gemini AI code reviews. Please read **[CONTRIBUTING.md](CONTRIBUTING.md)** before opening a PR.

---

## 📜 License

Dual-licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
