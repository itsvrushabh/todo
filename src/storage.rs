use crate::models::{TodoCreate, TodoItem, TodoStats, TodoUpdate};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::RwLock;

#[derive(Debug)]
pub enum StorageError {
    Io(io::Error),
    Json(serde_json::Error),
    Validation(String),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::Io(e) => write!(f, "IO error: {e}"),
            StorageError::Json(e) => write!(f, "JSON serialization error: {e}"),
            StorageError::Validation(e) => write!(f, "Validation error: {e}"),
        }
    }
}

impl std::error::Error for StorageError {}

impl From<io::Error> for StorageError {
    fn from(e: io::Error) -> Self {
        StorageError::Io(e)
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(e: serde_json::Error) -> Self {
        StorageError::Json(e)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct StorageData {
    next_id: i64,
    todos: Vec<TodoItem>,
}

impl Default for StorageData {
    fn default() -> Self {
        Self {
            next_id: 1,
            todos: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct JsonTodoStorage {
    file_path: PathBuf,
    lock: RwLock<()>,
}

impl JsonTodoStorage {
    fn write_lock(&self) -> std::sync::RwLockWriteGuard<'_, ()> {
        self.lock
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn read_lock(&self) -> std::sync::RwLockReadGuard<'_, ()> {
        self.lock
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn simulate_poison(&self) {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = self.lock.write().unwrap();
            panic!("Intentional lock poison simulation");
        }));
    }

    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let file_path = path.as_ref().to_path_buf();
        let storage = Self {
            file_path,
            lock: RwLock::new(()),
        };
        storage.ensure_storage()?;
        Ok(storage)
    }

    fn ensure_storage(&self) -> Result<(), StorageError> {
        let _guard = self.write_lock();
        if let Some(parent) = self.file_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }

        if !self.file_path.exists() {
            self.save_raw_locked(&StorageData::default())?;
        } else {
            // Check if valid JSON
            match fs::read_to_string(&self.file_path) {
                Ok(content) => {
                    if serde_json::from_str::<StorageData>(&content).is_err() {
                        let backup = self.file_path.with_extension("corrupt.bak");
                        let _ = fs::rename(&self.file_path, backup);
                        self.save_raw_locked(&StorageData::default())?;
                    }
                }
                Err(_) => {
                    self.save_raw_locked(&StorageData::default())?;
                }
            }
        }
        Ok(())
    }

    fn load_raw_locked(&self) -> Result<StorageData, StorageError> {
        let content = fs::read_to_string(&self.file_path)?;
        let data: StorageData = serde_json::from_str(&content)?;
        Ok(data)
    }

    fn save_raw_locked(&self, data: &StorageData) -> Result<(), StorageError> {
        let tmp_file = self.file_path.with_extension("tmp");
        {
            let mut file = File::create(&tmp_file)?;
            let json = serde_json::to_string_pretty(data)?;
            file.write_all(json.as_bytes())?;
            file.sync_all()?;
        }
        fs::rename(&tmp_file, &self.file_path)?;
        Ok(())
    }

    pub fn get_all(
        &self,
        status: Option<&str>,
        search: Option<&str>,
        priority: Option<&str>,
        tag: Option<&str>,
    ) -> Result<Vec<TodoItem>, StorageError> {
        let _guard = self.read_lock();
        let data = self.load_raw_locked()?;
        let mut items = data.todos;

        if let Some(s) = status {
            match s.to_lowercase().as_str() {
                "active" => items.retain(|t| !t.completed),
                "completed" => items.retain(|t| t.completed),
                _ => {}
            }
        }

        if let Some(p) = priority {
            let p_lower = p.to_lowercase();
            items.retain(|t| t.priority.to_string() == p_lower);
        }

        if let Some(t) = tag {
            let t_clean = t.trim().trim_start_matches('#').to_lowercase();
            if !t_clean.is_empty() {
                items.retain(|item| {
                    item.tags
                        .iter()
                        .any(|existing| existing.to_lowercase() == t_clean)
                });
            }
        }

        if let Some(query) = search {
            let q = query.trim().to_lowercase();
            if !q.is_empty() {
                items.retain(|t| {
                    t.title.to_lowercase().contains(&q)
                        || t.description
                            .as_deref()
                            .map(|d| d.to_lowercase().contains(&q))
                            .unwrap_or(false)
                });
            }
        }

        Ok(items)
    }

    pub fn get_by_id(&self, id: i64) -> Result<Option<TodoItem>, StorageError> {
        let _guard = self.read_lock();
        let data = self.load_raw_locked()?;
        Ok(data.todos.into_iter().find(|t| t.id == id))
    }

    pub fn get_by_uuid(&self, uuid: &str) -> Result<Option<TodoItem>, StorageError> {
        let _guard = self.read_lock();
        let data = self.load_raw_locked()?;
        let trimmed = uuid.trim();
        Ok(data
            .todos
            .into_iter()
            .find(|t| t.uuid.as_deref() == Some(trimmed)))
    }

    pub fn create(&self, todo_in: TodoCreate) -> Result<TodoItem, StorageError> {
        let trimmed_title = todo_in.title.trim().to_string();
        if trimmed_title.is_empty() {
            return Err(StorageError::Validation(
                "Title cannot be empty or whitespace only".to_string(),
            ));
        }

        let _guard = self.write_lock();
        let mut data = self.load_raw_locked()?;

        let mut item = TodoItem::new(
            data.next_id,
            trimmed_title,
            todo_in.description.map(|d| d.trim().to_string()),
            todo_in.priority.unwrap_or_default(),
        );
        if let Some(uuid_str) = todo_in.uuid {
            let trimmed_uuid = uuid_str.trim().to_string();
            if !trimmed_uuid.is_empty() {
                item.uuid = Some(trimmed_uuid);
            }
        }
        for tag in todo_in.tags {
            let clean = tag.trim().trim_start_matches('#').to_lowercase();
            if !clean.is_empty() && !item.tags.contains(&clean) {
                item.tags.push(clean);
            }
        }
        if !todo_in.subtasks.is_empty() {
            item.subtasks = todo_in.subtasks;
        }
        if let Some(due) = todo_in.due_date {
            let trimmed = due.trim().to_string();
            if !trimmed.is_empty() {
                item.due_date = Some(trimmed);
            }
        }
        item.recurrence = todo_in.recurrence;

        data.next_id += 1;
        data.todos.push(item.clone());
        self.save_raw_locked(&data)?;

        Ok(item)
    }

    pub fn update(&self, id: i64, todo_in: TodoUpdate) -> Result<Option<TodoItem>, StorageError> {
        let _guard = self.write_lock();
        let mut data = self.load_raw_locked()?;

        if let Some(item) = data.todos.iter_mut().find(|t| t.id == id) {
            if let Some(title) = todo_in.title {
                let trimmed = title.trim().to_string();
                if trimmed.is_empty() {
                    return Err(StorageError::Validation(
                        "Title cannot be empty or whitespace only".to_string(),
                    ));
                }
                item.title = trimmed;
            }
            if let Some(desc) = todo_in.description {
                item.description = Some(desc.trim().to_string());
            }
            if let Some(priority) = todo_in.priority {
                item.priority = priority;
            }
            if let Some(rec) = todo_in.recurrence {
                item.recurrence = Some(rec);
            }
            if let Some(due) = todo_in.due_date {
                let trimmed = due.trim().to_string();
                item.due_date = if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                };
            }
            if let Some(uuid_str) = todo_in.uuid {
                let trimmed = uuid_str.trim().to_string();
                if !trimmed.is_empty() {
                    item.uuid = Some(trimmed);
                }
            }
            if let Some(tags) = todo_in.tags {
                let mut clean_tags = Vec::new();
                for t in tags {
                    let clean = t.trim().trim_start_matches('#').to_lowercase();
                    if !clean.is_empty() && !clean_tags.contains(&clean) {
                        clean_tags.push(clean);
                    }
                }
                item.tags = clean_tags;
            }
            if let Some(subtasks) = todo_in.subtasks {
                item.subtasks = subtasks;
            }
            if let Some(completed) = todo_in.completed {
                if completed && !item.completed && item.recurrence.is_some() {
                    if let Some(next_due) = item.next_recurrence_date() {
                        item.due_date = Some(next_due);
                        item.completed = false;
                    } else {
                        item.completed = true;
                    }
                } else {
                    item.completed = completed;
                }
            }
            item.updated_at = Some(Utc::now().to_rfc3339());
            let updated = item.clone();

            self.save_raw_locked(&data)?;
            Ok(Some(updated))
        } else {
            Ok(None)
        }
    }

    pub fn toggle_subtask(
        &self,
        id: i64,
        subtask_id: &str,
    ) -> Result<Option<TodoItem>, StorageError> {
        let _guard = self.write_lock();
        let mut data = self.load_raw_locked()?;

        if let Some(item) = data.todos.iter_mut().find(|t| t.id == id) {
            if let Some(sub) = item.subtasks.iter_mut().find(|s| s.id == subtask_id) {
                sub.completed = !sub.completed;
                item.updated_at = Some(Utc::now().to_rfc3339());
                let updated = item.clone();
                self.save_raw_locked(&data)?;
                return Ok(Some(updated));
            }
        }
        Ok(None)
    }

    pub fn toggle(&self, id: i64) -> Result<Option<TodoItem>, StorageError> {
        let _guard = self.write_lock();
        let mut data = self.load_raw_locked()?;

        if let Some(item) = data.todos.iter_mut().find(|t| t.id == id) {
            if !item.completed && item.recurrence.is_some() {
                if let Some(next_due) = item.next_recurrence_date() {
                    item.due_date = Some(next_due);
                    item.completed = false;
                } else {
                    item.completed = true;
                }
            } else {
                item.completed = !item.completed;
            }
            item.updated_at = Some(Utc::now().to_rfc3339());
            let updated = item.clone();

            self.save_raw_locked(&data)?;
            Ok(Some(updated))
        } else {
            Ok(None)
        }
    }

    pub fn delete(&self, id: i64) -> Result<bool, StorageError> {
        let _guard = self.write_lock();
        let mut data = self.load_raw_locked()?;

        let original_len = data.todos.len();
        data.todos.retain(|t| t.id != id);

        if data.todos.len() != original_len {
            self.save_raw_locked(&data)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn clear_completed(&self) -> Result<usize, StorageError> {
        let _guard = self.write_lock();
        let mut data = self.load_raw_locked()?;

        let original_len = data.todos.len();
        data.todos.retain(|t| !t.completed);
        let removed = original_len - data.todos.len();

        if removed > 0 {
            self.save_raw_locked(&data)?;
        }
        Ok(removed)
    }

    pub fn get_stats(&self) -> Result<TodoStats, StorageError> {
        let _guard = self.read_lock();
        let data = self.load_raw_locked()?;

        let total = data.todos.len();
        let completed = data.todos.iter().filter(|t| t.completed).count();
        let active = total - completed;
        let completion_percentage = if total > 0 {
            (completed as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        Ok(TodoStats {
            total,
            completed,
            active,
            completion_percentage,
        })
    }
}
