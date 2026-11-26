# Project Name

<div align="center">

![Nocta](https://img.shields.io/badge/Nocta-Dark_Only-000?style=flat-square&logo=data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAxMjggMTI4Ij48cmVjdCB3aWR0aD0iMTI4IiBoZWlnaHQ9IjEyOCIgZmlsbD0iIzAwMCIvPjx0ZXh0IHg9IjY0IiB5PSI4OCIgZm9udC1mYW1pbHk9InN5c3RlbS11aSwic2Fucy1zZXJpZiIgZm9udC1zaXplPSI1NiIgZm9udC13ZWlnaHQ9ImJvbGQiIGZpbGw9IiNmZmYiIHRleHQtYW5jaG9yPSJtaWRkbGUiPk48L3RleHQ+PC9zdmc+)
![Accent](https://img.shields.io/badge/Accent-%23d4a843-d4a843?style=flat-square)
![Theme](https://img.shields.io/badge/Theme-Dark_Only-1a1a1a?style=flat-square)
![Version](https://img.shields.io/badge/Version-0.1.0-000?style=flat-square)
![License](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-000?style=flat-square)

</div>

---

## One-liner Description

**Tagline** — What problem does this solve?

<div align="center">

[![GitHub](https://img.shields.io/badge/GitHub-earnerbaymalay-181717?style=flat-square&logo=github)](https://github.com/earnerbaymalay/PROJECT_NAME)
[![Issues](https://img.shields.io/badge/Issues-Welcome-22c55e?style=flat-square)](https://github.com/earnerbaymalay/PROJECT_NAME/issues)
[![Discussions](https://img.shields.io/badge/Discussions-Open-d4a843?style=flat-square)](https://github.com/earnerbaymalay/PROJECT_NAME/discussions)

</div>

---

## Features

- Feature 1: Description
- Feature 2: Description
- Feature 3: Description

## Quick Start

```bash
# Minimal working example
git clone https://github.com/earnerbaymalay/PROJECT_NAME
cd PROJECT_NAME
./build.sh dev
```

## Installation

### Prerequisites

- **Rust**: 1.75+ ([rustup](https://rustup.rs/))
- **Node.js**: 20+ ([nvm](https://github.com/nvm-sh/nvm) / [fnm](https://github.com/Schniz/fnm))
- **Tauri Prerequisites**: See [Tauri docs](https://tauri.app/v1/guides/getting-started/prerequisites)

### Build

```bash
# Linux/macOS/WSL
./build.sh build

# Windows
build.cmd build
```

## Usage

```bash
# Basic usage
./build.sh dev          # Development mode
./build.sh build        # Production build
./build.sh test         # Run tests
./build.sh lint         # Lint code
./build.sh format       # Format code
```

## Configuration

```toml
# config.toml
[app]
name = "My App"
version = "0.1.0"
identifier = "com.example.myapp"

[window]
title = "My App"
width = 1200
height = 800

[security]
csp = "default-src 'self'; script-src 'self' 'unsafe-inline'"
devtools = false
```

## Architecture

```mermaid
graph TD
    A[Frontend] --> B[Tauri IPC]
    B --> C[Rust Core]
    C --> D[Shared Lib]
    D --> E[Plugins]
```

## Tech Stack

`Rust` `TypeScript` `Tauri 2` `Vite` `Ollama` `llama.cpp` `SQLite`

## Project Structure

```
PROJECT_NAME/
├── Cargo.toml              # Workspace root
├── tauri.conf.json         # Tauri configuration
├── build.sh / build.cmd    # Build scripts
├── .github/workflows/      # CI/CD
├── crates/
│   ├── app/                # Tauri entry point
│   ├── core/               # Business logic
│   ├── cli/                # CLI binary
│   └── shared/             # Shared utilities
├── frontend/               # TypeScript + Vite
└── docs/                   # Documentation
```

## Development

```bash
# Install dependencies
cd frontend && npm install

# Development
./build.sh dev

# Code quality
./build.sh lint
./build.sh format
./build.sh test
```

## Testing

```bash
# Rust tests
cargo test --workspace

# Frontend tests
cd frontend && npm test
```

## CI/CD

| Workflow | Status |
|----------|--------|
| CI | ![CI](https://github.com/earnerbaymalay/PROJECT_NAME/workflows/CI/badge.svg) |
| Release | ![Release](https://github.com/earnerbaymalay/PROJECT_NAME/workflows/Release/badge.svg) |
| Pages | ![Pages](https://github.com/earnerbaymalay/PROJECT_NAME/workflows/Deploy%20Pages/badge.svg) |

## License

MIT OR Apache-2.0

---

<div align="center">

**Built with Nocta aesthetic** · Fonts: Geist / Geist Mono · Accent: `#d4a843` · Dark-only, always

Deployed via GitHub Pages from [`earnerbaymalay/PROJECT_NAME`](https://github.com/earnerbaymalay/PROJECT_NAME)

</div>