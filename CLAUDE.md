# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust CLI application called "download_it" that appears to be in early development stages. The project uses Rust edition 2024 and has a modular structure with CLI argument parsing via clap.

## Build and Development Commands

```bash
# Build the project
cargo build

# Run the application
cargo run

# Run in release mode
cargo run --release

# Run tests
cargo test

# Check code without building
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy
```

## Architecture

- **Entry Point**: `src/main.rs` - Contains the main function and module declarations
- **CLI Module**: `src/cli.rs` - Handles command-line argument parsing (referenced but not yet implemented)
- **Dependencies**: 
  - `clap` with derive features for CLI argument parsing

## Development Notes

- The project is currently in initial setup phase with basic "Hello, world!" output
- CLI module is declared but not yet implemented
- Uses Rust edition 2024 (latest edition)
- Standard Rust project structure with Cargo.toml and src/ directory

## TODO List

- Implement a way to resume interrupted downloads
- Allow a safe interruption of a download or multi-download operation
- Display all progress bars at the same time