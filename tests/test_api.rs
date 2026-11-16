use std::sync::Arc;
use tempfile::{tempdir, TempDir};
use todo::models::{Priority, TodoCreate, TodoItem, TodoStats};
use todo::server::create_router;
use todo::storage::JsonTodoStorage;

async fn spawn_test_server() -> (String, Arc<JsonTodoStorage>, TempDir) {
    let dir = tempdir().unwrap();
    let storage_file = dir.path().join("todos.json");
    let storage = Arc::new(JsonTodoStorage::new(&storage_file).unwrap());

    let app = create_router(Arc::clone(&storage));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let base_url = format!("http://127.0.0.1:{port}");

    tokio::spawn(async move {
        axum::serve(listener, app.into_make_service())
            .await
            .unwrap();
    });

    (base_url, storage, dir)
}

#[tokio::test]
async fn test_health_check() {
    let (base_url, _, _dir) = spawn_test_server().await;
    let client = reqwest::Client::new();

    let res = client
        .get(format!("{base_url}/api/health"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn test_crud_endpoints() {
    let (base_url, _, _dir) = spawn_test_server().await;
    let client = reqwest::Client::new();

    // 1. Create a task
    let new_task = TodoCreate {
        title: "Test API Task".to_string(),
        description: Some("Description here".to_string()),
        priority: Some(Priority::High),
        due_date: Some("2026-11-25".to_string()),
        recurrence: None,
        uuid: None,
    };

    let res = client
        .post(format!("{base_url}/api/todos"))
        .json(&new_task)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 201);
    let created: TodoItem = res.json().await.unwrap();
    assert_eq!(created.id, 1);
    assert_eq!(created.title, "Test API Task");
    assert_eq!(created.priority, Priority::High);
    assert_eq!(created.due_date, Some("2026-11-25".to_string()));
    assert!(!created.completed);

    // 2. Fetch the task
    let res = client
        .get(format!("{base_url}/api/todos/1"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let fetched: TodoItem = res.json().await.unwrap();
    assert_eq!(fetched.id, 1);
    assert_eq!(fetched.due_date, Some("2026-11-25".to_string()));

    // 3. Toggle task
    let res = client
        .patch(format!("{base_url}/api/todos/1/toggle"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let toggled: TodoItem = res.json().await.unwrap();
    assert!(toggled.completed);

    // 4. Check stats
    let res = client
        .get(format!("{base_url}/api/stats"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let stats: TodoStats = res.json().await.unwrap();
    assert_eq!(stats.total, 1);
    assert_eq!(stats.completed, 1);

    // 5. Delete task
    let res = client
        .delete(format!("{base_url}/api/todos/1"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);

    // 6. Verify 404 after deletion
    let res = client
        .get(format!("{base_url}/api/todos/1"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn test_empty_title_validation() {
    let (base_url, _, _dir) = spawn_test_server().await;
    let client = reqwest::Client::new();

    let invalid_task = serde_json::json!({
        "title": "   ",
        "priority": "low"
    });

    let res = client
        .post(format!("{base_url}/api/todos"))
        .json(&invalid_task)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 400);
}

#[tokio::test]
async fn test_static_assets() {
    let (base_url, _, _dir) = spawn_test_server().await;
    let client = reqwest::Client::new();

    // Index page
    let res = client.get(format!("{base_url}/")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let text = res.text().await.unwrap();
    assert!(text.contains("Todo") || text.contains("<!DOCTYPE html>"));

    // Manifest
    let res = client
        .get(format!("{base_url}/manifest.json"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    // Fallback static resolution
    let res = client
        .get(format!("{base_url}/favicon.ico"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
}

#[tokio::test]
async fn test_recurring_task_api() {
    let (base_url, _, _dir) = spawn_test_server().await;
    let client = reqwest::Client::new();

    let new_task = TodoCreate {
        title: "Daily Sync".to_string(),
        description: None,
        priority: Some(Priority::Medium),
        due_date: Some("2026-01-01".to_string()),
        recurrence: Some(todo::models::RecurrencePattern::Daily),
        uuid: None,
    };

    let res = client
        .post(format!("{base_url}/api/todos"))
        .json(&new_task)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);
    let created: TodoItem = res.json().await.unwrap();
    assert_eq!(
        created.recurrence,
        Some(todo::models::RecurrencePattern::Daily)
    );

    // Toggle recurring task via API
    let res = client
        .patch(format!("{base_url}/api/todos/{}/toggle", created.id))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let toggled: TodoItem = res.json().await.unwrap();
    assert!(!toggled.completed);
    let today = chrono::Utc::now().date_naive();
    let expected = (today + chrono::Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();
    assert_eq!(toggled.due_date, Some(expected));
}
