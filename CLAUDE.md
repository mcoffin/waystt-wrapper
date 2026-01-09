# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
cargo build          # Debug build
cargo build --release # Release build
cargo run            # Run the application
cargo run -- --help  # Show CLI options
```

## What This Project Does

waystt-wrapper is a GTK4 Wayland overlay that wraps the `waystt` speech-to-text tool. It displays a visual indicator (microphone icon) while waystt is running and handles graceful shutdown via Escape key (sends SIGUSR1 to the child process).

## Architecture

The application follows this flow:
1. **main.rs** - GTK Application setup, event loop, and lifecycle management
2. **config.rs** - CLI argument parsing (clap) and configuration types
3. **overlay.rs** - GTK4 Layer Shell window creation and positioning
4. **process.rs** - Child process spawning and signal handling (SIGUSR1 for graceful stop)
5. **error.rs** - Centralized error types using thiserror

Key interaction pattern: Escape key triggers SIGUSR1 to child process, then waits for child exit before closing the GTK window. The exit code from the child process propagates to the wrapper's exit code.

## Dependencies

- **gtk4** + **gtk4-layer-shell** - Wayland overlay windows
- **nix** - Unix signal handling (SIGUSR1)
- **clap** - CLI argument parsing
- **thiserror** - Error type definitions
- **tracing** - Logging (controlled via RUST_LOG env var)
