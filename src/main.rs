use clap::Parser;
use std::env;
use std::sync::Arc;
use todo::cli::{self, Cli, Commands};
use todo::server::run_server;
use todo::storage::JsonTodoStorage;
use todo::tui::run_tui;

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
        Some(Commands::List {
            status,
            search,
            priority,
        }) => {
            cli::list_todos(
                storage,
                status.as_deref(),
                search.as_deref(),
                priority.as_deref(),
            )?;
        }
        Some(Commands::Add {
            title,
            desc,
            priority,
        }) => {
            cli::add_todo(storage, title, desc, priority)?;
        }
        Some(Commands::Done { id }) => {
            cli::toggle_todo(storage, id)?;
        }
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
