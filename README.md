# LootBox

Secure, encrypted credential storage for the terminal. Each file holds one or more key-value secrets encrypted with AES-256-GCM and a password you choose.

## Features

- **AES-256-GCM encryption** with Argon2 password-based key derivation
- **Interactive terminal UI** (ratatui) — arrow-key navigation, masked values, toggle reveal with Tab
- **CLI subcommands** for scripting: `save`, `list`, `read`, `update`, `remove`, `env`
- **Shell export** — `env` output is a valid shell script: `source <(lootbox env file.enc)`
- **MCP server mode** — exposes all commands as tools for Claude Code and other AI agents
- **File permissions 0o600** (owner-only read/write) on Unix
- 274 automated tests

## Requirements

- Rust 1.87+ (edition 2024)
- macOS or Linux

## Installation

```bash
git clone https://github.com/your-username/lootbox.git
cd lootbox
cargo build --release
cp target/release/lootbox ~/.local/bin/
```

Or run directly without installing:

```bash
cargo run --release -- <command> <file>
```

## Usage

### TUI — interactive mode

Pass only a file path (no subcommand) to open the interactive terminal UI:

```bash
lootbox credentials.enc
```

Enter your file password to unlock, then:

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate the credential list |
| `A` | Add a new credential |
| `U` | Update the highlighted credential |
| `R` | Remove the highlighted credential |
| `S` | Show the highlighted credential (value revealed with Tab) |
| `E` | Export all credentials as shell `export` statements |
| `Q` / `Esc` | Quit |

Secret values are masked as `●●●●●●` by default. Press **Tab** while editing or viewing to toggle visibility.

---

### CLI commands

All CLI commands prompt for the file password interactively (input is hidden).

#### `save` — add a new credential

```bash
lootbox save myfile.enc
# prompts: password, secret key, secret value
```

Creates the file if it does not exist. Returns an error if the file already exists — use `update` to modify existing credentials.

#### `list` — show all credentials

```bash
lootbox list myfile.enc
# prompts: password
```

Prints each credential with its ID and key in plain text; values are masked as `**********`.

#### `read` — show one credential in plain text

```bash
lootbox read myfile.enc
# prompts: password, credential ID
```

Prints the key and value of the specified credential in full.

#### `update` — edit a credential

```bash
lootbox update myfile.enc
# prompts: password, credential ID, new key (optional), new value (optional)
```

Press Enter without input to keep the current key or value unchanged.

#### `remove` — delete a credential

```bash
lootbox remove myfile.enc
# prompts: password, credential ID
```

Credentials that follow the removed one shift down by one position.

#### `env` — export as shell variables

```bash
lootbox env myfile.enc
# prompts: password (hidden, printed as a comment)
```

Prints `export KEY='value'` statements. Keys are uppercased with spaces replaced by underscores. Values are single-quoted and properly escaped for POSIX shells.

Load credentials directly into the current shell:

```bash
source <(lootbox env myfile.enc)
```

Inspect before loading:

```bash
lootbox env myfile.enc
```

---

### MCP server — AI agent integration

`lootbox mcp` starts a JSON-RPC 2.0 server on stdin/stdout, exposing all commands as tools for Claude Code and other MCP-compatible AI agents.

**Claude Code setup** — add to `.claude/mcp.json` (or configure via `claude mcp add`):

```json
{
  "mcpServers": {
    "lootbox": {
      "command": "/path/to/lootbox",
      "args": ["mcp"]
    }
  }
}
```

Available tools:

| Tool | Description |
|------|-------------|
| `save_credential` | Save a new key-value credential |
| `list_credentials` | List all credentials (values masked) |
| `read_credential` | Read one credential by ID in plain text |
| `update_credential` | Update key and/or value of a credential |
| `remove_credential` | Delete a credential by ID |
| `generate_env_vars` | Generate shell `export` statements |

---

## Encrypted file format

Each `.enc` file uses a compact binary layout:

```
[16 bytes: Argon2 salt][12 bytes: AES-GCM nonce][remaining: AES-256-GCM ciphertext]
```

The ciphertext decrypts to a JSON array of `{ "key": "…", "value": "…" }` objects. Salt and nonce are generated randomly per save, so the same password and data produce different ciphertext on every write.

## Security

- Passwords are never stored — used only for key derivation via Argon2
- Every write uses a fresh random salt and nonce
- File mode `0o600` restricts access to the owning user (Unix)
- Secret values are never written to disk unencrypted

---

## Development

### Prerequisites

- Rust 1.87+ — install or update with `rustup`:

  ```bash
  rustup update stable
  ```

### Build

```bash
cargo build           # debug build
cargo build --release # optimized build
```

### Run during development

```bash
cargo run -- save myfile.enc
cargo run -- list myfile.enc
cargo run -- myfile.enc       # TUI mode
```

### Tests

274 tests across 7 integration test suites plus unit tests embedded in each module.

```bash
cargo test                              # all tests
cargo test --test save_command_tests    # one integration suite
cargo test --test list_command_tests
cargo test --test read_command_tests
cargo test --test update_command_tests
cargo test --test remove_command_tests
cargo test --test env_command_tests
cargo test --test mcp_command_tests
cargo test --lib                        # unit tests only
cargo test -- --nocapture               # show stdout from tests
cargo test test_save_binary_format      # run by name pattern
```

### Module structure

| Module | Responsibility |
|--------|----------------|
| `crypto.rs` | Argon2 key derivation, AES-256-GCM encrypt/decrypt, random salt and nonce generation |
| `validation.rs` | Password, key, and value validation rules |
| `storage.rs` | Binary file format encoding/decoding; all credential CRUD operations |
| `tui.rs` | ratatui + crossterm interactive terminal UI |
| `mcp.rs` | JSON-RPC 2.0 MCP server message handler |
| `main.rs` | CLI entry point (clap); TUI dispatch when no subcommand is given |
| `lib.rs` | Public API re-exports |

### Validation rules

**Password:** minimum 8 characters; no leading or trailing whitespace; cannot be only whitespace.

**Secret key / value:** required (non-empty); cannot be only whitespace; exact bytes are stored without trimming.

## License

See [LICENSE](LICENSE).
