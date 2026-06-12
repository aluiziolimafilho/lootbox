# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

LootBox is a secure credential storage CLI tool written in Rust. It stores a single credential (key-value pair) per encrypted file using AES-256-GCM encryption with Argon2 password-based key derivation.

## Build and Test Commands

```bash
# Build the project
cargo build
cargo build --release

# Run all tests (65+ tests total)
cargo test

# Run specific test suites
cargo test --test save_command_tests
cargo test --test list_command_tests
cargo test --lib  # Run unit tests only

# Run specific test by name pattern
cargo test test_save_binary_format

# Run tests with output
cargo test -- --nocapture

# Run the CLI
cargo run -- save <filename>
cargo run -- list <filename>
./target/release/lootbox save <filename>
./target/release/lootbox list <filename>
```

## Architecture

### Module Structure

The codebase is organized into focused modules:

- **`crypto.rs`**: Low-level cryptography operations (Argon2 key derivation, AES-256-GCM encryption/decryption, random salt/nonce generation)
- **`validation.rs`**: Input validation for passwords, secret keys, and secret values
- **`storage.rs`**: High-level credential save/load operations and binary file format handling
- **`main.rs`**: CLI interface using clap, interactive prompts with rpassword
- **`lib.rs`**: Public API exports

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
