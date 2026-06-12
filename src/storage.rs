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
}
