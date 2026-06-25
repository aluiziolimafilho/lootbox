use crate::crypto::{decrypt, derive_key, encrypt, generate_nonce, generate_salt};
use crate::validation::{
    validate_description, validate_name, validate_password, validate_secret_key,
    validate_secret_value, validate_url,
};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Represents a single credential
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Credential {
    pub name: String,
    pub key: String,
    pub value: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

impl Credential {
    /// Display label for list views: name if set, otherwise falls back to key.
    pub fn display_name(&self) -> &str {
        if self.name.is_empty() {
            &self.key
        } else {
            &self.name
        }
    }
}

/// Binary file format (v2, new files):
/// - Magic: 4 bytes ("LBOX")
/// - Version length: 1 byte
/// - Version string: N bytes (UTF-8, e.g. "2.1.0")
/// - Salt: 16 bytes
/// - Nonce: 12 bytes
/// - Encrypted data: remaining bytes (contains JSON array of credentials)
///
/// Legacy format (v1, files without magic header):
/// - Salt: 16 bytes
/// - Nonce: 12 bytes
/// - Encrypted data: remaining bytes
const SALT_SIZE: usize = 16;
const NONCE_SIZE: usize = 12;
const MAGIC: &[u8; 4] = b"LBOX";

fn parse_semver(v: &str) -> Option<(u64, u64, u64)> {
    let mut parts = v.splitn(3, '.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    Some((major, minor, patch))
}

fn parse_file_content(bytes: &[u8]) -> Result<([u8; 16], [u8; 12], &[u8])> {
    let payload = if bytes.starts_with(MAGIC) {
        if bytes.len() < 5 {
            bail!("Invalid encrypted file - file is too small or corrupted");
        }
        let version_len = bytes[4] as usize;
        let payload_start = 5 + version_len;
        if bytes.len() < payload_start + SALT_SIZE + NONCE_SIZE {
            bail!("Invalid encrypted file - file is too small or corrupted");
        }
        let file_version_str = std::str::from_utf8(&bytes[5..5 + version_len])
            .context("File header contains invalid version string")?;
        if let (Some(app), Some(file)) = (
            parse_semver(env!("CARGO_PKG_VERSION")),
            parse_semver(file_version_str),
        ) {
            if file > app {
                bail!(
                    "This file was created with LootBox {}, but the current version is {}. \
                     Please upgrade LootBox to open this file.",
                    file_version_str,
                    env!("CARGO_PKG_VERSION")
                );
            }
        }
        &bytes[payload_start..]
    } else {
        bail!(
            "This file was not created by LootBox or uses an unsupported format. \
             Only files created with LootBox 2.0.0 or later can be opened."
        );
    };
    let salt: [u8; 16] = payload[0..SALT_SIZE]
        .try_into()
        .context("Failed to read salt from file")?;
    let nonce: [u8; 12] = payload[SALT_SIZE..SALT_SIZE + NONCE_SIZE]
        .try_into()
        .context("Failed to read nonce from file")?;
    Ok((salt, nonce, &payload[SALT_SIZE + NONCE_SIZE..]))
}

fn build_file_content(salt: &[u8; 16], nonce: &[u8; 12], encrypted_data: &[u8]) -> Vec<u8> {
    let version = env!("CARGO_PKG_VERSION").as_bytes();
    let mut content =
        Vec::with_capacity(5 + version.len() + SALT_SIZE + NONCE_SIZE + encrypted_data.len());
    content.extend_from_slice(MAGIC);
    content.push(version.len() as u8);
    content.extend_from_slice(version);
    content.extend_from_slice(salt);
    content.extend_from_slice(nonce);
    content.extend_from_slice(encrypted_data);
    content
}

fn write_encrypted_file<P: AsRef<Path>>(
    file_path: P,
    salt: &[u8; 16],
    nonce: &[u8; 12],
    encrypted_data: &[u8],
) -> Result<()> {
    let file_path = file_path.as_ref();
    fs::write(file_path, build_file_content(salt, nonce, encrypted_data))
        .context("Failed to write encrypted file")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(file_path)?.permissions();
        permissions.set_mode(0o600);
        fs::set_permissions(file_path, permissions)?;
    }
    Ok(())
}

fn encrypt_and_write<P: AsRef<Path>>(file_path: P, password: &str, credentials: &[Credential]) -> Result<()> {
    let credentials_json = serde_json::to_vec(credentials)
        .context("Failed to serialize credentials")?;
    let salt = generate_salt();
    let key = derive_key(password, &salt)?;
    let nonce = generate_nonce();
    let encrypted_data = encrypt(&credentials_json, &key, &nonce)?;
    write_encrypted_file(file_path, &salt, &nonce, &encrypted_data)
}

/// Saves a new credential to an encrypted file (creates or appends).
///
/// `name`, `description`, and `url` are optional at the API level; the CLI/TUI/GUI
/// enforce required-ness at their own layer. Passing `None` stores an empty name / no
/// url / no description. Passing `Some("")` behaves the same as `None` for optional
/// fields; for `name`, `Some("")` also stores an empty name (no validation error).
pub fn save_credential<P: AsRef<Path>>(
    file_path: P,
    password: &str,
    secret_key: &str,
    secret_value: &str,
    name: Option<&str>,
    description: Option<&str>,
    url: Option<&str>,
) -> Result<()> {
    let file_path = file_path.as_ref();

    validate_password(password)?;
    validate_secret_key(secret_key)?;
    validate_secret_value(secret_value)?;

    let name_str = name.unwrap_or("");
    if !name_str.is_empty() {
        validate_name(name_str)?;
    }

    let url_opt = url.filter(|s| !s.is_empty()).map(|s| {
        validate_url(s).map(|()| s.to_string())
    }).transpose()?;

    let desc_opt = description.filter(|s| !s.is_empty()).map(|s| {
        validate_description(s).map(|()| s.to_string())
    }).transpose()?;

    let new_credential = Credential {
        name: name_str.to_string(),
        key: secret_key.to_string(),
        value: secret_value.to_string(),
        url: url_opt,
        description: desc_opt,
    };

    let mut credentials = if file_path.exists() {
        list_credentials(file_path, password)?
    } else {
        Vec::new()
    };

    credentials.push(new_credential);
    encrypt_and_write(file_path, password, &credentials)
}

/// Reads and decrypts all credentials from an encrypted file.
pub fn list_credentials<P: AsRef<Path>>(file_path: P, password: &str) -> Result<Vec<Credential>> {
    let file_path = file_path.as_ref();

    validate_password(password)?;

    if !file_path.exists() {
        bail!("File not found: {}", file_path.display());
    }

    let file_content = fs::read(file_path)
        .context("Failed to read encrypted file")?;

    let (salt, nonce, encrypted_data) = parse_file_content(&file_content)?;

    let key = derive_key(password, &salt)?;
    let decrypted_data = decrypt(encrypted_data, &key, &nonce)?;

    let credentials: Vec<Credential> = serde_json::from_slice(&decrypted_data)
        .context("Failed to parse decrypted credential data")?;

    Ok(credentials)
}

/// Gets the display string for listing credentials.
/// Format: [n] Name: <name_or_key> Key: <key> Value: **********
pub fn get_list_display<P: AsRef<Path>>(file_path: P, password: &str) -> Result<String> {
    let credentials = list_credentials(file_path, password)?;

    let mut output = String::new();
    for (index, credential) in credentials.iter().enumerate() {
        let position = index + 1;
        output.push_str(&format!(
            "[{}] {} Key: {} Value: **********\n",
            position,
            credential.display_name(),
            credential.key
        ));
    }

    if output.ends_with('\n') {
        output.pop();
    }

    Ok(output)
}

/// Reads a specific credential by its 1-indexed position ID.
pub fn read_credential<P: AsRef<Path>>(
    file_path: P,
    password: &str,
    position_id: usize,
) -> Result<Credential> {
    let credentials = list_credentials(file_path, password)?;

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

    Ok(credentials[position_id - 1].clone())
}

/// Updates a specific credential by its 1-indexed position ID.
/// `None` for any field means "keep current". `Some("")` clears optional fields
/// (`url`, `description`) but is a no-op for `name` (keeps current).
pub fn update_credential<P: AsRef<Path>>(
    file_path: P,
    password: &str,
    position_id: usize,
    new_key: Option<&str>,
    new_value: Option<&str>,
    new_name: Option<&str>,
    new_description: Option<&str>,
    new_url: Option<&str>,
) -> Result<()> {
    let file_path = file_path.as_ref();

    validate_password(password)?;

    let mut credentials = list_credentials(file_path, password)?;

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

    let credential = &mut credentials[position_id - 1];

    if let Some(key) = new_key {
        validate_secret_key(key)?;
        credential.key = key.to_string();
    }

    if let Some(value) = new_value {
        validate_secret_value(value)?;
        credential.value = value.to_string();
    }

    if let Some(name) = new_name {
        if !name.is_empty() {
            validate_name(name)?;
            credential.name = name.to_string();
        }
        // Some("") keeps current name
    }

    if let Some(desc) = new_description {
        if desc.is_empty() {
            credential.description = None;
        } else {
            validate_description(desc)?;
            credential.description = Some(desc.to_string());
        }
    }

    if let Some(url) = new_url {
        if url.is_empty() {
            credential.url = None;
        } else {
            validate_url(url)?;
            credential.url = Some(url.to_string());
        }
    }

    encrypt_and_write(file_path, password, &credentials)
}

/// Removes a specific credential by its 1-indexed position ID.
pub fn remove_credential<P: AsRef<Path>>(
    file_path: P,
    password: &str,
    position_id: usize,
) -> Result<()> {
    let file_path = file_path.as_ref();

    validate_password(password)?;

    let mut credentials = list_credentials(file_path, password)?;

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

    credentials.remove(position_id - 1);
    encrypt_and_write(file_path, password, &credentials)
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

/// Reads a single credential by position ID and maps it to an environment variable.
pub fn generate_env_vars<P: AsRef<Path>>(
    file_path: P,
    password: &str,
    credential_id: usize,
) -> Result<EnvVarsResult> {
    let credential = read_credential(file_path, password, credential_id)?;

    let mut created: Vec<EnvEntry> = Vec::new();
    let mut invalid: Vec<InvalidEntry> = Vec::new();

    let env_name = credential.key.to_uppercase().replace(' ', "_");

    let invalid_char = env_name
        .chars()
        .find(|c| !matches!(c, 'A'..='Z' | '0'..='9' | '_'));

    if let Some(ch) = invalid_char {
        invalid.push(InvalidEntry {
            original_key: credential.key,
            reason: format!("invalid character '{}' in environment variable name", ch),
        });
    } else if env_name.starts_with(|c: char| c.is_ascii_digit()) {
        invalid.push(InvalidEntry {
            original_key: credential.key,
            reason: "environment variable name cannot start with a digit".to_string(),
        });
    } else if credential.value.contains('\0') {
        invalid.push(InvalidEntry {
            original_key: credential.key,
            reason: "value contains a null byte, which is not valid in shell environment variables"
                .to_string(),
        });
    } else {
        created.push(EnvEntry {
            original_key: credential.key,
            env_name,
            value: credential.value,
        });
    }

    Ok(EnvVarsResult { created, invalid })
}

// ============================================================================
// CSV helpers
// ============================================================================

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn csv_parse_row(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Option<Vec<String>> {
    if chars.peek().is_none() {
        return None;
    }
    let mut fields: Vec<String> = Vec::new();
    let mut field = String::new();
    let mut in_quotes = false;
    while let Some(&ch) = chars.peek() {
        chars.next();
        if in_quotes {
            if ch == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    field.push('"');
                } else {
                    in_quotes = false;
                }
            } else {
                field.push(ch);
            }
        } else {
            match ch {
                '"' => { in_quotes = true; }
                ',' => { fields.push(std::mem::take(&mut field)); }
                '\r' => {
                    if chars.peek() == Some(&'\n') {
                        chars.next();
                    }
                    fields.push(std::mem::take(&mut field));
                    return Some(fields);
                }
                '\n' => {
                    fields.push(std::mem::take(&mut field));
                    return Some(fields);
                }
                _ => { field.push(ch); }
            }
        }
    }
    fields.push(field);
    Some(fields)
}

fn csv_parse_rows(content: &str) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    let mut chars = content.chars().peekable();
    while let Some(row) = csv_parse_row(&mut chars) {
        if !(row.len() == 1 && row[0].is_empty()) {
            rows.push(row);
        }
    }
    rows
}

// ============================================================================
// CSV export / import
// ============================================================================

/// Exports all credentials from an encrypted file to a CSV file.
/// The CSV has a header row (`key,value`) followed by one row per credential.
pub fn export_credentials_to_csv<P: AsRef<Path>>(
    enc_path: P,
    password: &str,
    csv_path: P,
) -> Result<()> {
    let credentials = list_credentials(enc_path, password)?;
    let mut output = String::from("key,value\n");
    for cred in &credentials {
        output.push_str(&format!(
            "{},{}\n",
            csv_escape(&cred.key),
            csv_escape(&cred.value)
        ));
    }
    fs::write(csv_path.as_ref(), &output)
        .with_context(|| format!("Failed to write CSV file: {}", csv_path.as_ref().display()))?;
    Ok(())
}

/// Imports credentials from a CSV file into an encrypted vault.
/// The CSV must have a header row (`key,value`). Blank lines are skipped.
pub fn import_credentials_from_csv<P: AsRef<Path>>(
    enc_path: P,
    password: &str,
    csv_path: P,
) -> Result<usize> {
    validate_password(password)?;

    let content = fs::read_to_string(csv_path.as_ref())
        .with_context(|| format!("CSV file not found: {}", csv_path.as_ref().display()))?;

    let rows = csv_parse_rows(&content);

    let expected_header = vec!["key".to_string(), "value".to_string()];
    if rows.is_empty() || rows[0] != expected_header {
        bail!("CSV file must have a header row: key,value");
    }

    let mut count = 0;
    for (i, row) in rows[1..].iter().enumerate() {
        if row.len() != 2 {
            bail!(
                "CSV row {} has {} field(s), expected 2 (key and value)",
                i + 2,
                row.len()
            );
        }
        save_credential(enc_path.as_ref(), password, &row[0], &row[1], None, None, None)
            .with_context(|| format!("Failed to import CSV row {}", i + 2))?;
        count += 1;
    }

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_dir() -> TempDir {
        tempfile::tempdir().expect("Failed to create temp dir")
    }

    fn save(file_path: &std::path::Path, password: &str, key: &str, value: &str) {
        save_credential(file_path, password, key, value, None, None, None).unwrap();
    }

    #[test]
    fn test_save_and_list_single_credential() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("test.enc");

        save(&file_path, "password123", "my_key", "my_value");

        let credentials = list_credentials(&file_path, "password123").unwrap();
        assert_eq!(credentials.len(), 1);
        assert_eq!(credentials[0].key, "my_key");
        assert_eq!(credentials[0].value, "my_value");
    }

    #[test]
    fn test_save_multiple_credentials() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("test.enc");

        save(&file_path, "password123", "key1", "value1");
        save(&file_path, "password123", "key2", "value2");
        save(&file_path, "password123", "key3", "value3");

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

        save(&file_path, "correct123", "key1", "value1");

        let result = save_credential(&file_path, "wrong_password", "key2", "value2", None, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_fails_with_wrong_password() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("test.enc");

        save(&file_path, "correct123", "key", "value");

        let result = list_credentials(&file_path, "wrong456789");
        assert!(result.is_err());
    }

    #[test]
    fn test_display_shows_position_ids() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("test.enc");

        save(&file_path, "password123", "key1", "value1");
        save(&file_path, "password123", "key2", "value2");

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

        save(&file_path, "password123", "api_key", "super_secret");

        let display = get_list_display(&file_path, "password123").unwrap();
        assert!(display.contains("api_key"));
        assert!(!display.contains("super_secret"));
        assert_eq!(display.matches('*').count(), 10);
    }

    #[test]
    fn test_read_single_credential_by_id() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("test.enc");

        save(&file_path, "password123", "api_key", "secret_value");

        let credential = read_credential(&file_path, "password123", 1).unwrap();
        assert_eq!(credential.key, "api_key");
        assert_eq!(credential.value, "secret_value");
    }

    #[test]
    fn test_read_first_credential_from_multiple() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("test.enc");

        save(&file_path, "password123", "first", "value1");
        save(&file_path, "password123", "second", "value2");
        save(&file_path, "password123", "third", "value3");

        let credential = read_credential(&file_path, "password123", 1).unwrap();
        assert_eq!(credential.key, "first");
        assert_eq!(credential.value, "value1");
    }

    #[test]
    fn test_read_middle_credential_from_multiple() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("test.enc");

        save(&file_path, "password123", "first", "value1");
        save(&file_path, "password123", "second", "value2");
        save(&file_path, "password123", "third", "value3");

        let credential = read_credential(&file_path, "password123", 2).unwrap();
        assert_eq!(credential.key, "second");
        assert_eq!(credential.value, "value2");
    }

    #[test]
    fn test_read_last_credential_from_multiple() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("test.enc");

        save(&file_path, "password123", "first", "value1");
        save(&file_path, "password123", "second", "value2");
        save(&file_path, "password123", "third", "value3");

        let credential = read_credential(&file_path, "password123", 3).unwrap();
        assert_eq!(credential.key, "third");
        assert_eq!(credential.value, "value3");
    }

    #[test]
    fn test_read_returns_plain_text_value() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("test.enc");

        save(&file_path, "password123", "key", "super_secret_value");

        let credential = read_credential(&file_path, "password123", 1).unwrap();
        assert_eq!(credential.value, "super_secret_value");
        assert_ne!(credential.value, "**********");
    }

    #[test]
    fn test_read_fails_with_id_zero() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("test.enc");

        save(&file_path, "password123", "key", "value");

        let result = read_credential(&file_path, "password123", 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid"));
    }

    #[test]
    fn test_read_fails_with_id_greater_than_count() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("test.enc");

        save(&file_path, "password123", "key1", "value1");
        save(&file_path, "password123", "key2", "value2");

        let result = read_credential(&file_path, "password123", 5);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid"));
    }

    #[test]
    fn test_read_fails_with_wrong_password() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("test.enc");

        save(&file_path, "correct123", "key", "value");

        let result = read_credential(&file_path, "wrong_password", 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_with_duplicate_keys() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("test.enc");

        save(&file_path, "password123", "api_key", "first_value");
        save(&file_path, "password123", "api_key", "second_value");
        save(&file_path, "password123", "api_key", "third_value");

        let cred1 = read_credential(&file_path, "password123", 1).unwrap();
        let cred2 = read_credential(&file_path, "password123", 2).unwrap();
        let cred3 = read_credential(&file_path, "password123", 3).unwrap();

        assert_eq!(cred1.value, "first_value");
        assert_eq!(cred2.value, "second_value");
        assert_eq!(cred3.value, "third_value");
    }

    #[test]
    fn test_save_with_optional_fields() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("test.enc");

        save_credential(
            &file_path,
            "password123",
            "api_key",
            "secret",
            Some("My API Key"),
            Some("Used for testing"),
            Some("https://example.com"),
        )
        .unwrap();

        let creds = list_credentials(&file_path, "password123").unwrap();
        assert_eq!(creds[0].name, "My API Key");
        assert_eq!(creds[0].description, Some("Used for testing".to_string()));
        assert_eq!(creds[0].url, Some("https://example.com".to_string()));
    }

    #[test]
    fn test_save_optional_fields_default_to_none() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("test.enc");

        save(&file_path, "password123", "key", "value");

        let creds = list_credentials(&file_path, "password123").unwrap();
        assert_eq!(creds[0].name, "");
        assert_eq!(creds[0].url, None);
        assert_eq!(creds[0].description, None);
    }

    #[test]
    fn test_new_file_starts_with_magic_bytes() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("test.enc");

        save(&file_path, "password123", "k", "v");

        let raw = fs::read(&file_path).unwrap();
        assert_eq!(&raw[0..4], b"LBOX", "new files must start with LBOX magic bytes");
    }

    #[test]
    fn test_old_format_file_is_rejected() {
        use crate::crypto::{derive_key, encrypt, generate_nonce, generate_salt};

        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("old.enc");

        // Write a legacy file (no magic header: just salt + nonce + encrypted JSON)
        let creds = vec![Credential {
            name: "".into(),
            key: "legacy".into(),
            value: "val".into(),
            url: None,
            description: None,
        }];
        let json = serde_json::to_vec(&creds).unwrap();
        let salt = generate_salt();
        let key = derive_key("password123", &salt).unwrap();
        let nonce = generate_nonce();
        let encrypted = encrypt(&json, &key, &nonce).unwrap();
        let mut content = Vec::new();
        content.extend_from_slice(&salt);
        content.extend_from_slice(&nonce);
        content.extend_from_slice(&encrypted);
        fs::write(&file_path, content).unwrap();

        let result = list_credentials(&file_path, "password123");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("unsupported format"));
    }

    fn write_versioned_file(file_path: &std::path::Path, version: &str, password: &str) {
        use crate::crypto::{derive_key, encrypt, generate_nonce, generate_salt};

        let creds = vec![Credential {
            name: "".into(),
            key: "k".into(),
            value: "v".into(),
            url: None,
            description: None,
        }];
        let json = serde_json::to_vec(&creds).unwrap();
        let salt = generate_salt();
        let key = derive_key(password, &salt).unwrap();
        let nonce = generate_nonce();
        let encrypted = encrypt(&json, &key, &nonce).unwrap();
        let version_bytes = version.as_bytes();
        let mut content = Vec::new();
        content.extend_from_slice(b"LBOX");
        content.push(version_bytes.len() as u8);
        content.extend_from_slice(version_bytes);
        content.extend_from_slice(&salt);
        content.extend_from_slice(&nonce);
        content.extend_from_slice(&encrypted);
        fs::write(file_path, content).unwrap();
    }

    #[test]
    fn test_parse_semver() {
        assert_eq!(super::parse_semver("2.0.0"), Some((2, 0, 0)));
        assert_eq!(super::parse_semver("1.2.3"), Some((1, 2, 3)));
        assert_eq!(super::parse_semver("10.0.0"), Some((10, 0, 0)));
        assert!(super::parse_semver("bad").is_none());
    }

    #[test]
    fn test_file_from_future_version_is_rejected() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("future.enc");
        write_versioned_file(&file_path, "99.0.0", "password123");

        let result = list_credentials(&file_path, "password123");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("99.0.0"), "error should mention the file version");
        assert!(msg.contains("upgrade"), "error should suggest upgrading");
    }

    #[test]
    fn test_file_from_same_version_is_accepted() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("same.enc");
        write_versioned_file(&file_path, env!("CARGO_PKG_VERSION"), "password123");

        let result = list_credentials(&file_path, "password123").unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_file_from_older_version_is_accepted() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("older.enc");
        write_versioned_file(&file_path, "1.0.0", "password123");

        let result = list_credentials(&file_path, "password123").unwrap();
        assert_eq!(result.len(), 1);
    }
}
