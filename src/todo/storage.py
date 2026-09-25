import json
import os
from datetime import datetime, timezone
from pathlib import Path
from threading import RLock
from typing import Any, Dict, List, Optional

from todo.models import Priority, TodoCreate, TodoItem, TodoUpdate


class JsonTodoStorage:
    """Thread-safe JSON file storage for Todo items with atomic writes."""

    def __init__(self, file_path: str | Path = "todos.json") -> None:
        self.file_path = Path(file_path).resolve()
        self._lock = RLock()
        self._ensure_storage()

    def _ensure_storage(self) -> None:
        """Create file and parent directories if they do not exist."""
        with self._lock:
            self.file_path.parent.mkdir(parents=True, exist_ok=True)
            if not self.file_path.exists():
                self._save_raw({"next_id": 1, "todos": []})
            else:
                try:
                    with open(self.file_path, "r", encoding="utf-8") as f:
                        data = json.load(f)
                    if not isinstance(data, dict) or "todos" not in data:
                        self._save_raw({"next_id": 1, "todos": []})
                except (json.JSONDecodeError, OSError):
                    # In case of corruption, backup and re-initialize
                    backup_path = self.file_path.with_suffix(".corrupt.bak")
                    try:
                        self.file_path.rename(backup_path)
                    except OSError:
                        pass
                    self._save_raw({"next_id": 1, "todos": []})

    def _load_raw(self) -> Dict[str, Any]:
        """Load raw dictionary from JSON file."""
        with open(self.file_path, "r", encoding="utf-8") as f:
            return json.load(f)

    def _save_raw(self, data: Dict[str, Any]) -> None:
        """Atomically write data to disk."""
        tmp_file = self.file_path.with_name(f".{self.file_path.name}.tmp")
        with open(tmp_file, "w", encoding="utf-8") as f:
            json.dump(data, f, indent=2, ensure_ascii=False)
            f.flush()
            os.fsync(f.fileno())
        os.replace(tmp_file, self.file_path)

    def get_all(
        self,
        status: Optional[str] = None,
        search: Optional[str] = None,
        priority: Optional[str] = None,
    ) -> List[TodoItem]:
        """Retrieve todos with optional filtering by status (active/completed), search term, and priority."""
        with self._lock:
            raw = self._load_raw()
            todos = [TodoItem(**item) for item in raw.get("todos", [])]

            if status == "active":
                todos = [t for t in todos if not t.completed]
            elif status == "completed":
                todos = [t for t in todos if t.completed]

            if priority:
                todos = [t for t in todos if t.priority.value == priority]

            if search:
                query = search.strip().lower()
                todos = [
                    t
                    for t in todos
                    if query in t.title.lower() or (t.description and query in t.description.lower())
                ]

            return todos

    def get_by_id(self, todo_id: int) -> Optional[TodoItem]:
        """Find a todo item by ID."""
        with self._lock:
            raw = self._load_raw()
            for item in raw.get("todos", []):
                if item["id"] == todo_id:
                    return TodoItem(**item)
            return None

    def create(self, todo_in: TodoCreate) -> TodoItem:
        """Create a new todo item."""
        with self._lock:
            raw = self._load_raw()
            next_id = raw.get("next_id", 1)
            now = datetime.now(timezone.utc).isoformat()

            new_todo = TodoItem(
                id=next_id,
                title=todo_in.title.strip(),
                description=(todo_in.description or "").strip(),
                priority=todo_in.priority,
                completed=False,
                created_at=now,
                updated_at=None,
            )

            raw.setdefault("todos", []).append(new_todo.model_dump())
            raw["next_id"] = next_id + 1
            self._save_raw(raw)
            return new_todo

    def update(self, todo_id: int, todo_in: TodoUpdate) -> Optional[TodoItem]:
        """Update fields of an existing todo item."""
        with self._lock:
            raw = self._load_raw()
            todos = raw.get("todos", [])
            for i, item in enumerate(todos):
                if item["id"] == todo_id:
                    existing = TodoItem(**item)
                    update_data = todo_in.model_dump(exclude_unset=True)

                    if not update_data:
                        return existing

                    if "title" in update_data and update_data["title"] is not None:
                        existing.title = update_data["title"].strip()
                    if "description" in update_data and update_data["description"] is not None:
                        existing.description = update_data["description"].strip()
                    if "completed" in update_data and update_data["completed"] is not None:
                        existing.completed = update_data["completed"]
                    if "priority" in update_data and update_data["priority"] is not None:
                        existing.priority = update_data["priority"]

                    existing.updated_at = datetime.now(timezone.utc).isoformat()
                    todos[i] = existing.model_dump()
                    self._save_raw(raw)
                    return existing
            return None

    def toggle(self, todo_id: int) -> Optional[TodoItem]:
        """Toggle the completed state of a todo item."""
        with self._lock:
            raw = self._load_raw()
            todos = raw.get("todos", [])
            for i, item in enumerate(todos):
                if item["id"] == todo_id:
                    existing = TodoItem(**item)
                    existing.completed = not existing.completed
                    existing.updated_at = datetime.now(timezone.utc).isoformat()
                    todos[i] = existing.model_dump()
                    self._save_raw(raw)
                    return existing
            return None

    def delete(self, todo_id: int) -> bool:
        """Delete a todo item by ID. Returns True if deleted, False otherwise."""
        with self._lock:
            raw = self._load_raw()
            todos = raw.get("todos", [])
            initial_len = len(todos)
            raw["todos"] = [item for item in todos if item["id"] != todo_id]
            if len(raw["todos"]) != initial_len:
                self._save_raw(raw)
                return True
            return False

    def clear_completed(self) -> int:
        """Remove all completed todos. Returns count of deleted items."""
        with self._lock:
            raw = self._load_raw()
            todos = raw.get("todos", [])
            active_todos = [item for item in todos if not item.get("completed", False)]
            cleared_count = len(todos) - len(active_todos)
            if cleared_count > 0:
                raw["todos"] = active_todos
                self._save_raw(raw)
            return cleared_count

    def get_stats(self) -> Dict[str, Any]:
        """Return statistics on todos."""
        with self._lock:
            raw = self._load_raw()
            todos = [TodoItem(**item) for item in raw.get("todos", [])]
            total = len(todos)
            completed = sum(1 for t in todos if t.completed)
            active = total - completed
            by_priority = {
                "high": sum(1 for t in todos if t.priority == Priority.HIGH and not t.completed),
                "medium": sum(1 for t in todos if t.priority == Priority.MEDIUM and not t.completed),
                "low": sum(1 for t in todos if t.priority == Priority.LOW and not t.completed),
            }
            return {
                "total": total,
                "completed": completed,
                "active": active,
                "completion_percentage": round((completed / total * 100), 1) if total > 0 else 0.0,
                "by_priority": by_priority,
            }
