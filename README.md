# Duanwu Lab

A cross-platform desktop application built with [Tauri v2](https://tauri.app) and [Dioxus 0.7](https://dioxuslabs.com).

## Prerequisites

- [Rust](https://rustup.rs) (stable)
- Platform-specific build dependencies:

**Linux**
```bash
sudo apt install libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev libasound2-dev
```

**macOS** — Xcode Command Line Tools are required.

**Windows** — Visual Studio Build Tools with C++ workload.

## Getting Started

```bash
# Install CLI tools
cargo install dioxus-cli --locked
cargo install tauri-cli --locked

# Start dev server (hot-reload frontend + native window)
cargo tauri dev

# Frontend only (browser)
dx serve --port 1420
```

## Project Structure

```
├── src/                  # Dioxus frontend
│   ├── main.rs           # Entry point
│   └── app.rs            # App component
├── src-tauri/            # Tauri backend
│   ├── src/
│   │   ├── main.rs       # Binary entry
│   │   └── lib.rs        # Commands & app setup
│   └── tauri.conf.json   # Tauri configuration
├── assets/               # Static assets
├── dist/                 # Built frontend output
├── Cargo.toml            # Workspace root
└── Dioxus.toml           # Dioxus config
```

## Build

```bash
# Local build
cargo tauri build

# Cross-platform release (via GitHub Actions)
git tag v0.0.1
git push origin v0.0.1
```

Built artifacts are uploaded to [GitHub Releases](../../releases).

## Checks

```bash
cargo fmt --all             # Format code
cargo fmt --all --check     # Check formatting
cargo clippy                # Lint
cargo build                 # Compile
```
