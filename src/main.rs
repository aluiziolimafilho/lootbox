use clap::{Parser, Subcommand};
use lootbox::{generate_env_vars, get_list_display, read_credential, remove_credential, save_credential, update_credential};
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "lootbox")]
#[command(about = "A secure credential storage tool with AES-256 encryption", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Save a new credential to an encrypted file
    Save {
        /// The filename to save the encrypted credentials
        filename: PathBuf,
    },
    /// List credentials from an encrypted file
    List {
        /// The filename to read the encrypted credentials from
        filename: PathBuf,
    },
    /// Read a specific credential by ID from an encrypted file
    Read {
        /// The filename to read the encrypted credentials from
        filename: PathBuf,
    },
    /// Update a specific credential by ID in an encrypted file
    Update {
        /// The filename containing the encrypted credentials
        filename: PathBuf,
    },
    /// Remove a specific credential by ID from an encrypted file
    Remove {
        /// The filename containing the encrypted credentials
        filename: PathBuf,
    },
    /// Export credentials as environment variables
    Env {
        /// The filename containing the encrypted credentials
        filename: PathBuf,
    },
    /// Start an MCP server exposing all commands as tools (reads JSON-RPC from stdin)
    Mcp,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Save { filename } => handle_save(filename),
        Commands::List { filename } => handle_list(filename),
        Commands::Read { filename } => handle_read(filename),
        Commands::Update { filename } => handle_update(filename),
        Commands::Remove { filename } => handle_remove(filename),
        Commands::Env { filename } => handle_env(filename),
        Commands::Mcp => handle_mcp(),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn handle_save(filename: PathBuf) -> anyhow::Result<()> {
    println!("Saving credential to: {}", filename.display());
    println!();

    // Request password (hidden input)
    let password = rpassword::prompt_password("Enter file password: ")?;

    // Request secret_key
    print!("Enter secret key: ");
    io::stdout().flush()?;
    let mut secret_key = String::new();
    io::stdin().read_line(&mut secret_key)?;
    let secret_key = secret_key.trim();

    // Request secret_value (hidden input)
    let secret_value = rpassword::prompt_password("Enter secret value: ")?;

    // Save credential
    save_credential(&filename, &password, secret_key, &secret_value)?;

    println!();
    println!("Credential saved successfully!");

    Ok(())
}

fn handle_list(filename: PathBuf) -> anyhow::Result<()> {
    println!("Reading credential from: {}", filename.display());
    println!();

    // Request password (hidden input)
    let password = rpassword::prompt_password("Enter file password: ")?;

    // Get and display credential
    let display = get_list_display(&filename, &password)?;

    println!();
    println!("{}", display);

    Ok(())
}

fn handle_read(filename: PathBuf) -> anyhow::Result<()> {
    println!("Reading credential from: {}", filename.display());
    println!();

    // Request password (hidden input)
    let password = rpassword::prompt_password("Enter file password: ")?;

    // Request credential ID
    print!("Enter credential ID: ");
    io::stdout().flush()?;
    let mut id_input = String::new();
    io::stdin().read_line(&mut id_input)?;
    let id_str = id_input.trim();

    // Parse ID as usize
    let credential_id: usize = id_str.parse()
        .map_err(|_| anyhow::anyhow!("Invalid credential ID: must be a number"))?;

    // Read credential
    let credential = read_credential(&filename, &password, credential_id)?;

    // Display credential with ID, key, and value in plain text
    println!();
    println!("[{}] Key: {}", credential_id, credential.key);
    println!("Value: {}", credential.value);

    Ok(())
}

fn handle_update(filename: PathBuf) -> anyhow::Result<()> {
    println!("Updating credential in: {}", filename.display());
    println!();

    // Request password (hidden input)
    let password = rpassword::prompt_password("Enter file password: ")?;

    // Request credential ID
    print!("Enter credential ID: ");
    io::stdout().flush()?;
    let mut id_input = String::new();
    io::stdin().read_line(&mut id_input)?;
    let id_str = id_input.trim();

    // Parse ID as usize
    let credential_id: usize = id_str.parse()
        .map_err(|_| anyhow::anyhow!("Invalid credential ID: must be a number"))?;

    // Read current credential to display
    let current = read_credential(&filename, &password, credential_id)?;

    // Display current credential with masked value
    println!();
    println!("Current credential:");
    println!("[{}] Key: {}", credential_id, current.key);
    println!("Value: **********");
    println!();

    // Request new key (empty means keep current)
    print!("Enter new secret key (press Enter to keep current): ");
    io::stdout().flush()?;
    let mut new_key_input = String::new();
    io::stdin().read_line(&mut new_key_input)?;
    let new_key_trimmed = new_key_input.trim();
    let new_key = if new_key_trimmed.is_empty() {
        None
    } else {
        Some(new_key_trimmed)
    };

    // Request new value (hidden, empty means keep current)
    let new_value_input = rpassword::prompt_password("Enter new secret value (press Enter to keep current): ")?;
    let new_value = if new_value_input.is_empty() {
        None
    } else {
        Some(new_value_input.as_str())
    };

    // Update credential
    update_credential(&filename, &password, credential_id, new_key, new_value)?;

    println!();
    println!("Credential updated successfully!");

    Ok(())
}

fn handle_remove(filename: PathBuf) -> anyhow::Result<()> {
    println!("Removing credential from: {}", filename.display());
    println!();

    // Request password (hidden input)
    let password = rpassword::prompt_password("Enter file password: ")?;

    // Request credential ID
    print!("Enter credential ID: ");
    io::stdout().flush()?;
    let mut id_input = String::new();
    io::stdin().read_line(&mut id_input)?;
    let id_str = id_input.trim();

    // Parse ID as usize
    let credential_id: usize = id_str.parse()
        .map_err(|_| anyhow::anyhow!("Invalid credential ID: must be a number"))?;

    // Remove credential
    remove_credential(&filename, &password, credential_id)?;

    println!();
    println!("Credential removed successfully!");

    Ok(())
}

fn handle_env(filename: PathBuf) -> anyhow::Result<()> {
    let password = rpassword::prompt_password("# Enter file password: ")?;

    let result = generate_env_vars(&filename, &password)?;

    println!("# Exporting credentials from: {}", filename.display());

    for entry in &result.created {
        println!("export {}={}", entry.env_name, shell_escape(&entry.value));
    }

    if !result.created.is_empty() {
        println!("# Created ({}):", result.created.len());
        for entry in &result.created {
            println!("#   {} -> {}", entry.original_key, entry.env_name);
        }
    }

    if !result.invalid.is_empty() {
        println!("# Skipped ({}):", result.invalid.len());
        for entry in &result.invalid {
            println!("#   {} - {}", entry.original_key, entry.reason);
        }
    }

    Ok(())
}

/// Wraps a value in single quotes and escapes any single quotes within it,
/// producing a POSIX-safe shell string for use in `export KEY='value'`.
fn shell_escape(value: &str) -> String {
    // Replace every ' with '\'' (end quote, literal quote, reopen quote)
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn handle_mcp() -> anyhow::Result<()> {
    use std::io::{BufRead, BufReader, Write};

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let reader = BufReader::new(stdin.lock());

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let response = lootbox::mcp::handle_mcp_message(&line);
        if !response.is_empty() {
            let mut out = stdout.lock();
            writeln!(out, "{}", response)?;
            out.flush()?;
        }
    }

    Ok(())
}
