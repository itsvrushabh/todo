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
    let initial = storage.get_all(None, None, None).unwrap();
    assert_eq!(initial.len(), 0);

    // 2. Create item
    let created = storage
        .create(TodoCreate {
            title: "Test Task 1".to_string(),
            description: Some("Task details".to_string()),
            priority: Some(Priority::High),
            due_date: Some("2026-12-31".to_string()),
            recurrence: None,
        })
        .unwrap();
    assert_eq!(created.id, 1);
    assert_eq!(created.title, "Test Task 1");
    assert_eq!(created.priority, Priority::High);
    assert_eq!(created.due_date, Some("2026-12-31".to_string()));
    assert!(!created.completed);

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
        })
        .unwrap();

    storage
        .create(TodoCreate {
            title: "Write Rust tests".to_string(),
            description: Some("Integration & unit".to_string()),
            priority: Some(Priority::Medium),
            due_date: None,
            recurrence: None,
        })
        .unwrap();

    let task3 = storage
        .create(TodoCreate {
            title: "Clean desk".to_string(),
            description: None,
            priority: Some(Priority::Low),
            due_date: None,
            recurrence: None,
        })
        .unwrap();
    storage.toggle(task3.id).unwrap();

    // Filter active
    let active = storage.get_all(Some("active"), None, None).unwrap();
    assert_eq!(active.len(), 2);

    // Filter completed
    let completed = storage.get_all(Some("completed"), None, None).unwrap();
    assert_eq!(completed.len(), 1);

    // Filter priority
    let high = storage.get_all(None, None, Some("high")).unwrap();
    assert_eq!(high.len(), 1);
    assert_eq!(high[0].title, "Buy groceries");

    // Search query
    let search_results = storage.get_all(None, Some("bread"), None).unwrap();
    assert_eq!(search_results.len(), 1);
    assert_eq!(search_results[0].title, "Buy groceries");
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
                })
                .unwrap();
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let all = storage.get_all(None, None, None).unwrap();
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
        })
        .unwrap();
    assert_eq!(created.id, 1);

    // Intentionally poison the RwLock
    storage.simulate_poison();

    // Verify reads recover from poisoned lock
    let all = storage
        .get_all(None, None, None)
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
        })
        .expect("Write should recover gracefully from poisoned lock");
    assert_eq!(second.id, 2);
    assert_eq!(second.title, "Post-poison Task");
}
