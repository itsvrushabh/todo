# TaskMaster - Cross-Platform Modern Todo Application

A simple, fast, and feature-rich Todo application and CLI built with **Python 3.14**, **FastAPI**, and persistent **JSON storage**. Designed to run seamlessly in your local environment or local network (LAN) and installable across **Android**, **iPhone**, **Windows**, **macOS**, **Linux**, and **Web browsers** as a Progressive Web App (PWA).

---

## ✨ Features

- **Cross-Platform & Local Network Ready**:
  - Automatically binds to your local network (`0.0.0.0`) so phones, tablets, and laptops on your Wi-Fi can connect.
  - Generates an instant **ASCII QR code** in your terminal—scan with your phone camera to open immediately!
  - **Progressive Web App (PWA)**: Installable as a standalone app with native desktop/mobile windows, app icon, and offline cache.
- **Web Interface**:
  - Clean, responsive UI with **Light & Dark theme** support (with system auto-detection and persistence).
  - Live progress bar showing percentage of completed tasks.
  - Priority tags (**High**, **Medium**, **Low**) with visual badges and filter shortcuts.
  - Instant live search and filter tabs (**All**, **Active**, **Completed**).
  - Inline task editing, completion toggle, and single/bulk deletion.
  - Toast feedback notifications.
- **Storage**:
  - Human-readable and inspectable JSON storage (`todos.json`).
  - Thread-safe and atomic file writes to prevent data corruption.
- **RESTful API**:
  - Built with FastAPI with interactive Swagger API docs at [`/docs`](http://127.0.0.1:8000/docs).
- **CLI Commands**:
  - Integrated command-line commands for quick terminal use (`todo add`, `todo list`, `todo done`, `todo rm`).

---

## 🚀 Quick Start

### 1. Start the Application

Run using `uv`:

```bash
uv run todo
```

When started, your terminal displays:
- Local PC link: `http://localhost:8000`
- Mobile / LAN link: `http://<your-local-ip>:8000`
- A scannable **QR code** for Android & iPhone cameras!

---

## 📲 How to Install as an App on Each Device

### 📱 Android
1. Scan the terminal QR code or open `http://<your-local-ip>:8000` in **Chrome**.
2. Tap the **"Install"** button in the app header (or tap Chrome menu `⋮` → **"Install app"**).
3. The app will install with its own icon on your home screen and app drawer, opening full-screen without browser bars.

### 📱 iPhone / iPad (iOS)
1. Open `http://<your-local-ip>:8000` in **Safari**.
2. Tap the **Share** button (box with arrow pointing up) at the bottom.
3. Scroll down and tap **"Add to Home Screen"**, then tap **"Add"**.
4. TaskMaster will launch in standalone native app mode.

### 💻 Windows, Mac & Linux Desktop
1. Open `http://localhost:8000` (or local IP) in **Chrome**, **Edge**, or **Brave**.
2. Click the **"Install"** button in the top header (or the install icon in the browser address bar).
3. TaskMaster will install as an independent desktop application pinned to your Windows Start Menu / Taskbar, macOS Dock, or Linux Application Launcher.

---

### 2. CLI Usage

You can also manage tasks directly from your terminal:

```bash
# List all tasks
uv run todo list

# Add a new task with priority
uv run todo add "Buy groceries" --desc "Milk, bread, eggs" --priority high

# Mark task #1 as complete (or toggle status)
uv run todo done 1

# Delete task #2
uv run todo rm 2

# View help
uv run todo --help
```

---

## 📡 REST API Endpoints

| Method | Endpoint | Description |
|---|---|---|
| `GET` | `/api/todos` | List todos (supports `status`, `search`, `priority` query params) |
| `POST` | `/api/todos` | Create a new task |
| `GET` | `/api/todos/{id}` | Get task details |
| `PUT` | `/api/todos/{id}` | Update task title, description, or priority |
| `PATCH` | `/api/todos/{id}/toggle` | Toggle completion status |
| `DELETE` | `/api/todos/{id}` | Delete a task |
| `DELETE` | `/api/todos-completed/clear` | Clear all completed tasks |
| `GET` | `/api/stats` | Retrieve progress and priority statistics |
| `GET` | `/docs` | Interactive Swagger API documentation |

---

## 🧪 Testing & Code Quality

Run tests with `pytest`:

```bash
uv run pytest
```

Check code formatting and linting with `ruff`:

```bash
uv run ruff format --check
uv run ruff check
```

---

## 🔄 Continuous Integration

A GitHub Actions workflow is configured in [`.github/workflows/ci.yml`](.github/workflows/ci.yml) to automatically run formatting checks, linting, and tests on every push and pull request.
