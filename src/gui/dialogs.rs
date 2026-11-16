use crate::gui::types::SyncStatus;
use crate::gui::TodoGuiApp;
use crate::models::Priority;
use eframe::egui;

pub fn render_add_task_panel(app: &mut TodoGuiApp, ui: &mut egui::Ui) {
    if !app.show_add_dialog {
        return;
    }

    egui::Frame::new()
        .fill(app.theme.card_bg(false))
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
                    egui::TextEdit::singleline(&mut app.new_title)
                        .hint_text("Task title...")
                        .desired_width(300.0),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Details:");
                ui.add(
                    egui::TextEdit::singleline(&mut app.new_description)
                        .hint_text("Optional description...")
                        .desired_width(300.0),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Priority:");
                egui::ComboBox::from_id_salt("new_priority_combo")
                    .selected_text(match app.new_priority {
                        Priority::Low => "Low",
                        Priority::Medium => "Medium",
                        Priority::High => "High",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut app.new_priority, Priority::Low, "Low");
                        ui.selectable_value(&mut app.new_priority, Priority::Medium, "Medium");
                        ui.selectable_value(&mut app.new_priority, Priority::High, "High");
                    });
            });

            ui.horizontal(|ui| {
                ui.label("Due Date:");
                ui.add(
                    egui::TextEdit::singleline(&mut app.new_due_date)
                        .hint_text("YYYY-MM-DD")
                        .desired_width(160.0),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Recurrence:");
                let cur_rec_text = match &app.new_recurrence {
                    None => "None",
                    Some(crate::models::RecurrencePattern::Daily) => "Daily",
                    Some(crate::models::RecurrencePattern::Weekly) => "Weekly",
                    Some(crate::models::RecurrencePattern::Monthly) => "Monthly",
                    Some(crate::models::RecurrencePattern::CustomDays(_)) => "Custom",
                };
                egui::ComboBox::from_id_salt("new_recurrence_combo")
                    .selected_text(cur_rec_text)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut app.new_recurrence, None, "None");
                        ui.selectable_value(
                            &mut app.new_recurrence,
                            Some(crate::models::RecurrencePattern::Daily),
                            "Daily",
                        );
                        ui.selectable_value(
                            &mut app.new_recurrence,
                            Some(crate::models::RecurrencePattern::Weekly),
                            "Weekly",
                        );
                        ui.selectable_value(
                            &mut app.new_recurrence,
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
                    app.add_todo();
                }
                if ui.button("Cancel").clicked() {
                    app.show_add_dialog = false;
                }
            });
        });
    ui.add_space(14.0);
}

pub fn render_edit_task_panel(app: &mut TodoGuiApp, ui: &mut egui::Ui) {
    let edit_id = match app.editing_id {
        Some(id) => id,
        None => return,
    };

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
                ui.add(egui::TextEdit::singleline(&mut app.edit_title).desired_width(300.0));
            });

            ui.horizontal(|ui| {
                ui.label("Details:");
                ui.add(egui::TextEdit::singleline(&mut app.edit_description).desired_width(300.0));
            });

            ui.horizontal(|ui| {
                ui.label("Priority:");
                egui::ComboBox::from_id_salt("edit_priority_combo")
                    .selected_text(match app.edit_priority {
                        Priority::Low => "Low",
                        Priority::Medium => "Medium",
                        Priority::High => "High",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut app.edit_priority, Priority::Low, "Low");
                        ui.selectable_value(&mut app.edit_priority, Priority::Medium, "Medium");
                        ui.selectable_value(&mut app.edit_priority, Priority::High, "High");
                    });
            });

            ui.horizontal(|ui| {
                ui.label("Due Date:");
                ui.add(
                    egui::TextEdit::singleline(&mut app.edit_due_date)
                        .hint_text("YYYY-MM-DD")
                        .desired_width(160.0),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Recurrence:");
                let cur_rec_text = match &app.edit_recurrence {
                    None => "None",
                    Some(crate::models::RecurrencePattern::Daily) => "Daily",
                    Some(crate::models::RecurrencePattern::Weekly) => "Weekly",
                    Some(crate::models::RecurrencePattern::Monthly) => "Monthly",
                    Some(crate::models::RecurrencePattern::CustomDays(_)) => "Custom",
                };
                egui::ComboBox::from_id_salt("edit_recurrence_combo")
                    .selected_text(cur_rec_text)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut app.edit_recurrence, None, "None");
                        ui.selectable_value(
                            &mut app.edit_recurrence,
                            Some(crate::models::RecurrencePattern::Daily),
                            "Daily",
                        );
                        ui.selectable_value(
                            &mut app.edit_recurrence,
                            Some(crate::models::RecurrencePattern::Weekly),
                            "Weekly",
                        );
                        ui.selectable_value(
                            &mut app.edit_recurrence,
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
                    app.save_edit();
                }
                if ui.button("Cancel").clicked() {
                    app.editing_id = None;
                }
            });
        });
    ui.add_space(14.0);
}

pub fn render_export_dialog(app: &mut TodoGuiApp, ctx: &egui::Context) {
    if !app.show_export_dialog {
        return;
    }

    let mut close_export = false;
    let export_str = if app.export_markdown {
        app.export_to_markdown()
    } else {
        app.export_to_json()
    };

    egui::Window::new("📥 Export Tasks")
        .collapsible(false)
        .resizable(true)
        .default_width(500.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(!app.export_markdown, "JSON Format")
                    .clicked()
                {
                    app.export_markdown = false;
                }
                if ui
                    .selectable_label(app.export_markdown, "Markdown Checklist")
                    .clicked()
                {
                    app.export_markdown = true;
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
        app.show_export_dialog = false;
    }
}

pub fn render_import_dialog(app: &mut TodoGuiApp, ctx: &egui::Context) {
    if !app.show_import_dialog {
        return;
    }

    let mut close_import = false;
    let mut do_import = false;

    egui::Window::new("📤 Import Tasks")
        .collapsible(false)
        .resizable(true)
        .default_width(500.0)
        .show(ctx, |ui| {
            ui.label("Paste JSON array of tasks below:");
            ui.add(
                egui::TextEdit::multiline(&mut app.import_text)
                    .desired_rows(10)
                    .desired_width(f32::INFINITY)
                    .hint_text(r#"[{"title": "Sample Task", "priority": "high"}]"#)
                    .font(egui::TextStyle::Monospace),
            );

            if let Some(ref msg) = app.import_status_message {
                ui.add_space(4.0);
                ui.label(egui::RichText::new(msg).color(egui::Color32::from_rgb(243, 139, 168)));
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
        let to_import = app.import_text.clone();
        match app.import_from_json(&to_import) {
            Ok(_count) => {
                app.import_status_message = None;
                app.import_text.clear();
                app.show_import_dialog = false;
            }
            Err(err) => {
                app.import_status_message = Some(err);
            }
        }
    }

    if close_import {
        app.show_import_dialog = false;
        app.import_status_message = None;
    }
}

pub fn render_settings_dialog(app: &mut TodoGuiApp, ctx: &egui::Context) {
    if !app.show_settings_dialog {
        return;
    }

    let mut close_settings = false;
    let mut do_test_connection = false;
    let mut do_fetch = false;
    let mut do_disconnect = false;

    egui::Window::new("⚙️ Backend Sync Settings")
        .collapsible(false)
        .resizable(false)
        .default_width(450.0)
        .show(ctx, |ui| {
            ui.label("Connect to a REST server (e.g. todo server running locally or on VPS):");
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label("Server URL:");
                ui.add(
                    egui::TextEdit::singleline(&mut app.server_url)
                        .hint_text("http://localhost:8000")
                        .desired_width(280.0),
                );
            });

            ui.add_space(8.0);
            match &app.sync_status {
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

            if !app.pending_mutations.is_empty() {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "📦 Change Queue: {} pending items",
                            app.pending_mutations.len()
                        ))
                        .size(12.0)
                        .color(egui::Color32::from_rgb(203, 166, 247)),
                    );
                    if ui.button("Clear Queue").clicked() {
                        app.pending_mutations.clear();
                        app.save_todos();
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
                if !app.pending_mutations.is_empty() && ui.button("⚡ Flush Queue").clicked() {
                    app.drain_next_mutation(ctx.clone());
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
        app.test_connection(ctx.clone());
    }
    if do_fetch {
        app.fetch_from_server(ctx.clone());
    }
    if do_disconnect {
        app.server_url.clear();
        app.sync_status = SyncStatus::LocalOnly;
        app.save_todos();
    }
    if close_settings {
        app.show_settings_dialog = false;
    }
}
