# TaskMaster - Modern Todo Application

A simple, fast, and feature-rich Todo web application and CLI built with **Python 3.14**, **FastAPI**, and persistent **JSON storage**.

---

## ✨ Features

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

### 1. Start the Web App

Run using `uv`:

```bash
uv run todo
```

Or explicitly specify the host and port:

```bash
uv run todo serve --host 127.0.0.1 --port 8000
```

Open your browser at: **[http://127.0.0.1:8000](http://127.0.0.1:8000)**

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

## 🧪 Running Tests

Run the test suite with `pytest`:

```bash
uv run pytest
```
