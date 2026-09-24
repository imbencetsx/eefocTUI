# ⚡🐳 eefoctui — Docker + System + Network TUI

> A fast, keyboard-first terminal dashboard for containers, host metrics, and network diagnostics.

eefoctui is a Rust TUI built with `ratatui`, `tokio`, and `bollard`. It gives you one place to inspect containers, view system health, scan ports, and jump into an in-app container console.

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

## 🧠 Architecture Overview

- Event-driven state updates via `AppEvent`.
- Background polling tasks for Docker, system metrics, and network data.
- Separate UI views under `src/ui/views/`.
- Shared app state in `src/app.rs`.

## 🛠️ Development

```bash
cargo check
cargo fmt
cargo clippy -- -D warnings
cargo test
```

## 🤝 Contributing

PRs and issue reports are welcome. Keep changes focused, run `cargo fmt`, and make sure `cargo clippy -- -D warnings` passes before opening a PR.

---

Made with Rust, terminal obsession, and probably too many keyboard shortcuts. 😂
