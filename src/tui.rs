use crate::models::{Priority, TodoCreate, TodoItem, TodoStats};
use crate::storage::TodoStorage;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Wrap,
    },
    Frame, Terminal,
};
use std::io;
use std::sync::Arc;
use std::time::Duration;

#[derive(PartialEq, Eq)]
enum FilterMode {
    All,
    Active,
    Completed,
}

enum InputMode {
    Normal,
    Searching,
    AddingTitle,
    AddingDesc,
    ConfirmClear,
}

pub struct TuiApp {
    storage: Arc<dyn TodoStorage>,
    todos: Vec<TodoItem>,
    stats: TodoStats,
    list_state: ListState,
    filter: FilterMode,
    search_query: String,
    input_mode: InputMode,
    new_title: String,
    new_desc: String,
    new_priority: Priority,
    error_message: Option<String>,
    should_quit: bool,
}

impl TuiApp {
    pub fn new(storage: Arc<dyn TodoStorage>) -> Self {
        let mut app = Self {
            storage,
            todos: Vec::new(),
            stats: TodoStats {
                total: 0,
                completed: 0,
                active: 0,
                completion_percentage: 0.0,
            },
            list_state: ListState::default(),
            filter: FilterMode::All,
            search_query: String::new(),
            input_mode: InputMode::Normal,
            new_title: String::new(),
            new_desc: String::new(),
            new_priority: Priority::Medium,
            error_message: None,
            should_quit: false,
        };
        app.reload_data();
        if !app.todos.is_empty() {
            app.list_state.select(Some(0));
        }
        app
    }

    fn reload_data(&mut self) {
        let status_filter = match self.filter {
            FilterMode::All => None,
            FilterMode::Active => Some("active"),
            FilterMode::Completed => Some("completed"),
        };
        let search_filter = if self.search_query.is_empty() {
            None
        } else {
            Some(self.search_query.as_str())
        };

        if let Ok(items) = self
            .storage
            .get_all(status_filter, search_filter, None, None)
        {
            self.todos = items;
        }
        if let Ok(s) = self.storage.get_stats() {
            self.stats = s;
        }

        // Clamp selection
        if self.todos.is_empty() {
            self.list_state.select(None);
        } else if let Some(selected) = self.list_state.selected() {
            if selected >= self.todos.len() {
                self.list_state.select(Some(self.todos.len() - 1));
            }
        } else {
            self.list_state.select(Some(0));
        }
    }

    fn next(&mut self) {
        if self.todos.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.todos.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn previous(&mut self) {
        if self.todos.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.todos.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn toggle_selected(&mut self) {
        if let Some(i) = self.list_state.selected() {
            if let Some(item) = self.todos.get(i) {
                let id = item.id;
                let _ = self.storage.toggle(id);
                self.reload_data();
            }
        }
    }

    fn delete_selected(&mut self) {
        if let Some(i) = self.list_state.selected() {
            if let Some(item) = self.todos.get(i) {
                let id = item.id;
                let _ = self.storage.delete(id);
                self.reload_data();
            }
        }
    }

    fn submit_add(&mut self) {
        let title = self.new_title.trim().to_string();
        if !title.is_empty() {
            let desc = if self.new_desc.trim().is_empty() {
                None
            } else {
                Some(self.new_desc.trim().to_string())
            };

            let auto_tags = crate::models::TodoItem::extract_tags(&title);
            let _ = self.storage.create(TodoCreate {
                title,
                description: desc,
                priority: Some(self.new_priority),
                due_date: None,
                recurrence: None,
                uuid: None,
                tags: auto_tags,
                subtasks: Vec::new(),
            });
            self.new_title.clear();
            self.new_desc.clear();
            self.new_priority = Priority::Medium;
            self.input_mode = InputMode::Normal;
            self.reload_data();
            // Select newest
            if !self.todos.is_empty() {
                self.list_state.select(Some(self.todos.len() - 1));
            }
        } else {
            self.error_message = Some("Title cannot be empty".to_string());
        }
    }
}

pub fn run_tui(storage: Arc<dyn TodoStorage>) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = TuiApp::new(storage);

    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("TUI Error: {err:?}");
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut TuiApp,
) -> io::Result<()> {
    loop {
        terminal
            .draw(|f| ui(f, app))
            .map_err(|e| io::Error::other(e.to_string()))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != crossterm::event::KeyEventKind::Press {
                    continue;
                }

                match &app.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            if !app.search_query.is_empty() {
                                app.search_query.clear();
                                app.reload_data();
                            } else {
                                app.should_quit = true;
                            }
                        }
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.should_quit = true;
                        }
                        KeyCode::Char('j') | KeyCode::Down => app.next(),
                        KeyCode::Char('k') | KeyCode::Up => app.previous(),
                        KeyCode::Enter | KeyCode::Char(' ') => app.toggle_selected(),
                        KeyCode::Char('a') => {
                            app.input_mode = InputMode::AddingTitle;
                            app.new_title.clear();
                            app.new_desc.clear();
                            app.error_message = None;
                        }
                        KeyCode::Char('d') | KeyCode::Char('x') => app.delete_selected(),
                        KeyCode::Char('c') => {
                            app.input_mode = InputMode::ConfirmClear;
                        }
                        KeyCode::Char('/') => {
                            app.input_mode = InputMode::Searching;
                        }
                        KeyCode::Tab | KeyCode::Right => {
                            app.filter = match app.filter {
                                FilterMode::All => FilterMode::Active,
                                FilterMode::Active => FilterMode::Completed,
                                FilterMode::Completed => FilterMode::All,
                            };
                            app.reload_data();
                        }
                        KeyCode::Left => {
                            app.filter = match app.filter {
                                FilterMode::All => FilterMode::Completed,
                                FilterMode::Active => FilterMode::All,
                                FilterMode::Completed => FilterMode::Active,
                            };
                            app.reload_data();
                        }
                        KeyCode::Char('1') => {
                            app.filter = FilterMode::All;
                            app.reload_data();
                        }
                        KeyCode::Char('2') => {
                            app.filter = FilterMode::Active;
                            app.reload_data();
                        }
                        KeyCode::Char('3') => {
                            app.filter = FilterMode::Completed;
                            app.reload_data();
                        }
                        KeyCode::Char('r') => {
                            app.reload_data();
                        }
                        _ => {}
                    },
                    InputMode::Searching => match key.code {
                        KeyCode::Enter | KeyCode::Esc => {
                            app.input_mode = InputMode::Normal;
                        }
                        KeyCode::Backspace => {
                            app.search_query.pop();
                            app.reload_data();
                        }
                        KeyCode::Char(c) => {
                            app.search_query.push(c);
                            app.reload_data();
                        }
                        _ => {}
                    },
                    InputMode::AddingTitle => match key.code {
                        KeyCode::Esc => {
                            app.input_mode = InputMode::Normal;
                        }
                        KeyCode::Enter => {
                            app.input_mode = InputMode::AddingDesc;
                        }
                        KeyCode::Tab => {
                            app.new_priority = match app.new_priority {
                                Priority::Low => Priority::Medium,
                                Priority::Medium => Priority::High,
                                Priority::High => Priority::Low,
                            };
                        }
                        KeyCode::Backspace => {
                            app.new_title.pop();
                        }
                        KeyCode::Char(c) => {
                            app.new_title.push(c);
                        }
                        _ => {}
                    },
                    InputMode::AddingDesc => match key.code {
                        KeyCode::Esc => {
                            app.input_mode = InputMode::Normal;
                        }
                        KeyCode::Enter => {
                            app.submit_add();
                        }
                        KeyCode::Tab => {
                            app.new_priority = match app.new_priority {
                                Priority::Low => Priority::Medium,
                                Priority::Medium => Priority::High,
                                Priority::High => Priority::Low,
                            };
                        }
                        KeyCode::Backspace => {
                            app.new_desc.pop();
                        }
                        KeyCode::Char(c) => {
                            app.new_desc.push(c);
                        }
                        _ => {}
                    },
                    InputMode::ConfirmClear => match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                            let _ = app.storage.clear_completed();
                            app.input_mode = InputMode::Normal;
                            app.reload_data();
                        }
                        _ => {
                            app.input_mode = InputMode::Normal;
                        }
                    },
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn ui(f: &mut Frame, app: &TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header & progress
            Constraint::Min(8),    // Main content (List + Details)
            Constraint::Length(3), // Footer / Keybindings / Search
        ])
        .split(f.area());

    // 1. Header with Stats & Progress
    let filter_name = match app.filter {
        FilterMode::All => "All [1]",
        FilterMode::Active => "Active [2]",
        FilterMode::Completed => "Completed [3]",
    };

    let title_line = Line::from(vec![
        Span::styled(
            " ⚡ Todo TUI ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | Filter: "),
        Span::styled(
            filter_name,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!(
            " | Total: {} | Active: {} | Done: {}",
            app.stats.total, app.stats.active, app.stats.completed
        )),
    ]);

    let progress_ratio = if app.stats.total > 0 {
        (app.stats.completed as f64 / app.stats.total as f64).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(title_line),
        )
        .gauge_style(
            Style::default()
                .fg(Color::Green)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .ratio(progress_ratio)
        .label(format!("{:.1}% Done", app.stats.completion_percentage));

    f.render_widget(gauge, chunks[0]);

    // 2. Main Body: Split into List (60%) and Details (40%)
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks[1]);

    // List of Todos
    let items: Vec<ListItem> = app
        .todos
        .iter()
        .map(|item| {
            let status_mark = if item.completed {
                Span::styled(
                    "[✓] ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled("[ ] ", Style::default().fg(Color::Gray))
            };

            let (priority_badge, p_color) = match item.priority {
                Priority::High => ("[HIGH]", Color::Red),
                Priority::Medium => ("[MED] ", Color::Yellow),
                Priority::Low => ("[LOW] ", Color::Blue),
            };

            let priority_span = Span::styled(
                format!("{priority_badge} "),
                Style::default().fg(p_color).add_modifier(Modifier::BOLD),
            );

            let id_span = Span::styled(
                format!("#{} ", item.id),
                Style::default().fg(Color::DarkGray),
            );

            let title_style = if item.completed {
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::CROSSED_OUT)
            } else {
                Style::default().fg(Color::White)
            };

            let title_span = Span::styled(&item.title, title_style);

            let due_span = match item.due_status() {
                Some(crate::models::DueStatus::Overdue) => Span::styled(
                    " [OVERDUE]",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Some(crate::models::DueStatus::DueToday) => Span::styled(
                    " [TODAY]",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Some(crate::models::DueStatus::Upcoming) => Span::styled(
                    format!(" [{}]", item.due_date.as_deref().unwrap_or("")),
                    Style::default().fg(Color::DarkGray),
                ),
                None => Span::raw(""),
            };

            let rec_span = match &item.recurrence {
                Some(rec) => Span::styled(
                    format!(" [🔁 {rec}]"),
                    Style::default().fg(Color::LightMagenta),
                ),
                None => Span::raw(""),
            };

            let line = Line::from(vec![
                status_mark,
                id_span,
                priority_span,
                title_span,
                due_span,
                rec_span,
            ]);
            ListItem::new(line)
        })
        .collect();

    let list_title = if app.search_query.is_empty() {
        format!(" Tasks ({}) ", app.todos.len())
    } else {
        format!(
            " Tasks matching '{}' ({}) ",
            app.search_query,
            app.todos.len()
        )
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(list_title),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(30, 41, 59))
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, body_chunks[0], &mut app.list_state.clone());

    // Details Pane
    let selected_todo = app
        .list_state
        .selected()
        .and_then(|i| app.todos.get(i).cloned());

    let details_paragraph = match selected_todo {
        Some(item) => {
            let status_text = if item.completed {
                Span::styled("Completed", Style::default().fg(Color::Green))
            } else {
                Span::styled("Active / In Progress", Style::default().fg(Color::Yellow))
            };

            let (p_text, p_color) = match item.priority {
                Priority::High => ("High Priority", Color::Red),
                Priority::Medium => ("Medium Priority", Color::Yellow),
                Priority::Low => ("Low Priority", Color::Blue),
            };

            let title_text = item.title.clone();
            let created_text = item.created_at.clone();
            let updated_text = item
                .updated_at
                .clone()
                .unwrap_or_else(|| "Never".to_string());
            let desc_text = item
                .description
                .clone()
                .unwrap_or_else(|| "No details provided.".to_string());

            let lines = vec![
                Line::from(vec![
                    Span::styled(
                        "Task ID: ",
                        Style::default()
                            .fg(Color::Gray)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(format!("#{}", item.id)),
                ]),
                Line::from(vec![
                    Span::styled(
                        "Title: ",
                        Style::default()
                            .fg(Color::Gray)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        title_text,
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        "Priority: ",
                        Style::default()
                            .fg(Color::Gray)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(p_text, Style::default().fg(p_color)),
                ]),
                Line::from(vec![
                    Span::styled(
                        "Status: ",
                        Style::default()
                            .fg(Color::Gray)
                            .add_modifier(Modifier::BOLD),
                    ),
                    status_text,
                ]),
                Line::from(vec![
                    Span::styled(
                        "Created: ",
                        Style::default()
                            .fg(Color::Gray)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(created_text),
                ]),
                Line::from(vec![
                    Span::styled(
                        "Updated: ",
                        Style::default()
                            .fg(Color::Gray)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(updated_text),
                ]),
                Line::from(vec![
                    Span::styled(
                        "Due Date: ",
                        Style::default()
                            .fg(Color::Gray)
                            .add_modifier(Modifier::BOLD),
                    ),
                    match item.due_status() {
                        Some(crate::models::DueStatus::Overdue) => Span::styled(
                            format!("{} (OVERDUE)", item.due_date.as_deref().unwrap_or("")),
                            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                        ),
                        Some(crate::models::DueStatus::DueToday) => Span::styled(
                            format!("{} (DUE TODAY)", item.due_date.as_deref().unwrap_or("")),
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Some(crate::models::DueStatus::Upcoming) => Span::styled(
                            item.due_date.as_deref().unwrap_or("").to_string(),
                            Style::default().fg(Color::Cyan),
                        ),
                        None => Span::raw("None"),
                    },
                ]),
                Line::from(vec![
                    Span::styled(
                        "Recurrence:   ",
                        Style::default()
                            .fg(Color::Gray)
                            .add_modifier(Modifier::BOLD),
                    ),
                    match &item.recurrence {
                        Some(rec) => Span::styled(
                            format!("🔁 {rec}"),
                            Style::default().fg(Color::LightMagenta),
                        ),
                        None => Span::raw("None"),
                    },
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    "Description / Notes:",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::UNDERLINED),
                )),
                Line::from(desc_text),
            ];

            Paragraph::new(lines)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title(" Task Details "),
                )
                .wrap(Wrap { trim: true })
        }
        None => Paragraph::new("No task selected.")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" Task Details "),
            )
            .alignment(Alignment::Center),
    };

    f.render_widget(details_paragraph, body_chunks[1]);

    // 3. Footer / Status Bar
    let footer_text = match &app.input_mode {
        InputMode::Normal => Line::from(vec![
            Span::styled(" [↑/↓, j/k]", Style::default().fg(Color::Yellow)),
            Span::raw(" Move  "),
            Span::styled("[Space/Enter]", Style::default().fg(Color::Yellow)),
            Span::raw(" Toggle  "),
            Span::styled("[a]", Style::default().fg(Color::Yellow)),
            Span::raw(" Add  "),
            Span::styled("[d/x]", Style::default().fg(Color::Yellow)),
            Span::raw(" Delete  "),
            Span::styled("[Tab]", Style::default().fg(Color::Yellow)),
            Span::raw(" Filter  "),
            Span::styled("[/]", Style::default().fg(Color::Yellow)),
            Span::raw(" Search  "),
            Span::styled("[c]", Style::default().fg(Color::Yellow)),
            Span::raw(" Clear Done  "),
            Span::styled("[q]", Style::default().fg(Color::Yellow)),
            Span::raw(" Quit"),
        ]),
        InputMode::Searching => Line::from(vec![
            Span::styled(
                " Search Query: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(&app.search_query, Style::default().fg(Color::Cyan)),
            Span::styled("█", Style::default().fg(Color::White)),
            Span::raw("  (Press Enter/Esc to finish)"),
        ]),
        InputMode::AddingTitle | InputMode::AddingDesc => {
            Line::from(" (Editing new task in modal dialog...) ")
        }
        InputMode::ConfirmClear => Line::from(vec![
            Span::styled(
                " Clear ALL completed tasks? ",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw("Press 'y' to confirm, any other key to cancel"),
        ]),
    };

    let footer = Paragraph::new(footer_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Actions "),
    );
    f.render_widget(footer, chunks[2]);

    // Render Modal Dialog if Adding Task
    if matches!(
        app.input_mode,
        InputMode::AddingTitle | InputMode::AddingDesc
    ) {
        let area = centered_rect(60, 40, f.area());
        f.render_widget(Clear, area);

        let modal_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .title(" ➕ Add New Task ")
            .style(Style::default().bg(Color::Rgb(15, 23, 42)));

        let modal_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(3), // Description
                Constraint::Length(2), // Priority
                Constraint::Length(2), // Hints
            ])
            .split(area);

        f.render_widget(modal_block, area);

        // Title input
        let title_style = if matches!(app.input_mode, InputMode::AddingTitle) {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::White)
        };
        let title_widget = Paragraph::new(format!(
            "{}{}",
            app.new_title,
            if matches!(app.input_mode, InputMode::AddingTitle) {
                "█"
            } else {
                ""
            }
        ))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Task Title (Required) ")
                .border_style(title_style),
        );
        f.render_widget(title_widget, modal_chunks[0]);

        // Description input
        let desc_style = if matches!(app.input_mode, InputMode::AddingDesc) {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::White)
        };
        let desc_widget = Paragraph::new(format!(
            "{}{}",
            app.new_desc,
            if matches!(app.input_mode, InputMode::AddingDesc) {
                "█"
            } else {
                ""
            }
        ))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Description (Optional) ")
                .border_style(desc_style),
        );
        f.render_widget(desc_widget, modal_chunks[1]);

        // Priority display
        let (p_name, p_color) = match app.new_priority {
            Priority::Low => ("LOW", Color::Blue),
            Priority::Medium => ("MEDIUM", Color::Yellow),
            Priority::High => ("HIGH", Color::Red),
        };
        let priority_line = Line::from(vec![
            Span::raw("Priority: "),
            Span::styled(
                format!("[ {p_name} ]"),
                Style::default().fg(p_color).add_modifier(Modifier::BOLD),
            ),
            Span::raw("  (Press Tab to cycle Low / Med / High)"),
        ]);
        f.render_widget(Paragraph::new(priority_line), modal_chunks[2]);

        // Hints & Errors
        let hint_line = if let Some(ref err) = app.error_message {
            Line::from(Span::styled(
                format!("❌ {err}"),
                Style::default().fg(Color::Red),
            ))
        } else {
            Line::from(vec![
                Span::styled("Enter", Style::default().fg(Color::Yellow)),
                Span::raw(" Next / Save | "),
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::raw(" Cancel"),
            ])
        };
        f.render_widget(Paragraph::new(hint_line), modal_chunks[3]);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
