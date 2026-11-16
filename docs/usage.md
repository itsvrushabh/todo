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
- **Catppuccin Theme**: Dark-mode palette with mauve and blue accents.
- **Task Management**: Instant addition, completion toggling, in-place editing, and task deletion.
- **Real-Time Filters & Search**: Filter by `All`, `Active`, or `Completed` tasks, and search task titles and descriptions.
- **Progress Gauge**: Visual completion percentage and live task counts.

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
# Add with optional description and priority (low, medium, high)
cargo run -- add "Implement Rust tests" --desc "Write storage & API integration tests" --priority high
cargo run -- add "Buy milk" --priority low
```

### List Tasks
```bash
cargo run -- list
cargo run -- list --status active
cargo run -- list --priority high
cargo run -- list -q "milk"
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

### View Statistics
```bash
cargo run -- stats
```

---

## 📱 Cross-Platform PWA Installation

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
