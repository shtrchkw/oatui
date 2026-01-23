# oatui

A fast TUI application for browsing OpenAPI specifications in the terminal.

## Features

- Browse OpenAPI 3.0 specs without leaving your terminal
- Vim-style keyboard navigation with dual-pane focus
- Color-coded HTTP methods (GET, POST, PUT, DELETE, etc.)
- View endpoint details: parameters, request body, and responses
- Fast startup and lightweight

## Installation

Build from source:

```bash
git clone https://github.com/shtrchkw/oatui.git
cd oatui
cargo build --release
./target/release/oatui <openapi-file>
```

## Usage

```bash
oatui openapi.yaml
```

Supports both YAML and JSON OpenAPI 3.0 specifications.

## Key Bindings

### List Pane (default)

| Key | Action |
|-----|--------|
| `j` / `↓` | Next endpoint |
| `k` / `↑` | Previous endpoint |
| `Enter` | Focus detail pane |
| `q` | Quit |

### Detail Pane

| Key | Action |
|-----|--------|
| `j` / `↓` | Scroll down |
| `k` / `↑` | Scroll up |
| `Esc` | Return to list pane |
| `q` | Quit |

## License

MIT
