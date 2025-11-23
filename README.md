# Scow

A minimalist, "Transmit-like" terminal cleanup and transfer staging tool built with Rust.

## Overview

Scow is a terminal user interface (TUI) application designed to help you:
- Browse and visualize your filesystem with file sizes
- Stage files for deletion or transfer
- Execute operations with confirmation
- Transfer files to remote destinations via rsync

## Features

- **Split-pane TUI**: View file tree on the left, staging area on the right
- **Size-sorted display**: Files and directories sorted by size (largest first)
- **Staging operations**: Mark files for deletion or move to another location
- **Visual feedback**: Color-coded staging (red for delete, yellow for move)
- **Safe operations**: Confirmation dialog before executing staged operations
- **Remote transfers**: Support for rsync-based remote file transfers
- **Automatic rescan**: Filesystem updates after operations complete

## Installation

### Prerequisites

- Rust 1.70 or later
- For remote transfers: `rsync` installed on your system

### Building from source

```bash
git clone https://github.com/mitchins/scow.git
cd scow
cargo build --release
```

The binary will be available at `target/release/scow`.

## Usage

### Basic Usage

Run scow in the current directory:
```bash
scow
```

Or specify a directory to scan:
```bash
scow /path/to/directory
```

### Keyboard Shortcuts

- **q**: Quit the application
- **s**: Scan/rescan the current directory
- **Space**: Mark the current path for deletion (toggle)
- **m**: Open the destination picker to move files
- **c**: Clear all staged operations
- **Enter**: Commit staged operations (shows confirmation dialog)
- **y**: Confirm execution (in confirmation dialog)
- **n** or **Esc**: Cancel operation

### Configuration

Scow looks for a configuration file at `~/.config/scow/config.toml` (or `.scow.toml` in the current directory as a fallback).

Example configuration:

```toml
[[destinations]]
name = "Home"
path = "~"

[[destinations]]
name = "External Backup"
path = "/mnt/backup"

[[destinations]]
name = "Remote Server"
path = "user@server.com:/data/archive"
```

The `destinations` array defines locations where you can move files. Remote destinations using SSH syntax will automatically use `rsync` for transfer.

## Architecture

### Core Components

- **Scanner**: Parallel filesystem walking using `jwalk`
- **UI**: Split-pane interface using `ratatui`
- **Staging**: In-memory operation tracking with size calculations
- **Worker**: Execution engine for file operations (delete, move, rsync)

### Data Flow

1. User launches app → Scanner reads filesystem → Builds FileNode tree
2. User navigates and stages operations → Updates Staging hashmap
3. User commits → Confirmation dialog → Worker executes operations
4. Operations complete → Automatic rescan → Updated UI

## Development

### Running Tests

```bash
cargo test
```

All tests include:
- Unit tests for staging logic
- Integration tests for scanner
- Integration tests for worker operations

### Project Structure

```
scow/
├── src/
│   ├── main.rs       # Entry point and event loop
│   ├── app.rs        # Application state and data structures
│   ├── ui.rs         # Ratatui rendering logic
│   ├── events.rs     # Keyboard event handling
│   ├── scanner.rs    # Filesystem scanning
│   ├── worker.rs     # Operation execution
│   └── config.rs     # Configuration management
├── Cargo.toml
└── README.md
```

## Technical Details

### Dependencies

- **ratatui**: Terminal UI framework
- **crossterm**: Terminal manipulation and event handling
- **jwalk**: Parallel directory walking
- **serde/toml**: Configuration serialization
- **human_bytes**: Size formatting
- **anyhow**: Error handling
- **directories**: Cross-platform path utilities

### Design Decisions

1. **Immutable tree structure**: The FileNode tree is rebuilt on each scan rather than mutated, ensuring consistency
2. **Overlay staging**: Operations are tracked separately from the tree, allowing easy undo/clear
3. **Size-first sorting**: Children are sorted by size (descending) to highlight cleanup opportunities
4. **Synchronous MVP**: Initial implementation uses synchronous scanning and execution for simplicity

## Future Enhancements

- Async scanning with progress indicator
- Interactive tree navigation with cursor movement
- Collapsible/expandable directories
- Multi-select support
- Progress bars for long-running operations
- Filtering and search
- Dry-run mode
- Operation history/undo

## License

Licensed under the Apache License, Version 2.0. See LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
