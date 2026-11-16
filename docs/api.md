# REST API Reference

Todo provides a clean RESTful HTTP API built with Axum. All JSON responses use standard status codes and structured payloads.

Interactive rustdoc API documentation is also published at:
[https://itsvrushabh.github.io/todo/docs/todo/](https://itsvrushabh.github.io/todo/docs/todo/)

---

## 📡 Endpoints Overview

| Method | Endpoint | Description | Request Body |
|---|---|---|---|
| `GET` | `/api/health` | Health check endpoint (`{"status": "ok"}`) | None |
| `GET` | `/api/todos` | List todos with optional query filters | None |
| `POST` | `/api/todos` | Create a new task | `TodoCreate` JSON |
| `GET` | `/api/todos/{id}` | Retrieve task by numeric ID | None |
| `PUT` | `/api/todos/{id}` | Update title, description, priority, or status | `TodoUpdate` JSON |
| `PATCH` | `/api/todos/{id}/toggle` | Toggle task completion status | None |
| `DELETE` | `/api/todos/{id}` | Delete task by ID | None |
| `DELETE` | `/api/todos-completed/clear` | Clear all completed tasks | None |
| `GET` | `/api/stats` | Retrieve total, completed, pending, and percentage | None |

---

## 🔍 Query Parameters for `GET /api/todos`

| Parameter | Type | Description | Values |
|---|---|---|---|
| `status` | string | Filter by completion status | `active`, `completed` |
| `priority` | string | Filter by priority level | `low`, `medium`, `high` |
| `search` | string | Full-text query on title and description | e.g. `groceries` |

---

## 📋 JSON Schemas

### `TodoCreate`
```json
{
  "title": "Buy groceries",
  "description": "Milk, eggs, coffee",
  "priority": "high"
}
```

### `TodoItem`
```json
{
  "id": 1,
  "title": "Buy groceries",
  "description": "Milk, eggs, coffee",
  "completed": false,
  "priority": "high",
  "created_at": "2026-11-16T12:00:00Z",
  "updated_at": "2026-11-16T12:00:00Z"
}
```

### `TodoStats`
```json
{
  "total": 10,
  "completed": 7,
  "pending": 3,
  "completion_rate": 70.0
}
```
