# Contributing to Todo

Thank you for your interest in contributing to Todo! We follow an **issue-first, trunk-based workflow** to keep our codebase reliable and high quality.

---

## 🛠️ Contribution Workflow

1. **Find or Open an Issue**:
   - Every pull request must be linked to an open GitHub issue.
   - Use our [Bug Report](.github/ISSUE_TEMPLATE/bug_report.yml) or [Feature Request](.github/ISSUE_TEMPLATE/feature_request.yml) templates.

2. **Branch from `main`**:
   - Create a dedicated topic branch:
     ```bash
     git checkout -b feat/your-feature
     # or
     git checkout -b fix/your-bugfix
     # or
     git checkout -b docs/your-doc-update
     ```

3. **Verify Local Quality Checks**:
   Before submitting code, ensure all formatting, linter warnings, and tests pass with zero errors:
   ```bash
   # 1. Format check
   cargo fmt -- --check

   # 2. Strict clippy analysis
   cargo clippy --all-targets --all-features -- -D warnings

   # 3. Unit and integration tests
   cargo test --all-targets --all-features
   ```

4. **Submit a Pull Request**:
   - Reference the issue in your PR description: `Fixes #<issue-number>`.
   - Keep pull requests focused on a single concern.

5. **Automated Review & CI**:
   - Automated **Gemini AI Code Review** runs on every PR.
   - **GitHub Copilot Code Review**:
     - Guided by `.github/copilot-instructions.md`.
     - Automatically requested via `.github/workflows/copilot-pr-review.yml`.
     - Can also be toggled in repository settings: **Settings** ➔ **Copilot** ➔ **Code review** ➔ *"Automatically request review from Copilot"*.
   - Multi-platform CI (Ubuntu, Windows, macOS) validates builds, wasm targets, and test suites.

6. **Merge**:
   - Once approved and all checks pass, pull requests are merged via squash to `main`.

---

## 📜 Coding Guidelines

- **Idiomatic Rust**: Adhere to standard Rust 2021 conventions and rustfmt formatting.
- **Zero Warnings**: Code with compiler warnings or `clippy` lints will not be merged.
- **Safety**: Avoid `unsafe` blocks unless strictly necessary and documented with a safety invariant.
- **Atomic Operations**: Any storage modification must preserve crash-resilient atomic file writes.
