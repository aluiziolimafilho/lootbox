use lootbox::{list_credentials, save_credential, update_credential};
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
// Success Cases - Updating Credentials
// ============================================================================

#[test]
fn test_update_both_key_and_value() {
    // Given: An encrypted file with a credential
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "old_key", "old_value").unwrap();

    // When: Updating both key and value
    let result = update_credential(&file_path, password, 1, Some("new_key"), Some("new_value"));

    // Then: Should succeed and update the credential
    assert!(result.is_ok());

    let credentials = list_credentials(&file_path, password).unwrap();
    assert_eq!(credentials.len(), 1);
    assert_eq!(credentials[0].key, "new_key");
    assert_eq!(credentials[0].value, "new_value");
}

#[test]
fn test_update_only_key_keeps_value() {
    // Given: An encrypted file with a credential
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "old_key", "original_value").unwrap();

    // When: Updating only the key (value is None)
    let result = update_credential(&file_path, password, 1, Some("new_key"), None);

    // Then: Should update key but keep original value
    assert!(result.is_ok());

    let credentials = list_credentials(&file_path, password).unwrap();
    assert_eq!(credentials[0].key, "new_key");
    assert_eq!(credentials[0].value, "original_value");
}

#[test]
fn test_update_only_value_keeps_key() {
    // Given: An encrypted file with a credential
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "original_key", "old_value").unwrap();

    // When: Updating only the value (key is None)
    let result = update_credential(&file_path, password, 1, None, Some("new_value"));

    // Then: Should update value but keep original key
    assert!(result.is_ok());

    let credentials = list_credentials(&file_path, password).unwrap();
    assert_eq!(credentials[0].key, "original_key");
    assert_eq!(credentials[0].value, "new_value");
}

#[test]
fn test_update_with_both_none_keeps_both() {
    // Given: An encrypted file with a credential
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "original_key", "original_value").unwrap();

    // When: Updating with both None (simulating user pressing enter twice)
    let result = update_credential(&file_path, password, 1, None, None);

    // Then: Should succeed but keep both values unchanged
    assert!(result.is_ok());

    let credentials = list_credentials(&file_path, password).unwrap();
    assert_eq!(credentials[0].key, "original_key");
    assert_eq!(credentials[0].value, "original_value");
}

#[test]
fn test_update_specific_credential_in_multiple() {
    // Given: An encrypted file with multiple credentials
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "key1", "value1").unwrap();
    save_credential(&file_path, password, "key2", "value2").unwrap();
    save_credential(&file_path, password, "key3", "value3").unwrap();

    // When: Updating only the second credential (ID 2)
    let result = update_credential(&file_path, password, 2, Some("updated_key2"), Some("updated_value2"));

    // Then: Should update only the second credential
    assert!(result.is_ok());

    let credentials = list_credentials(&file_path, password).unwrap();
    assert_eq!(credentials.len(), 3);
    assert_eq!(credentials[0].key, "key1");
    assert_eq!(credentials[0].value, "value1");
    assert_eq!(credentials[1].key, "updated_key2");
    assert_eq!(credentials[1].value, "updated_value2");
    assert_eq!(credentials[2].key, "key3");
    assert_eq!(credentials[2].value, "value3");
}

#[test]
fn test_update_first_credential() {
    // Given: Multiple credentials
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "first", "value1").unwrap();
    save_credential(&file_path, password, "second", "value2").unwrap();
    save_credential(&file_path, password, "third", "value3").unwrap();

    // When: Updating first credential
    update_credential(&file_path, password, 1, Some("updated_first"), None).unwrap();

    // Then: First credential should be updated
    let credentials = list_credentials(&file_path, password).unwrap();
    assert_eq!(credentials[0].key, "updated_first");
    assert_eq!(credentials[0].value, "value1");
    assert_eq!(credentials[1].key, "second");
    assert_eq!(credentials[2].key, "third");
}

#[test]
fn test_update_last_credential() {
    // Given: Multiple credentials
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "first", "value1").unwrap();
    save_credential(&file_path, password, "second", "value2").unwrap();
    save_credential(&file_path, password, "third", "value3").unwrap();

    // When: Updating last credential
    update_credential(&file_path, password, 3, None, Some("updated_value3")).unwrap();

    // Then: Last credential should be updated
    let credentials = list_credentials(&file_path, password).unwrap();
    assert_eq!(credentials[2].key, "third");
    assert_eq!(credentials[2].value, "updated_value3");
    assert_eq!(credentials[0].key, "first");
    assert_eq!(credentials[1].key, "second");
}

#[test]
fn test_update_preserves_order_of_credentials() {
    // Given: Multiple credentials
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "a", "1").unwrap();
    save_credential(&file_path, password, "b", "2").unwrap();
    save_credential(&file_path, password, "c", "3").unwrap();

    // When: Updating middle credential
    update_credential(&file_path, password, 2, Some("b_updated"), Some("2_updated")).unwrap();

    // Then: Order should be preserved
    let credentials = list_credentials(&file_path, password).unwrap();
    assert_eq!(credentials[0].key, "a");
    assert_eq!(credentials[1].key, "b_updated");
    assert_eq!(credentials[2].key, "c");
}

#[test]
fn test_update_with_special_characters() {
    // Given: A credential
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "simple_key", "simple_value").unwrap();

    // When: Updating with special characters
    let special_key = "key@#$%^&*()";
    let special_value = "value!@#$%^&*(){}[]|\\:;\"'<>,.?/~`";
    update_credential(&file_path, password, 1, Some(special_key), Some(special_value)).unwrap();

    // Then: Should store special characters correctly
    let credentials = list_credentials(&file_path, password).unwrap();
    assert_eq!(credentials[0].key, special_key);
    assert_eq!(credentials[0].value, special_value);
}

#[test]
fn test_update_with_unicode_characters() {
    // Given: A credential
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "key", "value").unwrap();

    // When: Updating with unicode
    let unicode_key = "密钥🔑";
    let unicode_value = "秘密値🔐";
    update_credential(&file_path, password, 1, Some(unicode_key), Some(unicode_value)).unwrap();

    // Then: Should store unicode correctly
    let credentials = list_credentials(&file_path, password).unwrap();
    assert_eq!(credentials[0].key, unicode_key);
    assert_eq!(credentials[0].value, unicode_value);
}

#[test]
fn test_update_with_very_long_values() {
    // Given: A credential
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "key", "value").unwrap();

    // When: Updating with values at the maximum allowed lengths
    let long_key = "k".repeat(64);
    let long_value = "v".repeat(5000);
    update_credential(&file_path, password, 1, Some(&long_key), Some(&long_value)).unwrap();

    // Then: Should store long values correctly
    let credentials = list_credentials(&file_path, password).unwrap();
    assert_eq!(credentials[0].key, long_key);
    assert_eq!(credentials[0].value, long_value);
}

#[test]
fn test_update_with_newlines() {
    // Given: A credential
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "key", "value").unwrap();

    // When: Updating with newlines
    let key_with_newline = "key\nwith\nnewlines";
    let value_with_newline = "value\nwith\nnewlines";
    update_credential(&file_path, password, 1, Some(key_with_newline), Some(value_with_newline)).unwrap();

    // Then: Should store newlines correctly
    let credentials = list_credentials(&file_path, password).unwrap();
    assert_eq!(credentials[0].key, key_with_newline);
    assert_eq!(credentials[0].value, value_with_newline);
}

#[test]
fn test_update_after_multiple_updates() {
    // Given: A credential that has been updated multiple times
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "key1", "value1").unwrap();

    // When: Performing multiple updates
    update_credential(&file_path, password, 1, Some("key2"), Some("value2")).unwrap();
    update_credential(&file_path, password, 1, Some("key3"), None).unwrap();
    update_credential(&file_path, password, 1, None, Some("value3")).unwrap();

    // Then: Should have the final updated values
    let credentials = list_credentials(&file_path, password).unwrap();
    assert_eq!(credentials[0].key, "key3");
    assert_eq!(credentials[0].value, "value3");
}

// ============================================================================
// Error Cases - Invalid IDs
// ============================================================================

#[test]
fn test_update_fails_with_id_zero() {
    // Given: An encrypted file with credentials
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "key", "value").unwrap();

    // When: Attempting to update with ID 0
    let result = update_credential(&file_path, password, 0, Some("new_key"), Some("new_value"));

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Invalid") || error_msg.contains("ID") || error_msg.contains("position"));
}

#[test]
fn test_update_fails_with_id_greater_than_count() {
    // Given: An encrypted file with 2 credentials
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "key1", "value1").unwrap();
    save_credential(&file_path, password, "key2", "value2").unwrap();

    // When: Attempting to update with ID 5
    let result = update_credential(&file_path, password, 5, Some("new"), Some("new"));

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Invalid") || error_msg.contains("ID") || error_msg.contains("not found"));
}

#[test]
fn test_update_fails_with_id_one_more_than_count() {
    // Given: An encrypted file with 3 credentials
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "key1", "value1").unwrap();
    save_credential(&file_path, password, "key2", "value2").unwrap();
    save_credential(&file_path, password, "key3", "value3").unwrap();

    // When: Attempting to update with ID 4
    let result = update_credential(&file_path, password, 4, Some("new"), Some("new"));

    // Then: Should return an error
    assert!(result.is_err());
}

// ============================================================================
// Error Cases - File and Password Issues
// ============================================================================

#[test]
fn test_update_fails_with_wrong_password() {
    // Given: An encrypted file with correct password
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let correct_password = "correct123";
    let wrong_password = "wrong456789";

    save_credential(&file_path, correct_password, "key", "value").unwrap();

    // When: Attempting to update with wrong password
    let result = update_credential(&file_path, wrong_password, 1, Some("new"), Some("new"));

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("password") || error_msg.contains("decrypt") || error_msg.contains("authentication"));
}

#[test]
fn test_update_fails_when_file_does_not_exist() {
    // Given: A non-existent file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "nonexistent.enc");

    assert!(!file_path.exists());

    // When: Attempting to update
    let result = update_credential(&file_path, "password123", 1, Some("new"), Some("new"));

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("not found") || error_msg.contains("does not exist"));
}

#[test]
fn test_update_fails_with_corrupted_file() {
    // Given: A corrupted file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "corrupted.enc");

    fs::write(&file_path, "this is not encrypted data").expect("Failed to write corrupted file");

    // When: Attempting to update
    let result = update_credential(&file_path, "password123", 1, Some("new"), Some("new"));

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("decrypt") || error_msg.contains("invalid") || error_msg.contains("corrupted") || error_msg.contains("unsupported"));
}

#[test]
fn test_update_fails_with_empty_file() {
    // Given: An empty file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "empty.enc");

    fs::write(&file_path, "").expect("Failed to create empty file");

    // When: Attempting to update
    let result = update_credential(&file_path, "password123", 1, Some("new"), Some("new"));

    // Then: Should return an error
    assert!(result.is_err());
}

#[test]
fn test_update_fails_with_truncated_encrypted_file() {
    // Given: A truncated encrypted file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "key", "value").unwrap();

    // Truncate the file
    let mut file_content = fs::read(&file_path).unwrap();
    file_content.truncate(file_content.len() / 2);
    fs::write(&file_path, file_content).unwrap();

    // When: Attempting to update
    let result = update_credential(&file_path, "password123", 1, Some("new"), Some("new"));

    // Then: Should return an error
    assert!(result.is_err());
}

// ============================================================================
// Password Validation Tests
// ============================================================================

#[test]
fn test_update_with_empty_password() {
    // Given: An encrypted file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "key", "value").unwrap();

    // When: Attempting to update with empty password
    let result = update_credential(&file_path, "", 1, Some("new"), Some("new"));

    // Then: Should return an error (password validation)
    assert!(result.is_err());
}

#[test]
fn test_update_with_password_less_than_8_characters() {
    // Given: An encrypted file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "key", "value").unwrap();

    // When: Attempting to update with 7-character password
    let result = update_credential(&file_path, "pass123", 1, Some("new"), Some("new"));

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("8") || error_msg.contains("length") || error_msg.contains("characters"));
}

#[test]
fn test_update_with_password_only_whitespace() {
    // Given: An encrypted file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "key", "value").unwrap();

    // When: Attempting to update with whitespace-only password
    let result = update_credential(&file_path, "        ", 1, Some("new"), Some("new"));

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("whitespace") || error_msg.contains("invalid"));
}

#[test]
fn test_update_with_password_starting_with_whitespace() {
    // Given: An encrypted file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "key", "value").unwrap();

    // When: Attempting to update with password starting with whitespace
    let result = update_credential(&file_path, " password123", 1, Some("new"), Some("new"));

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("whitespace") || error_msg.contains("space"));
}

#[test]
fn test_update_with_password_ending_with_whitespace() {
    // Given: An encrypted file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "key", "value").unwrap();

    // When: Attempting to update with password ending with whitespace
    let result = update_credential(&file_path, "password123 ", 1, Some("new"), Some("new"));

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("whitespace") || error_msg.contains("space"));
}

// ============================================================================
// Validation Tests - New Key/Value
// ============================================================================

#[test]
fn test_update_fails_with_empty_new_key() {
    // Given: An encrypted file with a credential
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "old_key", "old_value").unwrap();

    // When: Attempting to update with empty string key (not None, but empty string)
    let result = update_credential(&file_path, password, 1, Some(""), Some("new_value"));

    // Then: Should return an error (empty key is invalid)
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("key") || error_msg.contains("empty") || error_msg.contains("required"));
}

#[test]
fn test_update_fails_with_empty_new_value() {
    // Given: An encrypted file with a credential
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "old_key", "old_value").unwrap();

    // When: Attempting to update with empty string value (not None, but empty string)
    let result = update_credential(&file_path, password, 1, Some("new_key"), Some(""));

    // Then: Should return an error (empty value is invalid)
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("value") || error_msg.contains("empty") || error_msg.contains("required"));
}

#[test]
fn test_update_fails_with_whitespace_only_new_key() {
    // Given: An encrypted file with a credential
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "old_key", "old_value").unwrap();

    // When: Attempting to update with whitespace-only key
    let result = update_credential(&file_path, password, 1, Some("   "), Some("new_value"));

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("whitespace") || error_msg.contains("key"));
}

#[test]
fn test_update_fails_with_whitespace_only_new_value() {
    // Given: An encrypted file with a credential
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "old_key", "old_value").unwrap();

    // When: Attempting to update with whitespace-only value
    let result = update_credential(&file_path, password, 1, Some("new_key"), Some("   "));

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("whitespace") || error_msg.contains("value"));
}

#[test]
fn test_update_with_secret_key_exactly_64_characters() {
    // Given: An encrypted file with a credential
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "old_key", "old_value").unwrap();

    // When: Updating with exactly 64-character key (maximum allowed)
    let key = "a".repeat(64);
    let result = update_credential(&file_path, password, 1, Some(&key), None);

    // Then: Should succeed
    assert!(result.is_ok());
}

#[test]
fn test_update_with_secret_key_exceeding_64_characters() {
    // Given: An encrypted file with a credential
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "old_key", "old_value").unwrap();

    // When: Attempting to update with 65-character key (exceeds maximum)
    let key = "a".repeat(65);
    let result = update_credential(&file_path, password, 1, Some(&key), None);

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("64") || error_msg.contains("maximum") || error_msg.contains("characters"));
}

#[test]
fn test_update_with_secret_value_exactly_5000_characters() {
    // Given: An encrypted file with a credential
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "old_key", "old_value").unwrap();

    // When: Updating with exactly 5000-character value (maximum allowed)
    let value = "a".repeat(5000);
    let result = update_credential(&file_path, password, 1, None, Some(&value));

    // Then: Should succeed
    assert!(result.is_ok());
}

#[test]
fn test_update_with_secret_value_exceeding_5000_characters() {
    // Given: An encrypted file with a credential
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "old_key", "old_value").unwrap();

    // When: Attempting to update with 5001-character value (exceeds maximum)
    let value = "a".repeat(5001);
    let result = update_credential(&file_path, password, 1, None, Some(&value));

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("5000") || error_msg.contains("maximum") || error_msg.contains("characters"));
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_update_does_not_affect_file_if_error() {
    // Given: An encrypted file with credentials
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "key1", "value1").unwrap();
    save_credential(&file_path, password, "key2", "value2").unwrap();

    // When: Attempting to update with invalid ID
    let result = update_credential(&file_path, password, 10, Some("new"), Some("new"));

    // Then: Should fail and file should remain unchanged
    assert!(result.is_err());

    let credentials = list_credentials(&file_path, password).unwrap();
    assert_eq!(credentials.len(), 2);
    assert_eq!(credentials[0].key, "key1");
    assert_eq!(credentials[1].key, "key2");
}

#[test]
fn test_update_single_credential_file() {
    // Given: A file with only one credential
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "only_key", "only_value").unwrap();

    // When: Updating the single credential
    update_credential(&file_path, password, 1, Some("updated_key"), Some("updated_value")).unwrap();

    // Then: Should update successfully
    let credentials = list_credentials(&file_path, password).unwrap();
    assert_eq!(credentials.len(), 1);
    assert_eq!(credentials[0].key, "updated_key");
    assert_eq!(credentials[0].value, "updated_value");
}

#[test]
fn test_update_can_create_duplicate_key() {
    // Given: Multiple credentials with different keys
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "key1", "value1").unwrap();
    save_credential(&file_path, password, "key2", "value2").unwrap();

    // When: Updating second credential to have same key as first
    update_credential(&file_path, password, 2, Some("key1"), Some("updated_value2")).unwrap();

    // Then: Should allow duplicate keys (they're distinguished by position)
    let credentials = list_credentials(&file_path, password).unwrap();
    assert_eq!(credentials[0].key, "key1");
    assert_eq!(credentials[0].value, "value1");
    assert_eq!(credentials[1].key, "key1");
    assert_eq!(credentials[1].value, "updated_value2");
}
