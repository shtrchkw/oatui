# oatui - OpenAPI TUI Viewer

## Overview

A TUI application for quickly browsing OpenAPI specifications in the terminal.

Built for developers who want to check API documentation without leaving their terminal environment (iTerm2, tmux, etc.).

## Motivation

### Problems with Existing Tools

| Tool | Issues |
|------|--------|
| Swagger UI | Heavy, outdated UI, requires browser |
| Redoc | Beautiful but lacks interactivity, requires browser |
| Others | No good options for quick terminal viewing |

### What oatui Solves

- Check API specs without leaving the terminal
- Fast startup, lightweight operation
- Keyboard-only operation

## Target Users

- Developers who work primarily in terminal environments
- Users of iTerm2 + tmux or similar setups
- People who frequently work with API development and integration

## Feature Requirements

### Phase 1: MVP (Minimum Viable Product)

#### Core Features

- [ ] Load OpenAPI 3.0 files (YAML/JSON)
- [ ] Display endpoint list
- [ ] Show endpoint details on selection
- [ ] Keyboard navigation (vim-style: j/k to move, Enter to select, q to quit)

#### Detail View Information

- HTTP method + path
- Summary / Description
- Parameter list (path, query, header, cookie)
- Request body schema
- Responses (by status code)

#### Usage Examples

```bash
# Basic usage
oatui openapi.yaml

# Display a specific endpoint directly
oatui openapi.yaml --path "/users/{id}" --method GET
```

### Phase 2: Usability Improvements

- [ ] Fuzzy search for endpoint filtering
- [ ] Schema tree view (expand/collapse)
- [ ] Generate curl commands & copy to clipboard
- [ ] Copy path to clipboard
- [ ] Multiple file support (tabs or switching)

### Phase 3: Advanced Features

- [ ] OpenAPI 3.1 support
- [ ] $ref resolution and inline display
- [ ] Search history
- [ ] Favorites/bookmarks
- [ ] Configuration file support (~/.config/oatui/config.toml)

### Future Considerations (Backlog)

- Send actual requests
- Diff comparison between multiple specs
- OpenAPI 2.0 (Swagger) support
- LSP integration

## Non-Functional Requirements

### Performance

- Startup time: under 100ms (for small to medium spec files)
- Memory usage: under 50MB

### Supported Environments

- OS: Linux, macOS
- Terminal: Terminal emulators supporting 256+ colors
- Shell: bash, zsh, fish, etc. (shell-independent)

## Tech Stack

| Purpose | Library |
|---------|---------|
| TUI framework | ratatui |
| Terminal control | crossterm |
| OpenAPI parser | openapiv3 |
| YAML/JSON parsing | serde, serde_yaml, serde_json |
| CLI arguments | clap |
| Error handling | anyhow, thiserror |
| Clipboard | arboard (Phase 2) |

## UI Design

### Layout

```
┌─ Endpoints ─────────────────┬─ Detail ────────────────────────────┐
│ > GET    /users             │ GET /users                          │
│   POST   /users             │ ─────────────────────────────────── │
│   GET    /users/{id}        │ Summary:                            │
│   PUT    /users/{id}        │   List all users                    │
│   DELETE /users/{id}        │                                     │
│   GET    /posts             │ Parameters:                         │
│   POST   /posts             │   limit  : integer (query)          │
│                             │   offset : integer (query)          │
│                             │                                     │
│                             │ Response 200:                       │
│                             │   application/json                  │
│                             │   { "users": [...] }                │
│                             │                                     │
├─────────────────────────────┴─────────────────────────────────────┤
│ [/] Search  [q] Quit  [Enter] Select  [j/k] Navigate              │
└───────────────────────────────────────────────────────────────────┘
```

### Key Bindings

| Key | Action |
|-----|--------|
| `j` / `↓` | Next endpoint |
| `k` / `↑` | Previous endpoint |
| `Enter` | Toggle detail view focus |
| `/` | Search mode |
| `Esc` | Cancel search / go back |
| `q` | Quit |
| `y` | Copy path (Phase 2) |
| `Y` | Copy curl command (Phase 2) |

### Color Scheme

- HTTP methods are color-coded:
  - GET: Green
  - POST: Blue
  - PUT: Yellow
  - DELETE: Red
  - PATCH: Cyan

## Project Structure

```
oatui/
├── Cargo.toml
├── README.md
├── LICENSE (MIT)
├── src/
│   ├── main.rs
│   ├── app.rs          # Application state management
│   ├── ui.rs           # UI rendering
│   ├── parser.rs       # OpenAPI parsing
│   ├── model.rs        # Internal data models
│   └── event.rs        # Event handling
└── tests/
    └── fixtures/       # Test OpenAPI files
```

## Development Roadmap

### Week 1-2: Foundation

- Project setup
- OpenAPI parser implementation
- Basic TUI framework

### Week 3-4: MVP Completion

- Endpoint list display
- Detail view
- Basic navigation

### Week 5-6: Phase 2 Features

- Fuzzy search
- Clipboard integration
- Schema tree view

### Beyond: Phase 3 + Community Feedback

## License

MIT License

## References

- [OpenAPI Specification](https://spec.openapis.org/oas/latest.html)
- [ratatui](https://github.com/ratatui-org/ratatui)
- [openapiv3 crate](https://crates.io/crates/openapiv3)
