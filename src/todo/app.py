import os
from pathlib import Path
from typing import Any, Dict, List, Optional

from fastapi import FastAPI, HTTPException, Query, status
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import FileResponse
from fastapi.staticfiles import StaticFiles

from todo.models import TodoCreate, TodoItem, TodoUpdate
from todo.storage import JsonTodoStorage

STATIC_DIR = Path(__file__).parent / "static"


def create_app(storage_path: Optional[str | Path] = None) -> FastAPI:
    if storage_path is None:
        storage_path = os.getenv("TODO_STORAGE_PATH", "todos.json")

    storage = JsonTodoStorage(storage_path)

    app = FastAPI(
        title="Todo App API",
        description="Clean, modern, and persistent Todo list application",
        version="0.1.0",
    )

    app.add_middleware(
        CORSMiddleware,
        allow_origins=["*"],
        allow_credentials=True,
        allow_methods=["*"],
        allow_headers=["*"],
    )

    # Expose storage on app state for tests / customization
    app.state.storage = storage

    @app.get("/api/health")
    def health_check() -> Dict[str, str]:
        return {"status": "ok"}

    @app.get("/api/todos", response_model=List[TodoItem])
    def get_todos(
        status: Optional[str] = Query(None, description="Filter by active, completed, or all"),
        search: Optional[str] = Query(None, description="Search keyword in title/description"),
        priority: Optional[str] = Query(None, description="Filter by low, medium, high priority"),
    ) -> List[TodoItem]:
        valid_status = status if status in ("active", "completed") else None
        return storage.get_all(status=valid_status, search=search, priority=priority)

    @app.post("/api/todos", response_model=TodoItem, status_code=status.HTTP_201_CREATED)
    def create_todo(todo_in: TodoCreate) -> TodoItem:
        if not todo_in.title.strip():
            raise HTTPException(status_code=400, detail="Title cannot be empty or whitespace only")
        return storage.create(todo_in)

    @app.get("/api/todos/{todo_id}", response_model=TodoItem)
    def get_todo(todo_id: int) -> TodoItem:
        item = storage.get_by_id(todo_id)
        if not item:
            raise HTTPException(status_code=404, detail=f"Todo #{todo_id} not found")
        return item

    @app.put("/api/todos/{todo_id}", response_model=TodoItem)
    def update_todo(todo_id: int, todo_in: TodoUpdate) -> TodoItem:
        updated = storage.update(todo_id, todo_in)
        if not updated:
            raise HTTPException(status_code=404, detail=f"Todo #{todo_id} not found")
        return updated

    @app.patch("/api/todos/{todo_id}/toggle", response_model=TodoItem)
    def toggle_todo(todo_id: int) -> TodoItem:
        toggled = storage.toggle(todo_id)
        if not toggled:
            raise HTTPException(status_code=404, detail=f"Todo #{todo_id} not found")
        return toggled

    @app.delete("/api/todos/{todo_id}", status_code=status.HTTP_200_OK)
    def delete_todo(todo_id: int) -> Dict[str, Any]:
        deleted = storage.delete(todo_id)
        if not deleted:
            raise HTTPException(status_code=404, detail=f"Todo #{todo_id} not found")
        return {"success": True, "message": f"Todo #{todo_id} deleted"}

    @app.delete("/api/todos-completed/clear", status_code=status.HTTP_200_OK)
    def clear_completed_todos() -> Dict[str, Any]:
        cleared_count = storage.clear_completed()
        return {"success": True, "cleared_count": cleared_count}

    @app.get("/api/stats")
    def get_stats() -> Dict[str, Any]:
        return storage.get_stats()

    # Serve static assets and web interface
    if STATIC_DIR.exists():
        app.mount("/static", StaticFiles(directory=STATIC_DIR), name="static")

    @app.get("/", include_in_schema=False)
    def serve_index() -> FileResponse:
        index_file = STATIC_DIR / "index.html"
        if not index_file.exists():
            raise HTTPException(status_code=404, detail="Frontend assets not found")
        return FileResponse(index_file)

    return app


app = create_app()
