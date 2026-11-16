use crate::gui::TodoGuiApp;
use crate::models::{Priority, TodoItem};
use eframe::egui;

#[derive(Default)]
pub struct ViewAction {
    pub toggled_id: Option<i64>,
    pub delete_id: Option<i64>,
    pub edit_id: Option<i64>,
    pub priority_change: Option<(i64, Priority)>,
    pub next_spotlight_idx: Option<usize>,
    pub toggle_subtask: Option<(i64, String)>,
    pub filter_tag: Option<String>,
}

pub fn render_desktop_planning_banner(app: &mut TodoGuiApp, ui: &mut egui::Ui) {
    egui::Frame::new()
        .fill(app.theme.top_panel_bg())
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(12, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("🖥️ Planning Overview")
                        .strong()
                        .size(12.0)
                        .color(egui::Color32::from_rgb(137, 180, 250)),
                );
                ui.separator();
                let high_count = app
                    .todos
                    .iter()
                    .filter(|t| !t.completed && t.priority == Priority::High)
                    .count();
                let med_count = app
                    .todos
                    .iter()
                    .filter(|t| !t.completed && t.priority == Priority::Medium)
                    .count();
                let low_count = app
                    .todos
                    .iter()
                    .filter(|t| !t.completed && t.priority == Priority::Low)
                    .count();
                let overdue_count = app
                    .todos
                    .iter()
                    .filter(|t| {
                        !t.completed && t.due_status() == Some(crate::models::DueStatus::Overdue)
                    })
                    .count();
                ui.label(
                    egui::RichText::new(format!(
                        "Workstreams: [🔴 High: {high_count}] [🟡 Med: {med_count}] [🟢 Low: {low_count}]"
                    ))
                    .size(11.0)
                    .color(egui::Color32::from_rgb(166, 173, 200)),
                );
                if overdue_count > 0 {
                    ui.label(
                        egui::RichText::new(format!("[⚠️ Overdue: {overdue_count}]"))
                            .color(egui::Color32::from_rgb(243, 139, 168))
                            .size(11.0)
                            .strong(),
                    );
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let btn_label = if app.desktop_board_view {
                        "📋 Switch to List View"
                    } else {
                        "📊 Switch to Planning Board"
                    };
                    if ui.button(btn_label).clicked() {
                        app.desktop_board_view = !app.desktop_board_view;
                    }
                    let scratchpad_btn = if app.show_planning_scratchpad {
                        "📝 Hide Scratchpad"
                    } else {
                        "📝 Open Scratchpad"
                    };
                    if ui.button(scratchpad_btn).clicked() {
                        app.show_planning_scratchpad = !app.show_planning_scratchpad;
                    }
                });
            });
        });
    ui.add_space(8.0);

    if app.show_planning_scratchpad {
        let mut convert_now = false;
        egui::Frame::new()
            .fill(app.theme.card_bg(false))
            .corner_radius(egui::CornerRadius::same(8))
            .stroke(egui::Stroke::new(1.0_f32, app.theme.card_border(false)))
            .inner_margin(egui::Margin::symmetric(14, 10))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("📝 Session Planning Scratchpad")
                            .strong()
                            .size(12.5)
                            .color(egui::Color32::from_rgb(137, 180, 250)),
                    );
                    ui.separator();
                    ui.label(
                        egui::RichText::new(
                            "Draft session thoughts, break down goals, or stage priorities",
                        )
                        .size(11.0)
                        .color(egui::Color32::from_rgb(166, 173, 200)),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .button("✕")
                            .on_hover_text("Close scratchpad")
                            .clicked()
                        {
                            app.show_planning_scratchpad = false;
                        }
                        if ui
                            .button("➕ Convert Lines to Tasks")
                            .on_hover_text("Convert bullet points or lines into actual tasks")
                            .clicked()
                        {
                            convert_now = true;
                        }
                        if !app.planning_scratchpad.is_empty() && ui.button("🗑️ Clear").clicked() {
                            app.planning_scratchpad.clear();
                            app.save_todos();
                        }
                    });
                });
                ui.add_space(6.0);
                let response = ui.add(
                    egui::TextEdit::multiline(&mut app.planning_scratchpad)
                        .hint_text("• Write session goals or task breakdown\n• Quick fix [15m]\n• Implement feature [1h]")
                        .desired_rows(4)
                        .desired_width(f32::INFINITY),
                );
                if response.changed() {
                    app.save_todos();
                }
            });
        if convert_now {
            app.convert_scratchpad_to_tasks();
        }
        ui.add_space(8.0);
    }
}

pub fn render_mobile_focus_banner(app: &mut TodoGuiApp, ui: &mut egui::Ui) {
    egui::Frame::new()
        .fill(app.theme.top_panel_bg())
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(12, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("📱 Mobile Focus")
                        .strong()
                        .size(12.0)
                        .color(egui::Color32::from_rgb(166, 227, 161)),
                );
                ui.separator();
                ui.label(
                    egui::RichText::new("Touch-optimized • Priority action hierarchy")
                        .size(11.0)
                        .color(egui::Color32::from_rgb(166, 173, 200)),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let meta_btn = if app.mobile_compact_meta {
                        "🔍 Expand Details"
                    } else {
                        "⚡ Compact View"
                    };
                    if ui.button(meta_btn).clicked() {
                        app.mobile_compact_meta = !app.mobile_compact_meta;
                    }
                });
            });
        });
    ui.add_space(8.0);
}

pub fn render_calm_banner(app: &TodoGuiApp, ui: &mut egui::Ui) {
    egui::Frame::new()
        .fill(app.theme.top_panel_bg())
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(12, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("🌿 Calm Focus")
                        .strong()
                        .size(12.0)
                        .color(egui::Color32::from_rgb(148, 226, 213)),
                );
                ui.separator();
                ui.label(
                    egui::RichText::new("Distraction-free • Step-by-step progress")
                        .size(11.0)
                        .color(egui::Color32::from_rgb(166, 173, 200)),
                );
            });
        });
    ui.add_space(8.0);
}

pub fn render_desktop_board(
    app: &TodoGuiApp,
    ui: &mut egui::Ui,
    filtered: &[&TodoItem],
    action: &mut ViewAction,
) {
    let high_todos: Vec<&TodoItem> = filtered
        .iter()
        .copied()
        .filter(|t| !t.completed && t.priority == Priority::High)
        .collect();
    let med_todos: Vec<&TodoItem> = filtered
        .iter()
        .copied()
        .filter(|t| !t.completed && t.priority == Priority::Medium)
        .collect();
    let low_todos: Vec<&TodoItem> = filtered
        .iter()
        .copied()
        .filter(|t| !t.completed && t.priority == Priority::Low)
        .collect();
    let done_todos: Vec<&TodoItem> = filtered.iter().copied().filter(|t| t.completed).collect();

    let mut render_board_col = |col_ui: &mut egui::Ui,
                                title: &str,
                                title_color: egui::Color32,
                                items: &[&TodoItem],
                                demote_to: Option<Priority>,
                                promote_to: Option<Priority>,
                                is_done_col: bool| {
        egui::Frame::new()
            .fill(app.theme.top_panel_bg())
            .corner_radius(egui::CornerRadius::same(6))
            .inner_margin(egui::Margin::symmetric(10, 8))
            .show(col_ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(title)
                            .strong()
                            .size(13.0)
                            .color(title_color),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("{}", items.len()))
                                .size(11.0)
                                .color(egui::Color32::from_rgb(166, 173, 200)),
                        );
                    });
                });
            });
        col_ui.add_space(6.0);

        egui::ScrollArea::vertical()
            .id_salt(title)
            .auto_shrink([false; 2])
            .show(col_ui, |ui| {
                for todo in items {
                    let card_bg = app.theme.card_bg(todo.completed);
                    egui::Frame::new()
                        .fill(card_bg)
                        .corner_radius(egui::CornerRadius::same(6))
                        .stroke(egui::Stroke::new(
                            1.0_f32,
                            app.theme.card_border(todo.completed),
                        ))
                        .inner_margin(egui::Margin::symmetric(10, 8))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let mut comp = todo.completed;
                                if ui.checkbox(&mut comp, "").changed() {
                                    action.toggled_id = Some(todo.id);
                                }
                                let title_text = if todo.completed {
                                    egui::RichText::new(&todo.title)
                                        .size(13.5)
                                        .strikethrough()
                                        .color(egui::Color32::from_rgb(108, 112, 134))
                                } else {
                                    egui::RichText::new(&todo.title)
                                        .size(13.5)
                                        .strong()
                                        .color(egui::Color32::from_rgb(205, 214, 244))
                                };
                                ui.label(title_text);
                            });

                            if let Some(ref desc) = todo.description {
                                if !desc.is_empty() {
                                    ui.add_space(2.0);
                                    ui.label(
                                        egui::RichText::new(desc)
                                            .size(11.0)
                                            .color(egui::Color32::from_rgb(147, 153, 178)),
                                    );
                                }
                            }

                            if let Some(status) = todo.due_status() {
                                let (due_txt, due_color) = match status {
                                    crate::models::DueStatus::Overdue => (
                                        format!(
                                            "⚠️ Due {}",
                                            todo.due_date.as_deref().unwrap_or("")
                                        ),
                                        egui::Color32::from_rgb(243, 139, 168),
                                    ),
                                    crate::models::DueStatus::DueToday => (
                                        "⏳ Today".to_string(),
                                        egui::Color32::from_rgb(250, 179, 135),
                                    ),
                                    crate::models::DueStatus::Upcoming => (
                                        format!("📅 {}", todo.due_date.as_deref().unwrap_or("")),
                                        egui::Color32::from_rgb(137, 180, 250),
                                    ),
                                };
                                ui.label(
                                    egui::RichText::new(due_txt)
                                        .size(10.5)
                                        .color(due_color)
                                        .strong(),
                                );
                            }

                            let est_mins = todo.effective_estimated_minutes();
                            let est_txt = if est_mins >= 60 {
                                format!("⏱️ {}h", est_mins / 60)
                            } else {
                                format!("⏱️ {}m", est_mins)
                            };
                            ui.label(
                                egui::RichText::new(est_txt)
                                    .size(10.5)
                                    .color(egui::Color32::from_rgb(180, 190, 254)),
                            );

                            if !todo.tags.is_empty() {
                                ui.horizontal_wrapped(|ui| {
                                    for tag in &todo.tags {
                                        if ui
                                            .link(
                                                egui::RichText::new(format!("#{tag}"))
                                                    .size(10.0)
                                                    .color(egui::Color32::from_rgb(137, 180, 250)),
                                            )
                                            .clicked()
                                        {
                                            action.filter_tag = Some(tag.clone());
                                        }
                                    }
                                });
                            }

                            if let Some((done, total)) = todo.subtask_progress() {
                                ui.label(
                                    egui::RichText::new(format!("☑️ {done}/{total}"))
                                        .size(10.0)
                                        .color(egui::Color32::from_rgb(166, 227, 161)),
                                );
                            }

                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                if !is_done_col {
                                    if let Some(target) = promote_to {
                                        if ui
                                            .small_button("▲")
                                            .on_hover_text("Increase priority")
                                            .clicked()
                                        {
                                            action.priority_change = Some((todo.id, target));
                                        }
                                    }
                                    if let Some(target) = demote_to {
                                        if ui
                                            .small_button("▼")
                                            .on_hover_text("Decrease priority")
                                            .clicked()
                                        {
                                            action.priority_change = Some((todo.id, target));
                                        }
                                    }
                                }
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui.small_button("🗑️").clicked() {
                                            action.delete_id = Some(todo.id);
                                        }
                                        if ui.small_button("✏️").clicked() {
                                            action.edit_id = Some(todo.id);
                                        }
                                    },
                                );
                            });
                        });
                    ui.add_space(6.0);
                }
            });
    };

    ui.columns(4, |cols| {
        render_board_col(
            &mut cols[0],
            "🔴 High Priority",
            egui::Color32::from_rgb(243, 139, 168),
            &high_todos,
            Some(Priority::Medium),
            None,
            false,
        );
        render_board_col(
            &mut cols[1],
            "🟡 Medium Priority",
            egui::Color32::from_rgb(250, 179, 135),
            &med_todos,
            Some(Priority::Low),
            Some(Priority::High),
            false,
        );
        render_board_col(
            &mut cols[2],
            "🟢 Low Priority",
            egui::Color32::from_rgb(166, 227, 161),
            &low_todos,
            None,
            Some(Priority::Medium),
            false,
        );
        render_board_col(
            &mut cols[3],
            "✅ Completed",
            egui::Color32::from_rgb(147, 153, 178),
            &done_todos,
            None,
            None,
            true,
        );
    });
}

pub fn render_mobile_focus_view(app: &TodoGuiApp, ui: &mut egui::Ui, action: &mut ViewAction) {
    let mobile_items = app.mobile_focus_todos();
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            for todo in mobile_items {
                let card_bg = app.theme.card_bg(todo.completed);

                egui::Frame::new()
                    .fill(card_bg)
                    .corner_radius(egui::CornerRadius::same(10))
                    .stroke(egui::Stroke::new(
                        1.0_f32,
                        app.theme.card_border(todo.completed),
                    ))
                    .inner_margin(egui::Margin::symmetric(14, 12))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Enlarged Touch Target Checkbox
                            let mut comp = todo.completed;
                            if ui
                                .add_sized([26.0, 26.0], egui::Checkbox::new(&mut comp, ""))
                                .changed()
                            {
                                action.toggled_id = Some(todo.id);
                            }

                            // Priority Chip
                            let (prio_text, prio_color) = match todo.priority {
                                Priority::High => ("HIGH", egui::Color32::from_rgb(243, 139, 168)),
                                Priority::Medium => ("MED", egui::Color32::from_rgb(250, 179, 135)),
                                Priority::Low => ("LOW", egui::Color32::from_rgb(166, 227, 161)),
                            };
                            ui.label(
                                egui::RichText::new(format!("[{prio_text}]"))
                                    .color(prio_color)
                                    .size(11.0)
                                    .strong(),
                            );

                            // Task Details
                            ui.vertical(|ui| {
                                let mut title_text = egui::RichText::new(&todo.title).size(15.5);
                                if todo.completed {
                                    title_text = title_text
                                        .strikethrough()
                                        .color(egui::Color32::from_rgb(108, 112, 134));
                                } else {
                                    title_text = title_text
                                        .strong()
                                        .color(egui::Color32::from_rgb(205, 214, 244));
                                }
                                ui.label(title_text);

                                if !app.mobile_compact_meta {
                                    if let Some(ref desc) = todo.description {
                                        if !desc.is_empty() {
                                            ui.label(
                                                egui::RichText::new(desc)
                                                    .size(12.0)
                                                    .color(egui::Color32::from_rgb(166, 173, 200)),
                                            );
                                        }
                                    }
                                }

                                if let Some(status) = todo.due_status() {
                                    let (due_txt, due_color) = match status {
                                        crate::models::DueStatus::Overdue => (
                                            format!(
                                                "⚠️ Overdue ({})",
                                                todo.due_date.as_deref().unwrap_or("")
                                            ),
                                            egui::Color32::from_rgb(243, 139, 168),
                                        ),
                                        crate::models::DueStatus::DueToday => (
                                            "⏳ Due Today".to_string(),
                                            egui::Color32::from_rgb(250, 179, 135),
                                        ),
                                        crate::models::DueStatus::Upcoming => (
                                            format!(
                                                "📅 {}",
                                                todo.due_date.as_deref().unwrap_or("")
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
                                            .color(egui::Color32::from_rgb(203, 166, 247))
                                            .strong(),
                                    );
                                }

                                let est_mins = todo.effective_estimated_minutes();
                                let est_txt = if est_mins >= 60 {
                                    format!("⏱️ {}h", est_mins / 60)
                                } else {
                                    format!("⏱️ {}m", est_mins)
                                };
                                ui.label(
                                    egui::RichText::new(est_txt)
                                        .size(11.0)
                                        .color(egui::Color32::from_rgb(180, 190, 254)),
                                );

                                if !todo.tags.is_empty() {
                                    ui.horizontal_wrapped(|ui| {
                                        for tag in &todo.tags {
                                            if ui
                                                .link(
                                                    egui::RichText::new(format!("#{tag}"))
                                                        .size(10.5)
                                                        .color(egui::Color32::from_rgb(
                                                            137, 180, 250,
                                                        )),
                                                )
                                                .clicked()
                                            {
                                                action.filter_tag = Some(tag.clone());
                                            }
                                        }
                                    });
                                }

                                if let Some((done, total)) = todo.subtask_progress() {
                                    ui.label(
                                        egui::RichText::new(format!("☑️ Subtasks: {done}/{total}"))
                                            .size(11.0)
                                            .color(egui::Color32::from_rgb(166, 227, 161)),
                                    );
                                }
                            });

                            // Right Touch Action Buttons
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .add_sized([38.0, 32.0], egui::Button::new("🗑️"))
                                        .clicked()
                                    {
                                        action.delete_id = Some(todo.id);
                                    }
                                    if ui
                                        .add_sized([38.0, 32.0], egui::Button::new("✏️"))
                                        .clicked()
                                    {
                                        action.edit_id = Some(todo.id);
                                    }
                                    let quick_action = if todo.completed { "↩" } else { "✓" };
                                    if ui
                                        .add_sized([44.0, 32.0], egui::Button::new(quick_action))
                                        .clicked()
                                    {
                                        action.toggled_id = Some(todo.id);
                                    }
                                },
                            );
                        });
                    });
                ui.add_space(6.0);
            }
        });
}

pub fn render_calm_view(
    app: &TodoGuiApp,
    ui: &mut egui::Ui,
    filtered: &[&TodoItem],
    action: &mut ViewAction,
) {
    let active_todos: Vec<&TodoItem> = filtered.iter().copied().filter(|t| !t.completed).collect();

    if !active_todos.is_empty() {
        let spotlight_idx = app
            .calm_spotlight_idx
            .min(active_todos.len().saturating_sub(1));
        let current_spotlight = active_todos[spotlight_idx];

        egui::Frame::new()
            .fill(egui::Color32::from_rgb(30, 40, 44))
            .corner_radius(egui::CornerRadius::same(10))
            .stroke(egui::Stroke::new(
                1.5_f32,
                egui::Color32::from_rgb(148, 226, 213),
            ))
            .inner_margin(egui::Margin::symmetric(20, 16))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "🌿 Step-by-Step Focus ({} of {})",
                            spotlight_idx + 1,
                            active_todos.len()
                        ))
                        .strong()
                        .size(13.0)
                        .color(egui::Color32::from_rgb(148, 226, 213)),
                    );
                });
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(&current_spotlight.title)
                        .size(18.0)
                        .strong()
                        .color(egui::Color32::from_rgb(205, 214, 244)),
                );
                if let Some(ref desc) = current_spotlight.description {
                    if !desc.is_empty() {
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(desc)
                                .size(13.0)
                                .color(egui::Color32::from_rgb(166, 173, 200)),
                        );
                    }
                }
                if let Some(ref due) = current_spotlight.due_date {
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(format!("📅 Due: {due}"))
                            .size(12.0)
                            .color(egui::Color32::from_rgb(148, 226, 213)),
                    );
                }
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if ui
                        .add_sized([160.0, 32.0], egui::Button::new("✨ Complete & Next"))
                        .clicked()
                    {
                        action.toggled_id = Some(current_spotlight.id);
                    }
                    if spotlight_idx > 0 && ui.button("⏮️ Previous").clicked() {
                        action.next_spotlight_idx = Some(spotlight_idx - 1);
                    }
                    if spotlight_idx + 1 < active_todos.len() && ui.button("⏭️ Next").clicked()
                    {
                        action.next_spotlight_idx = Some(spotlight_idx + 1);
                    }
                });
            });
        ui.add_space(12.0);
    }

    // Serene Checklist
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            for todo in filtered {
                let card_bg = app.theme.card_bg(todo.completed);

                egui::Frame::new()
                    .fill(card_bg)
                    .corner_radius(egui::CornerRadius::same(8))
                    .stroke(egui::Stroke::new(
                        1.0_f32,
                        app.theme.card_border(todo.completed),
                    ))
                    .inner_margin(egui::Margin::symmetric(14, 10))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let mut comp = todo.completed;
                            if ui.checkbox(&mut comp, "").changed() {
                                action.toggled_id = Some(todo.id);
                            }

                            // Subdued Priority Chip in Calm mode
                            let (prio_text, prio_color) = match todo.priority {
                                Priority::High => ("High", egui::Color32::from_rgb(243, 139, 168)),
                                Priority::Medium => ("Med", egui::Color32::from_rgb(250, 179, 135)),
                                Priority::Low => ("Low", egui::Color32::from_rgb(166, 227, 161)),
                            };
                            ui.label(
                                egui::RichText::new(format!("[{prio_text}]"))
                                    .color(prio_color)
                                    .size(11.0),
                            );

                            ui.vertical(|ui| {
                                let mut title_text = egui::RichText::new(&todo.title).size(14.5);
                                if todo.completed {
                                    title_text = title_text
                                        .strikethrough()
                                        .color(egui::Color32::from_rgb(108, 112, 134));
                                } else {
                                    title_text =
                                        title_text.color(egui::Color32::from_rgb(205, 214, 244));
                                }
                                ui.label(title_text);

                                if let Some(ref desc) = todo.description {
                                    if !desc.is_empty() {
                                        ui.label(
                                            egui::RichText::new(desc)
                                                .size(12.0)
                                                .color(egui::Color32::from_rgb(147, 153, 178)),
                                        );
                                    }
                                }

                                if let Some(ref due) = todo.due_date {
                                    ui.label(
                                        egui::RichText::new(format!("📅 {due}"))
                                            .size(11.0)
                                            .color(egui::Color32::from_rgb(148, 226, 213)),
                                    );
                                }
                            });

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.button("🗑️").clicked() {
                                        action.delete_id = Some(todo.id);
                                    }
                                    if ui.button("✏️").clicked() {
                                        action.edit_id = Some(todo.id);
                                    }
                                },
                            );
                        });
                    });
                ui.add_space(6.0);
            }
        });
}

pub fn render_standard_list_view(
    app: &TodoGuiApp,
    ui: &mut egui::Ui,
    filtered: &[&TodoItem],
    action: &mut ViewAction,
) {
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            for todo in filtered {
                let card_bg = app.theme.card_bg(todo.completed);

                egui::Frame::new()
                    .fill(card_bg)
                    .corner_radius(egui::CornerRadius::same(8))
                    .stroke(egui::Stroke::new(
                        1.0_f32,
                        app.theme.card_border(todo.completed),
                    ))
                    .inner_margin(egui::Margin::symmetric(14, 10))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Checkbox
                            let mut comp = todo.completed;
                            if ui.checkbox(&mut comp, "").changed() {
                                action.toggled_id = Some(todo.id);
                            }

                            // Priority Chip
                            let (prio_text, prio_color) = match todo.priority {
                                Priority::High => ("HIGH", egui::Color32::from_rgb(243, 139, 168)),
                                Priority::Medium => ("MED", egui::Color32::from_rgb(250, 179, 135)),
                                Priority::Low => ("LOW", egui::Color32::from_rgb(166, 227, 161)),
                            };
                            ui.label(
                                egui::RichText::new(format!("[{prio_text}]"))
                                    .color(prio_color)
                                    .size(11.0)
                                    .strong(),
                            );

                            // Task Content
                            ui.vertical(|ui| {
                                let mut title_text = egui::RichText::new(&todo.title).size(15.0);
                                if todo.completed {
                                    title_text = title_text
                                        .strikethrough()
                                        .color(egui::Color32::from_rgb(108, 112, 134));
                                } else {
                                    title_text = title_text
                                        .strong()
                                        .color(egui::Color32::from_rgb(205, 214, 244));
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
                                            egui::RichText::new(desc).size(12.0).color(desc_color),
                                        );
                                    }
                                }

                                if let Some(status) = todo.due_status() {
                                    let (due_txt, due_color) = match status {
                                        crate::models::DueStatus::Overdue => (
                                            format!(
                                                "⚠️ Overdue ({})",
                                                todo.due_date.as_deref().unwrap_or("")
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
                                                todo.due_date.as_deref().unwrap_or("")
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
                                            .color(egui::Color32::from_rgb(203, 166, 247))
                                            .strong(),
                                    );
                                }

                                let est_mins = todo.effective_estimated_minutes();
                                let est_txt = if est_mins >= 60 {
                                    format!("⏱️ {}h", est_mins / 60)
                                } else {
                                    format!("⏱️ {}m", est_mins)
                                };
                                ui.label(
                                    egui::RichText::new(est_txt)
                                        .size(11.0)
                                        .color(egui::Color32::from_rgb(180, 190, 254)),
                                );

                                if !todo.tags.is_empty() {
                                    ui.horizontal_wrapped(|ui| {
                                        for tag in &todo.tags {
                                            if ui
                                                .link(
                                                    egui::RichText::new(format!("#{tag}"))
                                                        .size(10.5)
                                                        .color(egui::Color32::from_rgb(
                                                            137, 180, 250,
                                                        )),
                                                )
                                                .clicked()
                                            {
                                                action.filter_tag = Some(tag.clone());
                                            }
                                        }
                                    });
                                }

                                if let Some((done, total)) = todo.subtask_progress() {
                                    ui.label(
                                        egui::RichText::new(format!("☑️ Subtasks: {done}/{total}"))
                                            .size(11.0)
                                            .color(egui::Color32::from_rgb(166, 227, 161)),
                                    );
                                }
                            });

                            // Action Buttons (Right-aligned)
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.button("🗑️").clicked() {
                                        action.delete_id = Some(todo.id);
                                    }
                                    if ui.button("✏️").clicked() {
                                        action.edit_id = Some(todo.id);
                                    }
                                },
                            );
                        });
                    });
                ui.add_space(6.0);
            }
        });
}
