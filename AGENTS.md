# AGENTS.md - Development Guide for eefoctui

This document provides essential information for agentic coding agents working on this codebase.

## Project Overview

eefoctui is a Rust TUI application for Docker container management, system monitoring, and network diagnostics. It uses ratatui for the terminal UI, bollard for Docker API, and tokio for async operations.

## Build & Development Commands

### Building
```bash
cargo build              # Debug build
cargo build --release    # Release build
cargo run                # Run in development
```

### Code Quality
```bash
cargo check              # Type check without building
cargo fmt                # Format code (run before committing)
cargo clippy             # Linting and static analysis
cargo clippy -- -D warnings  # Treat warnings as errors
```

### Testing
```bash
cargo test               # Run all tests
cargo test <test_name>   # Run a single test by name
cargo test -- --nocapture  # Show output during tests
```

**Note:** This project currently has no tests. When adding tests, place them in `tests/` directory or use inline `#[cfg(test)]` modules.

## Code Style Guidelines

### Formatting
- Use 4 spaces for indentation (Rust default)
- Run `cargo fmt` before committing
- Maximum line length: 100 characters (soft guideline)
- Use trailing commas in struct literals and match arms

### Naming Conventions
- **Types/Enums**: PascalCase (e.g., `AppEvent`, `DockerState`)
- **Functions/Variables**: snake_case (e.g., `refresh_containers`, `port_input_host`)
- **Constants**: SCREAMING_SNAKE_CASE
- **Modules**: snake_case (e.g., `mod docker;`)

### Imports Organization
Order imports in this sequence:
1. Crate-local imports (`crate::`)
2. External crate imports (e.g., `tokio::`, `ratatui::`)
3. Standard library imports (`std::`)
4. Alphabetical within each group

```rust
// Good import ordering
use crate::events::AppEvent;
use anyhow::Result;
use bollard::Docker;
use tokio::sync::mpsc::UnboundedSender;
use std::collections::HashMap;
```

### Error Handling
- Use `anyhow::Result<T>` for application code (easier error propagation)
- Use `thiserror` for library code requiring custom error types
- Propagate errors with `?` operator
- Include context in error messages: `map_err(|e| format!("failed to X: {e}"))`

### Async Patterns
- All async functions should use `#[tokio::main]` or `#[tokio::test]`
- Use `tokio::spawn` for background tasks
- Use unbounded channels (`mpsc::unbounded_channel`) for event dispatching
- Avoid blocking calls in async context; use async equivalents

### Module Structure
- Use `mod.rs` or inline module declarations
- One module per file is preferred for clarity
- Group related functionality (e.g., `services/docker.rs`, `ui/views/`)

```rust
// Module organization
src/
  main.rs      # Entry point
  app.rs       # App state and main logic
  config.rs    # Configuration
  events.rs    # Event definitions
  services/    # External service integrations
    docker.rs
    network.rs
    system.rs
  models/      # Data structures
    container.rs
    metrics.rs
  ui/          # TUI rendering
    mod.rs
    views/
    theme.rs
```

### Documentation
- Add doc comments (`///`) on public functions and types
- Include usage examples where helpful
- Document async behavior and error conditions

### TUI-Specific Patterns
- Use `ratatui` for all terminal rendering
- Store UI state in dedicated structs (e.g., `DockerState`, `NetworkState`)
- Use events for all state mutations
- Handle terminal restore in `main.rs` (see existing code)

### Testing Guidelines
- Add unit tests for pure functions
- Add integration tests for service modules
- Mock external dependencies where possible
- Use `#[tokio::test]` for async tests

## Key Dependencies

| Crate | Purpose |
|-------|---------|
| ratatui | Terminal UI framework |
| crossterm | Terminal capabilities |
| tokio | Async runtime |
| bollard | Docker API client |
| anyhow | Error handling |
| sysinfo | System metrics |
| serde | Serialization |

## Working with Docker

When modifying Docker functionality:
- Use `bollard` crate for API calls
- Fall back to CLI (`docker stats`, `docker attach`) for stats/console
- Handle both PTY (Unix) and pipe-based console modes

## Common Tasks

### Adding a New View
1. Create `src/ui/views/new_view.rs`
2. Implement render function with `impl Widget for &mut NewViewState`
3. Add to `View` enum in `app.rs`
4. Add event handlers in `app.rs::App::update()`
5. Add navigation keybindings in `events/input.rs`

### Adding a New Service
1. Create `src/services/new_service.rs`
2. Export in `src/services/mod.rs`
3. Add background task spawning in `services::spawn_background_tasks`
4. Use events to communicate with UI

### Running the Application
```bash
cargo run
```

Controls:
- Arrow keys: Navigate
- Enter: Select/activate
- Escape: Back/cancel
- Ctrl+C: Quit
