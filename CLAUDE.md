# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

oatui is a TUI application for browsing OpenAPI specifications in the terminal. Built with Rust using ratatui for the UI framework.

## Build Commands

```bash
cargo build           # Build the project
cargo run             # Run the application
cargo run -- openapi.yaml  # Run with an OpenAPI file
cargo test            # Run all tests
cargo test <name>     # Run a specific test
cargo clippy          # Run linter
cargo fmt             # Format code
```

## Architecture

The planned architecture follows this structure:

- `src/main.rs` - Entry point, CLI argument parsing with clap
- `src/app.rs` - Application state management
- `src/ui.rs` - UI rendering with ratatui
- `src/parser.rs` - OpenAPI file parsing with openapiv3
- `src/model.rs` - Internal data models
- `src/event.rs` - Keyboard/terminal event handling with crossterm

## Tech Stack

- **TUI**: ratatui + crossterm
- **OpenAPI parsing**: openapiv3, serde_yaml, serde_json
- **CLI**: clap
- **Error handling**: anyhow, thiserror

## Testing Requirements

For high and medium priority features, always write:

- **Unit tests**: Test individual functions and modules in isolation
- **Integration tests**: Test interactions between modules
- **E2E tests**: Test the application as a whole from user perspective

Run tests with `cargo test` before committing.
