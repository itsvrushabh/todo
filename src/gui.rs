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

pub struct TodoGuiApp {
    pub todos: Vec<TodoItem>,
    pub filter: FilterStatus,
    pub search_query: String,
    pub new_title: String,
    pub new_description: String,
    pub new_priority: Priority,
    pub show_add_dialog: bool,
    pub editing_id: Option<i64>,
    pub edit_title: String,
    pub edit_description: String,
    pub edit_priority: Priority,
    pub storage_path: Option<PathBuf>,
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
            show_add_dialog: false,
            editing_id: None,
            edit_title: String::new(),
            edit_description: String::new(),
            edit_priority: Priority::Medium,
            storage_path,
        };
        app.load_todos();
        app
    }

    pub fn load_todos(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(ref path) = self.storage_path {
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
                    if let Ok(json) = serde_json::to_string(&self.todos) {
                        let _ = storage.set_item("taskmaster_todos", &json);
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

        self.todos.push(TodoItem::new(
            new_id,
            trimmed.to_string(),
            desc,
            self.new_priority,
        ));

        self.new_title.clear();
        self.new_description.clear();
        self.new_priority = Priority::Medium;
        self.show_add_dialog = false;
        self.save_todos();
    }

    pub fn toggle_todo(&mut self, id: i64) {
        if let Some(todo) = self.todos.iter_mut().find(|t| t.id == id) {
            todo.completed = !todo.completed;
            todo.updated_at = Some(Utc::now().to_rfc3339());
            self.save_todos();
        }
    }

    pub fn delete_todo(&mut self, id: i64) {
        self.todos.retain(|t| t.id != id);
        self.save_todos();
    }

    pub fn start_edit(&mut self, id: i64) {
        if let Some(todo) = self.todos.iter().find(|t| t.id == id) {
            self.editing_id = Some(id);
            self.edit_title = todo.title.clone();
            self.edit_description = todo.description.clone().unwrap_or_default();
            self.edit_priority = todo.priority;
        }
    }

    pub fn save_edit(&mut self) {
        if let Some(id) = self.editing_id {
            let trimmed = self.edit_title.trim();
            if !trimmed.is_empty() {
                if let Some(todo) = self.todos.iter_mut().find(|t| t.id == id) {
                    todo.title = trimmed.to_string();
                    todo.description = if self.edit_description.trim().is_empty() {
                        None
                    } else {
                        Some(self.edit_description.trim().to_string())
                    };
                    todo.priority = self.edit_priority;
                    todo.updated_at = Some(Utc::now().to_rfc3339());
                    self.save_todos();
                }
            }
        }
        self.editing_id = None;
    }

    pub fn clear_completed(&mut self) {
        self.todos.retain(|t| !t.completed);
        self.save_todos();
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
        // Catppuccin Macchiato styled dark theme
        let mut visuals = egui::Visuals::dark();
        visuals.override_text_color = Some(egui::Color32::from_rgb(205, 214, 244));
        visuals.panel_fill = egui::Color32::from_rgb(24, 24, 37); // Base
        visuals.window_fill = egui::Color32::from_rgb(30, 30, 46); // Mantle
        visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(30, 30, 46);
        visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(49, 50, 68); // Surface0
        visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(69, 71, 90); // Surface1
        visuals.widgets.active.bg_fill = egui::Color32::from_rgb(137, 180, 250); // Blue
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
                    .fill(egui::Color32::from_rgb(30, 30, 46))
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

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .button(
                                egui::RichText::new("➕ New Task")
                                    .color(egui::Color32::from_rgb(17, 17, 27))
                                    .strong(),
                            )
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
                    });
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

                    ui.label("🔍");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.search_query)
                            .hint_text("Search tasks...")
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
                    .fill(egui::Color32::from_rgb(17, 17, 27))
                    .inner_margin(egui::Margin::symmetric(16, 8)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "Active: {active}  |  Completed: {completed}  |  Total: {total}"
                        ))
                        .size(12.0)
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
                    .fill(egui::Color32::from_rgb(24, 24, 37))
                    .inner_margin(egui::Margin::same(16)),
            )
            .show(ctx, |ui| {
                // Add Task Dialog / Card
                if self.show_add_dialog {
                    egui::Frame::new()
                        .fill(egui::Color32::from_rgb(30, 30, 46))
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
                                let card_bg = if todo.completed {
                                    egui::Color32::from_rgb(20, 20, 30)
                                } else {
                                    egui::Color32::from_rgb(30, 30, 46)
                                };

                                egui::Frame::new()
                                    .fill(card_bg)
                                    .corner_radius(egui::CornerRadius::same(8))
                                    .stroke(egui::Stroke::new(
                                        1.0_f32,
                                        if todo.completed {
                                            egui::Color32::from_rgb(49, 50, 68)
                                        } else {
                                            egui::Color32::from_rgb(69, 71, 90)
                                        },
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
