# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Trinket is a high-performance Windows system tray application for lightning-fast text snippet storage and retrieval. It provides global hotkeys for instant access to snippet management:

- **WIN+CTRL+PgUp** - Opens add snippet window with text editor
- **WIN+CTRL+PgDown** - Opens searchable snippet browser

## Development Commands

### Building and Running
```bash
# Build debug version
cargo build

# Build release version (optimized for size and performance)
cargo build --release

# Run the application - DO NOT RUN THIS, INSTEAD, ASK THE USER TO RUN IT
cargo run

# Run with logging
RUST_LOG=info cargo run
```

### Development Tools
```bash
# Check code formatting
cargo fmt --check

# Format code
cargo fmt

# Run clippy lints
cargo clippy

# Check for security vulnerabilities
cargo audit  # (if installed via cargo install cargo-audit)
```

## Architecture

### Core Components

**Main Application (`src/app.rs`)**
- `TrinketApp` - Main application state and eframe::App implementation
- `AppMode` enum - Controls whether app is hidden, adding snippets, or browsing snippets
- Manages hotkey events and coordinates between UI windows and storage

**Storage System (`src/storage/`)**
- `FileStorage` - Handles saving/loading snippets as individual .txt files
- `Snippet` struct - Core data model with content, preview, timestamps, and file path
- Snippets stored in `%LOCALAPPDATA%/trinket/snippets/` directory
- Files named with UUID + .txt extension for uniqueness

**UI Modules (`src/ui/`)**
- `AddWindowState` - Text editor window for creating new snippets
- `GetWindowState` - Searchable list/table for browsing and selecting snippets
- Built with egui immediate mode GUI framework

**System Integration**
- `src/hotkeys.rs` - Global hotkey event definitions
- `src/clipboard.rs` - Clipboard operations for copying selected snippets
- `src/main.rs` - Entry point, system tray setup, and hotkey registration

### Data Flow

1. Global hotkeys trigger `HotkeyEvent::Add` or `HotkeyEvent::Get`
2. Events change `AppMode` and show appropriate UI window
3. Add mode: User enters text → `FileStorage::save_snippet()` → Updates in-memory snippets list
4. Get mode: User searches/selects snippet → Copy to clipboard via `copy_to_clipboard()`
5. Both modes return to `AppMode::Hidden` when complete

### File Storage

- Snippets stored as individual `.txt` files in user's local app data directory
- Atomic writes using `tempfile` crate to prevent corruption
- Files loaded on startup into in-memory `Vec<Snippet>` for fast searching
- No database - simple file-based storage for portability

### Performance Optimizations

- Release profile uses `opt-level = "z"` (optimize for size)
- LTO enabled, symbols stripped, single codegen unit
- Snippet previews generated from first 3 lines (max 200 chars)
- Simple substring search for snippet filtering

## Platform Requirements

- **Windows only** - Uses Windows-specific dependencies for global hotkeys
- Requires system tray support
- Stores data in `%LOCALAPPDATA%/trinket/snippets/`

## Key Dependencies

- `eframe` + `egui` - Cross-platform GUI framework
- `global-hotkey` - System-wide hotkey registration
- `tray-icon` - System tray integration  
- `arboard` - Cross-platform clipboard access
- `tempfile` - Atomic file operations
- `uuid` - Unique snippet identifiers

## Testing Notes

No test framework is currently configured. The application is designed for manual testing:

1. Test hotkey registration and window showing
2. Test snippet creation, storage, and loading
3. Test search functionality and clipboard operations
4. Test system tray behavior and window focus handling