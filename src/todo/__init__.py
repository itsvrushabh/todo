import argparse
import os
import sys
from pathlib import Path

from todo.storage import JsonTodoStorage
from todo.models import Priority, TodoCreate, TodoUpdate


def run_server(host: str, port: int, storage_file: str, reload: bool = False) -> None:
    """Launch the FastAPI Todo Web Application."""
    os.environ["TODO_STORAGE_PATH"] = storage_file
    print("\n" + "=" * 55)
    print(" 🚀 Todo App Server Starting...")
    print(f" 🌐 Access the Web UI: http://{host}:{port}")
    print(f" 📁 Data stored at:    {Path(storage_file).resolve()}")
    print(" 💡 Press Ctrl+C to stop the server")
    print("=" * 55 + "\n")

    import uvicorn
    from todo.app import create_app

    app = create_app(storage_file)
    uvicorn.run(app, host=host, port=port, log_level="info")


def main() -> None:
    parser = argparse.ArgumentParser(
        prog="todo",
        description="TaskMaster - Modern Todo App with Web UI and CLI support",
    )
    parser.add_argument(
        "--file",
        "-f",
        default="todos.json",
        help="Path to JSON data file (default: todos.json)",
    )

    subparsers = parser.add_subparsers(dest="command", help="Available subcommands")

    # Command: serve (default)
    serve_parser = subparsers.add_parser("serve", help="Start the Todo web app server")
    serve_parser.add_argument("--host", default="127.0.0.1", help="Host to bind (default: 127.0.0.1)")
    serve_parser.add_argument("--port", type=int, default=8000, help="Port to bind (default: 8000)")
    serve_parser.add_argument("--reload", action="store_true", help="Enable auto-reload")

    # Command: list
    list_parser = subparsers.add_parser("list", help="List todos in the terminal")
    list_parser.add_argument(
        "--status",
        choices=["all", "active", "completed"],
        default="all",
        help="Filter by status (default: all)",
    )

    # Command: add
    add_parser = subparsers.add_parser("add", help="Add a new todo via CLI")
    add_parser.add_argument("title", help="Title of the task")
    add_parser.add_argument("--desc", "-d", default="", help="Optional description")
    add_parser.add_argument(
        "--priority",
        "-p",
        choices=["low", "medium", "high"],
        default="medium",
        help="Priority level (default: medium)",
    )

    # Command: done
    done_parser = subparsers.add_parser("done", help="Toggle or complete a task by ID")
    done_parser.add_argument("id", type=int, help="Task ID to mark complete")

    # Command: rm
    rm_parser = subparsers.add_parser("rm", help="Delete a task by ID")
    rm_parser.add_argument("id", type=int, help="Task ID to delete")

    # Also support top-level --host and --port when no subcommand is provided
    parser.add_argument("--host", default="127.0.0.1", help="Host when starting server")
    parser.add_argument("--port", type=int, default=8000, help="Port when starting server")

    args = parser.parse_args()

    storage = JsonTodoStorage(args.file)

    if args.command == "serve" or args.command is None:
        host = getattr(args, "host", "127.0.0.1")
        port = getattr(args, "port", 8000)
        reload_flag = getattr(args, "reload", False)
        run_server(host=host, port=port, storage_file=args.file, reload=reload_flag)

    elif args.command == "list":
        filter_status = None if args.status == "all" else args.status
        todos = storage.get_all(status=filter_status)
        if not todos:
            print("No tasks found.")
            return

        print(f"\n{'ID':<5} {'STATUS':<12} {'PRIORITY':<10} {'TITLE'}")
        print("-" * 55)
        for t in todos:
            status_str = "✔ Done" if t.completed else "○ Pending"
            print(f"{t.id:<5} {status_str:<12} {t.priority.value.upper():<10} {t.title}")
            if t.description:
                print(f"      └─ Note: {t.description}")
        print()

    elif args.command == "add":
        new_todo = storage.create(
            TodoCreate(
                title=args.title,
                description=args.desc,
                priority=Priority(args.priority),
            )
        )
        print(f"✓ Added task #{new_todo.id}: '{new_todo.title}' [{new_todo.priority.value.upper()}]")

    elif args.command == "done":
        toggled = storage.toggle(args.id)
        if toggled:
            state = "completed" if toggled.completed else "reopened"
            print(f"✓ Task #{toggled.id} is now {state}: '{toggled.title}'")
        else:
            print(f"✗ Task #{args.id} not found", file=sys.stderr)
            sys.exit(1)

    elif args.command == "rm":
        if storage.delete(args.id):
            print(f"✓ Deleted task #{args.id}")
        else:
            print(f"✗ Task #{args.id} not found", file=sys.stderr)
            sys.exit(1)


if __name__ == "__main__":
    main()
