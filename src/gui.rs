use crate::models::{Priority, TodoItem};
use chrono::Utc;
use eframe::egui;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterStatus {
    All,
    Active,
    Completed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GuiTheme {
    #[default]
    Macchiato,
    Latte,
    HighContrast,
}

impl GuiTheme {
    pub fn is_dark(&self) -> bool {
        match self {
            GuiTheme::Macchiato | GuiTheme::HighContrast => true,
            GuiTheme::Latte => false,
        }
    }

    pub fn bg(&self) -> egui::Color32 {
        match self {
            GuiTheme::Macchiato => egui::Color32::from_rgb(24, 24, 37), // Base
            GuiTheme::Latte => egui::Color32::from_rgb(239, 241, 245),  // Base
            GuiTheme::HighContrast => egui::Color32::from_rgb(0, 0, 0),
        }
    }

    pub fn top_panel_bg(&self) -> egui::Color32 {
        match self {
            GuiTheme::Macchiato => egui::Color32::from_rgb(30, 30, 46), // Mantle
            GuiTheme::Latte => egui::Color32::from_rgb(230, 233, 239),  // Mantle
            GuiTheme::HighContrast => egui::Color32::from_rgb(18, 18, 18),
        }
    }

    pub fn card_bg(&self, completed: bool) -> egui::Color32 {
        match self {
            GuiTheme::Macchiato => {
                if completed {
                    egui::Color32::from_rgb(20, 20, 30)
                } else {
                    egui::Color32::from_rgb(30, 30, 46)
                }
            }
            GuiTheme::Latte => {
                if completed {
                    egui::Color32::from_rgb(235, 237, 242)
                } else {
                    egui::Color32::from_rgb(255, 255, 255)
                }
            }
            GuiTheme::HighContrast => {
                if completed {
                    egui::Color32::from_rgb(15, 15, 15)
                } else {
                    egui::Color32::from_rgb(24, 24, 24)
                }
            }
        }
    }

    pub fn card_border(&self, completed: bool) -> egui::Color32 {
        match self {
            GuiTheme::Macchiato => {
                if completed {
                    egui::Color32::from_rgb(49, 50, 68)
                } else {
                    egui::Color32::from_rgb(69, 71, 90)
                }
            }
            GuiTheme::Latte => {
                if completed {
                    egui::Color32::from_rgb(204, 208, 218)
                } else {
                    egui::Color32::from_rgb(188, 192, 204)
                }
            }
            GuiTheme::HighContrast => {
                if completed {
                    egui::Color32::from_rgb(100, 100, 100)
                } else {
                    egui::Color32::from_rgb(255, 255, 255)
                }
            }
        }
    }

    pub fn primary_text(&self) -> egui::Color32 {
        match self {
            GuiTheme::Macchiato => egui::Color32::from_rgb(205, 214, 244),
            GuiTheme::Latte => egui::Color32::from_rgb(76, 79, 105),
            GuiTheme::HighContrast => egui::Color32::from_rgb(255, 255, 255),
        }
    }
}

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncStatus {
    LocalOnly,
    Syncing,
    Connected,
    Offline { pending_count: usize },
    Error(String),
}

pub enum SyncResult {
    FetchedTodos(Vec<TodoItem>),
    MutationDrained {
        success: bool,
        error: Option<String>,
    },
    SyncError(String),
    SuccessMessage(String),
}

impl crate::models::PendingMutation {
    pub fn to_request(&self, base_url: &str) -> ehttp::Request {
        let base = base_url.trim_end_matches('/');
        match self {
            crate::models::PendingMutation::Create(create) => {
                let body = serde_json::to_vec(create).unwrap_or_default();
                ehttp::Request::new(
                    ehttp::Method::POST,
                    format!("{base}/api/todos"),
                    &[("Accept", "*/*"), ("Content-Type", "application/json")],
                )
                .with_body(body)
            }
            crate::models::PendingMutation::Update { id, update } => {
                let body = serde_json::to_vec(update).unwrap_or_default();
                ehttp::Request::new(
                    ehttp::Method::PUT,
                    format!("{base}/api/todos/{id}"),
                    &[("Accept", "*/*"), ("Content-Type", "application/json")],
                )
                .with_body(body)
            }
            crate::models::PendingMutation::Delete { id } => ehttp::Request::new(
                ehttp::Method::DELETE,
                format!("{base}/api/todos/{id}"),
                &[("Accept", "*/*")],
            ),
            crate::models::PendingMutation::Toggle { id } => ehttp::Request::new(
                ehttp::Method::PATCH,
                format!("{base}/api/todos/{id}/toggle"),
                &[("Accept", "*/*")],
            ),
        }
    }
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
                    if let Ok(Some(url)) = storage.get_item("taskmaster_server_url") {
                        if !url.trim().is_empty() {
                            self.server_url = url;
                            self.sync_status = SyncStatus::Connected;
                        }
                    }
                    if let Ok(Some(data)) = storage.get_item("taskmaster_pending_mutations") {
                        if let Ok(mutations) =
                            serde_json::from_str::<Vec<crate::models::PendingMutation>>(&data)
                        {
                            self.pending_mutations = mutations;
                        }
                    }
                    if let Ok(Some(data)) = storage.get_item("taskmaster_todos") {
                        if let Ok(items) = serde_json::from_str::<Vec<TodoItem>>(&data) {
                            self.todos = items;
                            return;
                        }
                    }
                }
            }
        }

        // Default sample todos if empty
        if self.todos.is_empty() {
            self.todos = vec![
                TodoItem::new(
                    1,
                    "Welcome to TaskMaster Pure Rust GUI!".to_string(),
                    Some(
                        "Built with 100% Rust and egui. Zero HTML, zero CSS, zero JavaScript."
                            .to_string(),
                    ),
                    Priority::High,
                ),
                TodoItem::new(
                    2,
                    "Explore keyboard shortcuts and filters".to_string(),
                    Some(
                        "Filter by status, search in real-time, and manage your tasks.".to_string(),
                    ),
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
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    let _ = storage.set_item("taskmaster_server_url", &self.server_url);
                    if let Ok(json) = serde_json::to_string(&self.todos) {
                        let _ = storage.set_item("taskmaster_todos", &json);
                    }
                    if let Ok(json) = serde_json::to_string(&self.pending_mutations) {
                        let _ = storage.set_item("taskmaster_pending_mutations", &json);
                    }
                }
            }
        }
    }

    pub fn add_todo(&mut self) {
        let trimmed = self.new_title.trim();
        if trimmed.is_empty() {
            return;
        }

        let new_id = self.todos.iter().map(|t| t.id).max().unwrap_or(0) + 1;
        let desc = if self.new_description.trim().is_empty() {
            None
        } else {
            Some(self.new_description.trim().to_string())
        };

        let mut item = TodoItem::new(new_id, trimmed.to_string(), desc, self.new_priority);
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
            let trimmed = self.edit_title.trim();
            if !trimmed.is_empty() {
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
                    todo.title = trimmed.to_string();
                    todo.description = desc.clone();
                    todo.priority = self.edit_priority;
                    todo.due_date = due.clone();
                    todo.recurrence = self.edit_recurrence;
                    todo.updated_at = Some(Utc::now().to_rfc3339());

                    if !self.server_url.trim().is_empty() {
                        self.pending_mutations
                            .push(crate::models::PendingMutation::Update {
                                id,
                                update: crate::models::TodoUpdate {
                                    title: Some(trimmed.to_string()),
                                    description: desc,
                                    completed: None,
                                    priority: Some(self.edit_priority),
                                    due_date: due,
                                    recurrence: self.edit_recurrence,
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
        self.todos.retain(|t| !t.completed);
        self.save_todos();
    }

    pub fn export_to_json(&self) -> String {
        serde_json::to_string_pretty(&self.todos).unwrap_or_default()
    }

    pub fn export_to_markdown(&self) -> String {
        let mut out = String::from("# TaskMaster Export\n\n");
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

    pub fn filtered_todos(&self) -> Vec<&TodoItem> {
        self.todos
            .iter()
            .filter(|todo| match self.filter {
                FilterStatus::All => true,
                FilterStatus::Active => !todo.completed,
                FilterStatus::Completed => todo.completed,
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
                    in_title || in_desc
                }
            })
            .collect()
    }
}

impl eframe::App for TodoGuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
                        egui::RichText::new("🦀 TaskMaster")
                            .color(egui::Color32::from_rgb(203, 166, 247)) // Mauve
                            .strong()
                            .size(22.0),
                    );
                    ui.label(
                        egui::RichText::new("• 100% Pure Rust GUI")
                            .color(egui::Color32::from_rgb(166, 173, 200))
                            .size(13.0),
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
                });
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
                    ui.label(
                        egui::RichText::new(format!(
                            "Active: {active}  |  Completed: {completed}  |  Total: {total}    •    [Ctrl+N] New Task  |  [/] Focus Search  |  [Esc] Close Dialogs"
                        ))
                        .size(11.0)
                        .color(egui::Color32::from_rgb(147, 153, 178)),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new("Zero HTML • Zero CSS • Zero JavaScript")
                                .size(11.0)
                                .color(egui::Color32::from_rgb(166, 227, 161)),
                        );
                    });
                });
            });

        // Main Content Area
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(self.theme.bg())
                    .inner_margin(egui::Margin::same(16)),
            )
            .show(ctx, |ui| {
                // Add Task Dialog / Card
                if self.show_add_dialog {
                    egui::Frame::new()
                        .fill(self.theme.card_bg(false))
                        .stroke(egui::Stroke::new(
                            1.0_f32,
                            egui::Color32::from_rgb(137, 180, 250),
                        ))
                        .corner_radius(egui::CornerRadius::same(8))
                        .inner_margin(egui::Margin::same(14))
                        .show(ui, |ui| {
                            ui.heading(
                                egui::RichText::new("Add New Task")
                                    .size(16.0)
                                    .color(egui::Color32::from_rgb(137, 180, 250)),
                            );
                            ui.add_space(6.0);

                            ui.horizontal(|ui| {
                                ui.label("Title:");
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.new_title)
                                        .hint_text("Task title...")
                                        .desired_width(300.0),
                                );
                            });

                            ui.horizontal(|ui| {
                                ui.label("Details:");
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.new_description)
                                        .hint_text("Optional description...")
                                        .desired_width(300.0),
                                );
                            });

                            ui.horizontal(|ui| {
                                ui.label("Priority:");
                                egui::ComboBox::from_id_salt("new_priority_combo")
                                    .selected_text(match self.new_priority {
                                        Priority::Low => "Low",
                                        Priority::Medium => "Medium",
                                        Priority::High => "High",
                                    })
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            &mut self.new_priority,
                                            Priority::Low,
                                            "Low",
                                        );
                                        ui.selectable_value(
                                            &mut self.new_priority,
                                            Priority::Medium,
                                            "Medium",
                                        );
                                        ui.selectable_value(
                                            &mut self.new_priority,
                                            Priority::High,
                                            "High",
                                        );
                                    });
                            });

                            ui.horizontal(|ui| {
                                ui.label("Due Date:");
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.new_due_date)
                                        .hint_text("YYYY-MM-DD")
                                        .desired_width(160.0),
                                );
                            });

                            ui.horizontal(|ui| {
                                ui.label("Recurrence:");
                                let cur_rec_text = match &self.new_recurrence {
                                    None => "None",
                                    Some(crate::models::RecurrencePattern::Daily) => "Daily",
                                    Some(crate::models::RecurrencePattern::Weekly) => "Weekly",
                                    Some(crate::models::RecurrencePattern::Monthly) => "Monthly",
                                    Some(crate::models::RecurrencePattern::CustomDays(_)) => {
                                        "Custom"
                                    }
                                };
                                egui::ComboBox::from_id_salt("new_recurrence_combo")
                                    .selected_text(cur_rec_text)
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(&mut self.new_recurrence, None, "None");
                                        ui.selectable_value(
                                            &mut self.new_recurrence,
                                            Some(crate::models::RecurrencePattern::Daily),
                                            "Daily",
                                        );
                                        ui.selectable_value(
                                            &mut self.new_recurrence,
                                            Some(crate::models::RecurrencePattern::Weekly),
                                            "Weekly",
                                        );
                                        ui.selectable_value(
                                            &mut self.new_recurrence,
                                            Some(crate::models::RecurrencePattern::Monthly),
                                            "Monthly",
                                        );
                                    });
                            });

                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                if ui
                                    .button(
                                        egui::RichText::new("Save Task")
                                            .color(egui::Color32::from_rgb(17, 17, 27))
                                            .strong(),
                                    )
                                    .clicked()
                                {
                                    self.add_todo();
                                }
                                if ui.button("Cancel").clicked() {
                                    self.show_add_dialog = false;
                                }
                            });
                        });
                    ui.add_space(14.0);
                }

                // Edit Task Dialog / Card
                if let Some(edit_id) = self.editing_id {
                    egui::Frame::new()
                        .fill(egui::Color32::from_rgb(30, 30, 46))
                        .stroke(egui::Stroke::new(
                            1.0_f32,
                            egui::Color32::from_rgb(250, 179, 135),
                        ))
                        .corner_radius(egui::CornerRadius::same(8))
                        .inner_margin(egui::Margin::same(14))
                        .show(ui, |ui| {
                            ui.heading(
                                egui::RichText::new(format!("Edit Task #{edit_id}"))
                                    .size(16.0)
                                    .color(egui::Color32::from_rgb(250, 179, 135)),
                            );
                            ui.add_space(6.0);

                            ui.horizontal(|ui| {
                                ui.label("Title:");
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.edit_title)
                                        .desired_width(300.0),
                                );
                            });

                            ui.horizontal(|ui| {
                                ui.label("Details:");
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.edit_description)
                                        .desired_width(300.0),
                                );
                            });

                            ui.horizontal(|ui| {
                                ui.label("Priority:");
                                egui::ComboBox::from_id_salt("edit_priority_combo")
                                    .selected_text(match self.edit_priority {
                                        Priority::Low => "Low",
                                        Priority::Medium => "Medium",
                                        Priority::High => "High",
                                    })
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            &mut self.edit_priority,
                                            Priority::Low,
                                            "Low",
                                        );
                                        ui.selectable_value(
                                            &mut self.edit_priority,
                                            Priority::Medium,
                                            "Medium",
                                        );
                                        ui.selectable_value(
                                            &mut self.edit_priority,
                                            Priority::High,
                                            "High",
                                        );
                                    });
                            });

                            ui.horizontal(|ui| {
                                ui.label("Due Date:");
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.edit_due_date)
                                        .hint_text("YYYY-MM-DD")
                                        .desired_width(160.0),
                                );
                            });

                            ui.horizontal(|ui| {
                                ui.label("Recurrence:");
                                let cur_rec_text = match &self.edit_recurrence {
                                    None => "None",
                                    Some(crate::models::RecurrencePattern::Daily) => "Daily",
                                    Some(crate::models::RecurrencePattern::Weekly) => "Weekly",
                                    Some(crate::models::RecurrencePattern::Monthly) => "Monthly",
                                    Some(crate::models::RecurrencePattern::CustomDays(_)) => {
                                        "Custom"
                                    }
                                };
                                egui::ComboBox::from_id_salt("edit_recurrence_combo")
                                    .selected_text(cur_rec_text)
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            &mut self.edit_recurrence,
                                            None,
                                            "None",
                                        );
                                        ui.selectable_value(
                                            &mut self.edit_recurrence,
                                            Some(crate::models::RecurrencePattern::Daily),
                                            "Daily",
                                        );
                                        ui.selectable_value(
                                            &mut self.edit_recurrence,
                                            Some(crate::models::RecurrencePattern::Weekly),
                                            "Weekly",
                                        );
                                        ui.selectable_value(
                                            &mut self.edit_recurrence,
                                            Some(crate::models::RecurrencePattern::Monthly),
                                            "Monthly",
                                        );
                                    });
                            });

                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                if ui
                                    .button(
                                        egui::RichText::new("Update Task")
                                            .color(egui::Color32::from_rgb(17, 17, 27))
                                            .strong(),
                                    )
                                    .clicked()
                                {
                                    self.save_edit();
                                }
                                if ui.button("Cancel").clicked() {
                                    self.editing_id = None;
                                }
                            });
                        });
                    ui.add_space(14.0);
                }

                // Task List
                let filtered = self.filtered_todos();
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
                    let mut toggled_id = None;
                    let mut delete_id = None;
                    let mut edit_id = None;

                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            for todo in filtered {
                                let card_bg = self.theme.card_bg(todo.completed);

                                egui::Frame::new()
                                    .fill(card_bg)
                                    .corner_radius(egui::CornerRadius::same(8))
                                    .stroke(egui::Stroke::new(
                                        1.0_f32,
                                        self.theme.card_border(todo.completed),
                                    ))
                                    .inner_margin(egui::Margin::symmetric(14, 10))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            // Checkbox
                                            let mut comp = todo.completed;
                                            if ui.checkbox(&mut comp, "").changed() {
                                                toggled_id = Some(todo.id);
                                            }

                                            // Priority Chip
                                            let (prio_text, prio_color) = match todo.priority {
                                                Priority::High => {
                                                    ("HIGH", egui::Color32::from_rgb(243, 139, 168))
                                                }
                                                Priority::Medium => {
                                                    ("MED", egui::Color32::from_rgb(250, 179, 135))
                                                }
                                                Priority::Low => {
                                                    ("LOW", egui::Color32::from_rgb(166, 227, 161))
                                                }
                                            };
                                            ui.label(
                                                egui::RichText::new(format!("[{prio_text}]"))
                                                    .color(prio_color)
                                                    .size(11.0)
                                                    .strong(),
                                            );

                                            // Task Content
                                            ui.vertical(|ui| {
                                                let mut title_text =
                                                    egui::RichText::new(&todo.title).size(15.0);
                                                if todo.completed {
                                                    title_text = title_text.strikethrough().color(
                                                        egui::Color32::from_rgb(108, 112, 134),
                                                    );
                                                } else {
                                                    title_text = title_text.strong().color(
                                                        egui::Color32::from_rgb(205, 214, 244),
                                                    );
                                                }
                                                ui.label(title_text);

                                                if let Some(ref desc) = todo.description {
                                                    if !desc.is_empty() {
                                                        let desc_color = if todo.completed {
                                                            egui::Color32::from_rgb(88, 91, 112)
                                                        } else {
                                                            egui::Color32::from_rgb(166, 173, 200)
                                                        };
                                                        ui.label(
                                                            egui::RichText::new(desc)
                                                                .size(12.0)
                                                                .color(desc_color),
                                                        );
                                                    }
                                                }

                                                if let Some(status) = todo.due_status() {
                                                    let (due_txt, due_color) = match status {
                                                        crate::models::DueStatus::Overdue => (
                                                            format!(
                                                                "⚠️ Overdue ({})",
                                                                todo.due_date
                                                                    .as_deref()
                                                                    .unwrap_or("")
                                                            ),
                                                            egui::Color32::from_rgb(243, 139, 168),
                                                        ),
                                                        crate::models::DueStatus::DueToday => (
                                                            "⏳ Due Today".to_string(),
                                                            egui::Color32::from_rgb(250, 179, 135),
                                                        ),
                                                        crate::models::DueStatus::Upcoming => (
                                                            format!(
                                                                "📅 Due: {}",
                                                                todo.due_date
                                                                    .as_deref()
                                                                    .unwrap_or("")
                                                            ),
                                                            egui::Color32::from_rgb(137, 180, 250),
                                                        ),
                                                    };
                                                    ui.label(
                                                        egui::RichText::new(due_txt)
                                                            .size(11.0)
                                                            .color(due_color)
                                                            .strong(),
                                                    );
                                                }

                                                if let Some(ref rec) = todo.recurrence {
                                                    ui.label(
                                                        egui::RichText::new(format!("🔁 {rec}"))
                                                            .size(11.0)
                                                            .color(egui::Color32::from_rgb(
                                                                203, 166, 247,
                                                            ))
                                                            .strong(),
                                                    );
                                                }
                                            });

                                            // Action Buttons (Right-aligned)
                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    if ui.button("🗑️").clicked() {
                                                        delete_id = Some(todo.id);
                                                    }
                                                    if ui.button("✏️").clicked() {
                                                        edit_id = Some(todo.id);
                                                    }
                                                },
                                            );
                                        });
                                    });
                                ui.add_space(6.0);
                            }
                        });

                    if let Some(id) = toggled_id {
                        self.toggle_todo(id);
                    }
                    if let Some(id) = delete_id {
                        self.delete_todo(id);
                    }
                    if let Some(id) = edit_id {
                        self.start_edit(id);
                    }
                }
            });

        // Export Window Dialog
        if self.show_export_dialog {
            let mut close_export = false;
            let export_str = if self.export_markdown {
                self.export_to_markdown()
            } else {
                self.export_to_json()
            };

            egui::Window::new("📥 Export Tasks")
                .collapsible(false)
                .resizable(true)
                .default_width(500.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui
                            .selectable_label(!self.export_markdown, "JSON Format")
                            .clicked()
                        {
                            self.export_markdown = false;
                        }
                        if ui
                            .selectable_label(self.export_markdown, "Markdown Checklist")
                            .clicked()
                        {
                            self.export_markdown = true;
                        }
                    });

                    ui.add_space(8.0);
                    let mut text_display = export_str.clone();
                    ui.add(
                        egui::TextEdit::multiline(&mut text_display)
                            .desired_rows(12)
                            .desired_width(f32::INFINITY)
                            .font(egui::TextStyle::Monospace),
                    );

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("📋 Copy to Clipboard").clicked() {
                            ctx.copy_text(export_str);
                        }
                        if ui.button("Close").clicked() {
                            close_export = true;
                        }
                    });
                });

            if close_export {
                self.show_export_dialog = false;
            }
        }

        // Import Window Dialog
        if self.show_import_dialog {
            let mut close_import = false;
            let mut do_import = false;

            egui::Window::new("📤 Import Tasks")
                .collapsible(false)
                .resizable(true)
                .default_width(500.0)
                .show(ctx, |ui| {
                    ui.label("Paste JSON array of tasks below:");
                    ui.add(
                        egui::TextEdit::multiline(&mut self.import_text)
                            .desired_rows(10)
                            .desired_width(f32::INFINITY)
                            .hint_text(r#"[{"title": "Sample Task", "priority": "high"}]"#)
                            .font(egui::TextStyle::Monospace),
                    );

                    if let Some(ref msg) = self.import_status_message {
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(msg).color(egui::Color32::from_rgb(243, 139, 168)),
                        );
                    }

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("📥 Import Tasks").clicked() {
                            do_import = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close_import = true;
                        }
                    });
                });

            if do_import {
                let to_import = self.import_text.clone();
                match self.import_from_json(&to_import) {
                    Ok(_count) => {
                        self.import_status_message = None;
                        self.import_text.clear();
                        self.show_import_dialog = false;
                    }
                    Err(err) => {
                        self.import_status_message = Some(err);
                    }
                }
            }

            if close_import {
                self.show_import_dialog = false;
                self.import_status_message = None;
            }
        }

        // Backend Sync Settings Window Dialog
        if self.show_settings_dialog {
            let mut close_settings = false;
            let mut do_test_connection = false;
            let mut do_fetch = false;
            let mut do_disconnect = false;

            egui::Window::new("⚙️ Backend Sync Settings")
                .collapsible(false)
                .resizable(false)
                .default_width(450.0)
                .show(ctx, |ui| {
                    ui.label("Connect to a TaskMaster REST server (e.g. todo server running locally or on VPS):");
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label("Server URL:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.server_url)
                                .hint_text("http://localhost:8000")
                                .desired_width(280.0),
                        );
                    });

                    ui.add_space(8.0);
                    match &self.sync_status {
                        SyncStatus::Connected => {
                            ui.label(
                                egui::RichText::new("🟢 Status: Connected to backend API")
                                    .color(egui::Color32::from_rgb(166, 227, 161))
                                    .strong(),
                            );
                        }
                        SyncStatus::Syncing => {
                            ui.label(
                                egui::RichText::new("🔄 Status: Syncing with server...")
                                    .color(egui::Color32::from_rgb(250, 179, 135)),
                            );
                        }
                        SyncStatus::Offline { pending_count } => {
                            ui.label(
                                egui::RichText::new(format!(
                                    "⚠️ Status: Offline ({pending_count} pending mutations queued)"
                                ))
                                .color(egui::Color32::from_rgb(250, 179, 135))
                                .strong(),
                            );
                        }
                        SyncStatus::LocalOnly => {
                            ui.label(
                                egui::RichText::new("🟡 Status: Local Storage Only (Offline)")
                                    .color(egui::Color32::from_rgb(249, 226, 175)),
                            );
                        }
                        SyncStatus::Error(err) => {
                            ui.label(
                                egui::RichText::new(format!("🔴 Status Error: {err}"))
                                    .color(egui::Color32::from_rgb(243, 139, 168)),
                            );
                        }
                    }

                    if !self.pending_mutations.is_empty() {
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!(
                                    "📦 Change Queue: {} pending items",
                                    self.pending_mutations.len()
                                ))
                                .size(12.0)
                                .color(egui::Color32::from_rgb(203, 166, 247)),
                            );
                            if ui.button("Clear Queue").clicked() {
                                self.pending_mutations.clear();
                                self.save_todos();
                            }
                        });
                    }

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("🔍 Test Connection").clicked() {
                            do_test_connection = true;
                        }
                        if ui.button("🔄 Sync Todos Now").clicked() {
                            do_fetch = true;
                        }
                        if !self.pending_mutations.is_empty() && ui.button("⚡ Flush Queue").clicked() {
                            self.drain_next_mutation(ctx.clone());
                        }
                        if ui.button("Local Only").clicked() {
                            do_disconnect = true;
                        }
                        if ui.button("Close").clicked() {
                            close_settings = true;
                        }
                    });
                });

            if do_test_connection {
                self.test_connection(ctx.clone());
            }
            if do_fetch {
                self.fetch_from_server(ctx.clone());
            }
            if do_disconnect {
                self.server_url.clear();
                self.sync_status = SyncStatus::LocalOnly;
                self.save_todos();
            }
            if close_settings {
                self.show_settings_dialog = false;
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run_desktop(storage_path: PathBuf) -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([850.0, 650.0])
            .with_min_inner_size([400.0, 300.0])
            .with_title("TaskMaster (Pure Rust)"),
        ..Default::default()
    };

    eframe::run_native(
        "TaskMaster",
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
        assert!(md.contains("# TaskMaster Export"));
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
}
