use clap::{Parser, Subcommand};
use lootbox::{get_list_display, read_credential, save_credential};
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
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Save { filename } => handle_save(filename),
        Commands::List { filename } => handle_list(filename),
        Commands::Read { filename } => handle_read(filename),
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
