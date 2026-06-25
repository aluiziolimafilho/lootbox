use clap::{Parser, Subcommand};
use lootbox::{export_credentials_to_csv, generate_env_vars, get_list_display, import_credentials_from_csv, read_credential, remove_credential, save_credential, update_credential};
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "lootbox")]
#[command(about = "A secure credential storage tool with AES-256 encryption", long_about = None)]
#[command(version = concat!(env!("CARGO_PKG_VERSION"), " (", env!("GIT_HASH"), ")"))]
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
    /// Export all credentials to a CSV file (header: key,value)
    ExportCsv {
        /// Path to the encrypted credentials file
        filename: PathBuf,
        /// Path to write the CSV output
        csv_file: PathBuf,
    },
    /// Import credentials from a CSV file into the encrypted vault
    ImportCsv {
        /// Path to the encrypted credentials file
        filename: PathBuf,
        /// Path to the CSV file to read (must have header: key,value)
        csv_file: PathBuf,
    },
    /// Start an MCP server exposing all commands as tools (reads JSON-RPC from stdin)
    Mcp,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let known = ["save","list","read","update","remove","env","export-csv","import-csv","mcp","-h","--help","-V","--version"];
    if args.len() == 2 && !known.contains(&args[1].as_str()) {
        let path = PathBuf::from(&args[1]);
        if let Err(e) = lootbox::tui::run(path) {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
        return;
    }

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Save { filename } => handle_save(filename),
        Commands::List { filename } => handle_list(filename),
        Commands::Read { filename } => handle_read(filename),
        Commands::Update { filename } => handle_update(filename),
        Commands::Remove { filename } => handle_remove(filename),
        Commands::Env { filename } => handle_env(filename),
        Commands::ExportCsv { filename, csv_file } => handle_export_csv(filename, csv_file),
        Commands::ImportCsv { filename, csv_file } => handle_import_csv(filename, csv_file),
        Commands::Mcp => handle_mcp(),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn prompt_line(prompt: &str) -> anyhow::Result<String> {
    print!("{prompt}");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn prompt_optional(prompt: &str) -> anyhow::Result<Option<String>> {
    let s = prompt_line(prompt)?;
    Ok(if s.is_empty() { None } else { Some(s) })
}

fn handle_save(filename: PathBuf) -> anyhow::Result<()> {
    println!("Saving credential to: {}", filename.display());
    println!();

    let password = rpassword::prompt_password("Enter file password: ")?;

    let name = prompt_line("Enter name (required): ")?;
    let secret_key = prompt_line("Enter secret key: ")?;
    let secret_value = rpassword::prompt_password("Enter secret value: ")?;
    let url = prompt_optional("Enter URL (optional, press Enter to skip): ")?;
    let description = prompt_optional("Enter description (optional, press Enter to skip): ")?;

    save_credential(
        &filename,
        &password,
        &secret_key,
        &secret_value,
        Some(&name),
        description.as_deref(),
        url.as_deref(),
    )?;

    println!();
    println!("Credential saved successfully!");

    Ok(())
}

fn handle_list(filename: PathBuf) -> anyhow::Result<()> {
    println!("Reading credential from: {}", filename.display());
    println!();

    let password = rpassword::prompt_password("Enter file password: ")?;
    let display = get_list_display(&filename, &password)?;

    println!();
    println!("{}", display);

    Ok(())
}

fn handle_read(filename: PathBuf) -> anyhow::Result<()> {
    println!("Reading credential from: {}", filename.display());
    println!();

    let password = rpassword::prompt_password("Enter file password: ")?;
    let id_str = prompt_line("Enter credential ID: ")?;

    let credential_id: usize = id_str.parse()
        .map_err(|_| anyhow::anyhow!("Invalid credential ID: must be a number"))?;

    let credential = read_credential(&filename, &password, credential_id)?;

    println!();
    if !credential.name.is_empty() {
        println!("[{}] Name: {}", credential_id, credential.name);
    }
    println!("    Key: {}", credential.key);
    println!("    Value: {}", credential.value);
    if let Some(ref url) = credential.url {
        println!("    URL: {}", url);
    }
    if let Some(ref description) = credential.description {
        println!("    Description: {}", description);
    }

    Ok(())
}

fn handle_update(filename: PathBuf) -> anyhow::Result<()> {
    println!("Updating credential in: {}", filename.display());
    println!();

    let password = rpassword::prompt_password("Enter file password: ")?;
    let id_str = prompt_line("Enter credential ID: ")?;

    let credential_id: usize = id_str.parse()
        .map_err(|_| anyhow::anyhow!("Invalid credential ID: must be a number"))?;

    let current = read_credential(&filename, &password, credential_id)?;

    println!();
    println!("Current credential:");
    if !current.name.is_empty() {
        println!("[{}] Name: {}", credential_id, current.name);
    }
    println!("    Key: {}", current.key);
    println!("    Value: **********");
    if let Some(ref url) = current.url {
        println!("    URL: {}", url);
    }
    if let Some(ref description) = current.description {
        println!("    Description: {}", description);
    }
    println!();

    let new_name = prompt_optional("Enter new name (press Enter to keep current): ")?;
    let new_key_input = prompt_line("Enter new secret key (press Enter to keep current): ")?;
    let new_key = if new_key_input.is_empty() { None } else { Some(new_key_input.as_str()) };

    let new_value_input = rpassword::prompt_password("Enter new secret value (press Enter to keep current): ")?;
    let new_value = if new_value_input.is_empty() { None } else { Some(new_value_input.as_str()) };

    let new_url = prompt_optional("Enter new URL (press Enter to keep current, '-' to clear): ")?;
    let new_url = new_url.as_deref().map(|s| if s == "-" { "" } else { s });

    let new_description = prompt_optional("Enter new description (press Enter to keep current, '-' to clear): ")?;
    let new_description = new_description.as_deref().map(|s| if s == "-" { "" } else { s });

    update_credential(
        &filename,
        &password,
        credential_id,
        new_key,
        new_value,
        new_name.as_deref(),
        new_description,
        new_url,
    )?;

    println!();
    println!("Credential updated successfully!");

    Ok(())
}

fn handle_remove(filename: PathBuf) -> anyhow::Result<()> {
    println!("Removing credential from: {}", filename.display());
    println!();

    let password = rpassword::prompt_password("Enter file password: ")?;
    let id_str = prompt_line("Enter credential ID: ")?;

    let credential_id: usize = id_str.parse()
        .map_err(|_| anyhow::anyhow!("Invalid credential ID: must be a number"))?;

    remove_credential(&filename, &password, credential_id)?;

    println!();
    println!("Credential removed successfully!");

    Ok(())
}

fn handle_env(filename: PathBuf) -> anyhow::Result<()> {
    let password = rpassword::prompt_password("# Enter file password: ")?;
    let id_str = prompt_line("Enter credential ID: ")?;

    let credential_id: usize = id_str
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid credential ID: must be a number"))?;

    let result = generate_env_vars(&filename, &password, credential_id)?;

    println!("# Exporting credential from: {}", filename.display());

    let mut export_lines = String::new();
    for entry in &result.created {
        let line = format!("export {}={}", entry.env_name, shell_escape(&entry.value));
        println!("{}", line);
        export_lines.push_str(&line);
        export_lines.push('\n');
    }

    if !export_lines.is_empty() {
        match arboard::Clipboard::new().and_then(|mut c| c.set_text(export_lines.trim_end().to_string())) {
            Ok(()) => println!("# Copied to clipboard"),
            Err(_) => eprintln!("# Warning: clipboard unavailable"),
        }
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

fn handle_export_csv(filename: PathBuf, csv_file: PathBuf) -> anyhow::Result<()> {
    let password = rpassword::prompt_password("Enter file password: ")?;
    export_credentials_to_csv(&filename, &password, &csv_file)?;
    println!("Exported to {}", csv_file.display());
    Ok(())
}

fn handle_import_csv(filename: PathBuf, csv_file: PathBuf) -> anyhow::Result<()> {
    let password = rpassword::prompt_password("Enter file password: ")?;
    let count = import_credentials_from_csv(&filename, &password, &csv_file)?;
    println!("Imported {} credential(s).", count);
    Ok(())
}

/// Wraps a value in single quotes and escapes any single quotes within it,
/// producing a POSIX-safe shell string for use in `export KEY='value'`.
fn shell_escape(value: &str) -> String {
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
