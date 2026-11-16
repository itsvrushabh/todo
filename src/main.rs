#[cfg(not(target_arch = "wasm32"))]
use clap::Parser;
#[cfg(not(target_arch = "wasm32"))]
use std::env;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use todo::cli::{self, Cli, Commands};
#[cfg(not(target_arch = "wasm32"))]
use todo::server::run_server;
#[cfg(not(target_arch = "wasm32"))]
use todo::storage::JsonTodoStorage;
#[cfg(not(target_arch = "wasm32"))]
use todo::tui::run_tui;

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let storage_path = cli
        .file
        .map(|p| p.to_string_lossy().to_string())
        .or_else(|| env::var("TODO_STORAGE_PATH").ok())
        .unwrap_or_else(|| "todos.json".to_string());

    let storage = Arc::new(JsonTodoStorage::new(&storage_path)?);

    match cli.command {
        Some(Commands::Serve { host, port }) => {
            run_server(&host, port, storage, &storage_path).await?;
        }
        Some(Commands::Tui) => {
            run_tui(storage)?;
        }
        Some(Commands::Gui) => {
            todo::gui::run_desktop(std::path::PathBuf::from(&storage_path))
                .map_err(|e| format!("Failed to run GUI: {e}"))?;
        }
        Some(Commands::List {
            status,
            search,
            priority,
            tag,
        }) => {
            cli::list_todos(
                storage,
                status.as_deref(),
                search.as_deref(),
                priority.as_deref(),
                tag.as_deref(),
            )?;
        }
        Some(Commands::Add {
            title,
            desc,
            priority,
            due,
            repeat,
            tags,
            subtasks,
        }) => {
            let recurrence = repeat
                .as_deref()
                .map(|r| r.parse::<todo::models::RecurrencePattern>())
                .transpose()
                .map_err(|e| format!("Invalid recurrence: {e}"))?;
            cli::add_todo(
                storage, title, desc, priority, due, recurrence, tags, subtasks,
            )?;
        }
        Some(Commands::Done { id }) => {
            cli::toggle_todo(storage, id)?;
        }
        Some(Commands::SubtaskDone { id, subtask_id }) => match storage.toggle_subtask(id, &subtask_id)? {
            Some(_) => println!("🔄 Subtask '{subtask_id}' on task #{id} toggled"),
            None => eprintln!("❌ Error: Subtask '{subtask_id}' on task #{id} not found"),
        },
        Some(Commands::Rm { id }) => {
            cli::delete_todo(storage, id)?;
        }
        Some(Commands::Clear) => {
            cli::clear_completed(storage)?;
        }
        Some(Commands::Stats) => {
            cli::show_stats(storage)?;
        }
        None => {
            // Default action: start the server on 0.0.0.0:8000
            run_server("0.0.0.0", 8000, storage, &storage_path).await?;
        }
    }

    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {}
