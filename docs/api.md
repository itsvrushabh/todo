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
| `POST` | `/api/todos` | Create a new task (supports subtasks and tags) | `TodoCreate` JSON |
| `GET` | `/api/todos/{id}` | Retrieve task by numeric ID | None |
| `PUT` | `/api/todos/{id}` | Update title, description, priority, or status | `TodoUpdate` JSON |
| `PATCH` | `/api/todos/{id}/toggle` | Toggle task completion status | None |
| `DELETE` | `/api/todos/{id}` | Delete task by ID (records tombstone) | None |
| `DELETE` | `/api/todos-completed/clear` | Clear all completed tasks | None |
| `POST` | `/api/todos/{id}/subtasks` | Add a subtask to an existing task | `{"title": "..."}` JSON |
| `PATCH` | `/api/todos/{id}/subtasks/{subtask_id}/toggle` | Toggle completion status of a subtask | None |
| `GET` | `/api/sync` | Differential sync pull (`?since=<RFC3339>`) | None |
| `POST` | `/api/sync` | Two-way batch synchronization | `SyncPushRequest` JSON |
| `GET` | `/api/stats` | Retrieve total, completed, pending, and percentage | None |

---

## 🔍 Query Parameters for `GET /api/todos`

| Parameter | Type | Description | Values |
|---|---|---|---|
| `status` | string | Filter by completion status | `active`, `completed` |
| `priority` | string | Filter by priority level | `low`, `medium`, `high` |
| `tag` | string | Filter by tag name | e.g. `rust`, `work` |
| `search` | string | Full-text query on title, description, and tags | e.g. `groceries` |

---

## 🔄 Two-Way Differential Synchronization

### `GET /api/sync?since=2026-11-16T00:00:00Z`
Returns all tasks created or modified since the given timestamp, plus any deletion tombstones:
```json
{
  "items": [ ... ],
  "tombstones": [
    { "uuid": "c3e1a0b2-3f8d-4e92-9426-8c5e62f01742", "deleted_at": "2026-11-16T12:00:00Z" }
  ],
  "synced_at": "2026-11-17T00:00:00Z"
}
```

### `POST /api/sync`
Batch push local mutations and retrieve updated server state in a single round-trip:
```json
{
  "items": [ ... ],
  "deleted_uuids": ["c3e1a0b2-3f8d-4e92-9426-8c5e62f01742"],
  "last_synced_at": "2026-11-16T00:00:00Z"
}
```
Conflict resolution follows deterministic **Last-Write-Wins (LWW)** using RFC-3339 timestamps. Tombstones prevent resurrection of deleted items across offline peers.

---

## 📋 JSON Schemas

### `TodoCreate`
```json
{
  "title": "Buy groceries #errands",
  "description": "Milk, eggs, coffee",
  "priority": "high",
  "tags": ["errands"],
  "subtasks": [
    { "id": "sub-1", "title": "Oat milk", "completed": false },
    { "id": "sub-2", "title": "Fresh eggs", "completed": true }
  ]
}
```

### `TodoItem`
```json
{
  "id": 1,
  "uuid": "8f3b23c1-4b72-4bc4-b892-0bfa7d9d04ea",
  "title": "Buy groceries #errands",
  "description": "Milk, eggs, coffee",
  "completed": false,
  "priority": "high",
  "tags": ["errands"],
  "subtasks": [
    { "id": "sub-1", "title": "Oat milk", "completed": false }
  ],
  "created_at": "2026-11-16T12:00:00Z",
  "updated_at": "2026-11-16T12:05:00Z"
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
