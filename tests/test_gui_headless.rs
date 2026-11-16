use todo::gui::egui;
use todo::gui::theme::GuiTheme;
use todo::gui::types::{AdaptiveMode, FilterStatus};
use todo::gui::TodoGuiApp;
use todo::models::{Priority, TimeBudgetFilter};

#[test]
fn test_headless_frame_lifecycle() {
    let mut app = TodoGuiApp::new(None);
    let ctx = egui::Context::default();

    // 1. Initial frame simulation
    let output = ctx.run(egui::RawInput::default(), |ctx| {
        app.update_headless(ctx);
    });

    assert!(
        !output.shapes.is_empty(),
        "Headless frame should generate render shapes"
    );
    assert_eq!(app.todos.len(), 2, "Default sample tasks should be present");
}

#[test]
fn test_headless_layout_and_theme_cycles() {
    let mut app = TodoGuiApp::new(None);
    let ctx = egui::Context::default();

    // 1. Cycle all themes and ensure no panic during rendering
    for theme in [GuiTheme::Macchiato, GuiTheme::Latte, GuiTheme::HighContrast] {
        app.theme = theme;
        let output = ctx.run(egui::RawInput::default(), |ctx| {
            app.update_headless(ctx);
        });
        assert!(!output.shapes.is_empty());
    }

    // 2. Cycle all adaptive modes
    for mode in [
        AdaptiveMode::DesktopPlanning,
        AdaptiveMode::MobileFocus,
        AdaptiveMode::Calm,
    ] {
        app.adaptive_mode = mode;
        let output = ctx.run(egui::RawInput::default(), |ctx| {
            app.update_headless(ctx);
        });
        assert!(!output.shapes.is_empty());
    }
}

#[test]
fn test_headless_task_actions_and_filtering() {
    let mut app = TodoGuiApp::new(None);
    let ctx = egui::Context::default();
    app.todos.clear();

    // 1. Add tasks
    app.new_title = "Headless Test Task #ui #rust".to_string();
    app.new_description = "Simulating offscreen rendering".to_string();
    app.new_priority = Priority::High;
    app.add_todo();

    let output = ctx.run(egui::RawInput::default(), |ctx| {
        app.update_headless(ctx);
    });
    assert!(!output.shapes.is_empty());
    assert_eq!(app.todos.len(), 1);
    assert!(app.todos[0].tags.contains(&"ui".to_string()));

    // 2. Filter states
    app.filter = FilterStatus::Active;
    assert_eq!(app.filtered_todos().len(), 1);

    app.filter = FilterStatus::Completed;
    assert_eq!(app.filtered_todos().len(), 0);

    app.time_budget = TimeBudgetFilter::DeepWork;
    assert_eq!(app.filtered_todos().len(), 0); // High priority incomplete is active, but filter is Completed

    app.filter = FilterStatus::All;
    assert_eq!(app.filtered_todos().len(), 1);

    // 3. Toggle completion
    app.toggle_todo(app.todos[0].id);
    assert!(app.todos[0].completed);

    // 4. Scratchpad conversion
    app.planning_scratchpad = "- Convert headless scratchpad [30m]".to_string();
    let count = app.convert_scratchpad_to_tasks();
    assert_eq!(count, 1);
    assert_eq!(app.todos.len(), 2);

    let output = ctx.run(egui::RawInput::default(), |ctx| {
        app.update_headless(ctx);
    });
    assert!(!output.shapes.is_empty());
}

#[test]
fn test_headless_undo_redo_and_dialogs() {
    let mut app = TodoGuiApp::new(None);
    let ctx = egui::Context::default();
    app.todos.clear();

    app.new_title = "Action to undo".to_string();
    app.add_todo();
    assert_eq!(app.todos.len(), 1);

    // Dialog state
    app.show_add_dialog = true;
    app.show_export_dialog = true;
    app.show_import_dialog = true;
    app.show_settings_dialog = true;

    let output = ctx.run(egui::RawInput::default(), |ctx| {
        app.update_headless(ctx);
    });
    assert!(!output.shapes.is_empty());

    // Undo action
    app.undo();
    assert_eq!(app.todos.len(), 0);

    // Redo action
    app.redo();
    assert_eq!(app.todos.len(), 1);
    assert_eq!(app.todos[0].title, "Action to undo");

    // Export / Import roundtrip
    let json = app.export_to_json();
    let md = app.export_to_markdown();
    assert!(json.contains("Action to undo"));
    assert!(md.contains("Action to undo"));

    let imported = app.import_from_json(&json).unwrap();
    assert_eq!(imported, 1);
    assert_eq!(app.todos.len(), 2);
}
