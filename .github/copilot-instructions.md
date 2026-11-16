# GitHub Copilot PR Review & Coding Guidelines for TaskMaster

When reviewing Pull Requests or generating code in this repository, follow these standards:

## 1. Rust Best Practices & Architecture
- **Safety First**: Never use `unsafe` code unless strictly necessary with clear invariants and justification.
- **Error Handling**: Use explicit, strongly typed `Result<T, E>` and avoid unhandled `unwrap()` or `expect()` in production paths.
- **Thread Safety**: Storage modifications must remain thread-safe (`RwLock`, atomic filesystem renames).
- **Zero-Cost Abstractions**: Leverage iterators, borrow checker guarantees, and efficient memory allocations.
- **Clippy Cleanliness**: All code must pass `cargo clippy --all-targets --all-features -- -D warnings` without warnings.
- **Formatting**: Format using standard `cargo fmt`.

## 2. Cross-Platform & API Standards
- **REST API Compatibility**: All endpoints (`/api/todos`, `/api/health`, `/api/stats`, etc.) must follow JSON schema contracts and status codes (200, 201, 400, 404, 500).
- **PWA & Web UI**: Ensure responsive touch controls for mobile (Android, iOS/iPad) and desktop window modes.
- **TUI & CLI**: Maintain keyboard navigation shortcuts, terminal alternate screen cleanup, and clear terminal feedback.

## 3. Pull Request Requirements
- Every PR must reference a tracked GitHub Issue (e.g., `Fixes #...` or `Resolves #...`).
- Verify that unit tests or integration tests accompany new features or bug fixes.
- Ensure all CI checks (linting, tests, security audits) pass before requesting approval.
