use lootbox::{list_credentials, save_credential};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper function to create a temporary test directory
fn setup_test_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp dir")
}

/// Helper function to get test file path
fn get_test_file_path(dir: &TempDir, filename: &str) -> PathBuf {
    dir.path().join(filename)
}

#[test]
fn test_save_creates_new_file_when_not_exists() {
    // Given: A non-existent file path
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    assert!(!file_path.exists(), "File should not exist initially");

    // When: Saving a credential with password "mypassword123"
    save_credential(&file_path, "mypassword123", "api_key", "sk-1234567890").unwrap();

    // Then: File should be created
    assert!(file_path.exists());
    // And: File should not be empty
    assert!(fs::metadata(&file_path).unwrap().len() > 0);
    // And: Should contain exactly one credential
    let credentials = list_credentials(&file_path, "mypassword123").unwrap();
    assert_eq!(credentials.len(), 1);
}

#[test]
fn test_save_adds_to_existing_file() {
    // Given: An existing encrypted file with one credential
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "key1", "value1").unwrap();

    // When: Saving a second credential to the same file
    let result = save_credential(&file_path, password, "key2", "value2");

    // Then: Should succeed
    assert!(result.is_ok());
    // And: File should now contain both credentials
    let credentials = list_credentials(&file_path, password).unwrap();
    assert_eq!(credentials.len(), 2);
    assert!(credentials.iter().any(|c| c.key == "key1" && c.value == "value1"));
    assert!(credentials.iter().any(|c| c.key == "key2" && c.value == "value2"));
}

#[test]
fn test_save_multiple_credentials_to_same_file() {
    // Given: A new file path
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    // When: Saving multiple credentials sequentially
    save_credential(&file_path, password, "aws_key", "aws_secret_123").unwrap();
    save_credential(&file_path, password, "github_token", "ghp_token_456").unwrap();
    save_credential(&file_path, password, "api_key", "sk_key_789").unwrap();

    // Then: All credentials should be stored
    let credentials = list_credentials(&file_path, password).unwrap();
    assert_eq!(credentials.len(), 3);
    assert!(credentials.iter().any(|c| c.key == "aws_key"));
    assert!(credentials.iter().any(|c| c.key == "github_token"));
    assert!(credentials.iter().any(|c| c.key == "api_key"));
}

#[test]
fn test_save_fails_with_wrong_password_on_existing_file() {
    // Given: An encrypted file with existing credentials
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let original_password = "correct123";

    save_credential(&file_path, original_password, "key1", "value1").unwrap();

    // When: Attempting to save with wrong password
    let result = save_credential(&file_path, "wrong-password", "key2", "value2");

    // Then: Should return an error (cannot decrypt existing file)
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("password") || error_msg.contains("decrypt"));
}

#[test]
fn test_save_fails_with_corrupted_existing_file() {
    // Given: A corrupted file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "corrupted.enc");

    // Write invalid/corrupted data
    fs::write(&file_path, "this is not a valid encrypted file").unwrap();

    // When: Attempting to add a credential
    let result = save_credential(&file_path, "password123", "key", "value");

    // Then: Should return an error (cannot validate existing file)
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("decrypt") || error_msg.contains("invalid") || error_msg.contains("corrupted"));
}

#[test]
fn test_save_preserves_existing_credentials() {
    // Given: An encrypted file with existing credentials
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "mypassword";

    save_credential(&file_path, password, "original_key", "original_value").unwrap();

    // When: Adding a new credential
    save_credential(&file_path, password, "new_key", "new_value").unwrap();

    // Then: Original credential should still exist
    let credentials = list_credentials(&file_path, password).unwrap();
    assert_eq!(credentials.len(), 2);
    assert!(credentials.iter().any(|c| c.key == "original_key" && c.value == "original_value"));
    assert!(credentials.iter().any(|c| c.key == "new_key" && c.value == "new_value"));
}

#[test]
fn test_save_allows_duplicate_keys() {
    // Given: An encrypted file with a credential
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "api_key", "old_value").unwrap();

    // When: Saving with the same key but different value
    let result = save_credential(&file_path, password, "api_key", "new_value");

    // Then: Should succeed (allows duplicates - user manages them)
    assert!(result.is_ok());
    // And: Both entries should exist
    let credentials = list_credentials(&file_path, password).unwrap();
    let api_key_creds: Vec<_> = credentials.iter()
        .filter(|c| c.key == "api_key")
        .collect();
    assert_eq!(api_key_creds.len(), 2);
}

#[test]
fn test_save_encrypts_file_content() {
    // Given: A new credential file path
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    // When: Saving a credential with secret value "my-secret-value"
    save_credential(&file_path, "password123", "key1", "my-secret-value").unwrap();

    // Then: File content should not contain plain text secret
    let content = fs::read(&file_path).unwrap();
    let content_str = String::from_utf8_lossy(&content);
    assert!(!content_str.contains("my-secret-value"));
    assert!(!content_str.contains("key1"));
}

// ============================================================================
// Password Validation Tests
// ============================================================================

#[test]
fn test_save_with_empty_password() {
    // Given: A new file path
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    // When: Attempting to save with empty password (less than 8 characters)
    let result = save_credential(&file_path, "", "key", "value");

    // Then: Should return an error
    assert!(result.is_err());
    // And: File should not be created
    assert!(!file_path.exists());
}

#[test]
fn test_save_with_password_less_than_8_characters() {
    // Given: A new file path
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    // When: Attempting to save with 7-character password
    let result = save_credential(&file_path, "pass123", "key", "value");

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("8") || error_msg.contains("length") || error_msg.contains("characters"));
    // And: File should not be created
    assert!(!file_path.exists());
}

#[test]
fn test_save_with_password_exactly_8_characters() {
    // Given: A new file path
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    // When: Saving with exactly 8-character password
    let result = save_credential(&file_path, "password", "key", "value");

    // Then: Should succeed
    assert!(result.is_ok());
    assert!(file_path.exists());
}

#[test]
fn test_save_with_password_only_whitespace() {
    // Given: A new file path
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    // When: Attempting to save with whitespace-only password (even if >= 8 chars)
    let result = save_credential(&file_path, "        ", "key", "value");

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("whitespace") || error_msg.contains("invalid"));
    // And: File should not be created
    assert!(!file_path.exists());
}

#[test]
fn test_save_with_password_starting_with_whitespace() {
    // Given: A new file path
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    // When: Attempting to save with password starting with whitespace
    let result = save_credential(&file_path, " password123", "key", "value");

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("whitespace") || error_msg.contains("space"));
    // And: File should not be created
    assert!(!file_path.exists());
}

#[test]
fn test_save_with_password_ending_with_whitespace() {
    // Given: A new file path
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    // When: Attempting to save with password ending with whitespace
    let result = save_credential(&file_path, "password123 ", "key", "value");

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("whitespace") || error_msg.contains("space"));
    // And: File should not be created
    assert!(!file_path.exists());
}

#[test]
fn test_save_with_password_containing_whitespace_in_middle() {
    // Given: A new file path
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    // When: Saving with password containing whitespace in the middle (should be valid)
    let result = save_credential(&file_path, "pass word 123", "key", "value");

    // Then: Should succeed (whitespace in middle is allowed)
    assert!(result.is_ok());
    assert!(file_path.exists());
}

#[test]
fn test_save_with_password_exactly_32_characters() {
    // Given: A new file path
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    // When: Saving with exactly 32-character password (maximum allowed)
    let password = "a".repeat(32);
    let result = save_credential(&file_path, &password, "key", "value");

    // Then: Should succeed
    assert!(result.is_ok());
}

#[test]
fn test_save_with_password_exceeding_32_characters() {
    // Given: A new file path
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    // When: Attempting to save with 33-character password (exceeds maximum)
    let password = "a".repeat(33);
    let result = save_credential(&file_path, &password, "key", "value");

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("32") || error_msg.contains("maximum") || error_msg.contains("characters"));
    // And: File should not be created
    assert!(!file_path.exists());
}

// ============================================================================
// Secret Key Validation Tests
// ============================================================================

#[test]
fn test_save_with_empty_secret_key() {
    // Given: A new file path
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    // When: Attempting to save with empty secret_key (required field)
    let result = save_credential(&file_path, "password123", "", "value");

    // Then: Should return an error
    assert!(result.is_err());
    // And: File should not be created
    assert!(!file_path.exists());
}

#[test]
fn test_save_with_whitespace_only_key() {
    // Given: A new file path
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    // When: Attempting to save with whitespace-only key
    let result = save_credential(&file_path, "password123", "   ", "value");

    // Then: Should return an error
    assert!(result.is_err());
    // And: File should not be created
    assert!(!file_path.exists());
}

#[test]
fn test_save_with_secret_key_exactly_64_characters() {
    // Given: A new file path
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    // When: Saving with exactly 64-character secret key (maximum allowed)
    let key = "a".repeat(64);
    let result = save_credential(&file_path, "password123", &key, "value");

    // Then: Should succeed
    assert!(result.is_ok());
}

#[test]
fn test_save_with_secret_key_exceeding_64_characters() {
    // Given: A new file path
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    // When: Attempting to save with 65-character secret key (exceeds maximum)
    let key = "a".repeat(65);
    let result = save_credential(&file_path, "password123", &key, "value");

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("64") || error_msg.contains("maximum") || error_msg.contains("characters"));
    // And: File should not be created
    assert!(!file_path.exists());
}

// ============================================================================
// Secret Value Validation Tests
// ============================================================================

#[test]
fn test_save_with_empty_secret_value() {
    // Given: A new file path
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    // When: Attempting to save with empty secret_value (required field)
    let result = save_credential(&file_path, "password123", "key", "");

    // Then: Should return an error
    assert!(result.is_err());
    // And: File should not be created
    assert!(!file_path.exists());
}

#[test]
fn test_save_with_whitespace_only_value() {
    // Given: A new file path
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    // When: Attempting to save with whitespace-only value
    let result = save_credential(&file_path, "password123", "key", "   ");

    // Then: Should return an error
    assert!(result.is_err());
    // And: File should not be created
    assert!(!file_path.exists());
}

#[test]
fn test_save_with_secret_value_exactly_5000_characters() {
    // Given: A new file path
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    // When: Saving with exactly 5000-character secret value (maximum allowed)
    let value = "a".repeat(5000);
    let result = save_credential(&file_path, "password123", "key", &value);

    // Then: Should succeed
    assert!(result.is_ok());
}

#[test]
fn test_save_with_secret_value_exceeding_5000_characters() {
    // Given: A new file path
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    // When: Attempting to save with 5001-character secret value (exceeds maximum)
    let value = "a".repeat(5001);
    let result = save_credential(&file_path, "password123", "key", &value);

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("5000") || error_msg.contains("maximum") || error_msg.contains("characters"));
    // And: File should not be created
    assert!(!file_path.exists());
}

// ============================================================================
// Special Characters and Encoding Tests
// ============================================================================

#[test]
fn test_save_with_special_characters_in_key_and_value() {
    // Given: A new file path
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    // When: Saving with special characters
    let special_key = "key@#$%^&*()";
    let special_value = "value!@#$%^&*(){}[]|\\:;\"'<>,.?/~`";
    let result = save_credential(&file_path, "password123", special_key, special_value);

    // Then: Should save successfully
    assert!(result.is_ok());
    // And: Should be retrievable correctly
    let credentials = list_credentials(&file_path, "password123").unwrap();
    assert!(credentials.iter().any(|c| c.key == special_key && c.value == special_value));
}

#[test]
fn test_save_with_unicode_characters() {
    // Given: A new file path
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    // When: Saving with unicode characters
    let result = save_credential(&file_path, "password123", "日本語キー", "中文值🔑");

    // Then: Should save successfully
    assert!(result.is_ok());
    // And: Should be retrievable correctly
    let credentials = list_credentials(&file_path, "password123").unwrap();
    assert!(credentials.iter().any(|c| c.key == "日本語キー" && c.value == "中文值🔑"));
}

#[test]
fn test_save_with_newlines_in_key_and_value() {
    // Given: A new file path
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    // When: Saving with newlines
    let key_with_newline = "multi\nline\nkey";
    let value_with_newline = "multi\nline\nvalue";
    let result = save_credential(&file_path, "password123", key_with_newline, value_with_newline);

    // Then: Should save successfully
    assert!(result.is_ok());
    // And: Should be retrievable correctly
    let credentials = list_credentials(&file_path, "password123").unwrap();
    assert!(credentials.iter().any(|c| c.key == key_with_newline && c.value == value_with_newline));
}

#[test]
fn test_save_with_very_long_key_and_value() {
    // Given: A new file path
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    // When: Using key and value at the maximum allowed lengths
    let long_key = "a".repeat(64);
    let long_value = "b".repeat(5000);
    let result = save_credential(&file_path, "password123", &long_key, &long_value);

    // Then: Should handle properly
    assert!(result.is_ok());
}

// ============================================================================
// File System Tests
// ============================================================================

#[test]
fn test_save_creates_file_with_proper_permissions() {
    // Given: A new file path
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    // When: Saving a credential
    save_credential(&file_path, "password123", "key", "value").unwrap();

    // Then: File should have restricted permissions (on Unix systems)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(&file_path).unwrap();
        let permissions = metadata.permissions();
        // Should be readable/writable only by owner (0o600)
        assert_eq!(permissions.mode() & 0o777, 0o600);
    }
}

#[test]
fn test_save_in_nonexistent_directory() {
    // Given: A path in a non-existent directory
    let temp_dir = setup_test_dir();
    let file_path = temp_dir.path().join("nonexistent").join("credentials.enc");

    // When: Attempting to save
    let result = save_credential(&file_path, "password123", "key", "value");

    // Then: Should return an error
    assert!(result.is_err());
}

// ============================================================================
// Binary Format Tests
// ============================================================================

#[test]
fn test_save_creates_binary_file_not_json() {
    // Given: A new file path
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    // When: Saving a credential
    save_credential(&file_path, "password123", "key", "value").unwrap();

    // Then: File should be in binary format (not JSON wrapper)
    let content = fs::read(&file_path).unwrap();

    // File should start with 16-byte salt (not with '{' which is JSON)
    assert_ne!(content[0], b'{', "File should not start with JSON object");

    // Try to parse as JSON - should fail because it's binary format
    let json_parse_result = serde_json::from_slice::<serde_json::Value>(&content);
    assert!(json_parse_result.is_err(), "File should not be parseable as JSON");

    // Verify binary structure: at least 28 bytes (16 salt + 12 nonce + some encrypted data)
    assert!(content.len() >= 28, "File should contain salt, nonce, and encrypted data");
}

#[test]
fn test_save_binary_format_structure() {
    // Given: A new file path
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    // When: Saving a credential
    save_credential(&file_path, "password123", "key", "value").unwrap();

    // Then: File should have correct binary structure
    let content = fs::read(&file_path).unwrap();

    // First 16 bytes are salt
    // Next 12 bytes are nonce
    // Remaining bytes are encrypted data
    assert!(content.len() >= 28, "File should have at least salt (16) + nonce (12) bytes");

    // Verify we can read the structure
    let salt = &content[0..16];
    let nonce = &content[16..28];
    let encrypted_data = &content[28..];

    assert_eq!(salt.len(), 16);
    assert_eq!(nonce.len(), 12);
    assert!(encrypted_data.len() > 0);
}

// ============================================================================
// Encryption Tests
// ============================================================================

#[test]
fn test_save_different_passwords_create_different_encrypted_files() {
    // Given: Two different file paths
    let temp_dir = setup_test_dir();
    let file_path1 = get_test_file_path(&temp_dir, "file1.enc");
    let file_path2 = get_test_file_path(&temp_dir, "file2.enc");

    // When: Saving same credentials with different passwords
    save_credential(&file_path1, "password123", "key", "value").unwrap();
    save_credential(&file_path2, "different456", "key", "value").unwrap();

    // Then: Files should have different encrypted content
    let content1 = fs::read(&file_path1).unwrap();
    let content2 = fs::read(&file_path2).unwrap();
    assert_ne!(content1, content2);
}

#[test]
fn test_save_same_data_twice_creates_different_encrypted_files() {
    // Given: Two different file paths
    let temp_dir = setup_test_dir();
    let file_path1 = get_test_file_path(&temp_dir, "file1.enc");
    let file_path2 = get_test_file_path(&temp_dir, "file2.enc");

    // When: Saving same credentials with same password to different files
    save_credential(&file_path1, "password123", "key", "value").unwrap();
    save_credential(&file_path2, "password123", "key", "value").unwrap();

    // Then: Files should have different content (due to random nonce/salt)
    let content1 = fs::read(&file_path1).unwrap();
    let content2 = fs::read(&file_path2).unwrap();
    assert_ne!(content1, content2, "Encrypted files should differ due to random nonce");
}

#[test]
fn test_save_can_decrypt_and_retrieve_saved_credentials() {
    // Given: A new file path
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "my-secure-password";

    // When: Saving multiple credentials
    save_credential(&file_path, password, "key1", "value1").unwrap();
    save_credential(&file_path, password, "key2", "value2").unwrap();

    // Then: Should be able to decrypt and retrieve all values
    let credentials = list_credentials(&file_path, password).unwrap();
    assert_eq!(credentials.len(), 2);
    assert!(credentials.iter().any(|c| c.key == "key1" && c.value == "value1"));
    assert!(credentials.iter().any(|c| c.key == "key2" && c.value == "value2"));
}
