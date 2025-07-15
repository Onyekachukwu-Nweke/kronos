# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

Kronos is a database backup utility tool written in Rust that supports SQLite backups (with planned support for MySQL, PostgreSQL, and MongoDB). It creates compressed tar.gz backups and stores them locally or in cloud storage.

## Development Commands

### Build and Run
```bash
cargo build                          # Build the project
cargo run -- backup                 # Run with default config (config.toml)
cargo run -- backup --config path/to/config.toml  # Run with custom config
```

### Testing and Code Quality
```bash
cargo test                          # Run tests
cargo check                         # Check code without building
cargo clippy                        # Run linter
cargo fmt                           # Format code
```

## Architecture

### Core Components

- **main.rs**: Entry point with CLI argument parsing using clap
- **config.rs**: Configuration management with TOML parsing for database connections, storage settings, and scheduling
- **backup/**: Core backup logic
  - `performer.rs`: Main backup orchestrator that handles different database types
- **database/**: Database connection framework and implementations
  - `connection.rs`: Core `DatabaseConnection` trait and factory
  - `sqlite.rs`: SQLite backup using rusqlite's backup API
  - `mysql.rs`: MySQL backup using mysqldump
  - `postgres.rs`: PostgreSQL backup using pg_dump
  - `mongodb.rs`: MongoDB backup using mongodump
- **storage/**: Storage backends for backup files
  - `local.rs`: Local filesystem storage with compression
- **utils/**: Utility functions
  - `compression.rs`: Directory compression to tar.gz format
- **error.rs**: Centralized error handling
- **logger.rs**: Logging configuration

### Configuration Structure

The application uses TOML configuration files with these sections:
- `databases`: Database connection settings (mysql, postgres, sqlite, mongodb)
- `storage`: Storage backend configuration (local path or S3 settings)
- `schedule`: Optional cron scheduling (planned feature)

### Database Support Status

- **SQLite**: Fully implemented using rusqlite backup API
- **MySQL**: Implemented using mysqldump command-line tool
- **PostgreSQL**: Implemented using pg_dump command-line tool  
- **MongoDB**: Implemented using mongodump command-line tool
- **Pluggable Framework**: All database types use a common `DatabaseConnection` trait

### Key Dependencies

- `clap`: CLI argument parsing
- `tokio`: Async runtime
- `rusqlite`: SQLite database operations
- `serde`/`toml`: Configuration parsing
- `tar`/`flate2`: Compression utilities
- `log`/`env_logger`: Logging
- `async-trait`: Async trait support for database connections
- `serde_json`: JSON parsing for database metadata

## Example Usage

```bash
# Create example config
cp examples/config.toml config.toml

# Run backup with default config
cargo run -- backup

# Run backup with custom config
cargo run -- backup --config /path/to/config.toml
```

The tool will read the configuration, perform backups of specified databases, compress them, and store them in the configured location.