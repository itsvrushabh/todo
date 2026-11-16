use std::sync::Arc;
use std::thread;
use tempfile::tempdir;
use todo::models::{Priority, TodoCreate, TodoUpdate};
use todo::storage::JsonTodoStorage;

#[test]
fn test_storage_creation_and_crud() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("todos.json");

    let storage = JsonTodoStorage::new(&file_path).unwrap();

    // 1. Initial should be empty
    let initial = storage.get_all(None, None, None, None).unwrap();
    assert_eq!(initial.len(), 0);

    // 2. Create item
    let created = storage
        .create(TodoCreate {
            title: "Test Task 1".to_string(),
            description: Some("Task details".to_string()),
            priority: Some(Priority::High),
            due_date: Some("2026-12-31".to_string()),
            recurrence: None,
            uuid: None,
            tags: Vec::new(),
            subtasks: Vec::new(),
        })
        .unwrap();
    assert_eq!(created.id, 1);
    assert_eq!(created.title, "Test Task 1");
    assert_eq!(created.priority, Priority::High);
    assert_eq!(created.due_date, Some("2026-12-31".to_string()));
    assert!(!created.completed);
    assert!(created.uuid.is_some());

    // 3. Get by ID
    let fetched = storage.get_by_id(1).unwrap().expect("Item should exist");
    assert_eq!(fetched.id, 1);
    assert_eq!(fetched.title, "Test Task 1");
    assert_eq!(fetched.due_date, Some("2026-12-31".to_string()));

    // 4. Update item
    let updated = storage
        .update(
            1,
            TodoUpdate {
                title: Some("Updated Task 1".to_string()),
                description: Some("New desc".to_string()),
                completed: Some(false),
                priority: Some(Priority::Low),
                due_date: Some("2027-01-01".to_string()),
                recurrence: None,
                uuid: None,
                tags: None,
                subtasks: None,
            },
        )
        .unwrap()
        .expect("Should update");
    assert_eq!(updated.title, "Updated Task 1");
    assert_eq!(updated.priority, Priority::Low);
    assert_eq!(updated.due_date, Some("2027-01-01".to_string()));

    // 5. Toggle item
    let toggled = storage.toggle(1).unwrap().expect("Should toggle");
    assert!(toggled.completed);

    // 6. Stats
    let stats = storage.get_stats().unwrap();
    assert_eq!(stats.total, 1);
    assert_eq!(stats.completed, 1);
    assert_eq!(stats.active, 0);
    assert_eq!(stats.completion_percentage, 100.0);

    // 7. Delete item
    let deleted = storage.delete(1).unwrap();
    assert!(deleted);
    assert!(storage.get_by_id(1).unwrap().is_none());
}

#[test]
fn test_filtering_and_search() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("todos.json");
    let storage = JsonTodoStorage::new(&file_path).unwrap();

    storage
        .create(TodoCreate {
            title: "Buy groceries".to_string(),
            description: Some("Milk and bread".to_string()),
            priority: Some(Priority::High),
            due_date: None,
            recurrence: None,
            uuid: None,
            tags: vec!["errands".to_string()],
            subtasks: Vec::new(),
        })
        .unwrap();

    storage
        .create(TodoCreate {
            title: "Write Rust tests".to_string(),
            description: Some("Integration & unit".to_string()),
            priority: Some(Priority::Medium),
            due_date: None,
            recurrence: None,
            uuid: None,
            tags: vec!["dev".to_string(), "rust".to_string()],
            subtasks: Vec::new(),
        })
        .unwrap();

    let task3 = storage
        .create(TodoCreate {
            title: "Clean desk".to_string(),
            description: None,
            priority: Some(Priority::Low),
            due_date: None,
            recurrence: None,
            uuid: None,
            tags: vec!["chores".to_string()],
            subtasks: Vec::new(),
        })
        .unwrap();
    storage.toggle(task3.id).unwrap();

    // Filter active
    let active = storage.get_all(Some("active"), None, None, None).unwrap();
    assert_eq!(active.len(), 2);

    // Filter completed
    let completed = storage
        .get_all(Some("completed"), None, None, None)
        .unwrap();
    assert_eq!(completed.len(), 1);

    // Filter priority
    let high = storage.get_all(None, None, Some("high"), None).unwrap();
    assert_eq!(high.len(), 1);
    assert_eq!(high[0].title, "Buy groceries");

    // Search query
    let search_results = storage.get_all(None, Some("bread"), None, None).unwrap();
    assert_eq!(search_results.len(), 1);
    assert_eq!(search_results[0].title, "Buy groceries");

    // Filter tag
    let tag_results = storage.get_all(None, None, None, Some("dev")).unwrap();
    assert_eq!(tag_results.len(), 1);
    assert_eq!(tag_results[0].title, "Write Rust tests");
}

#[test]
fn test_concurrent_access() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("todos_concurrent.json");
    let storage = Arc::new(JsonTodoStorage::new(&file_path).unwrap());

    let mut handles = Vec::new();
    for i in 0..10 {
        let storage_clone = Arc::clone(&storage);
        handles.push(thread::spawn(move || {
            storage_clone
                .create(TodoCreate {
                    title: format!("Thread task {i}"),
                    description: None,
                    priority: Some(Priority::Medium),
                    due_date: None,
                    recurrence: None,
                    uuid: None,
                    tags: Vec::new(),
                    subtasks: Vec::new(),
                })
                .unwrap();
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let all = storage.get_all(None, None, None, None).unwrap();
    assert_eq!(all.len(), 10);
}

#[test]
fn test_recurring_task_toggle() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("todos_recurring.json");
    let storage = JsonTodoStorage::new(&file_path).unwrap();

    let task = storage
        .create(TodoCreate {
            title: "Daily Standup".to_string(),
            description: None,
            priority: Some(Priority::High),
            due_date: Some("2026-01-01".to_string()),
            recurrence: Some(todo::models::RecurrencePattern::Daily),
            uuid: None,
            tags: Vec::new(),
            subtasks: Vec::new(),
        })
        .unwrap();

    assert_eq!(task.due_date, Some("2026-01-01".to_string()));
    assert!(!task.completed);

    // Toggle recurring task
    let toggled = storage.toggle(task.id).unwrap().unwrap();
    // Should NOT be completed, but rolled forward to tomorrow
    assert!(!toggled.completed);
    assert_ne!(toggled.due_date, Some("2026-01-01".to_string()));
    let today = chrono::Utc::now().date_naive();
    let expected_due = (today + chrono::Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();
    assert_eq!(toggled.due_date, Some(expected_due));
}

#[test]
fn test_poisoned_lock_recovery() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("todos_poison.json");
    let storage = JsonTodoStorage::new(&file_path).unwrap();

    // Create an initial task
    let created = storage
        .create(TodoCreate {
            title: "Pre-poison Task".to_string(),
            description: None,
            priority: Some(Priority::Medium),
            due_date: None,
            recurrence: None,
            uuid: None,
            tags: Vec::new(),
            subtasks: Vec::new(),
        })
        .unwrap();
    assert_eq!(created.id, 1);

    // Intentionally poison the RwLock
    storage.simulate_poison();

    // Verify reads recover from poisoned lock
    let all = storage
        .get_all(None, None, None, None)
        .expect("Read should recover gracefully from poisoned lock");
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].title, "Pre-poison Task");

    // Verify writes recover from poisoned lock
    let second = storage
        .create(TodoCreate {
            title: "Post-poison Task".to_string(),
            description: None,
            priority: Some(Priority::High),
            due_date: None,
            recurrence: None,
            uuid: None,
            tags: Vec::new(),
            subtasks: Vec::new(),
        })
        .expect("Write should recover gracefully from poisoned lock");
    assert_eq!(second.id, 2);
    assert_eq!(second.title, "Post-poison Task");
}

#[test]
fn test_storage_uuid_generation_and_lookup() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("todos_uuid.json");
    let storage = JsonTodoStorage::new(&file_path).unwrap();

    // 1. Creation generates a UUID automatically when None is passed
    let auto = storage
        .create(TodoCreate {
            title: "Auto UUID task".to_string(),
            description: None,
            priority: Some(Priority::Medium),
            due_date: None,
            recurrence: None,
            uuid: None,
            tags: Vec::new(),
            subtasks: Vec::new(),
        })
        .unwrap();
    assert!(auto.uuid.is_some());
    let auto_uuid = auto.uuid.clone().unwrap();

    // Lookup by UUID
    let fetched = storage
        .get_by_uuid(&auto_uuid)
        .unwrap()
        .expect("Should find by UUID");
    assert_eq!(fetched.id, auto.id);
    assert_eq!(fetched.title, "Auto UUID task");

    // 2. Client-provided UUID is preserved
    let client_uuid = "client-gen-uuid-12345".to_string();
    let client_task = storage
        .create(TodoCreate {
            title: "Client UUID task".to_string(),
            description: None,
            priority: Some(Priority::High),
            due_date: None,
            recurrence: None,
            uuid: Some(client_uuid.clone()),
            tags: Vec::new(),
            subtasks: Vec::new(),
        })
        .unwrap();
    assert_eq!(client_task.uuid.as_deref(), Some(client_uuid.as_str()));

    let fetched_client = storage
        .get_by_uuid(&client_uuid)
        .unwrap()
        .expect("Should find client UUID");
    assert_eq!(fetched_client.id, client_task.id);

    // 3. Lookup non-existent UUID returns None
    let missing = storage.get_by_uuid("non-existent-uuid").unwrap();
    assert!(missing.is_none());
}

#[test]
fn test_storage_subtasks_lifecycle() {
    use todo::models::Subtask;

    let dir = tempdir().unwrap();
    let file_path = dir.path().join("todos_subtasks.json");
    let storage = JsonTodoStorage::new(&file_path).unwrap();

    let task = storage
        .create(TodoCreate {
            title: "Epic Task".to_string(),
            description: None,
            priority: Some(Priority::High),
            due_date: None,
            recurrence: None,
            uuid: None,
            tags: vec!["epic".to_string()],
            subtasks: vec![Subtask::new("Sub 1"), Subtask::new("Sub 2")],
        })
        .unwrap();

    assert_eq!(task.subtasks.len(), 2);
    assert_eq!(task.subtask_progress(), Some((0, 2)));

    let sub_id = task.subtasks[0].id.clone();
    let toggled = storage.toggle_subtask(task.id, &sub_id).unwrap().unwrap();
    assert!(toggled.subtasks[0].completed);
    assert_eq!(toggled.subtask_progress(), Some((1, 2)));

    let toggled_back = storage.toggle_subtask(task.id, &sub_id).unwrap().unwrap();
    assert!(!toggled_back.subtasks[0].completed);
    assert_eq!(toggled_back.subtask_progress(), Some((0, 2)));
}

#[test]
fn test_storage_tombstones_and_differential_sync() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("todos_sync.json");
    let storage = JsonTodoStorage::new(&file_path).unwrap();

    // 1. Create items
    let task1 = storage
        .create(TodoCreate {
            title: "Task 1 to delete".to_string(),
            description: None,
            priority: Some(Priority::Low),
            due_date: None,
            recurrence: None,
            uuid: Some("uuid-111".to_string()),
            tags: Vec::new(),
            subtasks: Vec::new(),
        })
        .unwrap();

    let task2 = storage
        .create(TodoCreate {
            title: "Task 2 to update".to_string(),
            description: None,
            priority: Some(Priority::Medium),
            due_date: None,
            recurrence: None,
            uuid: Some("uuid-222".to_string()),
            tags: Vec::new(),
            subtasks: Vec::new(),
        })
        .unwrap();

    let _task3 = storage
        .create(TodoCreate {
            title: "Task 3 untouched".to_string(),
            description: None,
            priority: Some(Priority::High),
            due_date: None,
            recurrence: None,
            uuid: Some("uuid-333".to_string()),
            tags: Vec::new(),
            subtasks: Vec::new(),
        })
        .unwrap();

    // Initial full sync (since = None)
    let full_sync = storage.get_sync(None).unwrap();
    assert_eq!(full_sync.updated_todos.len(), 3);
    assert_eq!(full_sync.deleted_uuids.len(), 0);

    // Save timestamp before modifications
    let checkpoint = full_sync.server_timestamp;

    // Small delay to ensure timestamp strictly advances
    std::thread::sleep(std::time::Duration::from_millis(15));

    // Delete task 1
    assert!(storage.delete(task1.id).unwrap());
    let tombstones = storage.get_tombstones().unwrap();
    assert_eq!(tombstones.len(), 1);
    assert_eq!(tombstones[0].uuid, "uuid-111");

    // Update task 2
    storage
        .update(
            task2.id,
            TodoUpdate {
                title: Some("Task 2 updated".to_string()),
                description: None,
                completed: Some(true),
                priority: None,
                due_date: None,
                recurrence: None,
                uuid: None,
                tags: None,
                subtasks: None,
            },
        )
        .unwrap();

    // Differential sync since checkpoint
    let diff_sync = storage.get_sync(Some(&checkpoint)).unwrap();
    assert_eq!(diff_sync.updated_todos.len(), 1);
    assert_eq!(diff_sync.updated_todos[0].uuid.as_deref(), Some("uuid-222"));
    assert_eq!(diff_sync.updated_todos[0].title, "Task 2 updated");
    assert_eq!(diff_sync.deleted_uuids, vec!["uuid-111"]);

    // Test apply_sync on another storage instance
    let client_path = dir.path().join("client_todos.json");
    let client_storage = JsonTodoStorage::new(&client_path).unwrap();

    // Client initially has task 1 and task 3
    client_storage
        .create(TodoCreate {
            title: "Task 1 on client".to_string(),
            description: None,
            priority: Some(Priority::Low),
            due_date: None,
            recurrence: None,
            uuid: Some("uuid-111".to_string()),
            tags: Vec::new(),
            subtasks: Vec::new(),
        })
        .unwrap();

    client_storage
        .create(TodoCreate {
            title: "Task 3 on client".to_string(),
            description: None,
            priority: Some(Priority::High),
            due_date: None,
            recurrence: None,
            uuid: Some("uuid-333".to_string()),
            tags: Vec::new(),
            subtasks: Vec::new(),
        })
        .unwrap();

    // Apply the differential sync to client
    let result = client_storage.apply_sync(diff_sync).unwrap();
    assert_eq!(result.applied_deletions, 1); // uuid-111 removed
    assert_eq!(result.applied_updates, 1); // uuid-222 added

    let client_todos = client_storage.get_all(None, None, None, None).unwrap();
    assert_eq!(client_todos.len(), 2);
    assert!(!client_todos
        .iter()
        .any(|t| t.uuid.as_deref() == Some("uuid-111")));
    assert!(client_todos
        .iter()
        .any(|t| t.uuid.as_deref() == Some("uuid-222")));
    assert!(client_todos
        .iter()
        .any(|t| t.uuid.as_deref() == Some("uuid-333")));
}

#[test]
fn test_memory_storage_engine() {
    use todo::storage::{MemoryTodoStorage, TodoStorage};

    let storage: Box<dyn TodoStorage> = Box::new(MemoryTodoStorage::new());

    // 1. Initial empty
    assert_eq!(storage.get_all(None, None, None, None).unwrap().len(), 0);

    // 2. Create
    let item = storage
        .create(TodoCreate {
            title: "Memory Task".into(),
            description: Some("In memory".into()),
            priority: Some(Priority::High),
            due_date: None,
            recurrence: None,
            uuid: Some("mem-uuid-1".into()),
            tags: vec!["ephemeral".into()],
            subtasks: Vec::new(),
        })
        .unwrap();
    assert_eq!(item.id, 1);
    assert_eq!(item.title, "Memory Task");

    // 3. Lookup by UUID & ID
    assert_eq!(
        storage.get_by_uuid("mem-uuid-1").unwrap().unwrap().title,
        "Memory Task"
    );
    assert_eq!(storage.get_by_id(1).unwrap().unwrap().title, "Memory Task");

    // 4. Toggle
    let toggled = storage.toggle(1).unwrap().unwrap();
    assert!(toggled.completed);

    // 5. Delete & Tombstone
    assert!(storage.delete(1).unwrap());
    assert_eq!(storage.get_all(None, None, None, None).unwrap().len(), 0);
    assert_eq!(storage.get_tombstones().unwrap().len(), 1);
    assert_eq!(storage.get_tombstones().unwrap()[0].uuid, "mem-uuid-1");
}

#[cfg(feature = "sqlite")]
#[test]
fn test_sqlite_storage_engine() {
    use todo::storage::{SqliteTodoStorage, TodoStorage};

    let storage: Box<dyn TodoStorage> = Box::new(SqliteTodoStorage::in_memory().unwrap());

    // 1. Initial empty
    assert_eq!(storage.get_all(None, None, None, None).unwrap().len(), 0);

    // 2. Create with subtasks and tags
    let item = storage
        .create(TodoCreate {
            title: "SQLite Task".into(),
            description: Some("Relational DB".into()),
            priority: Some(Priority::High),
            due_date: Some("2026-12-01".into()),
            recurrence: None,
            uuid: Some("sqlite-uuid-1".into()),
            tags: vec!["db".into(), "sql".into()],
            subtasks: vec![todo::models::Subtask::new("Migration")],
        })
        .unwrap();
    assert_eq!(item.id, 1);
    assert_eq!(item.title, "SQLite Task");
    assert_eq!(item.subtasks.len(), 1);

    // 3. Lookup
    let fetched = storage.get_by_uuid("sqlite-uuid-1").unwrap().unwrap();
    assert_eq!(fetched.id, 1);
    assert_eq!(fetched.tags, vec!["db", "sql"]);

    // 4. Toggle subtask
    let sub_id = &fetched.subtasks[0].id;
    let toggled_sub = storage.toggle_subtask(1, sub_id).unwrap().unwrap();
    assert!(toggled_sub.subtasks[0].completed);

    // 5. Filter by tag
    let tag_match = storage.get_all(None, None, None, Some("db")).unwrap();
    assert_eq!(tag_match.len(), 1);
    let no_match = storage
        .get_all(None, None, None, Some("nonexistent"))
        .unwrap();
    assert_eq!(no_match.len(), 0);

    // 6. Stats
    let stats = storage.get_stats().unwrap();
    assert_eq!(stats.total, 1);
    assert_eq!(stats.active, 1);
    assert_eq!(stats.completed, 0);

    // 7. Toggle item
    let toggled = storage.toggle(1).unwrap().unwrap();
    assert!(toggled.completed);
    assert_eq!(storage.get_stats().unwrap().completed, 1);

    // 8. Delete & Tombstones
    assert!(storage.delete(1).unwrap());
    assert_eq!(storage.get_all(None, None, None, None).unwrap().len(), 0);
    let tombstones = storage.get_tombstones().unwrap();
    assert_eq!(tombstones.len(), 1);
    assert_eq!(tombstones[0].uuid, "sqlite-uuid-1");
}
