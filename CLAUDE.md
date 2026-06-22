# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

LootBox is a secure credential storage tool written in Rust. It stores a single credential (key-value pair) per encrypted file using AES-256-GCM encryption with Argon2 password-based key derivation.

The repo is a Cargo workspace with two members:

- **`lootbox-core/`** — package name `lootbox`, the CLI + Ratatui TUI + business logic. Produces the `lootbox` binary.
- **`lootbox-gui/`** — a native desktop UI built with [GPUI](https://www.gpui.rs) + [gpui-component](https://github.com/longbridge/gpui-component), depending on `lootbox-core` via a path dependency. Holds no business logic of its own.

## Build and Test Commands

```bash
# Build the CLI/TUI (lootbox-core) — use -p so GPU deps aren't pulled in
cargo build -p lootbox
cargo build --release -p lootbox

# Run all CLI/TUI tests (65+ tests total)
cargo test -p lootbox

# Run specific test suites
cargo test -p lootbox --test save_command_tests
cargo test -p lootbox --test list_command_tests
cargo test -p lootbox --lib  # Run unit tests only

# Run specific test by name pattern
cargo test -p lootbox test_save_binary_format

# Run tests with output
cargo test -p lootbox -- --nocapture

# Run the CLI
cargo run -p lootbox -- save <filename>
cargo run -p lootbox -- list <filename>
./target/release/lootbox save <filename>
./target/release/lootbox list <filename>

# Build/run/test the GUI
cargo build -p lootbox-gui
cargo run -p lootbox-gui
cargo test -p lootbox-gui   # headless #[gpui::test] suite
```

## Architecture

### Module Structure

`lootbox-core/src/` is organized into focused modules:

- **`crypto.rs`**: Low-level cryptography operations (Argon2 key derivation, AES-256-GCM encryption/decryption, random salt/nonce generation)
- **`validation.rs`**: Input validation for passwords, secret keys, and secret values
- **`storage.rs`**: High-level credential save/load operations and binary file format handling
- **`tui.rs`**: ratatui + crossterm interactive terminal UI
- **`mcp.rs`**: JSON-RPC 2.0 MCP server message handler
- **`main.rs`**: CLI interface using clap, interactive prompts with rpassword
- **`lib.rs`**: Public API exports

`lootbox-gui/src/` mirrors the TUI's screen set (NewFileConfirm, Password, CredentialList, Add/Update form, RemoveConfirm, ReadView, EnvVars, CSV export/import) as GPUI screens, calling the same `lootbox::*` functions re-exported from `lootbox-core`'s `lib.rs` — no storage/crypto/validation logic is duplicated in the GUI crate.

### Binary File Format

The encrypted file uses a compact binary format (NOT JSON):

```
[16 bytes: salt][12 bytes: nonce][remaining bytes: encrypted data]
```

The encrypted data contains a JSON-serialized `Credential` struct with `key` and `value` fields.

### Validation Rules

**Password requirements:**
- Minimum 8 characters
- Cannot be only whitespace
- Cannot start or end with whitespace
- Whitespace in the middle is allowed

**Secret key and value requirements:**
- Required (non-empty)
- Cannot be only whitespace
- No trimming is performed (exact values are stored)

### Storage Behavior

- **Save command**: Always creates a NEW file. Returns error if file already exists (no overwriting).
- **List command**: Decrypts and displays credential with key in plain text and value masked as exactly 10 asterisks (`**********`).
- **File permissions**: On Unix systems, files are created with mode 0o600 (read/write for owner only).

### Cryptographic Flow

1. **Saving**:
   - Generate random 16-byte salt
   - Derive 32-byte AES key from password using Argon2
   - Generate random 12-byte nonce
   - Serialize credential to JSON
   - Encrypt JSON with AES-256-GCM
   - Write: salt + nonce + encrypted_data as binary

2. **Loading**:
   - Read binary file
   - Extract salt (first 16 bytes), nonce (next 12 bytes), encrypted data (remaining)
   - Derive key from password and salt
   - Decrypt data with AES-256-GCM
   - Deserialize credential from JSON

### Test Organization

- **Unit tests**: Embedded in source modules (`crypto.rs`, `validation.rs`, `storage.rs`)
- **Integration tests**:
  - `tests/save_command_tests.rs` - 27 tests covering save functionality, validation, encryption, binary format
  - `tests/list_command_tests.rs` - 25 tests covering list functionality (currently commented out)

Tests use `tempfile` crate for temporary directories to avoid file conflicts.

## Key Implementation Details

### Why Binary Format?

The file format was changed from JSON to binary to reduce file size and complexity. The outer structure is binary (salt + nonce + encrypted_data), while the credential payload inside the encrypted data remains JSON for flexibility.

### Error Handling

- All functions return `anyhow::Result<T>` for consistent error handling
- Cryptographic failures (wrong password, corrupted file) return descriptive errors
- Validation errors clearly state what requirement was violated

### Security Considerations

- Passwords are never stored, only used for key derivation
- Each file uses unique random salt and nonce (same password + data = different ciphertext)
- File permissions restrict access to owner only (Unix)
- Password input is hidden using `rpassword`
