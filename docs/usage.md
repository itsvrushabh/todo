# User Guide & Platform Usage

This guide covers interacting with Todo via the **Native Desktop GUI**, the **Interactive Terminal UI (TUI)**, the **CLI subcommands**, and the **WebAssembly App**.

---

## 🖥️ Native Desktop Graphical User Interface (GUI)

Todo includes an immediate-mode graphical window built in Rust using `egui` and `eframe`.

Launch the desktop app directly:
```bash
cargo run --release -- gui
```

### ✨ GUI Capabilities
- **Themes**: Switch instantly between Catppuccin Macchiato, Latte, and High Contrast.
- **Adaptive Modes**: Desktop Planning (with Kanban columns and scratchpad converter), Mobile Focus (single prioritized view), and Calm Mode (distraction-free spotlight).
- **Undo / Redo History**: 50-step circular snapshot history with `Ctrl+Z` / `Cmd+Z` (Undo) and `Ctrl+Y` / `Cmd+Shift+Z` (Redo) or header controls.
- **Hierarchical Subtasks & #Tags**: Add and toggle subtasks within cards, and click tag chips to filter instantly.
- **Offline Sync & LWW Conflict Resolution**: Automatic differential sync with deletion tombstones.
- **Task Management**: Instant addition, completion toggling, in-place editing, and task deletion.
- **Real-Time Filters & Search**: Filter by `All`, `Active`, or `Completed` tasks, priority, time budgets (Quick Win, Deep Work), and hashtag taxonomy.
- **Import / Export**: Full JSON and Markdown export and import directly in the UI.

---

## 📺 Interactive Terminal UI (TUI)

Launch the TUI at any time from your terminal:
```bash
cargo run --release -- tui
```

### ⌨️ Keyboard Navigation & Shortcuts

| Key | Action |
|---|---|
| `↑` / `k`, `↓` / `j` | Navigate tasks up and down |
| `Space` or `Enter` | Toggle task completion status |
| `a` | Add new task (opens modal; press `Tab` to cycle priority) |
| `d` or `x` | Delete selected task |
| `Tab` / `Right` | Cycle through filters (**All** ➔ **Active** ➔ **Completed**) |
| `1`, `2`, `3` | Jump directly to filter (**1**: All, **2**: Active, **3**: Done) |
| `/` | Enter search filter mode |
| `c` | Clear all completed tasks (press `y` to confirm) |
| `r` | Refresh/reload tasks from storage |
| `q` or `Esc` | Quit TUI / Close modal |

---

## 💻 CLI Subcommands

Manage tasks directly in shell scripts, cron jobs, or terminals:

### Add Tasks
```bash
# Add with optional description, priority, tags, and subtasks
cargo run -- add "Implement Rust tests #dev" --desc "Write storage & API integration tests" --priority high --subtasks "Unit tests,API tests,Headless GUI"
cargo run -- add "Buy groceries" --tag shopping --subtasks "Oat milk,Apples"
```

### List Tasks
```bash
cargo run -- list
cargo run -- list --status active
cargo run -- list --priority high
cargo run -- list --tag dev
cargo run -- list -q "milk"
```

### Subtasks
```bash
# Mark subtask #2 of task #1 as completed
cargo run -- subtask-done 1 2
```

### Toggle, Delete & Clear
```bash
# Toggle status of task #1
cargo run -- done 1

# Delete task #2
cargo run -- rm 2

# Clear all completed tasks
cargo run -- clear
```

### Shell Autocompletions
```bash
# Generate completion scripts for your shell
cargo run -- completions bash > ~/.local/share/bash-completion/completions/todo
cargo run -- completions zsh > ~/.zfunc/_todo
cargo run -- completions fish > ~/.config/fish/completions/todo.fish
```

### Storage Engines
```bash
# Default JSON file backend (crash-resilient, poisoned lock recovery)
cargo run -- --storage todos.json tui

# Optional SQLite backend (high performance, indexed lookup)
cargo run --features sqlite -- --storage todos.db gui
```

### View Statistics
```bash
cargo run -- stats
```

---

## 📦 Native Desktop Installation & Packaging

### Linux (.deb & Desktop Entry)
```bash
# Build standard Debian / Ubuntu .deb package
./scripts/package-deb.sh 0.2.0 amd64 target/release/todo dist/
sudo dpkg -i dist/todo_0.2.0_amd64.deb

# Or manually install desktop entry & icon:
install -Dm644 packaging/desktop/todo.desktop ~/.local/share/applications/todo.desktop
install -Dm644 packaging/desktop/todo.png ~/.local/share/icons/hicolor/512x512/apps/todo.png
```

### macOS (.app Bundle)
```bash
# Build standalone Todo.app bundle
./scripts/package-macos-app.sh 0.2.0 target/release/todo dist/
# Resulting bundle is at dist/Todo.app and dist/todo-macos-app.tar.gz
```

### Windows (MSI & ZIP)
Standalone Windows executables (`todo.exe`) are bundled with icons into `todo-windows-amd64.zip` in CI release builds.

---

## 📱 Pure Rust Native Mobile Apps

### 🤖 Android Native App (APK)
Todo compiles to a pure Rust native Android application (`android.app.NativeActivity`) with zero JavaScript:
```bash
# Install cargo-apk tool
cargo install cargo-apk

# Ensure Android NDK is exported
export ANDROID_NDK_ROOT=/path/to/android-ndk

# Build release APK
./scripts/build-android-apk.sh
# Output APK: target/release/apk/todo.apk
```
Install directly onto an Android device via ADB:
```bash
adb install -r target/release/apk/todo.apk
```

### 🍎 iOS Native Library & Bundle
Todo compiles to an iOS universal static library and application bundle using pure Rust:
```bash
# Build for iOS device or simulator
./scripts/build-ios.sh aarch64-apple-ios-sim
# Output bundle structure: target/ios/Todo.app
```

---

## 🌐 Cross-Platform PWA Installation (Zero JavaScript)

Todo embeds a high-performance web client and Progressive Web App directly into the compiled executable.

### 📱 Android
1. Start the server (`cargo run --release -- serve`) and scan the terminal ASCII QR code with your camera (or visit `http://<lan-ip>:8000` in **Chrome**).
2. Tap **"Install"** in the app header (or Chrome menu `⋮` ➔ **"Install app"**).
3. The app installs to your home screen and drawer, opening as a full-screen standalone application.

### 📱 iPhone / iPad (iOS)
1. Open `http://<lan-ip>:8000` in **Safari**.
2. Tap the **Share** button (box with upward arrow).
3. Tap **"Add to Home Screen"** and confirm **"Add"**.
4. Todo will launch in standalone native app window mode with full touch support.

### 💻 Desktop (Linux, Windows, macOS)
1. Open `http://localhost:8000` in Chrome, Edge, or Brave.
2. Click the **Install** button in the URL address bar or top app bar.
3. Todo will install as an independent desktop window pinned to your taskbar/dock and system app launcher.
4. On Linux desktops, a desktop launcher entry is also provided at `deploy/todo.desktop`.
