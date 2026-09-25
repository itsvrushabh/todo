import pytest
from pathlib import Path
from starlette.testclient import TestClient

from todo.app import create_app


@pytest.fixture
def client(tmp_path: Path):
    test_db = tmp_path / "test_api_todos.json"
    app = create_app(storage_path=test_db)
    return TestClient(app)


def test_health_endpoint(client: TestClient):
    res = client.get("/api/health")
    assert res.status_code == 200
    assert res.json() == {"status": "ok"}


def test_create_and_get_todo(client: TestClient):
    # Empty list initially
    res = client.get("/api/todos")
    assert res.status_code == 200
    assert res.json() == []

    # Create new task
    create_payload = {
        "title": "Learn FastAPI",
        "description": "Build a modern API",
        "priority": "high",
    }
    create_res = client.post("/api/todos", json=create_payload)
    assert create_res.status_code == 201
    item = create_res.json()
    assert item["id"] == 1
    assert item["title"] == "Learn FastAPI"
    assert item["description"] == "Build a modern API"
    assert item["priority"] == "high"
    assert item["completed"] is False

    # Get by ID
    get_res = client.get(f"/api/todos/{item['id']}")
    assert get_res.status_code == 200
    assert get_res.json()["title"] == "Learn FastAPI"


def test_create_todo_validation(client: TestClient):
    # Empty title
    res = client.post("/api/todos", json={"title": "   "})
    assert res.status_code == 400


def test_update_todo(client: TestClient):
    create_res = client.post("/api/todos", json={"title": "Old title"})
    todo_id = create_res.json()["id"]

    update_res = client.put(
        f"/api/todos/{todo_id}",
        json={"title": "Updated Title", "priority": "low"},
    )
    assert update_res.status_code == 200
    updated = update_res.json()
    assert updated["title"] == "Updated Title"
    assert updated["priority"] == "low"


def test_toggle_todo(client: TestClient):
    create_res = client.post("/api/todos", json={"title": "Toggle Me"})
    todo_id = create_res.json()["id"]

    toggle_res = client.patch(f"/api/todos/{todo_id}/toggle")
    assert toggle_res.status_code == 200
    assert toggle_res.json()["completed"] is True

    toggle_res_2 = client.patch(f"/api/todos/{todo_id}/toggle")
    assert toggle_res_2.status_code == 200
    assert toggle_res_2.json()["completed"] is False


def test_delete_todo(client: TestClient):
    create_res = client.post("/api/todos", json={"title": "Delete Me"})
    todo_id = create_res.json()["id"]

    del_res = client.delete(f"/api/todos/{todo_id}")
    assert del_res.status_code == 200
    assert del_res.json()["success"] is True

    # 404 on subsequent get
    assert client.get(f"/api/todos/{todo_id}").status_code == 404


def test_clear_completed(client: TestClient):
    t1 = client.post("/api/todos", json={"title": "Task 1"}).json()
    t2 = client.post("/api/todos", json={"title": "Task 2"}).json()

    client.patch(f"/api/todos/{t1['id']}/toggle")

    clear_res = client.delete("/api/todos-completed/clear")
    assert clear_res.status_code == 200
    assert clear_res.json()["cleared_count"] == 1

    remaining = client.get("/api/todos").json()
    assert len(remaining) == 1
    assert remaining[0]["id"] == t2["id"]


def test_stats_endpoint(client: TestClient):
    client.post("/api/todos", json={"title": "T1", "priority": "high"})
    t2 = client.post("/api/todos", json={"title": "T2", "priority": "low"}).json()
    client.patch(f"/api/todos/{t2['id']}/toggle")

    stats = client.get("/api/stats").json()
    assert stats["total"] == 2
    assert stats["completed"] == 1
    assert stats["active"] == 1
    assert stats["completion_percentage"] == 50.0
    assert stats["by_priority"]["high"] == 1
    assert stats["by_priority"]["low"] == 0  # low is completed, so active is 0


def test_index_serves_html(client: TestClient):
    res = client.get("/")
    assert res.status_code == 200
    assert "TaskMaster" in res.text
    assert "text/html" in res.headers.get("content-type", "")
