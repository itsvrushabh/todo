use crate::models::{
    SyncApplyResult, SyncResponse, TodoCreate, TodoItem, TodoStats, TodoUpdate, Tombstone,
};
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
    #[cfg(feature = "sqlite")]
    Sqlite(rusqlite::Error),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::Io(e) => write!(f, "IO error: {e}"),
            StorageError::Json(e) => write!(f, "JSON serialization error: {e}"),
            StorageError::Validation(e) => write!(f, "Validation error: {e}"),
            #[cfg(feature = "sqlite")]
            StorageError::Sqlite(e) => write!(f, "SQLite error: {e}"),
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

#[cfg(feature = "sqlite")]
impl From<rusqlite::Error> for StorageError {
    fn from(e: rusqlite::Error) -> Self {
        StorageError::Sqlite(e)
    }
}

pub trait TodoStorage: Send + Sync {
    fn get_all(
        &self,
        status: Option<&str>,
        search: Option<&str>,
        priority: Option<&str>,
        tag: Option<&str>,
    ) -> Result<Vec<TodoItem>, StorageError>;

    fn get_by_id(&self, id: i64) -> Result<Option<TodoItem>, StorageError>;
    fn get_by_uuid(&self, uuid: &str) -> Result<Option<TodoItem>, StorageError>;
    fn create(&self, todo_in: TodoCreate) -> Result<TodoItem, StorageError>;
    fn update(&self, id: i64, todo_in: TodoUpdate) -> Result<Option<TodoItem>, StorageError>;
    fn toggle(&self, id: i64) -> Result<Option<TodoItem>, StorageError>;
    fn toggle_subtask(&self, id: i64, subtask_id: &str) -> Result<Option<TodoItem>, StorageError>;
    fn delete(&self, id: i64) -> Result<bool, StorageError>;
    fn delete_by_uuid(&self, uuid: &str) -> Result<bool, StorageError>;
    fn clear_completed(&self) -> Result<usize, StorageError>;
    fn get_stats(&self) -> Result<TodoStats, StorageError>;
    fn get_tombstones(&self) -> Result<Vec<Tombstone>, StorageError>;
    fn get_sync(&self, since: Option<&str>) -> Result<SyncResponse, StorageError>;
    fn apply_sync(&self, sync: SyncResponse) -> Result<SyncApplyResult, StorageError>;
}

#[derive(Debug, Serialize, Deserialize)]
struct StorageData {
    next_id: i64,
    todos: Vec<TodoItem>,
    #[serde(default)]
    tombstones: Vec<Tombstone>,
}

impl Default for StorageData {
    fn default() -> Self {
        Self {
            next_id: 1,
            todos: Vec::new(),
            tombstones: Vec::new(),
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

        if let Some(pos) = data.todos.iter().position(|t| t.id == id) {
            let removed = data.todos.remove(pos);
            if let Some(uuid) = removed.uuid {
                if !uuid.trim().is_empty() {
                    data.tombstones.push(Tombstone {
                        uuid,
                        deleted_at: Utc::now().to_rfc3339(),
                    });
                }
            }
            self.save_raw_locked(&data)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn delete_by_uuid(&self, uuid: &str) -> Result<bool, StorageError> {
        let trimmed = uuid.trim();
        if trimmed.is_empty() {
            return Ok(false);
        }
        let _guard = self.write_lock();
        let mut data = self.load_raw_locked()?;

        if let Some(pos) = data
            .todos
            .iter()
            .position(|t| t.uuid.as_deref() == Some(trimmed))
        {
            let removed = data.todos.remove(pos);
            if let Some(u) = removed.uuid {
                data.tombstones.push(Tombstone {
                    uuid: u,
                    deleted_at: Utc::now().to_rfc3339(),
                });
            }
            self.save_raw_locked(&data)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn clear_completed(&self) -> Result<usize, StorageError> {
        let _guard = self.write_lock();
        let mut data = self.load_raw_locked()?;

        let mut removed = 0;
        let mut now_tombstones = Vec::new();
        let now_ts = Utc::now().to_rfc3339();
        data.todos.retain(|t| {
            if t.completed {
                if let Some(u) = &t.uuid {
                    if !u.trim().is_empty() {
                        now_tombstones.push(Tombstone {
                            uuid: u.clone(),
                            deleted_at: now_ts.clone(),
                        });
                    }
                }
                removed += 1;
                false
            } else {
                true
            }
        });
        data.tombstones.extend(now_tombstones);

        if removed > 0 {
            self.save_raw_locked(&data)?;
        }
        Ok(removed)
    }

    pub fn get_tombstones(&self) -> Result<Vec<Tombstone>, StorageError> {
        let _guard = self.read_lock();
        let data = self.load_raw_locked()?;
        Ok(data.tombstones)
    }

    pub fn get_sync(&self, since: Option<&str>) -> Result<SyncResponse, StorageError> {
        let _guard = self.read_lock();
        let data = self.load_raw_locked()?;
        let server_timestamp = Utc::now().to_rfc3339();

        match since {
            None => Ok(SyncResponse {
                updated_todos: data.todos,
                deleted_uuids: data.tombstones.into_iter().map(|t| t.uuid).collect(),
                server_timestamp,
            }),
            Some(since_str) => {
                let updated_todos: Vec<TodoItem> = data
                    .todos
                    .into_iter()
                    .filter(|t| {
                        let ts = t.updated_at.as_deref().unwrap_or(&t.created_at);
                        ts > since_str
                    })
                    .collect();

                let deleted_uuids: Vec<String> = data
                    .tombstones
                    .into_iter()
                    .filter(|t| t.deleted_at.as_str() > since_str)
                    .map(|t| t.uuid)
                    .collect();

                Ok(SyncResponse {
                    updated_todos,
                    deleted_uuids,
                    server_timestamp,
                })
            }
        }
    }

    pub fn apply_sync(&self, sync: SyncResponse) -> Result<SyncApplyResult, StorageError> {
        let _guard = self.write_lock();
        let mut data = self.load_raw_locked()?;

        let mut applied_deletions = 0;
        let mut applied_updates = 0;

        // 1. Process deletions
        for del_uuid in &sync.deleted_uuids {
            let trimmed = del_uuid.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Some(pos) = data
                .todos
                .iter()
                .position(|t| t.uuid.as_deref() == Some(trimmed))
            {
                data.todos.remove(pos);
                applied_deletions += 1;
            }
            if !data.tombstones.iter().any(|t| t.uuid == trimmed) {
                data.tombstones.push(Tombstone {
                    uuid: trimmed.to_string(),
                    deleted_at: Utc::now().to_rfc3339(),
                });
            }
        }

        // 2. Process updates with Last-Write-Wins (LWW)
        for remote in sync.updated_todos {
            let remote_uuid = match &remote.uuid {
                Some(u) if !u.trim().is_empty() => u.clone(),
                _ => continue,
            };

            // If we have a local tombstone newer than remote.updated_at, skip
            let remote_ts = remote
                .updated_at
                .as_deref()
                .unwrap_or(&remote.created_at)
                .to_string();
            if let Some(tomb) = data.tombstones.iter().find(|t| t.uuid == remote_uuid) {
                if tomb.deleted_at >= remote_ts {
                    continue;
                }
            }

            if let Some(local) = data
                .todos
                .iter_mut()
                .find(|t| t.uuid.as_deref() == Some(&remote_uuid))
            {
                let local_ts = local.updated_at.as_deref().unwrap_or(&local.created_at);
                if remote_ts.as_str() >= local_ts {
                    local.title = remote.title;
                    local.description = remote.description;
                    local.completed = remote.completed;
                    local.priority = remote.priority;
                    local.due_date = remote.due_date;
                    local.recurrence = remote.recurrence;
                    local.tags = remote.tags;
                    local.subtasks = remote.subtasks;
                    local.estimated_minutes = remote.estimated_minutes;
                    local.updated_at = remote.updated_at.or(Some(remote_ts));
                    applied_updates += 1;
                }
            } else {
                let mut new_item = remote;
                new_item.id = data.next_id;
                data.next_id += 1;
                data.todos.push(new_item);
                applied_updates += 1;
            }
        }

        if applied_deletions > 0 || applied_updates > 0 {
            self.save_raw_locked(&data)?;
        }

        Ok(SyncApplyResult {
            applied_updates,
            applied_deletions,
        })
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

impl TodoStorage for JsonTodoStorage {
    fn get_all(
        &self,
        status: Option<&str>,
        search: Option<&str>,
        priority: Option<&str>,
        tag: Option<&str>,
    ) -> Result<Vec<TodoItem>, StorageError> {
        self.get_all(status, search, priority, tag)
    }

    fn get_by_id(&self, id: i64) -> Result<Option<TodoItem>, StorageError> {
        self.get_by_id(id)
    }

    fn get_by_uuid(&self, uuid: &str) -> Result<Option<TodoItem>, StorageError> {
        self.get_by_uuid(uuid)
    }

    fn create(&self, todo_in: TodoCreate) -> Result<TodoItem, StorageError> {
        self.create(todo_in)
    }

    fn update(&self, id: i64, todo_in: TodoUpdate) -> Result<Option<TodoItem>, StorageError> {
        self.update(id, todo_in)
    }

    fn toggle(&self, id: i64) -> Result<Option<TodoItem>, StorageError> {
        self.toggle(id)
    }

    fn toggle_subtask(&self, id: i64, subtask_id: &str) -> Result<Option<TodoItem>, StorageError> {
        self.toggle_subtask(id, subtask_id)
    }

    fn delete(&self, id: i64) -> Result<bool, StorageError> {
        self.delete(id)
    }

    fn delete_by_uuid(&self, uuid: &str) -> Result<bool, StorageError> {
        self.delete_by_uuid(uuid)
    }

    fn clear_completed(&self) -> Result<usize, StorageError> {
        self.clear_completed()
    }

    fn get_stats(&self) -> Result<TodoStats, StorageError> {
        self.get_stats()
    }

    fn get_tombstones(&self) -> Result<Vec<Tombstone>, StorageError> {
        self.get_tombstones()
    }

    fn get_sync(&self, since: Option<&str>) -> Result<SyncResponse, StorageError> {
        self.get_sync(since)
    }

    fn apply_sync(&self, sync: SyncResponse) -> Result<SyncApplyResult, StorageError> {
        self.apply_sync(sync)
    }
}

#[derive(Debug, Default)]
pub struct MemoryTodoStorage {
    data: RwLock<StorageData>,
}

impl MemoryTodoStorage {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(StorageData::default()),
        }
    }
}

impl TodoStorage for MemoryTodoStorage {
    fn get_all(
        &self,
        status: Option<&str>,
        search: Option<&str>,
        priority: Option<&str>,
        tag: Option<&str>,
    ) -> Result<Vec<TodoItem>, StorageError> {
        let guard = self
            .data
            .read()
            .map_err(|_| StorageError::Validation("Lock poisoned".into()))?;
        let mut results: Vec<TodoItem> = guard.todos.clone();

        if let Some(status_filter) = status {
            match status_filter.to_lowercase().as_str() {
                "active" => results.retain(|t| !t.completed),
                "completed" => results.retain(|t| t.completed),
                "all" => {}
                _ => {}
            }
        }

        if let Some(query) = search {
            let q = query.to_lowercase();
            results.retain(|t| {
                t.title.to_lowercase().contains(&q)
                    || t.description
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(&q))
                        .unwrap_or(false)
            });
        }

        if let Some(prio_filter) = priority {
            if let Ok(target_prio) = prio_filter.parse::<crate::models::Priority>() {
                results.retain(|t| t.priority == target_prio);
            }
        }

        if let Some(tag_filter) = tag {
            let clean_tag = tag_filter.trim().trim_start_matches('#').to_lowercase();
            if !clean_tag.is_empty() {
                results.retain(|t| t.tags.contains(&clean_tag));
            }
        }

        Ok(results)
    }

    fn get_by_id(&self, id: i64) -> Result<Option<TodoItem>, StorageError> {
        let guard = self
            .data
            .read()
            .map_err(|_| StorageError::Validation("Lock poisoned".into()))?;
        Ok(guard.todos.iter().find(|t| t.id == id).cloned())
    }

    fn get_by_uuid(&self, uuid: &str) -> Result<Option<TodoItem>, StorageError> {
        let trimmed = uuid.trim();
        let guard = self
            .data
            .read()
            .map_err(|_| StorageError::Validation("Lock poisoned".into()))?;
        Ok(guard
            .todos
            .iter()
            .find(|t| t.uuid.as_deref() == Some(trimmed))
            .cloned())
    }

    fn create(&self, todo_in: TodoCreate) -> Result<TodoItem, StorageError> {
        let trimmed_title = todo_in.title.trim().to_string();
        if trimmed_title.is_empty() {
            return Err(StorageError::Validation("Title cannot be empty".into()));
        }
        let mut guard = self
            .data
            .write()
            .map_err(|_| StorageError::Validation("Lock poisoned".into()))?;

        let mut item = TodoItem::new(
            guard.next_id,
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

        guard.next_id += 1;
        guard.todos.push(item.clone());
        Ok(item)
    }

    fn update(&self, id: i64, todo_in: TodoUpdate) -> Result<Option<TodoItem>, StorageError> {
        let mut guard = self
            .data
            .write()
            .map_err(|_| StorageError::Validation("Lock poisoned".into()))?;
        if let Some(item) = guard.todos.iter_mut().find(|t| t.id == id) {
            if let Some(title) = todo_in.title {
                let trimmed = title.trim().to_string();
                if trimmed.is_empty() {
                    return Err(StorageError::Validation("Title cannot be empty".into()));
                }
                item.title = trimmed;
            }
            if let Some(desc) = todo_in.description {
                let trimmed = desc.trim().to_string();
                item.description = if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                };
            }
            if let Some(priority) = todo_in.priority {
                item.priority = priority;
            }
            if let Some(due) = todo_in.due_date {
                let trimmed = due.trim().to_string();
                item.due_date = if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                };
            }
            if let Some(rec) = todo_in.recurrence {
                item.recurrence = Some(rec);
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
            Ok(Some(item.clone()))
        } else {
            Ok(None)
        }
    }

    fn toggle(&self, id: i64) -> Result<Option<TodoItem>, StorageError> {
        let mut guard = self
            .data
            .write()
            .map_err(|_| StorageError::Validation("Lock poisoned".into()))?;
        if let Some(item) = guard.todos.iter_mut().find(|t| t.id == id) {
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
            Ok(Some(item.clone()))
        } else {
            Ok(None)
        }
    }

    fn toggle_subtask(&self, id: i64, subtask_id: &str) -> Result<Option<TodoItem>, StorageError> {
        let mut guard = self
            .data
            .write()
            .map_err(|_| StorageError::Validation("Lock poisoned".into()))?;
        if let Some(item) = guard.todos.iter_mut().find(|t| t.id == id) {
            if let Some(sub) = item.subtasks.iter_mut().find(|s| s.id == subtask_id) {
                sub.completed = !sub.completed;
                item.updated_at = Some(Utc::now().to_rfc3339());
                return Ok(Some(item.clone()));
            }
        }
        Ok(None)
    }

    fn delete(&self, id: i64) -> Result<bool, StorageError> {
        let mut guard = self
            .data
            .write()
            .map_err(|_| StorageError::Validation("Lock poisoned".into()))?;
        if let Some(pos) = guard.todos.iter().position(|t| t.id == id) {
            let removed = guard.todos.remove(pos);
            if let Some(uuid) = removed.uuid {
                if !uuid.trim().is_empty() {
                    guard.tombstones.push(Tombstone {
                        uuid,
                        deleted_at: Utc::now().to_rfc3339(),
                    });
                }
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn delete_by_uuid(&self, uuid: &str) -> Result<bool, StorageError> {
        let trimmed = uuid.trim();
        if trimmed.is_empty() {
            return Ok(false);
        }
        let mut guard = self
            .data
            .write()
            .map_err(|_| StorageError::Validation("Lock poisoned".into()))?;
        if let Some(pos) = guard
            .todos
            .iter()
            .position(|t| t.uuid.as_deref() == Some(trimmed))
        {
            let removed = guard.todos.remove(pos);
            if let Some(u) = removed.uuid {
                guard.tombstones.push(Tombstone {
                    uuid: u,
                    deleted_at: Utc::now().to_rfc3339(),
                });
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn clear_completed(&self) -> Result<usize, StorageError> {
        let mut guard = self
            .data
            .write()
            .map_err(|_| StorageError::Validation("Lock poisoned".into()))?;
        let mut removed = 0;
        let mut now_tombstones = Vec::new();
        let now_ts = Utc::now().to_rfc3339();
        guard.todos.retain(|t| {
            if t.completed {
                if let Some(u) = &t.uuid {
                    if !u.trim().is_empty() {
                        now_tombstones.push(Tombstone {
                            uuid: u.clone(),
                            deleted_at: now_ts.clone(),
                        });
                    }
                }
                removed += 1;
                false
            } else {
                true
            }
        });
        guard.tombstones.extend(now_tombstones);
        Ok(removed)
    }

    fn get_stats(&self) -> Result<TodoStats, StorageError> {
        let guard = self
            .data
            .read()
            .map_err(|_| StorageError::Validation("Lock poisoned".into()))?;
        let total = guard.todos.len();
        let completed = guard.todos.iter().filter(|t| t.completed).count();
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

    fn get_tombstones(&self) -> Result<Vec<Tombstone>, StorageError> {
        let guard = self
            .data
            .read()
            .map_err(|_| StorageError::Validation("Lock poisoned".into()))?;
        Ok(guard.tombstones.clone())
    }

    fn get_sync(&self, since: Option<&str>) -> Result<SyncResponse, StorageError> {
        let guard = self
            .data
            .read()
            .map_err(|_| StorageError::Validation("Lock poisoned".into()))?;
        let server_timestamp = Utc::now().to_rfc3339();

        match since {
            None => Ok(SyncResponse {
                updated_todos: guard.todos.clone(),
                deleted_uuids: guard.tombstones.iter().map(|t| t.uuid.clone()).collect(),
                server_timestamp,
            }),
            Some(since_str) => {
                let updated_todos: Vec<TodoItem> = guard
                    .todos
                    .iter()
                    .filter(|t| {
                        let ts = t.updated_at.as_deref().unwrap_or(&t.created_at);
                        ts > since_str
                    })
                    .cloned()
                    .collect();

                let deleted_uuids: Vec<String> = guard
                    .tombstones
                    .iter()
                    .filter(|t| t.deleted_at.as_str() > since_str)
                    .map(|t| t.uuid.clone())
                    .collect();

                Ok(SyncResponse {
                    updated_todos,
                    deleted_uuids,
                    server_timestamp,
                })
            }
        }
    }

    fn apply_sync(&self, sync: SyncResponse) -> Result<SyncApplyResult, StorageError> {
        let mut guard = self
            .data
            .write()
            .map_err(|_| StorageError::Validation("Lock poisoned".into()))?;
        let mut applied_deletions = 0;
        let mut applied_updates = 0;

        for del_uuid in &sync.deleted_uuids {
            let trimmed = del_uuid.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Some(pos) = guard
                .todos
                .iter()
                .position(|t| t.uuid.as_deref() == Some(trimmed))
            {
                guard.todos.remove(pos);
                applied_deletions += 1;
            }
            if !guard.tombstones.iter().any(|t| t.uuid == trimmed) {
                guard.tombstones.push(Tombstone {
                    uuid: trimmed.to_string(),
                    deleted_at: Utc::now().to_rfc3339(),
                });
            }
        }

        for remote in sync.updated_todos {
            let remote_uuid = match &remote.uuid {
                Some(u) if !u.trim().is_empty() => u.clone(),
                _ => continue,
            };

            let remote_ts = remote
                .updated_at
                .as_deref()
                .unwrap_or(&remote.created_at)
                .to_string();
            if let Some(tomb) = guard.tombstones.iter().find(|t| t.uuid == remote_uuid) {
                if tomb.deleted_at >= remote_ts {
                    continue;
                }
            }

            if let Some(local) = guard
                .todos
                .iter_mut()
                .find(|t| t.uuid.as_deref() == Some(&remote_uuid))
            {
                let local_ts = local.updated_at.as_deref().unwrap_or(&local.created_at);
                if remote_ts.as_str() >= local_ts {
                    local.title = remote.title;
                    local.description = remote.description;
                    local.completed = remote.completed;
                    local.priority = remote.priority;
                    local.due_date = remote.due_date;
                    local.recurrence = remote.recurrence;
                    local.tags = remote.tags;
                    local.subtasks = remote.subtasks;
                    local.estimated_minutes = remote.estimated_minutes;
                    local.updated_at = remote.updated_at.or(Some(remote_ts));
                    applied_updates += 1;
                }
            } else {
                let mut new_item = remote;
                new_item.id = guard.next_id;
                guard.next_id += 1;
                guard.todos.push(new_item);
                applied_updates += 1;
            }
        }

        Ok(SyncApplyResult {
            applied_updates,
            applied_deletions,
        })
    }
}

#[cfg(feature = "sqlite")]
pub struct SqliteTodoStorage {
    conn: std::sync::Mutex<rusqlite::Connection>,
}

#[cfg(feature = "sqlite")]
impl SqliteTodoStorage {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let conn = rusqlite::Connection::open(path)?;
        Self::init(&conn)?;
        Ok(Self {
            conn: std::sync::Mutex::new(conn),
        })
    }

    pub fn in_memory() -> Result<Self, StorageError> {
        let conn = rusqlite::Connection::open_in_memory()?;
        Self::init(&conn)?;
        Ok(Self {
            conn: std::sync::Mutex::new(conn),
        })
    }

    fn init(conn: &rusqlite::Connection) -> Result<(), StorageError> {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS todos (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                uuid TEXT UNIQUE,
                title TEXT NOT NULL,
                description TEXT,
                priority TEXT NOT NULL,
                completed INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT,
                due_date TEXT,
                recurrence TEXT,
                estimated_minutes INTEGER,
                tags TEXT NOT NULL DEFAULT '[]',
                subtasks TEXT NOT NULL DEFAULT '[]'
            );
            CREATE TABLE IF NOT EXISTS tombstones (
                uuid TEXT PRIMARY KEY,
                deleted_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_todos_uuid ON todos(uuid);
            "#,
        )?;
        Ok(())
    }

    fn row_to_item(row: &rusqlite::Row) -> rusqlite::Result<TodoItem> {
        let priority_str: String = row.get(4)?;
        let priority = priority_str
            .parse::<crate::models::Priority>()
            .unwrap_or_default();
        let recurrence_str: Option<String> = row.get(9)?;
        let recurrence =
            recurrence_str.and_then(|r| r.parse::<crate::models::RecurrencePattern>().ok());
        let tags_str: String = row.get(11)?;
        let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
        let subtasks_str: String = row.get(12)?;
        let subtasks: Vec<crate::models::Subtask> =
            serde_json::from_str(&subtasks_str).unwrap_or_default();

        Ok(TodoItem {
            id: row.get(0)?,
            uuid: row.get(1)?,
            title: row.get(2)?,
            description: row.get(3)?,
            priority,
            completed: row.get::<_, i64>(5)? != 0,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
            due_date: row.get(8)?,
            recurrence,
            estimated_minutes: row.get(10)?,
            tags,
            subtasks,
        })
    }
}

#[cfg(feature = "sqlite")]
impl TodoStorage for SqliteTodoStorage {
    fn get_all(
        &self,
        status: Option<&str>,
        search: Option<&str>,
        priority: Option<&str>,
        tag: Option<&str>,
    ) -> Result<Vec<TodoItem>, StorageError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| StorageError::Validation("Lock poisoned".into()))?;
        let mut query = "SELECT id, uuid, title, description, priority, completed, created_at, updated_at, due_date, recurrence, estimated_minutes, tags, subtasks FROM todos WHERE 1=1".to_string();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(status_filter) = status {
            match status_filter.to_lowercase().as_str() {
                "active" => query.push_str(" AND completed = 0"),
                "completed" => query.push_str(" AND completed = 1"),
                _ => {}
            }
        }

        if let Some(q) = search {
            query.push_str(" AND (LOWER(title) LIKE ? OR LOWER(description) LIKE ?)");
            let pattern = format!("%{}%", q.to_lowercase());
            params.push(Box::new(pattern.clone()));
            params.push(Box::new(pattern));
        }

        if let Some(p) = priority {
            query.push_str(" AND LOWER(priority) = ?");
            params.push(Box::new(p.to_lowercase()));
        }

        query.push_str(" ORDER BY id ASC");
        let mut stmt = conn.prepare(&query)?;
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let items_iter = stmt.query_map(param_refs.as_slice(), Self::row_to_item)?;

        let mut items = Vec::new();
        for item_res in items_iter {
            let item = item_res?;
            if let Some(tag_filter) = tag {
                let clean = tag_filter.trim().trim_start_matches('#').to_lowercase();
                if !clean.is_empty() && !item.tags.contains(&clean) {
                    continue;
                }
            }
            items.push(item);
        }
        Ok(items)
    }

    fn get_by_id(&self, id: i64) -> Result<Option<TodoItem>, StorageError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| StorageError::Validation("Lock poisoned".into()))?;
        let mut stmt = conn.prepare("SELECT id, uuid, title, description, priority, completed, created_at, updated_at, due_date, recurrence, estimated_minutes, tags, subtasks FROM todos WHERE id = ?1")?;
        let mut rows = stmt.query_map([id], Self::row_to_item)?;
        match rows.next() {
            Some(res) => Ok(Some(res?)),
            None => Ok(None),
        }
    }

    fn get_by_uuid(&self, uuid: &str) -> Result<Option<TodoItem>, StorageError> {
        let trimmed = uuid.trim();
        let conn = self
            .conn
            .lock()
            .map_err(|_| StorageError::Validation("Lock poisoned".into()))?;
        let mut stmt = conn.prepare("SELECT id, uuid, title, description, priority, completed, created_at, updated_at, due_date, recurrence, estimated_minutes, tags, subtasks FROM todos WHERE uuid = ?1")?;
        let mut rows = stmt.query_map([trimmed], Self::row_to_item)?;
        match rows.next() {
            Some(res) => Ok(Some(res?)),
            None => Ok(None),
        }
    }

    fn create(&self, todo_in: TodoCreate) -> Result<TodoItem, StorageError> {
        let trimmed_title = todo_in.title.trim().to_string();
        if trimmed_title.is_empty() {
            return Err(StorageError::Validation("Title cannot be empty".into()));
        }
        let conn = self
            .conn
            .lock()
            .map_err(|_| StorageError::Validation("Lock poisoned".into()))?;

        let mut item = TodoItem::new(
            0,
            trimmed_title,
            todo_in.description.map(|d| d.trim().to_string()),
            todo_in.priority.unwrap_or_default(),
        );
        if let Some(uuid_str) = todo_in.uuid {
            let trimmed = uuid_str.trim().to_string();
            if !trimmed.is_empty() {
                item.uuid = Some(trimmed);
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

        let tags_json = serde_json::to_string(&item.tags).unwrap_or_else(|_| "[]".into());
        let subtasks_json = serde_json::to_string(&item.subtasks).unwrap_or_else(|_| "[]".into());
        let rec_str = item.recurrence.map(|r| r.to_string());

        conn.execute(
            r#"INSERT INTO todos (uuid, title, description, priority, completed, created_at, updated_at, due_date, recurrence, estimated_minutes, tags, subtasks)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)"#,
            rusqlite::params![
                item.uuid,
                item.title,
                item.description,
                item.priority.to_string(),
                if item.completed { 1 } else { 0 },
                item.created_at,
                item.updated_at,
                item.due_date,
                rec_str,
                item.estimated_minutes,
                tags_json,
                subtasks_json,
            ],
        )?;

        let id = conn.last_insert_rowid();
        item.id = id;
        Ok(item)
    }

    fn update(&self, id: i64, todo_in: TodoUpdate) -> Result<Option<TodoItem>, StorageError> {
        let item_opt = self.get_by_id(id)?;
        let mut item = match item_opt {
            Some(i) => i,
            None => return Ok(None),
        };

        if let Some(title) = todo_in.title {
            let trimmed = title.trim().to_string();
            if trimmed.is_empty() {
                return Err(StorageError::Validation("Title cannot be empty".into()));
            }
            item.title = trimmed;
        }
        if let Some(desc) = todo_in.description {
            let trimmed = desc.trim().to_string();
            item.description = if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            };
        }
        if let Some(priority) = todo_in.priority {
            item.priority = priority;
        }
        if let Some(due) = todo_in.due_date {
            let trimmed = due.trim().to_string();
            item.due_date = if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            };
        }
        if let Some(rec) = todo_in.recurrence {
            item.recurrence = Some(rec);
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

        let tags_json = serde_json::to_string(&item.tags).unwrap_or_else(|_| "[]".into());
        let subtasks_json = serde_json::to_string(&item.subtasks).unwrap_or_else(|_| "[]".into());
        let rec_str = item.recurrence.map(|r| r.to_string());

        let conn = self
            .conn
            .lock()
            .map_err(|_| StorageError::Validation("Lock poisoned".into()))?;
        conn.execute(
            r#"UPDATE todos SET uuid = ?1, title = ?2, description = ?3, priority = ?4, completed = ?5,
               created_at = ?6, updated_at = ?7, due_date = ?8, recurrence = ?9, estimated_minutes = ?10,
               tags = ?11, subtasks = ?12 WHERE id = ?13"#,
            rusqlite::params![
                item.uuid,
                item.title,
                item.description,
                item.priority.to_string(),
                if item.completed { 1 } else { 0 },
                item.created_at,
                item.updated_at,
                item.due_date,
                rec_str,
                item.estimated_minutes,
                tags_json,
                subtasks_json,
                id
            ],
        )?;

        Ok(Some(item))
    }

    fn toggle(&self, id: i64) -> Result<Option<TodoItem>, StorageError> {
        let item_opt = self.get_by_id(id)?;
        let mut item = match item_opt {
            Some(i) => i,
            None => return Ok(None),
        };

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

        let rec_str = item.recurrence.map(|r| r.to_string());
        let conn = self
            .conn
            .lock()
            .map_err(|_| StorageError::Validation("Lock poisoned".into()))?;
        conn.execute(
            "UPDATE todos SET completed = ?1, due_date = ?2, recurrence = ?3, updated_at = ?4 WHERE id = ?5",
            rusqlite::params![
                if item.completed { 1 } else { 0 },
                item.due_date,
                rec_str,
                item.updated_at,
                id
            ],
        )?;

        Ok(Some(item))
    }

    fn toggle_subtask(&self, id: i64, subtask_id: &str) -> Result<Option<TodoItem>, StorageError> {
        let item_opt = self.get_by_id(id)?;
        let mut item = match item_opt {
            Some(i) => i,
            None => return Ok(None),
        };

        if let Some(sub) = item.subtasks.iter_mut().find(|s| s.id == subtask_id) {
            sub.completed = !sub.completed;
            item.updated_at = Some(Utc::now().to_rfc3339());

            let subtasks_json =
                serde_json::to_string(&item.subtasks).unwrap_or_else(|_| "[]".into());
            let conn = self
                .conn
                .lock()
                .map_err(|_| StorageError::Validation("Lock poisoned".into()))?;
            conn.execute(
                "UPDATE todos SET subtasks = ?1, updated_at = ?2 WHERE id = ?3",
                rusqlite::params![subtasks_json, item.updated_at, id],
            )?;

            return Ok(Some(item));
        }

        Ok(None)
    }

    fn delete(&self, id: i64) -> Result<bool, StorageError> {
        let item_opt = self.get_by_id(id)?;
        let item = match item_opt {
            Some(i) => i,
            None => return Ok(false),
        };

        let conn = self
            .conn
            .lock()
            .map_err(|_| StorageError::Validation("Lock poisoned".into()))?;
        conn.execute("DELETE FROM todos WHERE id = ?1", [id])?;
        if let Some(uuid) = item.uuid {
            if !uuid.trim().is_empty() {
                conn.execute(
                    "INSERT OR REPLACE INTO tombstones (uuid, deleted_at) VALUES (?1, ?2)",
                    rusqlite::params![uuid, Utc::now().to_rfc3339()],
                )?;
            }
        }
        Ok(true)
    }

    fn delete_by_uuid(&self, uuid: &str) -> Result<bool, StorageError> {
        let trimmed = uuid.trim();
        if trimmed.is_empty() {
            return Ok(false);
        }
        let conn = self
            .conn
            .lock()
            .map_err(|_| StorageError::Validation("Lock poisoned".into()))?;
        let affected = conn.execute("DELETE FROM todos WHERE uuid = ?1", [trimmed])?;
        if affected > 0 {
            conn.execute(
                "INSERT OR REPLACE INTO tombstones (uuid, deleted_at) VALUES (?1, ?2)",
                rusqlite::params![trimmed, Utc::now().to_rfc3339()],
            )?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn clear_completed(&self) -> Result<usize, StorageError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| StorageError::Validation("Lock poisoned".into()))?;
        let mut stmt =
            conn.prepare("SELECT uuid FROM todos WHERE completed = 1 AND uuid IS NOT NULL")?;
        let uuids: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        let removed = conn.execute("DELETE FROM todos WHERE completed = 1", [])?;
        let now_ts = Utc::now().to_rfc3339();
        for u in uuids {
            conn.execute(
                "INSERT OR REPLACE INTO tombstones (uuid, deleted_at) VALUES (?1, ?2)",
                rusqlite::params![u, now_ts],
            )?;
        }
        Ok(removed)
    }

    fn get_stats(&self) -> Result<TodoStats, StorageError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| StorageError::Validation("Lock poisoned".into()))?;
        let total: usize = conn.query_row("SELECT COUNT(*) FROM todos", [], |row| row.get(0))?;
        let completed: usize = conn.query_row(
            "SELECT COUNT(*) FROM todos WHERE completed = 1",
            [],
            |row| row.get(0),
        )?;
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

    fn get_tombstones(&self) -> Result<Vec<Tombstone>, StorageError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| StorageError::Validation("Lock poisoned".into()))?;
        let mut stmt =
            conn.prepare("SELECT uuid, deleted_at FROM tombstones ORDER BY deleted_at ASC")?;
        let rows = stmt.query_map([], |row| {
            Ok(Tombstone {
                uuid: row.get(0)?,
                deleted_at: row.get(1)?,
            })
        })?;
        let mut tombstones = Vec::new();
        for r in rows {
            tombstones.push(r?);
        }
        Ok(tombstones)
    }

    fn get_sync(&self, since: Option<&str>) -> Result<SyncResponse, StorageError> {
        let server_timestamp = Utc::now().to_rfc3339();
        let all_todos = self.get_all(None, None, None, None)?;
        let tombstones = self.get_tombstones()?;

        match since {
            None => Ok(SyncResponse {
                updated_todos: all_todos,
                deleted_uuids: tombstones.into_iter().map(|t| t.uuid).collect(),
                server_timestamp,
            }),
            Some(since_str) => {
                let updated_todos: Vec<TodoItem> = all_todos
                    .into_iter()
                    .filter(|t| {
                        let ts = t.updated_at.as_deref().unwrap_or(&t.created_at);
                        ts > since_str
                    })
                    .collect();

                let deleted_uuids: Vec<String> = tombstones
                    .into_iter()
                    .filter(|t| t.deleted_at.as_str() > since_str)
                    .map(|t| t.uuid)
                    .collect();

                Ok(SyncResponse {
                    updated_todos,
                    deleted_uuids,
                    server_timestamp,
                })
            }
        }
    }

    fn apply_sync(&self, sync: SyncResponse) -> Result<SyncApplyResult, StorageError> {
        let mut applied_deletions = 0;
        let mut applied_updates = 0;

        for del_uuid in &sync.deleted_uuids {
            if self.delete_by_uuid(del_uuid)? {
                applied_deletions += 1;
            }
        }

        for remote in sync.updated_todos {
            let remote_uuid = match &remote.uuid {
                Some(u) if !u.trim().is_empty() => u.clone(),
                _ => continue,
            };

            let remote_ts = remote
                .updated_at
                .as_deref()
                .unwrap_or(&remote.created_at)
                .to_string();

            if let Some(tomb) = self
                .get_tombstones()?
                .into_iter()
                .find(|t| t.uuid == remote_uuid)
            {
                if tomb.deleted_at >= remote_ts {
                    continue;
                }
            }

            if let Some(local) = self.get_by_uuid(&remote_uuid)? {
                let local_ts = local.updated_at.as_deref().unwrap_or(&local.created_at);
                if remote_ts.as_str() >= local_ts {
                    self.update(
                        local.id,
                        TodoUpdate {
                            title: Some(remote.title),
                            description: remote.description,
                            completed: Some(remote.completed),
                            priority: Some(remote.priority),
                            due_date: remote.due_date,
                            recurrence: remote.recurrence,
                            uuid: Some(remote_uuid),
                            tags: Some(remote.tags),
                            subtasks: Some(remote.subtasks),
                        },
                    )?;
                    applied_updates += 1;
                }
            } else {
                self.create(TodoCreate {
                    title: remote.title,
                    description: remote.description,
                    priority: Some(remote.priority),
                    due_date: remote.due_date,
                    recurrence: remote.recurrence,
                    uuid: Some(remote_uuid),
                    tags: remote.tags,
                    subtasks: remote.subtasks,
                })?;
                applied_updates += 1;
            }
        }

        Ok(SyncApplyResult {
            applied_updates,
            applied_deletions,
        })
    }
}
