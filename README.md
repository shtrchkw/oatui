# oatui

A fast TUI application for browsing OpenAPI specifications in the terminal.

## Features

- Browse OpenAPI 3.0 specs without leaving your terminal
- Vim-style keyboard navigation
- Color-coded HTTP methods
- Fast startup and lightweight

## Installation

```bash
cargo install oatui
```

Or build from source:

```bash
git clone https://github.com/shtrchkw/oatui.git
cd oatui
cargo build --release
```

## Usage

```bash
oatui openapi.yaml
```

## Key Bindings

| Key | Action |
|-----|--------|
| `j` / `↓` | Next endpoint |
| `k` / `↑` | Previous endpoint |
| `Enter` | Toggle detail view |
| `/` | Search |
| `Esc` | Cancel / go back |
| `q` | Quit |

## License

MIT
