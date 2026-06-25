use lootbox::{read_credential, save_credential};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// ============================================================================
// Test Utilities
// ============================================================================

fn setup_test_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp dir")
}

fn get_test_file_path(dir: &TempDir, filename: &str) -> PathBuf {
    dir.path().join(filename)
}

// ============================================================================
// Success Cases - Reading Credentials by ID
// ============================================================================

#[test]
fn test_read_single_credential_by_id_1() {
    // Given: An encrypted file with one credential
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "api_key", "secret_value_123").unwrap();

    // When: Reading credential with ID 1
    let result = read_credential(&file_path, password, 1);

    // Then: Should return the credential with correct values
    assert!(result.is_ok());
    let credential = result.unwrap();
    assert_eq!(credential.key, "api_key");
    assert_eq!(credential.value, "secret_value_123");
}

#[test]
fn test_read_first_credential_from_multiple() {
    // Given: An encrypted file with three credentials
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "first_key", "first_value").unwrap();
    save_credential(&file_path, password, "second_key", "second_value").unwrap();
    save_credential(&file_path, password, "third_key", "third_value").unwrap();

    // When: Reading credential with ID 1
    let credential = read_credential(&file_path, password, 1).unwrap();

    // Then: Should return the first credential
    assert_eq!(credential.key, "first_key");
    assert_eq!(credential.value, "first_value");
}

#[test]
fn test_read_middle_credential_from_multiple() {
    // Given: An encrypted file with three credentials
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "first_key", "first_value").unwrap();
    save_credential(&file_path, password, "second_key", "second_value").unwrap();
    save_credential(&file_path, password, "third_key", "third_value").unwrap();

    // When: Reading credential with ID 2
    let credential = read_credential(&file_path, password, 2).unwrap();

    // Then: Should return the second credential
    assert_eq!(credential.key, "second_key");
    assert_eq!(credential.value, "second_value");
}

#[test]
fn test_read_last_credential_from_multiple() {
    // Given: An encrypted file with three credentials
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "first_key", "first_value").unwrap();
    save_credential(&file_path, password, "second_key", "second_value").unwrap();
    save_credential(&file_path, password, "third_key", "third_value").unwrap();

    // When: Reading credential with ID 3
    let credential = read_credential(&file_path, password, 3).unwrap();

    // Then: Should return the third credential
    assert_eq!(credential.key, "third_key");
    assert_eq!(credential.value, "third_value");
}

#[test]
fn test_read_returns_actual_value_not_masked() {
    // Given: An encrypted file with a credential
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";
    let secret_value = "ThisIsMySecretValue123";

    save_credential(&file_path, password, "api_key", secret_value).unwrap();

    // When: Reading credential with ID 1
    let credential = read_credential(&file_path, password, 1).unwrap();

    // Then: Should return the actual value, not masked
    assert_eq!(credential.value, secret_value);
    assert_ne!(credential.value, "**********");
}

#[test]
fn test_read_credential_with_special_characters() {
    // Given: An encrypted file with special characters in key and value
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";
    let special_key = "key@#$%^&*()";
    let special_value = "value!@#$%^&*(){}[]|\\:;\"'<>,.?/~`";

    save_credential(&file_path, password, special_key, special_value).unwrap();

    // When: Reading credential with ID 1
    let credential = read_credential(&file_path, password, 1).unwrap();

    // Then: Should return special characters correctly
    assert_eq!(credential.key, special_key);
    assert_eq!(credential.value, special_value);
}

#[test]
fn test_read_credential_with_unicode_characters() {
    // Given: An encrypted file with unicode characters
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";
    let unicode_key = "密钥🔑";
    let unicode_value = "秘密値🔐";

    save_credential(&file_path, password, unicode_key, unicode_value).unwrap();

    // When: Reading credential with ID 1
    let credential = read_credential(&file_path, password, 1).unwrap();

    // Then: Should return unicode characters correctly
    assert_eq!(credential.key, unicode_key);
    assert_eq!(credential.value, unicode_value);
}

#[test]
fn test_read_credential_with_newlines() {
    // Given: An encrypted file with newlines in key and value
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";
    let key_with_newline = "key\nwith\nnewlines";
    let value_with_newline = "value\nwith\nnewlines";

    save_credential(&file_path, password, key_with_newline, value_with_newline).unwrap();

    // When: Reading credential with ID 1
    let credential = read_credential(&file_path, password, 1).unwrap();

    // Then: Should return newlines correctly
    assert_eq!(credential.key, key_with_newline);
    assert_eq!(credential.value, value_with_newline);
}

#[test]
fn test_read_credential_with_very_long_values() {
    // Given: An encrypted file with very long key and value
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";
    let long_key = "k".repeat(64);
    let long_value = "v".repeat(5000);

    save_credential(&file_path, password, &long_key, &long_value).unwrap();

    // When: Reading credential with ID 1
    let credential = read_credential(&file_path, password, 1).unwrap();

    // Then: Should return full values without truncation
    assert_eq!(credential.key, long_key);
    assert_eq!(credential.value, long_value);
}

#[test]
fn test_read_retrieves_exact_saved_data() {
    // Given: Multiple credentials with various data
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "aws_key", "AKIAIOSFODNN7EXAMPLE").unwrap();
    save_credential(&file_path, password, "github_token", "ghp_1234567890abcdefghijklmnopqrstuv").unwrap();
    save_credential(&file_path, password, "db_password", "P@ssw0rd!2024").unwrap();

    // When: Reading each credential
    let cred1 = read_credential(&file_path, password, 1).unwrap();
    let cred2 = read_credential(&file_path, password, 2).unwrap();
    let cred3 = read_credential(&file_path, password, 3).unwrap();

    // Then: Should retrieve exactly what was saved
    assert_eq!(cred1.key, "aws_key");
    assert_eq!(cred1.value, "AKIAIOSFODNN7EXAMPLE");
    assert_eq!(cred2.key, "github_token");
    assert_eq!(cred2.value, "ghp_1234567890abcdefghijklmnopqrstuv");
    assert_eq!(cred3.key, "db_password");
    assert_eq!(cred3.value, "P@ssw0rd!2024");
}

// ============================================================================
// Error Cases - Invalid IDs
// ============================================================================

#[test]
fn test_read_fails_with_id_zero() {
    // Given: An encrypted file with credentials
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "key", "value").unwrap();

    // When: Attempting to read with ID 0 (invalid, IDs are 1-indexed)
    let result = read_credential(&file_path, password, 0);

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Invalid") || error_msg.contains("ID") || error_msg.contains("position"));
}

#[test]
fn test_read_fails_with_id_greater_than_count() {
    // Given: An encrypted file with 2 credentials
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "key1", "value1").unwrap();
    save_credential(&file_path, password, "key2", "value2").unwrap();

    // When: Attempting to read with ID 5 (only 2 credentials exist)
    let result = read_credential(&file_path, password, 5);

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Invalid") || error_msg.contains("ID") || error_msg.contains("position") || error_msg.contains("not found"));
}

#[test]
fn test_read_fails_with_id_one_more_than_count() {
    // Given: An encrypted file with 3 credentials
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "key1", "value1").unwrap();
    save_credential(&file_path, password, "key2", "value2").unwrap();
    save_credential(&file_path, password, "key3", "value3").unwrap();

    // When: Attempting to read with ID 4 (only 3 credentials exist)
    let result = read_credential(&file_path, password, 4);

    // Then: Should return an error
    assert!(result.is_err());
}

// ============================================================================
// Error Cases - File and Password Issues
// ============================================================================

#[test]
fn test_read_fails_with_wrong_password() {
    // Given: An encrypted file with correct password
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let correct_password = "correct123";
    let wrong_password = "wrong456789";

    save_credential(&file_path, correct_password, "key", "value").unwrap();

    // When: Attempting to read with wrong password
    let result = read_credential(&file_path, wrong_password, 1);

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("password") || error_msg.contains("decrypt") || error_msg.contains("authentication"));
}

#[test]
fn test_read_fails_when_file_does_not_exist() {
    // Given: A non-existent file path
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "nonexistent.enc");

    assert!(!file_path.exists());

    // When: Attempting to read credential
    let result = read_credential(&file_path, "password123", 1);

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("not found") || error_msg.contains("does not exist"));
}

#[test]
fn test_read_fails_with_corrupted_file() {
    // Given: A corrupted file (not properly encrypted)
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "corrupted.enc");

    fs::write(&file_path, "this is not encrypted data").expect("Failed to write corrupted file");

    // When: Attempting to read credential
    let result = read_credential(&file_path, "password123", 1);

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("decrypt") || error_msg.contains("invalid") || error_msg.contains("corrupted") || error_msg.contains("unsupported"));
}

#[test]
fn test_read_fails_with_empty_file() {
    // Given: An empty file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "empty.enc");

    fs::write(&file_path, "").expect("Failed to create empty file");

    // When: Attempting to read credential
    let result = read_credential(&file_path, "password123", 1);

    // Then: Should return an error
    assert!(result.is_err());
}

#[test]
fn test_read_fails_with_truncated_encrypted_file() {
    // Given: A truncated encrypted file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "key", "value").unwrap();

    // Truncate the file
    let mut file_content = fs::read(&file_path).unwrap();
    file_content.truncate(file_content.len() / 2);
    fs::write(&file_path, file_content).unwrap();

    // When: Attempting to read credential
    let result = read_credential(&file_path, "password123", 1);

    // Then: Should return an error
    assert!(result.is_err());
}

// ============================================================================
// Password Validation Tests
// ============================================================================

#[test]
fn test_read_with_empty_password() {
    // Given: An encrypted file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "key", "value").unwrap();

    // When: Attempting to read with empty password
    let result = read_credential(&file_path, "", 1);

    // Then: Should return an error (password validation)
    assert!(result.is_err());
}

#[test]
fn test_read_with_password_less_than_8_characters() {
    // Given: An encrypted file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "key", "value").unwrap();

    // When: Attempting to read with 7-character password
    let result = read_credential(&file_path, "pass123", 1);

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("8") || error_msg.contains("length") || error_msg.contains("characters"));
}

#[test]
fn test_read_with_password_only_whitespace() {
    // Given: An encrypted file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "key", "value").unwrap();

    // When: Attempting to read with whitespace-only password
    let result = read_credential(&file_path, "        ", 1);

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("whitespace") || error_msg.contains("invalid"));
}

#[test]
fn test_read_with_password_starting_with_whitespace() {
    // Given: An encrypted file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "key", "value").unwrap();

    // When: Attempting to read with password starting with whitespace
    let result = read_credential(&file_path, " password123", 1);

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("whitespace") || error_msg.contains("space"));
}

#[test]
fn test_read_with_password_ending_with_whitespace() {
    // Given: An encrypted file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "key", "value").unwrap();

    // When: Attempting to read with password ending with whitespace
    let result = read_credential(&file_path, "password123 ", 1);

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("whitespace") || error_msg.contains("space"));
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_read_after_multiple_saves_returns_correct_credential() {
    // Given: Multiple saves to the same file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "first", "value1").unwrap();
    save_credential(&file_path, password, "second", "value2").unwrap();
    save_credential(&file_path, password, "third", "value3").unwrap();
    save_credential(&file_path, password, "fourth", "value4").unwrap();
    save_credential(&file_path, password, "fifth", "value5").unwrap();

    // When: Reading specific credentials by ID
    let cred2 = read_credential(&file_path, password, 2).unwrap();
    let cred4 = read_credential(&file_path, password, 4).unwrap();

    // Then: Should return the correct credentials
    assert_eq!(cred2.key, "second");
    assert_eq!(cred2.value, "value2");
    assert_eq!(cred4.key, "fourth");
    assert_eq!(cred4.value, "value4");
}

#[test]
fn test_read_with_duplicate_keys_returns_correct_by_id() {
    // Given: An encrypted file with duplicate keys
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "api_key", "first_value").unwrap();
    save_credential(&file_path, password, "api_key", "second_value").unwrap();
    save_credential(&file_path, password, "api_key", "third_value").unwrap();

    // When: Reading by ID
    let cred1 = read_credential(&file_path, password, 1).unwrap();
    let cred2 = read_credential(&file_path, password, 2).unwrap();
    let cred3 = read_credential(&file_path, password, 3).unwrap();

    // Then: Should return correct credentials based on position, not key
    assert_eq!(cred1.value, "first_value");
    assert_eq!(cred2.value, "second_value");
    assert_eq!(cred3.value, "third_value");
}

#[test]
fn test_read_single_character_value() {
    // Given: A credential with single character value
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "key", "x").unwrap();

    // When: Reading the credential
    let credential = read_credential(&file_path, password, 1).unwrap();

    // Then: Should return the single character value correctly
    assert_eq!(credential.value, "x");
}

#[test]
fn test_read_empty_key_if_saved() {
    // Given: A file that might have credentials
    // Note: This test assumes validation may allow or reject empty keys
    // If empty keys are not allowed during save, this test will document that behavior
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    // This should fail during save due to validation
    let save_result = save_credential(&file_path, "password123", "", "value");

    // Then: Save should fail (based on existing validation)
    assert!(save_result.is_err());
}
