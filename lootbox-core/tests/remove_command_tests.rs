use lootbox::{list_credentials, remove_credential, save_credential};
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
// Success Cases - Removing Credentials
// ============================================================================

#[test]
fn test_remove_only_credential_leaves_empty_file() {
    // Given: An encrypted file with a single credential
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "only_key", "only_value", None, None, None).unwrap();

    // When: Removing the only credential
    let result = remove_credential(&file_path, password, 1);

    // Then: Should succeed and file should have no credentials
    assert!(result.is_ok());

    let credentials = list_credentials(&file_path, password).unwrap();
    assert_eq!(credentials.len(), 0);
}

#[test]
fn test_remove_first_credential_from_multiple() {
    // Given: An encrypted file with multiple credentials
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "key1", "value1", None, None, None).unwrap();
    save_credential(&file_path, password, "key2", "value2", None, None, None).unwrap();
    save_credential(&file_path, password, "key3", "value3", None, None, None).unwrap();

    // When: Removing the first credential (ID 1)
    let result = remove_credential(&file_path, password, 1);

    // Then: Should succeed and shift remaining credentials
    assert!(result.is_ok());

    let credentials = list_credentials(&file_path, password).unwrap();
    assert_eq!(credentials.len(), 2);
    assert_eq!(credentials[0].key, "key2");
    assert_eq!(credentials[0].value, "value2");
    assert_eq!(credentials[1].key, "key3");
    assert_eq!(credentials[1].value, "value3");
}

#[test]
fn test_remove_middle_credential_from_multiple() {
    // Given: An encrypted file with multiple credentials
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "key1", "value1", None, None, None).unwrap();
    save_credential(&file_path, password, "key2", "value2", None, None, None).unwrap();
    save_credential(&file_path, password, "key3", "value3", None, None, None).unwrap();

    // When: Removing the middle credential (ID 2)
    let result = remove_credential(&file_path, password, 2);

    // Then: Should succeed, first and third remain
    assert!(result.is_ok());

    let credentials = list_credentials(&file_path, password).unwrap();
    assert_eq!(credentials.len(), 2);
    assert_eq!(credentials[0].key, "key1");
    assert_eq!(credentials[0].value, "value1");
    assert_eq!(credentials[1].key, "key3");
    assert_eq!(credentials[1].value, "value3");
}

#[test]
fn test_remove_last_credential_from_multiple() {
    // Given: An encrypted file with multiple credentials
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "key1", "value1", None, None, None).unwrap();
    save_credential(&file_path, password, "key2", "value2", None, None, None).unwrap();
    save_credential(&file_path, password, "key3", "value3", None, None, None).unwrap();

    // When: Removing the last credential (ID 3)
    let result = remove_credential(&file_path, password, 3);

    // Then: Should succeed, first two remain unchanged
    assert!(result.is_ok());

    let credentials = list_credentials(&file_path, password).unwrap();
    assert_eq!(credentials.len(), 2);
    assert_eq!(credentials[0].key, "key1");
    assert_eq!(credentials[1].key, "key2");
}

#[test]
fn test_remove_decreases_count_by_one() {
    // Given: An encrypted file with 4 credentials
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "key1", "value1", None, None, None).unwrap();
    save_credential(&file_path, password, "key2", "value2", None, None, None).unwrap();
    save_credential(&file_path, password, "key3", "value3", None, None, None).unwrap();
    save_credential(&file_path, password, "key4", "value4", None, None, None).unwrap();

    // When: Removing one credential
    remove_credential(&file_path, password, 2).unwrap();

    // Then: Count should be 3
    let credentials = list_credentials(&file_path, password).unwrap();
    assert_eq!(credentials.len(), 3);
}

#[test]
fn test_remove_preserves_order_of_remaining_credentials() {
    // Given: An encrypted file with credentials
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "a", "1", None, None, None).unwrap();
    save_credential(&file_path, password, "b", "2", None, None, None).unwrap();
    save_credential(&file_path, password, "c", "3", None, None, None).unwrap();
    save_credential(&file_path, password, "d", "4", None, None, None).unwrap();

    // When: Removing the second credential
    remove_credential(&file_path, password, 2).unwrap();

    // Then: Remaining credentials preserve their original order
    let credentials = list_credentials(&file_path, password).unwrap();
    assert_eq!(credentials[0].key, "a");
    assert_eq!(credentials[0].value, "1");
    assert_eq!(credentials[1].key, "c");
    assert_eq!(credentials[1].value, "3");
    assert_eq!(credentials[2].key, "d");
    assert_eq!(credentials[2].value, "4");
}

#[test]
fn test_remove_shifts_ids_of_subsequent_credentials() {
    // Given: Three credentials
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "key1", "value1", None, None, None).unwrap();
    save_credential(&file_path, password, "key2", "value2", None, None, None).unwrap();
    save_credential(&file_path, password, "key3", "value3", None, None, None).unwrap();

    // When: Removing credential 1, then removing what was credential 3 (now at ID 2)
    remove_credential(&file_path, password, 1).unwrap();
    remove_credential(&file_path, password, 2).unwrap();

    // Then: Only original key2 should remain at ID 1
    let credentials = list_credentials(&file_path, password).unwrap();
    assert_eq!(credentials.len(), 1);
    assert_eq!(credentials[0].key, "key2");
    assert_eq!(credentials[0].value, "value2");
}

#[test]
fn test_remove_all_credentials_sequentially() {
    // Given: Three credentials
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "key1", "value1", None, None, None).unwrap();
    save_credential(&file_path, password, "key2", "value2", None, None, None).unwrap();
    save_credential(&file_path, password, "key3", "value3", None, None, None).unwrap();

    // When: Removing all credentials one by one (always removing ID 1)
    remove_credential(&file_path, password, 1).unwrap();
    remove_credential(&file_path, password, 1).unwrap();
    remove_credential(&file_path, password, 1).unwrap();

    // Then: File should have no credentials
    let credentials = list_credentials(&file_path, password).unwrap();
    assert_eq!(credentials.len(), 0);
}

#[test]
fn test_remove_does_not_alter_other_credentials_data() {
    // Given: Credentials with specific values
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "key1", "sensitive_value_1", None, None, None).unwrap();
    save_credential(&file_path, password, "key2", "sensitive_value_2", None, None, None).unwrap();
    save_credential(&file_path, password, "key3", "sensitive_value_3", None, None, None).unwrap();

    // When: Removing the middle credential
    remove_credential(&file_path, password, 2).unwrap();

    // Then: Keys and values of remaining credentials are byte-for-byte unchanged
    let credentials = list_credentials(&file_path, password).unwrap();
    assert_eq!(credentials[0].key, "key1");
    assert_eq!(credentials[0].value, "sensitive_value_1");
    assert_eq!(credentials[1].key, "key3");
    assert_eq!(credentials[1].value, "sensitive_value_3");
}

// ============================================================================
// Error Cases - Invalid IDs
// ============================================================================

#[test]
fn test_remove_fails_with_id_zero() {
    // Given: An encrypted file with a credential
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "key", "value", None, None, None).unwrap();

    // When: Attempting to remove with ID 0
    let result = remove_credential(&file_path, password, 0);

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Invalid") || error_msg.contains("ID") || error_msg.contains("position"));
}

#[test]
fn test_remove_fails_with_id_greater_than_count() {
    // Given: An encrypted file with 2 credentials
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "key1", "value1", None, None, None).unwrap();
    save_credential(&file_path, password, "key2", "value2", None, None, None).unwrap();

    // When: Attempting to remove with ID 5
    let result = remove_credential(&file_path, password, 5);

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Invalid") || error_msg.contains("ID") || error_msg.contains("not found"));
}

#[test]
fn test_remove_fails_with_id_one_more_than_count() {
    // Given: An encrypted file with 3 credentials
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "key1", "value1", None, None, None).unwrap();
    save_credential(&file_path, password, "key2", "value2", None, None, None).unwrap();
    save_credential(&file_path, password, "key3", "value3", None, None, None).unwrap();

    // When: Attempting to remove with ID 4 (one beyond last)
    let result = remove_credential(&file_path, password, 4);

    // Then: Should return an error
    assert!(result.is_err());
}

#[test]
fn test_remove_fails_on_empty_credential_list() {
    // Given: A file whose credential list has been emptied
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "key", "value", None, None, None).unwrap();
    remove_credential(&file_path, password, 1).unwrap();

    // When: Attempting to remove from an empty list
    let result = remove_credential(&file_path, password, 1);

    // Then: Should return an error
    assert!(result.is_err());
}

// ============================================================================
// Error Cases - File and Password Issues
// ============================================================================

#[test]
fn test_remove_fails_with_wrong_password() {
    // Given: An encrypted file with correct password
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let correct_password = "correct123";
    let wrong_password = "wrong456789";

    save_credential(&file_path, correct_password, "key", "value", None, None, None).unwrap();

    // When: Attempting to remove with wrong password
    let result = remove_credential(&file_path, wrong_password, 1);

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("password") || error_msg.contains("decrypt") || error_msg.contains("authentication"));
}

#[test]
fn test_remove_fails_when_file_does_not_exist() {
    // Given: A non-existent file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "nonexistent.enc");

    assert!(!file_path.exists());

    // When: Attempting to remove
    let result = remove_credential(&file_path, "password123", 1);

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("not found") || error_msg.contains("does not exist"));
}

#[test]
fn test_remove_fails_with_corrupted_file() {
    // Given: A corrupted file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "corrupted.enc");

    fs::write(&file_path, "this is not encrypted data").expect("Failed to write corrupted file");

    // When: Attempting to remove
    let result = remove_credential(&file_path, "password123", 1);

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("decrypt") || error_msg.contains("invalid") || error_msg.contains("corrupted") || error_msg.contains("unsupported"));
}

#[test]
fn test_remove_fails_with_empty_file() {
    // Given: An empty file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "empty.enc");

    fs::write(&file_path, "").expect("Failed to create empty file");

    // When: Attempting to remove
    let result = remove_credential(&file_path, "password123", 1);

    // Then: Should return an error
    assert!(result.is_err());
}

#[test]
fn test_remove_fails_with_truncated_encrypted_file() {
    // Given: A truncated encrypted file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "key", "value", None, None, None).unwrap();

    let mut file_content = fs::read(&file_path).unwrap();
    file_content.truncate(file_content.len() / 2);
    fs::write(&file_path, file_content).unwrap();

    // When: Attempting to remove
    let result = remove_credential(&file_path, "password123", 1);

    // Then: Should return an error
    assert!(result.is_err());
}

// ============================================================================
// Password Validation Tests
// ============================================================================

#[test]
fn test_remove_with_empty_password() {
    // Given: An encrypted file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "key", "value", None, None, None).unwrap();

    // When: Attempting to remove with empty password
    let result = remove_credential(&file_path, "", 1);

    // Then: Should return an error (password validation)
    assert!(result.is_err());
}

#[test]
fn test_remove_with_password_less_than_8_characters() {
    // Given: An encrypted file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "key", "value", None, None, None).unwrap();

    // When: Attempting to remove with 7-character password
    let result = remove_credential(&file_path, "pass123", 1);

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("8") || error_msg.contains("length") || error_msg.contains("characters"));
}

#[test]
fn test_remove_with_password_only_whitespace() {
    // Given: An encrypted file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "key", "value", None, None, None).unwrap();

    // When: Attempting to remove with whitespace-only password
    let result = remove_credential(&file_path, "        ", 1);

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("whitespace") || error_msg.contains("invalid"));
}

#[test]
fn test_remove_with_password_starting_with_whitespace() {
    // Given: An encrypted file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "key", "value", None, None, None).unwrap();

    // When: Attempting to remove with password starting with whitespace
    let result = remove_credential(&file_path, " password123", 1);

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("whitespace") || error_msg.contains("space"));
}

#[test]
fn test_remove_with_password_ending_with_whitespace() {
    // Given: An encrypted file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "key", "value", None, None, None).unwrap();

    // When: Attempting to remove with password ending with whitespace
    let result = remove_credential(&file_path, "password123 ", 1);

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("whitespace") || error_msg.contains("space"));
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_remove_does_not_affect_file_if_error() {
    // Given: An encrypted file with credentials
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "key1", "value1", None, None, None).unwrap();
    save_credential(&file_path, password, "key2", "value2", None, None, None).unwrap();

    // When: Attempting to remove with an invalid ID
    let result = remove_credential(&file_path, password, 10);

    // Then: Should fail and file should remain unchanged
    assert!(result.is_err());

    let credentials = list_credentials(&file_path, password).unwrap();
    assert_eq!(credentials.len(), 2);
    assert_eq!(credentials[0].key, "key1");
    assert_eq!(credentials[1].key, "key2");
}

#[test]
fn test_remove_with_special_characters_in_remaining_credentials() {
    // Given: Credentials containing special characters
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    let special_key = "key@#$%^&*()";
    let special_value = "value!@#$%^&*(){}[]|\\:;\"'<>,.?/~`";

    save_credential(&file_path, password, "key1", "value1", None, None, None).unwrap();
    save_credential(&file_path, password, special_key, special_value, None, None, None).unwrap();

    // When: Removing the first credential
    remove_credential(&file_path, password, 1).unwrap();

    // Then: The credential with special characters should remain intact
    let credentials = list_credentials(&file_path, password).unwrap();
    assert_eq!(credentials.len(), 1);
    assert_eq!(credentials[0].key, special_key);
    assert_eq!(credentials[0].value, special_value);
}

#[test]
fn test_remove_with_unicode_characters_in_remaining_credentials() {
    // Given: Credentials containing unicode
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    let unicode_key = "密钥🔑";
    let unicode_value = "秘密値🔐";

    save_credential(&file_path, password, "key1", "value1", None, None, None).unwrap();
    save_credential(&file_path, password, unicode_key, unicode_value, None, None, None).unwrap();

    // When: Removing the first credential
    remove_credential(&file_path, password, 1).unwrap();

    // Then: The unicode credential should remain intact
    let credentials = list_credentials(&file_path, password).unwrap();
    assert_eq!(credentials.len(), 1);
    assert_eq!(credentials[0].key, unicode_key);
    assert_eq!(credentials[0].value, unicode_value);
}

#[test]
fn test_remove_file_remains_accessible_after_removal() {
    // Given: A file with two credentials
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "key1", "value1", None, None, None).unwrap();
    save_credential(&file_path, password, "key2", "value2", None, None, None).unwrap();

    // When: Removing one credential
    remove_credential(&file_path, password, 1).unwrap();

    // Then: File still exists and is readable with the same password
    assert!(file_path.exists());
    let credentials = list_credentials(&file_path, password).unwrap();
    assert_eq!(credentials.len(), 1);
}

#[test]
fn test_remove_then_save_appends_correctly() {
    // Given: A file with two credentials where one is removed
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "key1", "value1", None, None, None).unwrap();
    save_credential(&file_path, password, "key2", "value2", None, None, None).unwrap();
    remove_credential(&file_path, password, 1).unwrap();

    // When: Saving a new credential to the same file
    save_credential(&file_path, password, "key3", "value3", None, None, None).unwrap();

    // Then: File should contain the remaining and new credential
    let credentials = list_credentials(&file_path, password).unwrap();
    assert_eq!(credentials.len(), 2);
    assert_eq!(credentials[0].key, "key2");
    assert_eq!(credentials[1].key, "key3");
}

#[test]
fn test_remove_specific_credential_among_duplicates() {
    // Given: Credentials with duplicate keys
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "same_key", "value1", None, None, None).unwrap();
    save_credential(&file_path, password, "same_key", "value2", None, None, None).unwrap();
    save_credential(&file_path, password, "same_key", "value3", None, None, None).unwrap();

    // When: Removing the second entry (ID 2)
    remove_credential(&file_path, password, 2).unwrap();

    // Then: The first and third entries should remain
    let credentials = list_credentials(&file_path, password).unwrap();
    assert_eq!(credentials.len(), 2);
    assert_eq!(credentials[0].key, "same_key");
    assert_eq!(credentials[0].value, "value1");
    assert_eq!(credentials[1].key, "same_key");
    assert_eq!(credentials[1].value, "value3");
}
