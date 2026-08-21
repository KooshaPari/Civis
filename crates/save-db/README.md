# save-db

> Session-scoped SQLite metadata index for save files.

## Overview

The `save-db` crate manages the metadata associated with Civis save files. It provides a thread-safe, session-scoped SQLite database to index save slots and autosaves. It tracks essential data such as UUID identifiers, session info, tick numbers, and file paths.

This crate ensures that save management is robust and concurrent-safe. It abstracts away the raw SQLite interactions, providing high-level structs for `SaveSlotRecord` and `AutosaveRecord`.

## Features

- Session-scoped SQLite storage for save metadata
- UUID-based slot and autosave identification
- Thread-safe connection handling via Mutex
- Tracking of tick numbers and session timestamps
- High-level record types for easy manipulation

## Usage

```rust
use save_db::*;
```

## Architecture

The `SaveDb` struct wraps a `Mutex<Connection>` to ensure thread safety. It exposes methods to create and query `SaveSlotRecord` and `AutosaveRecord` entries. The schema is designed to be compact and efficient for session-local lookups.

## License

Part of the [Civis](https://github.com/KooshaPari/Civis) project.
