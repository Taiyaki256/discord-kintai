# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Discord-Kintai is a Discord bot for attendance tracking (勤怠記録) written in Rust. The bot allows users to track work hours via Discord slash commands with features for time corrections, reporting, and admin management.

## Key Technologies

- **Language**: Rust (edition 2025)
- **Discord Framework**: Serenity v0.12 + Poise v0.6 for slash commands and interactions
- **Database**: SQLite with SQLx for async database operations
- **Async Runtime**: Tokio with multi-thread runtime
- **Time Handling**: chrono for date/time operations
- **Logging**: tracing + tracing-subscriber
- **Config**: dotenv for environment variable management

## Development Commands

```bash
# Build the project
cargo build

# Run in development mode
cargo run

# Run tests
cargo test

# Format code
cargo fmt

# Run linter
cargo clippy

# Release build
cargo build --release

# Setup database (when implemented)
cargo run --bin setup_db
```

## Environment Setup

1. Copy `.env.example` to `.env` and configure:
   - `DISCORD_TOKEN`: Bot token from Discord Developer Portal
   - `DATABASE_URL`: SQLite database path (default: `sqlite:attendance.db`)
   - `RUST_LOG`: Log level (info, debug, warn, error)
   - `ADMIN_ROLE_ID`: Discord role ID for admin commands (optional)

2. Create Discord application at Discord Developer Portal with bot permissions:
   - `applications.commands` (for slash commands)
   - `bot` (basic bot functionality)

## Architecture Overview

The project follows a modular architecture with planned structure:

- **`src/main.rs`**: Application entry point
- **`src/bot/`**: Discord bot implementation
  - `commands/`: Slash command handlers (attendance, status, reports, admin)
  - `handlers/`: Discord event handlers
  - `interactions/`: Button/modal interaction handlers for status corrections
- **`src/database/`**: Database layer
  - `models.rs`: Data structures for users, attendance_records, work_sessions
  - `queries.rs`: Database query functions
  - `migrations.rs`: Database schema migrations
- **`src/utils/`**: Utility functions for time calculations, formatting, validation
- **`src/config.rs`**: Configuration management

## Core Database Schema

Three main tables:
- **`users`**: Discord user information
- **`attendance_records`**: Individual start/end records with modification tracking
- **`work_sessions`**: Aggregated work sessions for reporting

## Key Features to Implement

1. **Basic Commands**: `/start`, `/end`, `/status`
2. **Status Command**: Interactive UI with buttons/modals for time corrections, deletions, history viewing
3. **Reporting**: `/daily`, `/weekly`, `/monthly` commands
4. **Admin Features**: User-specific reports and data export
5. **Time Correction System**: Modal dialogs for HH:MM time input with modification history

## Interaction Patterns

The bot heavily uses Discord's interaction system:
- **Buttons**: For status command menu options
- **Modals**: For time input (HH:MM format)
- **Select Menus**: For date/history selection
- **Multi-step Interactions**: Confirmation dialogs for deletions

## Development Notes

- The project is in early development stage (main.rs currently contains placeholder code)
- Comprehensive Japanese documentation exists in README.md with detailed feature specifications
- Follow async/await patterns throughout (tokio runtime)
- Use proper error handling with anyhow for error propagation
- Implement proper logging with tracing for debugging
- Database operations should be transactional where appropriate
- All user inputs require validation, especially time formats (HH:MM)