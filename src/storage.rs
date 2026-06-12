use crate::crypto::{decrypt, derive_key, encrypt, generate_nonce, generate_salt};
use crate::validation::{validate_password, validate_secret_key, validate_secret_value};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Represents a single credential (key-value pair)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Credential {
    pub key: String,
    pub value: String,
}

/// Binary file format:
/// - Salt: 16 bytes
/// - Nonce: 12 bytes
/// - Encrypted data: remaining bytes (contains JSON array of credentials)
const SALT_SIZE: usize = 16;
const NONCE_SIZE: usize = 12;

/// Saves a credential to an encrypted file
/// If file exists, validates password and appends the credential
/// If file doesn't exist, creates a new file
///
/// # Arguments
/// * `file_path` - Path to the encrypted file
/// * `password` - Password to encrypt the file (min 8 chars, validated)
/// * `secret_key` - The credential key (required, non-empty)
/// * `secret_value` - The credential value (required, non-empty)
///
/// # Returns
/// Ok(()) on success, Err on validation failure or wrong password
pub fn save_credential<P: AsRef<Path>>(
    file_path: P,
    password: &str,
    secret_key: &str,
    secret_value: &str,
) -> Result<()> {
    let file_path = file_path.as_ref();

    // Validate inputs
    validate_password(password)?;
    validate_secret_key(secret_key)?;
    validate_secret_value(secret_value)?;

    // Create new credential
    let new_credential = Credential {
        key: secret_key.to_string(),
        value: secret_value.to_string(),
    };

    // Load existing credentials or start with empty list
    let mut credentials = if file_path.exists() {
        // Validate existing file and load credentials
        list_credentials(file_path, password)?
    } else {
        Vec::new()
    };

    // Append new credential
    credentials.push(new_credential);

    // Serialize all credentials to JSON
    let credentials_json = serde_json::to_vec(&credentials)
        .context("Failed to serialize credentials")?;

    // Generate salt and derive key
    let salt = generate_salt();
    let key = derive_key(password, &salt)?;

    // Generate nonce and encrypt
    let nonce = generate_nonce();
    let encrypted_data = encrypt(&credentials_json, &key, &nonce)?;

    // Create binary file content: salt (16 bytes) + nonce (12 bytes) + encrypted_data
    let mut file_content = Vec::with_capacity(SALT_SIZE + NONCE_SIZE + encrypted_data.len());
    file_content.extend_from_slice(&salt);
    file_content.extend_from_slice(&nonce);
    file_content.extend_from_slice(&encrypted_data);

    // Write to file
    fs::write(file_path, file_content)
        .context("Failed to write encrypted file")?;

    // Set file permissions (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(file_path)?.permissions();
        permissions.set_mode(0o600); // rw-------
        fs::set_permissions(file_path, permissions)?;
    }

    Ok(())
}

/// Reads and decrypts all credentials from an encrypted file
///
/// # Arguments
/// * `file_path` - Path to the encrypted file
/// * `password` - Password to decrypt the file
///
/// # Returns
/// Ok(Vec<Credential>) on success, Err on validation failure or decryption error
pub fn list_credentials<P: AsRef<Path>>(file_path: P, password: &str) -> Result<Vec<Credential>> {
    let file_path = file_path.as_ref();

    // Validate password
    validate_password(password)?;

    // Check if file exists
    if !file_path.exists() {
        bail!("File not found: {}", file_path.display());
    }

    // Read file
    let file_content = fs::read(file_path)
        .context("Failed to read encrypted file")?;

    // Verify minimum file size
    if file_content.len() < SALT_SIZE + NONCE_SIZE {
        bail!("Invalid encrypted file - file is too small or corrupted");
    }

    // Parse binary format: salt (16 bytes) + nonce (12 bytes) + encrypted_data
    let salt: [u8; 16] = file_content[0..SALT_SIZE]
        .try_into()
        .context("Failed to read salt from file")?;

    let nonce: [u8; 12] = file_content[SALT_SIZE..SALT_SIZE + NONCE_SIZE]
        .try_into()
        .context("Failed to read nonce from file")?;

    let encrypted_data = &file_content[SALT_SIZE + NONCE_SIZE..];

    // Derive key from password and salt
    let key = derive_key(password, &salt)?;

    // Decrypt data
    let decrypted_data = decrypt(encrypted_data, &key, &nonce)?;

    // Deserialize credentials array
    let credentials: Vec<Credential> = serde_json::from_slice(&decrypted_data)
        .context("Failed to parse decrypted credential data")?;

    Ok(credentials)
}

/// Gets the display string for listing credentials
/// Shows all credentials with position IDs, keys in plain text, and values masked with 10 asterisks
///
/// Format: [n] Key: <key> Value: **********
pub fn get_list_display<P: AsRef<Path>>(file_path: P, password: &str) -> Result<String> {
    let credentials = list_credentials(file_path, password)?;

    let mut output = String::new();
    for (index, credential) in credentials.iter().enumerate() {
        let position = index + 1; // 1-indexed
        output.push_str(&format!(
            "[{}] Key: {} Value: **********\n",
            position, credential.key
        ));
    }

    // Remove trailing newline
    if output.ends_with('\n') {
        output.pop();
    }

    Ok(output)
}

/// Reads a specific credential by its position ID
/// Position IDs are 1-indexed (first credential is ID 1)
///
/// # Arguments
/// * `file_path` - Path to the encrypted file
/// * `password` - Password to decrypt the file
/// * `position_id` - The 1-indexed position ID of the credential to read
///
/// # Returns
/// Ok(Credential) with plain text key and value, Err if invalid ID or decryption fails
pub fn read_credential<P: AsRef<Path>>(
    file_path: P,
    password: &str,
    position_id: usize,
) -> Result<Credential> {
    // Load all credentials
    let credentials = list_credentials(file_path, password)?;

    // Validate position_id (must be >= 1 and <= credentials.len())
    if position_id == 0 {
        bail!("Invalid credential ID: ID must be at least 1");
    }

    if position_id > credentials.len() {
        bail!(
            "Invalid credential ID: {} (file contains {} credential{})",
            position_id,
            credentials.len(),
            if credentials.len() == 1 { "" } else { "s" }
        );
    }

    // Return the credential at position_id - 1 (convert from 1-indexed to 0-indexed)
    Ok(credentials[position_id - 1].clone())
}

/// Updates a specific credential by its position ID
/// Position IDs are 1-indexed (first credential is ID 1)
/// If new_key or new_value is None, the current value is kept
/// If new_key or new_value is Some(""), validation will fail
///
/// # Arguments
/// * `file_path` - Path to the encrypted file
/// * `password` - Password to decrypt and re-encrypt the file
/// * `position_id` - The 1-indexed position ID of the credential to update
/// * `new_key` - Optional new key (None = keep current, Some("") = validation error)
/// * `new_value` - Optional new value (None = keep current, Some("") = validation error)
///
/// # Returns
/// Ok(()) on success, Err if invalid ID, validation fails, or decryption fails
pub fn update_credential<P: AsRef<Path>>(
    file_path: P,
    password: &str,
    position_id: usize,
    new_key: Option<&str>,
    new_value: Option<&str>,
) -> Result<()> {
    let file_path = file_path.as_ref();

    // Validate password
    validate_password(password)?;

    // Load all credentials
    let mut credentials = list_credentials(file_path, password)?;

    // Validate position_id (must be >= 1 and <= credentials.len())
    if position_id == 0 {
        bail!("Invalid credential ID: ID must be at least 1");
    }

    if position_id > credentials.len() {
        bail!(
            "Invalid credential ID: {} (file contains {} credential{})",
            position_id,
            credentials.len(),
            if credentials.len() == 1 { "" } else { "s" }
        );
    }

    // Get mutable reference to the credential at position_id - 1
    let credential = &mut credentials[position_id - 1];

    // Update key if provided
    if let Some(key) = new_key {
        validate_secret_key(key)?;
        credential.key = key.to_string();
    }

    // Update value if provided
    if let Some(value) = new_value {
        validate_secret_value(value)?;
        credential.value = value.to_string();
    }

    // Serialize all credentials to JSON
    let credentials_json = serde_json::to_vec(&credentials)
        .context("Failed to serialize credentials")?;

    // Generate salt and derive key
    let salt = generate_salt();
    let key = derive_key(password, &salt)?;

    // Generate nonce and encrypt
    let nonce = generate_nonce();
    let encrypted_data = encrypt(&credentials_json, &key, &nonce)?;

    // Create binary file content: salt (16 bytes) + nonce (12 bytes) + encrypted_data
    let mut file_content = Vec::with_capacity(SALT_SIZE + NONCE_SIZE + encrypted_data.len());
    file_content.extend_from_slice(&salt);
    file_content.extend_from_slice(&nonce);
    file_content.extend_from_slice(&encrypted_data);

    // Write to file
    fs::write(file_path, file_content)
        .context("Failed to write encrypted file")?;

    // Set file permissions (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(file_path)?.permissions();
        permissions.set_mode(0o600); // rw-------
        fs::set_permissions(file_path, permissions)?;
    }

    Ok(())
}

/// Removes a specific credential by its position ID
/// Position IDs are 1-indexed (first credential is ID 1)
/// After removal, subsequent credentials shift down by one position
///
/// # Arguments
/// * `file_path` - Path to the encrypted file
/// * `password` - Password to decrypt and re-encrypt the file
/// * `position_id` - The 1-indexed position ID of the credential to remove
///
/// # Returns
/// Ok(()) on success, Err if invalid ID, decryption fails, or file not found
pub fn remove_credential<P: AsRef<Path>>(
    file_path: P,
    password: &str,
    position_id: usize,
) -> Result<()> {
    let file_path = file_path.as_ref();

    // Validate password
    validate_password(password)?;

    // Load all credentials
    let mut credentials = list_credentials(file_path, password)?;

    // Validate position_id (must be >= 1 and <= credentials.len())
    if position_id == 0 {
        bail!("Invalid credential ID: ID must be at least 1");
    }

    if position_id > credentials.len() {
        bail!(
            "Invalid credential ID: {} (file contains {} credential{})",
            position_id,
            credentials.len(),
            if credentials.len() == 1 { "" } else { "s" }
        );
    }

    // Remove the credential at position_id - 1 (convert from 1-indexed to 0-indexed)
    credentials.remove(position_id - 1);

    // Serialize all credentials to JSON
    let credentials_json = serde_json::to_vec(&credentials)
        .context("Failed to serialize credentials")?;

    // Generate salt and derive key
    let salt = generate_salt();
    let key = derive_key(password, &salt)?;

    // Generate nonce and encrypt
    let nonce = generate_nonce();
    let encrypted_data = encrypt(&credentials_json, &key, &nonce)?;

    // Create binary file content: salt (16 bytes) + nonce (12 bytes) + encrypted_data
    let mut file_content = Vec::with_capacity(SALT_SIZE + NONCE_SIZE + encrypted_data.len());
    file_content.extend_from_slice(&salt);
    file_content.extend_from_slice(&nonce);
    file_content.extend_from_slice(&encrypted_data);

    // Write to file
    fs::write(file_path, file_content)
        .context("Failed to write encrypted file")?;

    // Set file permissions (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(file_path)?.permissions();
        permissions.set_mode(0o600); // rw-------
        fs::set_permissions(file_path, permissions)?;
    }

    Ok(())
}

/// A successfully mapped credential ready to be exported as an environment variable
#[derive(Debug, Clone, PartialEq)]
pub struct EnvEntry {
    pub original_key: String,
    pub env_name: String,
    pub value: String,
}

/// A credential that could not be exported as an environment variable
#[derive(Debug, Clone, PartialEq)]
pub struct InvalidEntry {
    pub original_key: String,
    pub reason: String,
}

/// Result of generating environment variables from a credential file
#[derive(Debug, Clone, PartialEq)]
pub struct EnvVarsResult {
    pub created: Vec<EnvEntry>,
    pub invalid: Vec<InvalidEntry>,
}

/// Reads all credentials from an encrypted file and maps each one to an environment variable.
///
/// Transformation rules for the env var name:
///   - Letters are uppercased
///   - Spaces become `_`
///   - All other characters are left as-is before validation
///
/// A credential is placed in `invalid` (with a reason) when:
///   - Its transformed name contains a character outside `[A-Z0-9_]`
///   - Its transformed name starts with a digit
///   - Its value contains a null byte (`\0`)
///   - Its transformed name duplicates one already created (first wins)
pub fn generate_env_vars<P: AsRef<Path>>(file_path: P, password: &str) -> Result<EnvVarsResult> {
    let credentials = list_credentials(file_path, password)?;

    let mut created: Vec<EnvEntry> = Vec::new();
    let mut invalid: Vec<InvalidEntry> = Vec::new();

    for credential in credentials {
        let env_name = credential
            .key
            .to_uppercase()
            .replace(' ', "_");

        // Validate env var name: must match [A-Z0-9_]+ and not start with a digit
        let invalid_char = env_name
            .chars()
            .find(|c| !matches!(c, 'A'..='Z' | '0'..='9' | '_'));

        if let Some(ch) = invalid_char {
            invalid.push(InvalidEntry {
                original_key: credential.key,
                reason: format!("invalid character '{}' in environment variable name", ch),
            });
            continue;
        }

        if env_name.starts_with(|c: char| c.is_ascii_digit()) {
            invalid.push(InvalidEntry {
                original_key: credential.key,
                reason: "environment variable name cannot start with a digit".to_string(),
            });
            continue;
        }

        // Reject duplicates — first mapping wins
        if created.iter().any(|e| e.env_name == env_name) {
            invalid.push(InvalidEntry {
                original_key: credential.key,
                reason: format!("duplicate environment variable name '{}' already exists", env_name),
            });
            continue;
        }

        // Reject values containing null bytes — invalid in POSIX shell environment variables
        if credential.value.contains('\0') {
            invalid.push(InvalidEntry {
                original_key: credential.key,
                reason: "value contains a null byte, which is not valid in shell environment variables".to_string(),
            });
            continue;
        }

        created.push(EnvEntry {
            original_key: credential.key,
            env_name,
            value: credential.value,
        });
    }

    Ok(EnvVarsResult { created, invalid })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_dir() -> TempDir {
        tempfile::tempdir().expect("Failed to create temp dir")
    }

    #[test]
    fn test_save_and_list_single_credential() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("test.enc");

        save_credential(&file_path, "password123", "my_key", "my_value").unwrap();

        let credentials = list_credentials(&file_path, "password123").unwrap();
        assert_eq!(credentials.len(), 1);
        assert_eq!(credentials[0].key, "my_key");
        assert_eq!(credentials[0].value, "my_value");
    }

    #[test]
    fn test_save_multiple_credentials() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("test.enc");

        save_credential(&file_path, "password123", "key1", "value1").unwrap();
        save_credential(&file_path, "password123", "key2", "value2").unwrap();
        save_credential(&file_path, "password123", "key3", "value3").unwrap();

        let credentials = list_credentials(&file_path, "password123").unwrap();
        assert_eq!(credentials.len(), 3);
        assert_eq!(credentials[0].key, "key1");
        assert_eq!(credentials[1].key, "key2");
        assert_eq!(credentials[2].key, "key3");
    }

    #[test]
    fn test_save_fails_with_wrong_password_on_existing_file() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("test.enc");

        save_credential(&file_path, "correct123", "key1", "value1").unwrap();

        let result = save_credential(&file_path, "wrong_password", "key2", "value2");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_fails_with_wrong_password() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("test.enc");

        save_credential(&file_path, "correct123", "key", "value").unwrap();

        let result = list_credentials(&file_path, "wrong456789");
        assert!(result.is_err());
    }

    #[test]
    fn test_display_shows_position_ids() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("test.enc");

        save_credential(&file_path, "password123", "key1", "value1").unwrap();
        save_credential(&file_path, "password123", "key2", "value2").unwrap();

        let display = get_list_display(&file_path, "password123").unwrap();
        assert!(display.contains("[1]"));
        assert!(display.contains("[2]"));
        assert!(display.contains("key1"));
        assert!(display.contains("key2"));
        assert!(!display.contains("value1"));
        assert!(!display.contains("value2"));
        assert_eq!(display.matches('*').count(), 20); // 10 asterisks per credential × 2
    }

    #[test]
    fn test_display_masks_values() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("test.enc");

        save_credential(&file_path, "password123", "api_key", "super_secret").unwrap();

        let display = get_list_display(&file_path, "password123").unwrap();
        assert!(display.contains("api_key"));
        assert!(!display.contains("super_secret"));
        assert_eq!(display.matches('*').count(), 10);
    }

    #[test]
    fn test_read_single_credential_by_id() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("test.enc");

        save_credential(&file_path, "password123", "api_key", "secret_value").unwrap();

        let credential = read_credential(&file_path, "password123", 1).unwrap();
        assert_eq!(credential.key, "api_key");
        assert_eq!(credential.value, "secret_value");
    }

    #[test]
    fn test_read_first_credential_from_multiple() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("test.enc");

        save_credential(&file_path, "password123", "first", "value1").unwrap();
        save_credential(&file_path, "password123", "second", "value2").unwrap();
        save_credential(&file_path, "password123", "third", "value3").unwrap();

        let credential = read_credential(&file_path, "password123", 1).unwrap();
        assert_eq!(credential.key, "first");
        assert_eq!(credential.value, "value1");
    }

    #[test]
    fn test_read_middle_credential_from_multiple() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("test.enc");

        save_credential(&file_path, "password123", "first", "value1").unwrap();
        save_credential(&file_path, "password123", "second", "value2").unwrap();
        save_credential(&file_path, "password123", "third", "value3").unwrap();

        let credential = read_credential(&file_path, "password123", 2).unwrap();
        assert_eq!(credential.key, "second");
        assert_eq!(credential.value, "value2");
    }

    #[test]
    fn test_read_last_credential_from_multiple() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("test.enc");

        save_credential(&file_path, "password123", "first", "value1").unwrap();
        save_credential(&file_path, "password123", "second", "value2").unwrap();
        save_credential(&file_path, "password123", "third", "value3").unwrap();

        let credential = read_credential(&file_path, "password123", 3).unwrap();
        assert_eq!(credential.key, "third");
        assert_eq!(credential.value, "value3");
    }

    #[test]
    fn test_read_returns_plain_text_value() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("test.enc");

        save_credential(&file_path, "password123", "key", "super_secret_value").unwrap();

        let credential = read_credential(&file_path, "password123", 1).unwrap();
        assert_eq!(credential.value, "super_secret_value");
        assert_ne!(credential.value, "**********");
    }

    #[test]
    fn test_read_fails_with_id_zero() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("test.enc");

        save_credential(&file_path, "password123", "key", "value").unwrap();

        let result = read_credential(&file_path, "password123", 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid"));
    }

    #[test]
    fn test_read_fails_with_id_greater_than_count() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("test.enc");

        save_credential(&file_path, "password123", "key1", "value1").unwrap();
        save_credential(&file_path, "password123", "key2", "value2").unwrap();

        let result = read_credential(&file_path, "password123", 5);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid"));
    }

    #[test]
    fn test_read_fails_with_wrong_password() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("test.enc");

        save_credential(&file_path, "correct123", "key", "value").unwrap();

        let result = read_credential(&file_path, "wrong_password", 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_with_duplicate_keys() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("test.enc");

        save_credential(&file_path, "password123", "api_key", "first_value").unwrap();
        save_credential(&file_path, "password123", "api_key", "second_value").unwrap();
        save_credential(&file_path, "password123", "api_key", "third_value").unwrap();

        let cred1 = read_credential(&file_path, "password123", 1).unwrap();
        let cred2 = read_credential(&file_path, "password123", 2).unwrap();
        let cred3 = read_credential(&file_path, "password123", 3).unwrap();

        assert_eq!(cred1.value, "first_value");
        assert_eq!(cred2.value, "second_value");
        assert_eq!(cred3.value, "third_value");
    }
}
