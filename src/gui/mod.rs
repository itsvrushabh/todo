pub mod dialogs;
pub mod theme;
pub mod types;
pub mod views;

pub use dialogs::*;
pub use eframe::egui;
pub use theme::*;
pub use types::*;
pub use views::*;

use crate::models::{Priority, TimeBudgetFilter, TodoItem};
use chrono::Utc;
use std::path::PathBuf;

pub struct TodoGuiApp {
    pub todos: Vec<TodoItem>,
    pub filter: FilterStatus,
    pub search_query: String,
    pub new_title: String,
    pub new_description: String,
    pub new_priority: Priority,
    pub new_due_date: String,
    pub new_recurrence: Option<crate::models::RecurrencePattern>,
    pub show_add_dialog: bool,
    pub editing_id: Option<i64>,
    pub edit_title: String,
    pub edit_description: String,
    pub edit_priority: Priority,
    pub edit_due_date: String,
    pub edit_recurrence: Option<crate::models::RecurrencePattern>,
    pub storage_path: Option<PathBuf>,
    pub theme: GuiTheme,
    pub adaptive_mode: AdaptiveMode,
    pub calm_spotlight_idx: usize,
    pub desktop_board_view: bool,
    pub mobile_compact_meta: bool,
    pub time_budget: TimeBudgetFilter,
    pub selected_tag: Option<String>,
    pub planning_scratchpad: String,
    pub show_planning_scratchpad: bool,
    pub show_export_dialog: bool,
    pub export_markdown: bool,
    pub show_import_dialog: bool,
    pub import_text: String,
    pub import_status_message: Option<String>,
    pub server_url: String,
    pub sync_status: SyncStatus,
    pub show_settings_dialog: bool,
    pub pending_sync_events: std::sync::Arc<std::sync::Mutex<Vec<SyncResult>>>,
    pub pending_mutations: Vec<crate::models::PendingMutation>,
    pub is_draining_queue: bool,
    pub last_sync_timestamp: Option<String>,
    pub undo_stack: Vec<Vec<TodoItem>>,
    pub redo_stack: Vec<Vec<TodoItem>>,
}

impl TodoGuiApp {
    pub fn new(storage_path: Option<PathBuf>) -> Self {
        let mut app = Self {
            todos: Vec::new(),
            filter: FilterStatus::All,
            search_query: String::new(),
            new_title: String::new(),
            new_description: String::new(),
            new_priority: Priority::Medium,
            new_due_date: String::new(),
            new_recurrence: None,
            show_add_dialog: false,
            editing_id: None,
            edit_title: String::new(),
            edit_description: String::new(),
            edit_priority: Priority::Medium,
            edit_due_date: String::new(),
            edit_recurrence: None,
            storage_path,
            theme: GuiTheme::Macchiato,
            adaptive_mode: AdaptiveMode::default(),
            calm_spotlight_idx: 0,
            desktop_board_view: true,
            mobile_compact_meta: true,
            time_budget: TimeBudgetFilter::default(),
            selected_tag: None,
            planning_scratchpad: String::new(),
            show_planning_scratchpad: true,
            show_export_dialog: false,
            export_markdown: false,
            show_import_dialog: false,
            import_text: String::new(),
            import_status_message: None,
            server_url: String::new(),
            sync_status: SyncStatus::LocalOnly,
            show_settings_dialog: false,
            pending_sync_events: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            pending_mutations: Vec::new(),
            is_draining_queue: false,
            last_sync_timestamp: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        };
        app.load_todos();
        app
    }

    pub fn load_todos(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(ref path) = self.storage_path {
                let queue_path = path.with_extension("queue.json");
                if queue_path.exists() {
                    if let Ok(data) = std::fs::read_to_string(&queue_path) {
                        if let Ok(mutations) =
                            serde_json::from_str::<Vec<crate::models::PendingMutation>>(&data)
                        {
                            self.pending_mutations = mutations;
                        }
                    }
                }
                let scratchpad_path = path.with_extension("scratchpad.txt");
                if scratchpad_path.exists() {
                    if let Ok(content) = std::fs::read_to_string(&scratchpad_path) {
                        self.planning_scratchpad = content;
                    }
                }
                if path.exists() {
                    if let Ok(data) = std::fs::read_to_string(path) {
                        if let Ok(items) = serde_json::from_str::<Vec<TodoItem>>(&data) {
                            self.todos = items;
                            return;
                        }
                    }
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    let server_url_val = storage
                        .get_item("todo_server_url")
                        .ok()
                        .flatten()
                        .or_else(|| storage.get_item("taskmaster_server_url").ok().flatten());
                    if let Some(url) = server_url_val {
                        if !url.trim().is_empty() {
                            self.server_url = url;
                            self.sync_status = SyncStatus::Connected;
                        }
                    }

                    let pending_val = storage
                        .get_item("todo_pending_mutations")
                        .ok()
                        .flatten()
                        .or_else(|| {
                            storage
                                .get_item("taskmaster_pending_mutations")
                                .ok()
                                .flatten()
                        });
                    if let Some(data) = pending_val {
                        if let Ok(mutations) =
                            serde_json::from_str::<Vec<crate::models::PendingMutation>>(&data)
                        {
                            self.pending_mutations = mutations;
                        }
                    }

                    let todos_val = storage
                        .get_item("todo_todos")
                        .ok()
                        .flatten()
                        .or_else(|| storage.get_item("taskmaster_todos").ok().flatten());
                    if let Some(data) = todos_val {
                        if let Ok(items) = serde_json::from_str::<Vec<TodoItem>>(&data) {
                            self.todos = items;
                            return;
                        }
                    }

                    let scratchpad_val = storage
                        .get_item("todo_scratchpad")
                        .ok()
                        .flatten()
                        .or_else(|| storage.get_item("taskmaster_scratchpad").ok().flatten());
                    if let Some(content) = scratchpad_val {
                        self.planning_scratchpad = content;
                    }
                }
            }
        }

        // Default sample todos if empty
        if self.todos.is_empty() {
            self.todos = vec![
                TodoItem::new(
                    1,
                    "Welcome to your Todo App!".to_string(),
                    Some("Fast, reliable, and responsive task management.".to_string()),
                    Priority::High,
                ),
                TodoItem::new(
                    2,
                    "Explore keyboard shortcuts and layouts".to_string(),
                    Some("Switch layout modes, use shortcuts, and manage your tasks.".to_string()),
                    Priority::Medium,
                ),
            ];
            self.save_todos();
        }
    }

    pub fn save_todos(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(ref path) = self.storage_path {
                let queue_path = path.with_extension("queue.json");
                if self.pending_mutations.is_empty() {
                    let _ = std::fs::remove_file(&queue_path);
                } else if let Ok(json) = serde_json::to_string_pretty(&self.pending_mutations) {
                    let _ = std::fs::write(&queue_path, json);
                }

                if let Ok(json) = serde_json::to_string_pretty(&self.todos) {
                    if let Some(parent) = path.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    let _ = std::fs::write(path, json);
                }

                let scratchpad_path = path.with_extension("scratchpad.txt");
                let _ = std::fs::write(&scratchpad_path, &self.planning_scratchpad);
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    let _ = storage.set_item("todo_server_url", &self.server_url);
                    if let Ok(json) = serde_json::to_string(&self.todos) {
                        let _ = storage.set_item("todo_todos", &json);
                    }
                    if let Ok(json) = serde_json::to_string(&self.pending_mutations) {
                        let _ = storage.set_item("todo_pending_mutations", &json);
                    }
                    let _ = storage.set_item("todo_scratchpad", &self.planning_scratchpad);
                }
            }
        }
    }

    pub fn effective_mode(&self, screen_width: f32) -> AdaptiveMode {
        match self.adaptive_mode {
            AdaptiveMode::Auto => {
                if screen_width < 650.0 {
                    AdaptiveMode::MobileFocus
                } else {
                    AdaptiveMode::DesktopPlanning
                }
            }
            mode => mode,
        }
    }

    pub fn record_undo_snapshot(&mut self) {
        if self.undo_stack.len() >= 50 {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(self.todos.clone());
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(self.todos.clone());
            self.todos = prev;
            self.save_todos();
        }
    }

    pub fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.todos.clone());
            self.todos = next;
            self.save_todos();
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn add_todo(&mut self) {
        let trimmed = self.new_title.trim().to_string();
        if trimmed.is_empty() {
            return;
        }

        self.record_undo_snapshot();
        let new_id = self.todos.iter().map(|t| t.id).max().unwrap_or(0) + 1;
        let desc = if self.new_description.trim().is_empty() {
            None
        } else {
            Some(self.new_description.trim().to_string())
        };

        let mut item = TodoItem::new(new_id, trimmed, desc, self.new_priority);
        let due_trimmed = self.new_due_date.trim();
        if !due_trimmed.is_empty() {
            item.due_date = Some(due_trimmed.to_string());
        }
        item.recurrence = self.new_recurrence;

        self.todos.push(item.clone());

        if !self.server_url.trim().is_empty() {
            self.pending_mutations
                .push(crate::models::PendingMutation::Create(
                    crate::models::TodoCreate {
                        title: item.title.clone(),
                        description: item.description.clone(),
                        priority: Some(item.priority),
                        due_date: item.due_date.clone(),
                        recurrence: item.recurrence,
                        uuid: item.uuid.clone(),
                        tags: item.tags.clone(),
                        subtasks: item.subtasks.clone(),
                    },
                ));
        }

        self.new_title.clear();
        self.new_description.clear();
        self.new_priority = Priority::Medium;
        self.new_due_date.clear();
        self.new_recurrence = None;
        self.show_add_dialog = false;
        self.save_todos();
    }

    pub fn toggle_todo(&mut self, id: i64) {
        self.record_undo_snapshot();
        if let Some(todo) = self.todos.iter_mut().find(|t| t.id == id) {
            if !todo.completed && todo.recurrence.is_some() {
                if let Some(next_due) = todo.next_recurrence_date() {
                    todo.due_date = Some(next_due);
                    todo.completed = false;
                } else {
                    todo.completed = true;
                }
            } else {
                todo.completed = !todo.completed;
            }
            todo.updated_at = Some(Utc::now().to_rfc3339());

            if !self.server_url.trim().is_empty() {
                self.pending_mutations
                    .push(crate::models::PendingMutation::Toggle { id });
            }

            self.save_todos();
        }
    }

    pub fn delete_todo(&mut self, id: i64) {
        self.record_undo_snapshot();
        self.todos.retain(|t| t.id != id);
        if !self.server_url.trim().is_empty() {
            self.pending_mutations
                .push(crate::models::PendingMutation::Delete { id });
        }
        self.save_todos();
    }

    pub fn start_edit(&mut self, id: i64) {
        if let Some(todo) = self.todos.iter().find(|t| t.id == id) {
            self.editing_id = Some(id);
            self.edit_title = todo.title.clone();
            self.edit_description = todo.description.clone().unwrap_or_default();
            self.edit_priority = todo.priority;
            self.edit_due_date = todo.due_date.clone().unwrap_or_default();
            self.edit_recurrence = todo.recurrence;
        }
    }

    pub fn save_edit(&mut self) {
        if let Some(id) = self.editing_id {
            let trimmed = self.edit_title.trim().to_string();
            if !trimmed.is_empty() {
                self.record_undo_snapshot();
                let desc = if self.edit_description.trim().is_empty() {
                    None
                } else {
                    Some(self.edit_description.trim().to_string())
                };
                let due = if self.edit_due_date.trim().is_empty() {
                    None
                } else {
                    Some(self.edit_due_date.trim().to_string())
                };

                if let Some(todo) = self.todos.iter_mut().find(|t| t.id == id) {
                    todo.title = trimmed.clone();
                    todo.description = desc.clone();
                    todo.priority = self.edit_priority;
                    todo.due_date = due.clone();
                    todo.recurrence = self.edit_recurrence;
                    todo.updated_at = Some(Utc::now().to_rfc3339());
                    for t in TodoItem::extract_tags(&trimmed) {
                        if !todo.tags.contains(&t) {
                            todo.tags.push(t);
                        }
                    }

                    if !self.server_url.trim().is_empty() {
                        self.pending_mutations
                            .push(crate::models::PendingMutation::Update {
                                id,
                                update: crate::models::TodoUpdate {
                                    title: Some(trimmed),
                                    description: desc,
                                    completed: None,
                                    priority: Some(self.edit_priority),
                                    due_date: due,
                                    recurrence: self.edit_recurrence,
                                    uuid: todo.uuid.clone(),
                                    tags: Some(todo.tags.clone()),
                                    subtasks: Some(todo.subtasks.clone()),
                                },
                            });
                    }

                    self.save_todos();
                }
            }
        }
        self.editing_id = None;
        self.edit_due_date.clear();
        self.edit_recurrence = None;
    }

    pub fn clear_completed(&mut self) {
        self.record_undo_snapshot();
        self.todos.retain(|t| !t.completed);
        self.save_todos();
    }

    pub fn export_to_json(&self) -> String {
        serde_json::to_string_pretty(&self.todos).unwrap_or_default()
    }

    pub fn export_to_markdown(&self) -> String {
        let mut out = String::from("# Todo Export\n\n");
        for todo in &self.todos {
            let check = if todo.completed { "[x]" } else { "[ ]" };
            let prio_badge = match todo.priority {
                Priority::High => "[HIGH]",
                Priority::Medium => "[MED]",
                Priority::Low => "[LOW]",
            };
            out.push_str(&format!("- {check} {prio_badge} **{}**", todo.title));
            if let Some(ref d) = todo.due_date {
                out.push_str(&format!(" *(Due: {d})*"));
            }
            out.push('\n');
            if let Some(ref desc) = todo.description {
                if !desc.is_empty() {
                    out.push_str(&format!("  ↳ {desc}\n"));
                }
            }
        }
        out
    }

    pub fn import_from_json(&mut self, json_str: &str) -> Result<usize, String> {
        let items: Vec<TodoItem> =
            serde_json::from_str(json_str).map_err(|e| format!("Invalid JSON format: {e}"))?;
        let count = items.len();
        if count > 0 {
            self.record_undo_snapshot();
        }
        for item in items {
            let new_id = self.todos.iter().map(|t| t.id).max().unwrap_or(0) + 1;
            let mut new_item = item;
            new_item.id = new_id;
            self.todos.push(new_item);
        }
        self.save_todos();
        Ok(count)
    }

    pub fn drain_next_mutation(&mut self, ctx: egui::Context) {
        if self.is_draining_queue || self.pending_mutations.is_empty() {
            return;
        }

        let trimmed_url = self.server_url.trim().trim_end_matches('/');
        if trimmed_url.is_empty() {
            self.sync_status = SyncStatus::LocalOnly;
            return;
        }

        self.is_draining_queue = true;
        self.sync_status = SyncStatus::Syncing;

        let mutation = self.pending_mutations[0].clone();
        let request = mutation.to_request(trimmed_url);
        let pending = self.pending_sync_events.clone();

        ehttp::fetch(request, move |result| {
            if let Ok(mut guard) = pending.lock() {
                match result {
                    Ok(response) => {
                        if response.status >= 200 && response.status < 300 {
                            guard.push(SyncResult::MutationDrained {
                                success: true,
                                error: None,
                            });
                        } else {
                            guard.push(SyncResult::MutationDrained {
                                success: false,
                                error: Some(format!("Server returned HTTP {}", response.status)),
                            });
                        }
                    }
                    Err(err) => {
                        guard.push(SyncResult::MutationDrained {
                            success: false,
                            error: Some(format!("Network unreachable: {err}")),
                        });
                    }
                }
            }
            ctx.request_repaint();
        });
    }

    pub fn fetch_from_server(&mut self, ctx: egui::Context) {
        let trimmed_url = self.server_url.trim().trim_end_matches('/');
        if trimmed_url.is_empty() {
            self.sync_status = SyncStatus::LocalOnly;
            return;
        }

        self.sync_status = SyncStatus::Syncing;
        let url = format!("{trimmed_url}/api/todos");
        let request = ehttp::Request::get(&url);
        let pending = self.pending_sync_events.clone();

        ehttp::fetch(request, move |result| {
            if let Ok(mut guard) = pending.lock() {
                match result {
                    Ok(response) => {
                        if response.status == 200 {
                            if let Ok(items) =
                                serde_json::from_slice::<Vec<TodoItem>>(&response.bytes)
                            {
                                guard.push(SyncResult::FetchedTodos(items));
                            } else {
                                guard.push(SyncResult::SyncError(
                                    "Failed to parse JSON response from server".to_string(),
                                ));
                            }
                        } else {
                            guard.push(SyncResult::SyncError(format!(
                                "Server error: HTTP {}",
                                response.status
                            )));
                        }
                    }
                    Err(err) => {
                        guard.push(SyncResult::SyncError(format!("Network error: {err}")));
                    }
                }
            }
            ctx.request_repaint();
        });
    }

    pub fn fetch_differential_sync(&mut self, ctx: egui::Context) {
        let trimmed_url = self.server_url.trim().trim_end_matches('/');
        if trimmed_url.is_empty() {
            self.sync_status = SyncStatus::LocalOnly;
            return;
        }

        self.sync_status = SyncStatus::Syncing;
        let url = match &self.last_sync_timestamp {
            Some(since) => format!("{trimmed_url}/api/sync?since={since}"),
            None => format!("{trimmed_url}/api/sync"),
        };
        let request = ehttp::Request::get(&url);
        let pending = self.pending_sync_events.clone();

        ehttp::fetch(request, move |result| {
            if let Ok(mut guard) = pending.lock() {
                match result {
                    Ok(response) => {
                        if response.status == 200 {
                            if let Ok(sync_resp) = serde_json::from_slice::<
                                crate::models::SyncResponse,
                            >(&response.bytes)
                            {
                                guard.push(SyncResult::DifferentialSync(sync_resp));
                            } else {
                                guard.push(SyncResult::SyncError(
                                    "Failed to parse SyncResponse from server".to_string(),
                                ));
                            }
                        } else {
                            guard.push(SyncResult::SyncError(format!(
                                "Server error: HTTP {}",
                                response.status
                            )));
                        }
                    }
                    Err(err) => {
                        guard.push(SyncResult::SyncError(format!("Network error: {err}")));
                    }
                }
            }
            ctx.request_repaint();
        });
    }

    pub fn merge_differential_sync(&mut self, sync: crate::models::SyncResponse) {
        self.last_sync_timestamp = Some(sync.server_timestamp);

        // 1. Apply remote deletions
        for del_uuid in &sync.deleted_uuids {
            let trimmed = del_uuid.trim();
            if !trimmed.is_empty() {
                self.todos.retain(|t| t.uuid.as_deref() != Some(trimmed));
            }
        }

        // 2. Apply remote updates with Last-Write-Wins (LWW)
        for remote in sync.updated_todos {
            if let Some(remote_uuid) = &remote.uuid {
                let remote_ts = remote
                    .updated_at
                    .as_deref()
                    .unwrap_or(&remote.created_at)
                    .to_string();
                if let Some(local) = self
                    .todos
                    .iter_mut()
                    .find(|t| t.uuid.as_deref() == Some(remote_uuid))
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
                    }
                } else {
                    self.todos.push(remote);
                }
            } else {
                self.todos.push(remote);
            }
        }
        self.save_todos();
    }

    pub fn test_connection(&mut self, ctx: egui::Context) {
        let trimmed_url = self.server_url.trim().trim_end_matches('/');
        if trimmed_url.is_empty() {
            self.sync_status = SyncStatus::LocalOnly;
            return;
        }

        self.sync_status = SyncStatus::Syncing;
        let url = format!("{trimmed_url}/api/health");
        let request = ehttp::Request::get(&url);
        let pending = self.pending_sync_events.clone();

        ehttp::fetch(request, move |result| {
            if let Ok(mut guard) = pending.lock() {
                match result {
                    Ok(response) => {
                        if response.status == 200 {
                            guard.push(SyncResult::SuccessMessage(
                                "Connected successfully to backend API".to_string(),
                            ));
                        } else {
                            guard.push(SyncResult::SyncError(format!(
                                "Server returned HTTP {}",
                                response.status
                            )));
                        }
                    }
                    Err(err) => {
                        guard.push(SyncResult::SyncError(format!("Network error: {err}")));
                    }
                }
            }
            ctx.request_repaint();
        });
    }

    pub fn all_tags(&self) -> Vec<String> {
        let mut tags = Vec::new();
        for todo in &self.todos {
            for tag in &todo.tags {
                if !tags.contains(tag) {
                    tags.push(tag.clone());
                }
            }
        }
        tags.sort();
        tags
    }

    pub fn toggle_subtask(&mut self, todo_id: i64, subtask_id: &str) {
        self.record_undo_snapshot();
        if let Some(todo) = self.todos.iter_mut().find(|t| t.id == todo_id) {
            if let Some(sub) = todo.subtasks.iter_mut().find(|s| s.id == subtask_id) {
                sub.completed = !sub.completed;
                todo.updated_at = Some(Utc::now().to_rfc3339());

                if !self.server_url.trim().is_empty() {
                    self.pending_mutations
                        .push(crate::models::PendingMutation::ToggleSubtask {
                            id: todo_id,
                            subtask_id: subtask_id.to_string(),
                        });
                }

                self.save_todos();
            }
        }
    }

    pub fn filtered_todos(&self) -> Vec<&TodoItem> {
        self.todos
            .iter()
            .filter(|todo| match self.filter {
                FilterStatus::All => true,
                FilterStatus::Active => !todo.completed,
                FilterStatus::Completed => todo.completed,
            })
            .filter(|todo| self.time_budget.matches(todo.effective_estimated_minutes()))
            .filter(|todo| {
                if let Some(ref tag) = self.selected_tag {
                    todo.tags.iter().any(|t| t.eq_ignore_ascii_case(tag))
                } else {
                    true
                }
            })
            .filter(|todo| {
                if self.search_query.trim().is_empty() {
                    true
                } else {
                    let q = self.search_query.to_lowercase();
                    let in_title = todo.title.to_lowercase().contains(&q);
                    let in_desc = todo
                        .description
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(&q))
                        .unwrap_or(false);
                    let in_tag = todo.tags.iter().any(|t| t.to_lowercase().contains(&q));
                    in_title || in_desc || in_tag
                }
            })
            .collect()
    }

    pub fn convert_scratchpad_to_tasks(&mut self) -> usize {
        let mut added_count = 0;
        let mut new_items = Vec::new();

        for line in self.planning_scratchpad.lines() {
            let trimmed = line.trim();
            let clean = trimmed
                .trim_start_matches("- [ ]")
                .trim_start_matches("- [x]")
                .trim_start_matches("- ")
                .trim_start_matches("* ")
                .trim_start_matches("• ")
                .trim();

            if !clean.is_empty() {
                let next_id = self
                    .todos
                    .iter()
                    .map(|t| t.id)
                    .chain(new_items.iter().map(|t: &TodoItem| t.id))
                    .max()
                    .unwrap_or(0)
                    + 1;
                let duration = TodoItem::parse_duration_str(clean);
                let item = TodoItem {
                    id: next_id,
                    uuid: Some(uuid::Uuid::new_v4().to_string()),
                    title: clean.to_string(),
                    description: None,
                    priority: Priority::Medium,
                    completed: false,
                    created_at: Utc::now().to_rfc3339(),
                    updated_at: None,
                    due_date: None,
                    recurrence: None,
                    estimated_minutes: duration,
                    tags: TodoItem::extract_tags(clean),
                    subtasks: Vec::new(),
                };
                if !self.server_url.trim().is_empty() {
                    self.pending_mutations
                        .push(crate::models::PendingMutation::Create(
                            crate::models::TodoCreate {
                                title: item.title.clone(),
                                description: None,
                                priority: Some(item.priority),
                                due_date: None,
                                recurrence: None,
                                uuid: item.uuid.clone(),
                                tags: item.tags.clone(),
                                subtasks: Vec::new(),
                            },
                        ));
                }
                new_items.push(item);
                added_count += 1;
            }
        }

        if added_count > 0 {
            self.record_undo_snapshot();
            self.todos.extend(new_items);
            self.planning_scratchpad.clear();
            self.save_todos();
        }
        added_count
    }

    pub fn update_priority(&mut self, id: i64, new_priority: Priority) {
        self.record_undo_snapshot();
        if let Some(todo) = self.todos.iter_mut().find(|t| t.id == id) {
            todo.priority = new_priority;
            todo.updated_at = Some(Utc::now().to_rfc3339());

            if !self.server_url.trim().is_empty() {
                self.pending_mutations
                    .push(crate::models::PendingMutation::Update {
                        id,
                        update: crate::models::TodoUpdate {
                            title: None,
                            description: None,
                            completed: None,
                            priority: Some(new_priority),
                            due_date: None,
                            recurrence: None,
                            uuid: todo.uuid.clone(),
                            tags: None,
                            subtasks: None,
                        },
                    });
            }

            self.save_todos();
        }
    }

    pub fn mobile_focus_todos(&self) -> Vec<&TodoItem> {
        let mut list = self.filtered_todos();
        list.sort_by_key(|t| {
            if t.completed {
                100
            } else if t.due_status() == Some(crate::models::DueStatus::Overdue) {
                0
            } else if t.priority == Priority::High {
                1
            } else if t.due_status() == Some(crate::models::DueStatus::DueToday) {
                2
            } else if t.priority == Priority::Medium {
                3
            } else {
                4
            }
        });
        list
    }

    pub fn update_headless(&mut self, ctx: &egui::Context) {
        self.update_internal(ctx);
    }

    pub fn update_internal(&mut self, ctx: &egui::Context) {
        // Keyboard Shortcuts
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::N)) {
            self.show_add_dialog = true;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.show_add_dialog = false;
            self.editing_id = None;
            self.show_export_dialog = false;
            self.show_import_dialog = false;
            self.show_settings_dialog = false;
        }

        // Process pending async background sync events
        let events: Vec<SyncResult> = match self.pending_sync_events.try_lock() {
            Ok(mut guard) => guard.drain(..).collect(),
            Err(_) => Vec::new(),
        };

        let mut trigger_drain = false;
        let mut trigger_fetch = false;

        for event in events {
            match event {
                SyncResult::FetchedTodos(items) => {
                    self.todos = items;
                    self.sync_status = SyncStatus::Connected;
                    self.save_todos();
                }
                SyncResult::DifferentialSync(sync) => {
                    self.merge_differential_sync(sync);
                    self.sync_status = SyncStatus::Connected;
                }
                SyncResult::MutationDrained { success, error: _ } => {
                    self.is_draining_queue = false;
                    if success {
                        if !self.pending_mutations.is_empty() {
                            self.pending_mutations.remove(0);
                            self.save_todos();
                        }
                        if !self.pending_mutations.is_empty() {
                            trigger_drain = true;
                        } else {
                            self.sync_status = SyncStatus::Connected;
                            trigger_fetch = true;
                        }
                    } else {
                        self.sync_status = SyncStatus::Offline {
                            pending_count: self.pending_mutations.len(),
                        };
                        ctx.request_repaint_after(std::time::Duration::from_secs(5));
                    }
                }
                SyncResult::SyncError(err) => {
                    if !self.pending_mutations.is_empty() {
                        self.sync_status = SyncStatus::Offline {
                            pending_count: self.pending_mutations.len(),
                        };
                        ctx.request_repaint_after(std::time::Duration::from_secs(5));
                    } else {
                        self.sync_status = SyncStatus::Error(err);
                    }
                }
                SyncResult::SuccessMessage(_) => {
                    self.sync_status = SyncStatus::Connected;
                }
            }
        }

        if trigger_drain {
            self.drain_next_mutation(ctx.clone());
        } else if trigger_fetch {
            self.fetch_from_server(ctx.clone());
        }

        // Global Undo/Redo Shortcuts: Ctrl+Z / Cmd+Z and Ctrl+Y / Cmd+Shift+Z
        let (trigger_undo, trigger_redo) = ctx.input(|i| {
            let undo = i.modifiers.command && i.key_pressed(egui::Key::Z) && !i.modifiers.shift;
            let redo = (i.modifiers.command && i.key_pressed(egui::Key::Y))
                || (i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::Z));
            (undo, redo)
        });
        if trigger_undo {
            self.undo();
        } else if trigger_redo {
            self.redo();
        }

        // Automatic retry/drain if queue is non-empty and not currently draining
        if !self.pending_mutations.is_empty()
            && !self.is_draining_queue
            && !self.server_url.trim().is_empty()
        {
            match &self.sync_status {
                SyncStatus::Offline { .. } => {
                    ctx.request_repaint_after(std::time::Duration::from_secs(5));
                }
                _ => {
                    self.drain_next_mutation(ctx.clone());
                }
            }
        }

        // Apply Dynamic Theme
        let is_dark = self.theme.is_dark();
        let mut visuals = if is_dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        visuals.override_text_color = Some(self.theme.primary_text());
        visuals.panel_fill = self.theme.bg();
        visuals.window_fill = self.theme.top_panel_bg();
        ctx.set_visuals(visuals);

        let current_mode = self.effective_mode(ctx.screen_rect().width());
        let total = self.todos.len();
        let completed = self.todos.iter().filter(|t| t.completed).count();
        let active = total.saturating_sub(completed);
        let completion_rate = if total > 0 {
            (completed as f32 / total as f32) * 100.0_f32
        } else {
            0.0_f32
        };

        // Top Header Panel
        egui::TopBottomPanel::top("header_panel")
            .frame(
                egui::Frame::new()
                    .fill(self.theme.top_panel_bg())
                    .inner_margin(egui::Margin::same(16)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading(
                        egui::RichText::new("🦀 Todo")
                            .color(egui::Color32::from_rgb(203, 166, 247)) // Mauve
                            .strong()
                            .size(22.0),
                    );

                    let mut trigger_fetch = false;

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .button(
                                egui::RichText::new("➕ New Task")
                                    .color(egui::Color32::from_rgb(17, 17, 27))
                                    .strong(),
                            )
                            .on_hover_text("Shortcut: Ctrl+N / Cmd+N")
                            .clicked()
                        {
                            self.show_add_dialog = !self.show_add_dialog;
                        }

                        let can_redo = self.can_redo();
                        if ui
                            .add_enabled(can_redo, egui::Button::new("⟳ Redo"))
                            .on_hover_text("Redo last undone action (Ctrl+Y / Cmd+Shift+Z)")
                            .clicked()
                        {
                            self.redo();
                        }

                        let can_undo = self.can_undo();
                        if ui
                            .add_enabled(can_undo, egui::Button::new("⟲ Undo"))
                            .on_hover_text("Undo last action (Ctrl+Z / Cmd+Z)")
                            .clicked()
                        {
                            self.undo();
                        }

                        // In Calm Mode, hide Clear Done, Export, Import, Settings, and Sync Status
                        if current_mode != AdaptiveMode::Calm {
                            if completed > 0
                                && ui
                                    .button(
                                        egui::RichText::new("🗑️ Clear Done")
                                            .color(egui::Color32::from_rgb(243, 139, 168)),
                                    )
                                    .clicked()
                            {
                                self.clear_completed();
                            }

                            if ui.button("📥 Export").clicked() {
                                self.show_export_dialog = true;
                            }

                            if ui.button("📤 Import").clicked() {
                                self.show_import_dialog = true;
                            }

                            if ui.button("⚙️ Sync Settings").clicked() {
                                self.show_settings_dialog = true;
                            }

                            // Sync Status Indicator & Quick Action
                            match &self.sync_status {
                                SyncStatus::Connected => {
                                    if ui
                                        .small_button("🔄")
                                        .on_hover_text("Sync from server now")
                                        .clicked()
                                    {
                                        trigger_fetch = true;
                                    }
                                    let (label_text, label_color) =
                                        if !self.pending_mutations.is_empty() {
                                            (
                                                format!("⚠️ {} queued", self.pending_mutations.len()),
                                                egui::Color32::from_rgb(250, 179, 135),
                                            )
                                        } else {
                                            (
                                                "🟢 Synchronized".to_string(),
                                                egui::Color32::from_rgb(166, 227, 161),
                                            )
                                        };
                                    ui.label(
                                        egui::RichText::new(label_text)
                                            .color(label_color)
                                            .size(11.0)
                                            .strong(),
                                    )
                                    .on_hover_text(format!("Connected to {}", self.server_url));
                                }
                                SyncStatus::Syncing => {
                                    let label_text = if !self.pending_mutations.is_empty() {
                                        format!(
                                            "🔄 Syncing ({} pending)...",
                                            self.pending_mutations.len()
                                        )
                                    } else {
                                        "🔄 Syncing...".to_string()
                                    };
                                    ui.label(
                                        egui::RichText::new(label_text)
                                            .color(egui::Color32::from_rgb(137, 180, 250))
                                            .size(11.0),
                                    );
                                }
                                SyncStatus::Offline { pending_count } => {
                                    if ui
                                        .small_button("🔄")
                                        .on_hover_text("Retry connection & sync queue")
                                        .clicked()
                                    {
                                        trigger_drain = true;
                                    }
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "⚠️ Offline ({pending_count} pending)"
                                        ))
                                        .color(egui::Color32::from_rgb(250, 179, 135))
                                        .size(11.0)
                                        .strong(),
                                    )
                                    .on_hover_text("Backend unreachable. Mutations saved locally and will auto-sync on reconnect.");
                                }
                                SyncStatus::Error(err) => {
                                    if ui
                                        .small_button("🔄")
                                        .on_hover_text("Retry server sync")
                                        .clicked()
                                    {
                                        trigger_fetch = true;
                                    }
                                    ui.label(
                                        egui::RichText::new("🔴 Offline")
                                            .color(egui::Color32::from_rgb(243, 139, 168))
                                            .size(11.0)
                                            .strong(),
                                    )
                                    .on_hover_text(err);
                                }
                                SyncStatus::LocalOnly => {
                                    ui.label(
                                        egui::RichText::new("🟡 Local")
                                            .color(egui::Color32::from_rgb(249, 226, 175))
                                            .size(11.0),
                                    )
                                    .on_hover_text(
                                        "Using local storage. Click ⚙️ to configure remote sync.",
                                    );
                                }
                            }
                        }

                        egui::ComboBox::from_id_salt("adaptive_mode_selector")
                            .selected_text(self.adaptive_mode.display_label())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.adaptive_mode,
                                    AdaptiveMode::Auto,
                                    "🤖 Auto Layout",
                                );
                                ui.selectable_value(
                                    &mut self.adaptive_mode,
                                    AdaptiveMode::DesktopPlanning,
                                    "🖥️ Desktop Planning",
                                );
                                ui.selectable_value(
                                    &mut self.adaptive_mode,
                                    AdaptiveMode::MobileFocus,
                                    "📱 Mobile Focus",
                                );
                                ui.selectable_value(
                                    &mut self.adaptive_mode,
                                    AdaptiveMode::Calm,
                                    "🌿 Calm Mode",
                                );
                            });

                        egui::ComboBox::from_id_salt("theme_selector")
                            .selected_text(match self.theme {
                                GuiTheme::Macchiato => "🎨 Dark",
                                GuiTheme::Latte => "☀️ Light",
                                GuiTheme::HighContrast => "⬛ High Contrast",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.theme,
                                    GuiTheme::Macchiato,
                                    "🎨 Dark (Macchiato)",
                                );
                                ui.selectable_value(
                                    &mut self.theme,
                                    GuiTheme::Latte,
                                    "☀️ Light (Latte)",
                                );
                                ui.selectable_value(
                                    &mut self.theme,
                                    GuiTheme::HighContrast,
                                    "⬛ High Contrast",
                                );
                            });
                    });

                    if trigger_fetch {
                        self.fetch_from_server(ctx.clone());
                    }
                });

                // In Calm Mode, hide Progress Bar and Filter & Search Controls
                if current_mode != AdaptiveMode::Calm {
                    ui.add_space(10.0);

                    // Progress Bar
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("Progress ({completed}/{total})"))
                                .size(12.0)
                                .color(egui::Color32::from_rgb(166, 173, 200)),
                        );
                        let progress = if total > 0 {
                            completed as f32 / total as f32
                        } else {
                            0.0_f32
                        };
                        ui.add(
                            egui::ProgressBar::new(progress)
                                .text(format!("{:.0}%", completion_rate))
                                .fill(egui::Color32::from_rgb(166, 227, 161)), // Green
                        );
                    });

                    ui.add_space(10.0);

                    // Filter & Search Controls
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.filter, FilterStatus::All, "All");
                        ui.selectable_value(&mut self.filter, FilterStatus::Active, "Active");
                        ui.selectable_value(&mut self.filter, FilterStatus::Completed, "Completed");

                        ui.separator();

                        ui.label("⏱️ Time:");
                        ui.selectable_value(&mut self.time_budget, TimeBudgetFilter::All, "All");
                        ui.selectable_value(
                            &mut self.time_budget,
                            TimeBudgetFilter::QuickWins,
                            "⚡ Quick (≤15m)",
                        );
                        ui.selectable_value(
                            &mut self.time_budget,
                            TimeBudgetFilter::ShortSession,
                            "⏱️ Short (≤30m)",
                        );
                        ui.selectable_value(
                            &mut self.time_budget,
                            TimeBudgetFilter::DeepWork,
                            "🎯 Deep (>30m)",
                        );

                        ui.separator();

                        let search_id = egui::Id::new("gui_search_input");
                        if ctx.input(|i| !i.modifiers.command && i.key_pressed(egui::Key::Slash))
                            && !ctx.wants_keyboard_input()
                        {
                            ui.memory_mut(|m| m.request_focus(search_id));
                        }

                        ui.label("🔍");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.search_query)
                                .id(search_id)
                                .hint_text("Search tasks... [/]")
                                .desired_width(200.0),
                        );

                        if !self.search_query.is_empty() && ui.small_button("✖").clicked() {
                            self.search_query.clear();
                        }

                        let tags = self.all_tags();
                        if !tags.is_empty() {
                            ui.separator();
                            ui.label("🏷️");
                            let is_all_tags = self.selected_tag.is_none();
                            if ui.selectable_label(is_all_tags, "All").clicked() {
                                self.selected_tag = None;
                            }
                            for tag in &tags {
                                let is_sel = self.selected_tag.as_deref() == Some(tag.as_str());
                                if ui.selectable_label(is_sel, format!("#{tag}")).clicked() {
                                    if is_sel {
                                        self.selected_tag = None;
                                    } else {
                                        self.selected_tag = Some(tag.clone());
                                    }
                                }
                            }
                        }
                    });
                }
            });

        // Bottom Status Bar
        egui::TopBottomPanel::bottom("status_bar")
            .frame(
                egui::Frame::new()
                    .fill(self.theme.top_panel_bg())
                    .inner_margin(egui::Margin::symmetric(16, 8)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if current_mode == AdaptiveMode::Calm {
                        ui.label(
                            egui::RichText::new("🌿 Calm Focus Mode    •    [Ctrl+N] New Task  |  [Esc] Close Dialogs")
                                .size(11.0)
                                .color(egui::Color32::from_rgb(147, 153, 178)),
                        );
                    } else {
                        ui.label(
                            egui::RichText::new(format!(
                                "Active: {active}  |  Completed: {completed}  |  Total: {total}    •    [Ctrl+N] New Task  |  [/] Focus Search  |  [Esc] Close Dialogs"
                            ))
                            .size(11.0)
                            .color(egui::Color32::from_rgb(147, 153, 178)),
                        );
                    }
                });
            });

        // Central Content Panel
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(self.theme.bg())
                    .inner_margin(egui::Margin::same(16)),
            )
            .show(ctx, |ui| {
                dialogs::render_add_task_panel(self, ui);
                dialogs::render_edit_task_panel(self, ui);

                match current_mode {
                    AdaptiveMode::DesktopPlanning => {
                        views::render_desktop_planning_banner(self, ui);
                    }
                    AdaptiveMode::MobileFocus => {
                        views::render_mobile_focus_banner(self, ui);
                    }
                    AdaptiveMode::Calm => {
                        views::render_calm_banner(self, ui);
                    }
                    AdaptiveMode::Auto => {}
                }

                let filtered = if current_mode == AdaptiveMode::Calm {
                    self.todos.iter().filter(|t| !t.completed).collect()
                } else {
                    self.filtered_todos()
                };
                if filtered.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(40.0);
                        ui.label(
                            egui::RichText::new("✨ No tasks to display!")
                                .size(18.0)
                                .color(egui::Color32::from_rgb(147, 153, 178)),
                        );
                        ui.label(
                            egui::RichText::new("Click '+ New Task' above to add your first item.")
                                .size(13.0)
                                .color(egui::Color32::from_rgb(108, 112, 134)),
                        );
                    });
                } else {
                    let mut action = views::ViewAction::default();

                    match current_mode {
                        AdaptiveMode::DesktopPlanning if self.desktop_board_view => {
                            views::render_desktop_board(self, ui, &filtered, &mut action);
                        }
                        AdaptiveMode::MobileFocus => {
                            views::render_mobile_focus_view(self, ui, &mut action);
                        }
                        AdaptiveMode::Calm => {
                            views::render_calm_view(self, ui, &filtered, &mut action);
                        }
                        _ => {
                            views::render_standard_list_view(self, ui, &filtered, &mut action);
                        }
                    }

                    if let Some(id) = action.toggled_id {
                        self.toggle_todo(id);
                    }
                    if let Some(id) = action.delete_id {
                        self.delete_todo(id);
                    }
                    if let Some(id) = action.edit_id {
                        self.start_edit(id);
                    }
                    if let Some((id, new_prio)) = action.priority_change {
                        self.update_priority(id, new_prio);
                    }
                    if let Some(idx) = action.next_spotlight_idx {
                        self.calm_spotlight_idx = idx;
                    }
                    if let Some((id, sub_id)) = action.toggle_subtask {
                        self.toggle_subtask(id, &sub_id);
                    }
                    if let Some(tag) = action.filter_tag {
                        self.selected_tag = Some(tag);
                    }
                }
            });

        // Floating Dialogs
        dialogs::render_export_dialog(self, ctx);
        dialogs::render_import_dialog(self, ctx);
        dialogs::render_settings_dialog(self, ctx);
    }
}

impl eframe::App for TodoGuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_internal(ctx);
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run_desktop(storage_path: PathBuf) -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([850.0, 650.0])
            .with_min_inner_size([400.0, 300.0])
            .with_title("Todo"),
        ..Default::default()
    };

    eframe::run_native(
        "Todo",
        native_options,
        Box::new(move |_cc| Ok(Box::new(TodoGuiApp::new(Some(storage_path))))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gui_export_json_and_markdown() {
        let mut app = TodoGuiApp::new(None);
        app.todos.clear();
        app.todos.push(
            TodoItem::new(
                1,
                "Buy Coffee".to_string(),
                Some("Beans from roastery".to_string()),
                Priority::High,
            )
            .with_due_date(Some("2026-11-30".to_string())),
        );

        let json = app.export_to_json();
        assert!(json.contains("Buy Coffee"));
        assert!(json.contains("2026-11-30"));

        let md = app.export_to_markdown();
        assert!(md.contains("# Todo Export"));
        assert!(md.contains("- [ ] [HIGH] **Buy Coffee**"));
        assert!(md.contains("Due: 2026-11-30"));
        assert!(md.contains("Beans from roastery"));
    }

    #[test]
    fn test_gui_import_json() {
        let mut app = TodoGuiApp::new(None);
        app.todos.clear();

        let json = r#"[
            {"title": "Imported 1", "priority": "high", "completed": false, "created_at": "2026-01-01T00:00:00Z"},
            {"title": "Imported 2", "priority": "low", "completed": true, "created_at": "2026-01-01T00:00:00Z"}
        ]"#;

        let count = app.import_from_json(json).expect("Import should succeed");
        assert_eq!(count, 2);
        assert_eq!(app.todos.len(), 2);
        assert_eq!(app.todos[0].title, "Imported 1");
        assert_eq!(app.todos[1].title, "Imported 2");
        assert!(app.todos[1].completed);
    }

    #[test]
    fn test_sync_status_transitions() {
        let mut app = TodoGuiApp::new(None);
        assert_eq!(app.sync_status, SyncStatus::LocalOnly);

        app.sync_status = SyncStatus::Connected;
        assert_eq!(app.sync_status, SyncStatus::Connected);

        app.sync_status = SyncStatus::Offline { pending_count: 3 };
        assert_eq!(app.sync_status, SyncStatus::Offline { pending_count: 3 });

        app.sync_status = SyncStatus::Error("Connection refused".to_string());
        match app.sync_status {
            SyncStatus::Error(ref msg) => assert!(msg.contains("Connection refused")),
            _ => panic!("Expected SyncStatus::Error"),
        }
    }

    #[test]
    fn test_offline_mutation_queue() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("gui_todos.json");
        let mut app = TodoGuiApp::new(Some(file_path.clone()));
        app.server_url = "http://localhost:8000".to_string();
        app.todos.clear();
        app.pending_mutations.clear();

        // 1. Add task while configured with server URL
        app.new_title = "Queued Task".to_string();
        app.new_priority = Priority::High;
        app.add_todo();
        assert_eq!(app.todos.len(), 1);
        assert_eq!(app.pending_mutations.len(), 1);
        match &app.pending_mutations[0] {
            crate::models::PendingMutation::Create(create) => {
                assert_eq!(create.title, "Queued Task");
                assert_eq!(create.priority, Some(Priority::High));
            }
            _ => panic!("Expected Create mutation"),
        }

        // 2. Toggle task
        let task_id = app.todos[0].id;
        app.toggle_todo(task_id);
        assert_eq!(app.pending_mutations.len(), 2);
        match &app.pending_mutations[1] {
            crate::models::PendingMutation::Toggle { id } => assert_eq!(*id, task_id),
            _ => panic!("Expected Toggle mutation"),
        }

        // 3. Edit task
        app.start_edit(task_id);
        app.edit_title = "Updated Queued Task".to_string();
        app.save_edit();
        assert_eq!(app.pending_mutations.len(), 3);
        match &app.pending_mutations[2] {
            crate::models::PendingMutation::Update { id, update } => {
                assert_eq!(*id, task_id);
                assert_eq!(update.title, Some("Updated Queued Task".to_string()));
            }
            _ => panic!("Expected Update mutation"),
        }

        // 4. Delete task
        app.delete_todo(task_id);
        assert_eq!(app.todos.len(), 0);
        assert_eq!(app.pending_mutations.len(), 4);
        match &app.pending_mutations[3] {
            crate::models::PendingMutation::Delete { id } => assert_eq!(*id, task_id),
            _ => panic!("Expected Delete mutation"),
        }

        // 5. Verify persistence in file queue
        let reloaded_app = TodoGuiApp::new(Some(file_path));
        assert_eq!(reloaded_app.pending_mutations.len(), 4);
    }

    #[test]
    fn test_adaptive_mode_resolution() {
        let mut app = TodoGuiApp::new(None);
        assert_eq!(app.adaptive_mode, AdaptiveMode::Auto);

        // Auto mode responsive breakpoint (< 650 is MobileFocus)
        assert_eq!(app.effective_mode(500.0), AdaptiveMode::MobileFocus);
        assert_eq!(app.effective_mode(1200.0), AdaptiveMode::DesktopPlanning);

        // Explicit overrides
        app.adaptive_mode = AdaptiveMode::MobileFocus;
        assert_eq!(app.effective_mode(1400.0), AdaptiveMode::MobileFocus);

        app.adaptive_mode = AdaptiveMode::Calm;
        assert_eq!(app.effective_mode(400.0), AdaptiveMode::Calm);

        app.adaptive_mode = AdaptiveMode::DesktopPlanning;
        assert_eq!(app.effective_mode(400.0), AdaptiveMode::DesktopPlanning);
    }

    #[test]
    fn test_mobile_focus_hierarchy() {
        let mut app = TodoGuiApp::new(None);
        app.todos = vec![
            TodoItem {
                id: 1,
                uuid: None,
                title: "Low Priority".into(),
                description: None,
                completed: false,
                priority: Priority::Low,
                due_date: None,
                recurrence: None,
                estimated_minutes: None,
                created_at: Utc::now().to_rfc3339(),
                updated_at: None,
                tags: Vec::new(),
                subtasks: Vec::new(),
            },
            TodoItem {
                id: 2,
                uuid: None,
                title: "High Priority".into(),
                description: None,
                completed: false,
                priority: Priority::High,
                due_date: None,
                recurrence: None,
                estimated_minutes: None,
                created_at: Utc::now().to_rfc3339(),
                updated_at: None,
                tags: Vec::new(),
                subtasks: Vec::new(),
            },
            TodoItem {
                id: 3,
                uuid: None,
                title: "Completed High".into(),
                description: None,
                completed: true,
                priority: Priority::High,
                due_date: None,
                recurrence: None,
                estimated_minutes: None,
                created_at: Utc::now().to_rfc3339(),
                updated_at: None,
                tags: Vec::new(),
                subtasks: Vec::new(),
            },
            TodoItem {
                id: 4,
                uuid: None,
                title: "Medium Priority".into(),
                description: None,
                completed: false,
                priority: Priority::Medium,
                due_date: None,
                recurrence: None,
                estimated_minutes: None,
                created_at: Utc::now().to_rfc3339(),
                updated_at: None,
                tags: Vec::new(),
                subtasks: Vec::new(),
            },
        ];

        let mobile_list = app.mobile_focus_todos();
        assert_eq!(mobile_list.len(), 4);
        assert_eq!(mobile_list[0].id, 2); // High priority incomplete first
        assert_eq!(mobile_list[1].id, 4); // Medium priority incomplete second
        assert_eq!(mobile_list[2].id, 1); // Low priority incomplete third
        assert_eq!(mobile_list[3].id, 3); // Completed last
    }

    #[test]
    fn test_desktop_planning_priority_update() {
        let mut app = TodoGuiApp::new(None);
        app.server_url = "http://localhost:3000".to_string();
        app.todos = vec![TodoItem {
            id: 42,
            uuid: None,
            title: "Task to bump".into(),
            description: None,
            completed: false,
            priority: Priority::Medium,
            due_date: None,
            recurrence: None,
            estimated_minutes: None,
            created_at: Utc::now().to_rfc3339(),
            updated_at: None,
            tags: Vec::new(),
            subtasks: Vec::new(),
        }];

        app.update_priority(42, Priority::High);
        assert_eq!(app.todos[0].priority, Priority::High);
        assert!(app.todos[0].updated_at.is_some());
        assert_eq!(app.pending_mutations.len(), 1);
        match &app.pending_mutations[0] {
            crate::models::PendingMutation::Update { id, update } => {
                assert_eq!(*id, 42);
                assert_eq!(update.priority, Some(Priority::High));
            }
            _ => panic!("Expected update mutation"),
        }
    }

    #[test]
    fn test_calm_spotlight_indices() {
        let mut app = TodoGuiApp::new(None);
        app.todos = vec![
            TodoItem {
                id: 1,
                uuid: None,
                title: "Calm Task 1".into(),
                description: None,
                completed: false,
                priority: Priority::Medium,
                due_date: None,
                recurrence: None,
                estimated_minutes: None,
                created_at: Utc::now().to_rfc3339(),
                updated_at: None,
                tags: Vec::new(),
                subtasks: Vec::new(),
            },
            TodoItem {
                id: 2,
                uuid: None,
                title: "Calm Task 2".into(),
                description: None,
                completed: false,
                priority: Priority::Low,
                due_date: None,
                recurrence: None,
                estimated_minutes: None,
                created_at: Utc::now().to_rfc3339(),
                updated_at: None,
                tags: Vec::new(),
                subtasks: Vec::new(),
            },
        ];

        assert_eq!(app.calm_spotlight_idx, 0);
        app.calm_spotlight_idx = 5; // Simulating out-of-bounds index
        let active_count = app.todos.iter().filter(|t| !t.completed).count();
        let clamped = app.calm_spotlight_idx.min(active_count.saturating_sub(1));
        assert_eq!(clamped, 1);
    }

    #[test]
    fn test_convert_scratchpad_to_tasks() {
        let mut app = TodoGuiApp::new(None);
        app.todos.clear();
        app.planning_scratchpad = r#"
- [ ] Brainstorm features
- Implement OAuth provider [1h]
* Quick bug fix [15m]
• Draft documentation
"#
        .to_string();

        let count = app.convert_scratchpad_to_tasks();
        assert_eq!(count, 4);
        assert_eq!(app.todos.len(), 4);
        assert_eq!(app.todos[0].title, "Brainstorm features");
        assert_eq!(app.todos[1].title, "Implement OAuth provider [1h]");
        assert_eq!(app.todos[1].estimated_minutes, Some(60));
        assert_eq!(app.todos[2].title, "Quick bug fix [15m]");
        assert_eq!(app.todos[2].estimated_minutes, Some(15));
        assert_eq!(app.todos[3].title, "Draft documentation");
        assert!(app.planning_scratchpad.is_empty());
    }

    #[test]
    fn test_time_budget_gui_filtering() {
        let mut app = TodoGuiApp::new(None);
        app.todos = vec![
            TodoItem {
                id: 1,
                uuid: None,
                title: "Quick win".into(),
                description: None,
                completed: false,
                priority: Priority::Low, // default 15m
                due_date: None,
                recurrence: None,
                estimated_minutes: None,
                created_at: Utc::now().to_rfc3339(),
                updated_at: None,
                tags: Vec::new(),
                subtasks: Vec::new(),
            },
            TodoItem {
                id: 2,
                uuid: None,
                title: "Deep work task".into(),
                description: None,
                completed: false,
                priority: Priority::High, // default 60m
                due_date: None,
                recurrence: None,
                estimated_minutes: None,
                created_at: Utc::now().to_rfc3339(),
                updated_at: None,
                tags: Vec::new(),
                subtasks: Vec::new(),
            },
        ];

        // All filter
        app.time_budget = TimeBudgetFilter::All;
        assert_eq!(app.filtered_todos().len(), 2);

        // QuickWins filter
        app.time_budget = TimeBudgetFilter::QuickWins;
        let quick = app.filtered_todos();
        assert_eq!(quick.len(), 1);
        assert_eq!(quick[0].id, 1);

        // DeepWork filter
        app.time_budget = TimeBudgetFilter::DeepWork;
        let deep = app.filtered_todos();
        assert_eq!(deep.len(), 1);
        assert_eq!(deep[0].id, 2);
    }

    #[test]
    fn test_scratchpad_persistence() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("todos.json");

        let mut app = TodoGuiApp::new(Some(file_path.clone()));
        app.planning_scratchpad = "Session Goal: Finish Stage 3 testing".to_string();
        app.save_todos();

        let reloaded = TodoGuiApp::new(Some(file_path));
        assert_eq!(
            reloaded.planning_scratchpad,
            "Session Goal: Finish Stage 3 testing"
        );
    }

    #[test]
    fn test_gui_tag_filtering_and_subtasks() {
        use crate::models::Subtask;

        let mut app = TodoGuiApp::new(None);
        app.server_url = "http://localhost:3000".to_string();
        app.todos = vec![
            TodoItem {
                id: 1,
                uuid: None,
                title: "Frontend work".into(),
                description: None,
                completed: false,
                priority: Priority::Medium,
                due_date: None,
                recurrence: None,
                estimated_minutes: None,
                created_at: Utc::now().to_rfc3339(),
                updated_at: None,
                tags: vec!["gui".to_string(), "rust".to_string()],
                subtasks: vec![Subtask::new("Button"), Subtask::new("Modal")],
            },
            TodoItem {
                id: 2,
                uuid: None,
                title: "Backend work".into(),
                description: None,
                completed: false,
                priority: Priority::High,
                due_date: None,
                recurrence: None,
                estimated_minutes: None,
                created_at: Utc::now().to_rfc3339(),
                updated_at: None,
                tags: vec!["api".to_string(), "rust".to_string()],
                subtasks: Vec::new(),
            },
        ];

        let tags = app.all_tags();
        assert_eq!(tags, vec!["api", "gui", "rust"]);

        app.selected_tag = Some("gui".to_string());
        let filtered = app.filtered_todos();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);

        app.selected_tag = Some("rust".to_string());
        assert_eq!(app.filtered_todos().len(), 2);

        // Subtask toggle
        assert!(!app.todos[0].subtasks[0].completed);
        let sub_id = app.todos[0].subtasks[0].id.clone();
        app.toggle_subtask(1, &sub_id);
        assert!(app.todos[0].subtasks[0].completed);
        assert_eq!(app.pending_mutations.len(), 1);
    }

    #[test]
    fn test_gui_differential_sync_merge_lww() {
        use crate::models::SyncResponse;

        let mut app = TodoGuiApp::new(None);
        app.todos = vec![
            TodoItem {
                id: 1,
                uuid: Some("uuid-to-delete".to_string()),
                title: "To be deleted".into(),
                description: None,
                completed: false,
                priority: Priority::Low,
                due_date: None,
                recurrence: None,
                estimated_minutes: None,
                created_at: "2026-11-16T10:00:00Z".to_string(),
                updated_at: None,
                tags: Vec::new(),
                subtasks: Vec::new(),
            },
            TodoItem {
                id: 2,
                uuid: Some("uuid-to-update".to_string()),
                title: "Local version".into(),
                description: None,
                completed: false,
                priority: Priority::Medium,
                due_date: None,
                recurrence: None,
                estimated_minutes: None,
                created_at: "2026-11-16T10:00:00Z".to_string(),
                updated_at: Some("2026-11-16T11:00:00Z".to_string()),
                tags: Vec::new(),
                subtasks: Vec::new(),
            },
        ];

        let sync_resp = SyncResponse {
            deleted_uuids: vec!["uuid-to-delete".to_string()],
            updated_todos: vec![
                // Remote update with newer timestamp (12:00:00Z > 11:00:00Z)
                TodoItem {
                    id: 99,
                    uuid: Some("uuid-to-update".to_string()),
                    title: "Remote updated version".into(),
                    description: Some("New desc".into()),
                    completed: true,
                    priority: Priority::High,
                    due_date: None,
                    recurrence: None,
                    estimated_minutes: None,
                    created_at: "2026-11-16T10:00:00Z".to_string(),
                    updated_at: Some("2026-11-16T12:00:00Z".to_string()),
                    tags: vec!["synced".to_string()],
                    subtasks: Vec::new(),
                },
                // Brand new remote item
                TodoItem {
                    id: 100,
                    uuid: Some("uuid-new-remote".to_string()),
                    title: "Brand new remote task".into(),
                    description: None,
                    completed: false,
                    priority: Priority::Low,
                    due_date: None,
                    recurrence: None,
                    estimated_minutes: None,
                    created_at: "2026-11-16T12:00:00Z".to_string(),
                    updated_at: None,
                    tags: Vec::new(),
                    subtasks: Vec::new(),
                },
            ],
            server_timestamp: "2026-11-16T12:05:00Z".to_string(),
        };

        app.merge_differential_sync(sync_resp);

        assert_eq!(
            app.last_sync_timestamp.as_deref(),
            Some("2026-11-16T12:05:00Z")
        );
        assert_eq!(app.todos.len(), 2);
        // "uuid-to-delete" was deleted
        assert!(!app
            .todos
            .iter()
            .any(|t| t.uuid.as_deref() == Some("uuid-to-delete")));
        // "uuid-to-update" was updated
        let updated = app
            .todos
            .iter()
            .find(|t| t.uuid.as_deref() == Some("uuid-to-update"))
            .unwrap();
        assert_eq!(updated.title, "Remote updated version");
        assert!(updated.completed);
        assert_eq!(updated.priority, Priority::High);
        // "uuid-new-remote" was added
        assert!(app
            .todos
            .iter()
            .any(|t| t.uuid.as_deref() == Some("uuid-new-remote")));
    }

    #[test]
    fn test_gui_undo_redo_lifecycle() {
        let mut app = TodoGuiApp::new(None);
        app.todos.clear();
        app.undo_stack.clear();
        app.redo_stack.clear();

        assert!(!app.can_undo());
        assert!(!app.can_redo());

        // 1. Add task
        app.new_title = "Undoable Task".to_string();
        app.add_todo();
        assert_eq!(app.todos.len(), 1);
        assert!(app.can_undo());
        assert!(!app.can_redo());

        // 2. Undo addition
        app.undo();
        assert_eq!(app.todos.len(), 0);
        assert!(!app.can_undo());
        assert!(app.can_redo());

        // 3. Redo addition
        app.redo();
        assert_eq!(app.todos.len(), 1);
        assert_eq!(app.todos[0].title, "Undoable Task");
        assert!(app.can_undo());
        assert!(!app.can_redo());

        // 4. Toggle task and undo
        let task_id = app.todos[0].id;
        assert!(!app.todos[0].completed);
        app.toggle_todo(task_id);
        assert!(app.todos[0].completed);
        app.undo();
        assert!(!app.todos[0].completed);

        // 5. Delete task and undo
        app.delete_todo(task_id);
        assert_eq!(app.todos.len(), 0);
        app.undo();
        assert_eq!(app.todos.len(), 1);
        assert_eq!(app.todos[0].id, task_id);
    }

    #[test]
    fn test_gui_undo_redo_capacity_limit() {
        let mut app = TodoGuiApp::new(None);
        app.todos.clear();
        app.undo_stack.clear();

        for i in 0..60 {
            app.new_title = format!("Task {i}");
            app.add_todo();
        }

        // Maximum snapshots capped at 50
        assert_eq!(app.undo_stack.len(), 50);
        assert_eq!(app.todos.len(), 60);

        // Can undo 50 times
        for _ in 0..50 {
            app.undo();
        }
        assert!(!app.can_undo());
        assert_eq!(app.redo_stack.len(), 50);
    }
}
