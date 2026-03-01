# ⚡🐳 eefoctui — Docker + System + Network TUI

> A fast, keyboard-first terminal dashboard for containers, host metrics, and network diagnostics.

eefoctui is a Rust TUI built with `ratatui`, `tokio`, and `bollard`. It gives you one place to inspect containers, view system health, scan ports, and jump into an in-app container console.

## ✨ Highlights

- **Docker control panel**: list containers, start/stop/restart, delete with confirmation.
- **In-app container console**: attach, stream output, send input, scroll, and detach.
- **System observability**: live CPU, memory, disk, and network metrics.
- **Network tools**: interface + connection view and an interactive TCP port scanner.
- **Multi-view workflow**: `Docker`, `System`, `Network`, and `Help` tabs with smooth keyboard navigation.

## 🧰 Tech Stack

- `Rust` (edition 2021)
- `ratatui` + `crossterm`
- `tokio` async runtime
- `bollard` Docker API
- `sysinfo` host metrics

## 🚀 Quick Start

### 1) Prerequisites

- Rust toolchain (`cargo`, `rustc`)
- Docker installed and running
- Terminal with ANSI color support

### 2) Run

```bash
cargo run
```

### 3) Build release

```bash
cargo build --release
```

## ⌨️ Keybindings

### Global

- `q` quit
- `Ctrl+Left` / `Ctrl+Right` switch views
- `Up` / `Down` move selection
- `Esc` dismiss transient UI state/errors

### Docker view

- `Enter` open in-app console for selected container
- `Insert` detach/cancel action
- `o` start selected container
- `s` stop selected container
- `r` restart selected container
- `Delete` delete selected container (type container name to confirm)
- `Alt+Up` / `Alt+Down` scroll container table

### Network view

- `Tab` / `Shift+Tab` switch sub-views
- `i` focus port-scanner input mode
- `Tab` switch scanner input field (host/ports)
- `p` cycle scanner port presets
- `Enter` run scan

## 🧠 Architecture Overview

- Event-driven state updates via `AppEvent`.
- Background polling tasks for Docker, system metrics, and network data.
- Separate UI views under `src/ui/views/`.
- Shared app state in `src/app.rs`.

## 📁 Project Layout

```text
src/
  main.rs
  app.rs
  config.rs
  events/
  models/
  services/
  ui/
```

## 🛠️ Development

```bash
cargo check
cargo fmt
cargo clippy -- -D warnings
cargo test
```

## 🗺️ Roadmap Ideas

- Persisted configuration from file/env
- Search/filter for large container lists
- Export snapshots of metrics/network state
- More protocol-aware network diagnostics

## 🤝 Contributing

PRs and issue reports are welcome. Keep changes focused, run `cargo fmt`, and make sure `cargo clippy -- -D warnings` passes before opening a PR.

---

Made with Rust, terminal obsession, and probably too many keyboard shortcuts. ⌨️
