import json
import pytest
from pathlib import Path

from todo.models import Priority, TodoCreate, TodoUpdate
from todo.storage import JsonTodoStorage


@pytest.fixture
def storage(tmp_path: Path) -> JsonTodoStorage:
    data_file = tmp_path / "test_todos.json"
    return JsonTodoStorage(data_file)


def test_init_creates_empty_storage(storage: JsonTodoStorage):
    assert storage.file_path.exists()
    todos = storage.get_all()
    assert todos == []
    stats = storage.get_stats()
    assert stats["total"] == 0
    assert stats["completed"] == 0
    assert stats["active"] == 0


def test_create_todo(storage: JsonTodoStorage):
    todo = storage.create(TodoCreate(title="Buy milk", description="Oat milk", priority=Priority.HIGH))
    assert todo.id == 1
    assert todo.title == "Buy milk"
    assert todo.description == "Oat milk"
    assert todo.priority == Priority.HIGH
    assert not todo.completed
    assert todo.created_at is not None

    all_todos = storage.get_all()
    assert len(all_todos) == 1
    assert all_todos[0].id == 1


def test_get_by_id(storage: JsonTodoStorage):
    todo1 = storage.create(TodoCreate(title="Task 1"))
    todo2 = storage.create(TodoCreate(title="Task 2"))

    found = storage.get_by_id(todo2.id)
    assert found is not None
    assert found.title == "Task 2"

    not_found = storage.get_by_id(999)
    assert not_found is None


def test_update_todo(storage: JsonTodoStorage):
    todo = storage.create(TodoCreate(title="Original Title", priority=Priority.LOW))
    updated = storage.update(
        todo.id,
        TodoUpdate(title="New Title", priority=Priority.HIGH, completed=True),
    )
    assert updated is not None
    assert updated.title == "New Title"
    assert updated.priority == Priority.HIGH
    assert updated.completed is True
    assert updated.updated_at is not None

    # Check persistence
    fetched = storage.get_by_id(todo.id)
    assert fetched.title == "New Title"
    assert fetched.completed is True


def test_toggle_todo(storage: JsonTodoStorage):
    todo = storage.create(TodoCreate(title="Toggle test"))
    assert not todo.completed

    toggled = storage.toggle(todo.id)
    assert toggled.completed is True

    toggled_again = storage.toggle(todo.id)
    assert toggled_again.completed is False


def test_delete_todo(storage: JsonTodoStorage):
    todo = storage.create(TodoCreate(title="To be deleted"))
    assert storage.delete(todo.id) is True
    assert storage.get_by_id(todo.id) is None
    assert storage.delete(todo.id) is False


def test_clear_completed(storage: JsonTodoStorage):
    t1 = storage.create(TodoCreate(title="Active task"))
    t2 = storage.create(TodoCreate(title="Completed task 1"))
    t3 = storage.create(TodoCreate(title="Completed task 2"))

    storage.toggle(t2.id)
    storage.toggle(t3.id)

    cleared = storage.clear_completed()
    assert cleared == 2

    remaining = storage.get_all()
    assert len(remaining) == 1
    assert remaining[0].id == t1.id


def test_filtering_and_searching(storage: JsonTodoStorage):
    storage.create(TodoCreate(title="Buy apples", priority=Priority.LOW))
    storage.create(TodoCreate(title="Buy bananas", description="yellow fruit", priority=Priority.MEDIUM))
    t3 = storage.create(TodoCreate(title="Fix critical bug", priority=Priority.HIGH))
    storage.toggle(t3.id)

    # Active filter
    active = storage.get_all(status="active")
    assert len(active) == 2

    # Completed filter
    completed = storage.get_all(status="completed")
    assert len(completed) == 1
    assert completed[0].title == "Fix critical bug"

    # Search filter
    search_fruit = storage.get_all(search="fruit")
    assert len(search_fruit) == 1
    assert search_fruit[0].title == "Buy bananas"

    # Priority filter
    high_priority = storage.get_all(priority="high")
    assert len(high_priority) == 1
    assert high_priority[0].title == "Fix critical bug"


def test_corrupt_file_recovery(tmp_path: Path):
    corrupt_file = tmp_path / "corrupt_todos.json"
    corrupt_file.write_text("{ this is invalid json ]", encoding="utf-8")

    storage = JsonTodoStorage(corrupt_file)
    assert storage.get_all() == []
    # Test adding new item works
    item = storage.create(TodoCreate(title="Recovered task"))
    assert item.id == 1
