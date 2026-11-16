use crate::models::{Priority, TodoCreate, TodoItem, TodoStats};
use crate::storage::JsonTodoStorage;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(
    name = "todo",
    about = "TaskMaster - Cross-Platform Modern Todo App with Web UI, PWA, TUI, and CLI",
    version
)]
pub struct Cli {
    #[arg(
        short,
        long,
        global = true,
        help = "Path to JSON storage file (defaults to TODO_STORAGE_PATH or todos.json)"
    )]
    pub file: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start the Web and API server (default)
    Serve {
        #[arg(
            long,
            default_value = "0.0.0.0",
            help = "Host address to bind to (0.0.0.0 for LAN/mobile access)"
        )]
        host: String,
        #[arg(short, long, default_value_t = 8000, help = "Port number")]
        port: u16,
    },
    /// Launch the interactive Terminal User Interface (TUI)
    Tui,
    /// Launch the native Desktop Graphical User Interface (GUI)
    Gui,
    /// List all tasks with optional filters
    List {
        #[arg(short, long, help = "Filter by status: active or completed")]
        status: Option<String>,
        #[arg(short = 'q', long, help = "Search term in title or description")]
        search: Option<String>,
        #[arg(short, long, help = "Filter by priority: low, medium, high")]
        priority: Option<String>,
    },
    /// Add a new task
    Add {
        #[arg(help = "Title of the task")]
        title: String,
        #[arg(short, long, help = "Optional detailed description")]
        desc: Option<String>,
        #[arg(short, long, help = "Priority: low, medium, or high")]
        priority: Option<Priority>,
        #[arg(long, help = "Optional due date (YYYY-MM-DD or RFC3339)")]
        due: Option<String>,
        #[arg(
            long,
            help = "Optional recurrence: daily, weekly, monthly, or Nd (e.g. 3d)"
        )]
        repeat: Option<String>,
    },
    /// Toggle completion status of a task
    Done {
        #[arg(help = "Task ID")]
        id: i64,
    },
    /// Delete a task
    Rm {
        #[arg(help = "Task ID")]
        id: i64,
    },
    /// Clear all completed tasks
    Clear,
    /// Display summary statistics
    Stats,
}

pub fn list_todos(
    storage: Arc<JsonTodoStorage>,
    status: Option<&str>,
    search: Option<&str>,
    priority: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let items = storage.get_all(status, search, priority)?;
    let stats = storage.get_stats()?;

    println!("\n{}", "=".repeat(60));
    println!(" 📋 TaskMaster - Task List");
    println!("{}", "=".repeat(60));

    if items.is_empty() {
        println!(" (No tasks found matching criteria)");
    } else {
        for item in &items {
            let mark = if item.completed { "✓" } else { " " };
            let p_badge = match item.priority {
                Priority::High => "[HIGH]",
                Priority::Medium => "[MED] ",
                Priority::Low => "[LOW] ",
            };
            let due_badge = match item.due_status() {
                Some(crate::models::DueStatus::Overdue) => format!(
                    " \x1b[31;1m[OVERDUE: {}]\x1b[0m",
                    item.due_date.as_deref().unwrap_or("")
                ),
                Some(crate::models::DueStatus::DueToday) => {
                    " \x1b[33;1m[DUE TODAY]\x1b[0m".to_string()
                }
                Some(crate::models::DueStatus::Upcoming) => format!(
                    " \x1b[36m[Due: {}]\x1b[0m",
                    item.due_date.as_deref().unwrap_or("")
                ),
                None => String::new(),
            };
            let rec_badge = match &item.recurrence {
                Some(rec) => format!(" \x1b[35m[🔁 {}]\x1b[0m", rec),
                None => String::new(),
            };
            println!(
                " [{mark}] #{:<3} {:<6} {}{due_badge}{rec_badge}",
                item.id, p_badge, item.title
            );
            if let Some(ref desc) = item.description {
                if !desc.is_empty() {
                    println!("       ↳ {desc}");
                }
            }
        }
    }

    println!("{}", "-".repeat(60));
    println!(
        " Total: {} | Active: {} | Completed: {} ({:.1}% done)",
        stats.total, stats.active, stats.completed, stats.completion_percentage
    );
    println!("{}\n", "=".repeat(60));
    Ok(())
}

pub fn add_todo(
    storage: Arc<JsonTodoStorage>,
    title: String,
    description: Option<String>,
    priority: Option<Priority>,
    due_date: Option<String>,
    recurrence: Option<crate::models::RecurrencePattern>,
) -> Result<TodoItem, Box<dyn std::error::Error>> {
    let item = storage.create(TodoCreate {
        title,
        description,
        priority,
        due_date,
        recurrence,
    })?;
    let rec_msg = item
        .recurrence
        .map(|r| format!(" (repeats {r})"))
        .unwrap_or_default();
    println!("✅ Added task #{}: '{}{}'", item.id, item.title, rec_msg);
    Ok(item)
}

pub fn toggle_todo(
    storage: Arc<JsonTodoStorage>,
    id: i64,
) -> Result<(), Box<dyn std::error::Error>> {
    match storage.toggle(id)? {
        Some(item) => {
            if let Some(ref rec) = item.recurrence {
                let due_info = item
                    .due_date
                    .as_deref()
                    .map(|d| format!(" (Next due: {d})"))
                    .unwrap_or_default();
                println!(
                    "🔁 Recurring task #{} ({rec}) rolled forward{due_info}",
                    item.id
                );
            } else {
                let state = if item.completed {
                    "completed"
                } else {
                    "active"
                };
                println!("🔄 Task #{} is now {}", item.id, state);
            }
        }
        None => {
            eprintln!("❌ Error: Todo #{id} not found.");
        }
    }
    Ok(())
}

pub fn delete_todo(
    storage: Arc<JsonTodoStorage>,
    id: i64,
) -> Result<(), Box<dyn std::error::Error>> {
    if storage.delete(id)? {
        println!("🗑️  Deleted task #{id}");
    } else {
        eprintln!("❌ Error: Todo #{id} not found.");
    }
    Ok(())
}

pub fn clear_completed(storage: Arc<JsonTodoStorage>) -> Result<(), Box<dyn std::error::Error>> {
    let count = storage.clear_completed()?;
    println!("🧹 Cleared {count} completed task(s)");
    Ok(())
}

pub fn show_stats(storage: Arc<JsonTodoStorage>) -> Result<TodoStats, Box<dyn std::error::Error>> {
    let stats = storage.get_stats()?;
    println!("\n📊 TaskMaster Statistics:");
    println!("   Total tasks:          {}", stats.total);
    println!("   Active tasks:         {}", stats.active);
    println!("   Completed tasks:      {}", stats.completed);
    println!(
        "   Completion rate:      {:.1}%\n",
        stats.completion_percentage
    );
    Ok(stats)
}
