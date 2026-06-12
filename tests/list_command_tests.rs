use lootbox::{get_list_display, list_credentials, save_credential};
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

// ============================================================================
// Basic List Functionality Tests
// ============================================================================

#[test]
fn test_list_successfully_decrypts_and_displays_credentials() {
    // Given: An encrypted file with multiple credentials
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "key1", "value1").unwrap();
    save_credential(&file_path, password, "key2", "value2").unwrap();

    // When: Listing credentials with correct password
    let result = list_credentials(&file_path, password);

    // Then: Should succeed
    assert!(result.is_ok());
    let credentials = result.unwrap();
    assert_eq!(credentials.len(), 2);
    assert!(credentials.iter().any(|c| c.key == "key1" && c.value == "value1"));
    assert!(credentials.iter().any(|c| c.key == "key2" && c.value == "value2"));
}

#[test]
fn test_list_single_credential() {
    // Given: An encrypted file with one credential
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "api_key", "secret_value").unwrap();

    // When: Listing credentials
    let credentials = list_credentials(&file_path, password).unwrap();

    // Then: Should return exactly one credential
    assert_eq!(credentials.len(), 1);
    assert_eq!(credentials[0].key, "api_key");
    assert_eq!(credentials[0].value, "secret_value");
}

#[test]
fn test_list_multiple_credentials() {
    // Given: An encrypted file with three credentials
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "aws_key", "aws_secret").unwrap();
    save_credential(&file_path, password, "github_token", "ghp_token").unwrap();
    save_credential(&file_path, password, "api_key", "sk_key").unwrap();

    // When: Listing credentials
    let credentials = list_credentials(&file_path, password).unwrap();

    // Then: Should return all three credentials
    assert_eq!(credentials.len(), 3);
}

#[test]
fn test_list_display_shows_position_ids_in_bracket_format() {
    // Given: An encrypted file with multiple credentials
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "key1", "value1").unwrap();
    save_credential(&file_path, password, "key2", "value2").unwrap();
    save_credential(&file_path, password, "key3", "value3").unwrap();

    // When: Getting display output
    let display = get_list_display(&file_path, password).unwrap();

    // Then: Should show position IDs in [n] format (e.g., [1], [2], [3])
    assert!(display.contains("[1]"), "Display should contain [1]");
    assert!(display.contains("[2]"), "Display should contain [2]");
    assert!(display.contains("[3]"), "Display should contain [3]");
}

#[test]
fn test_list_display_shows_keys_in_plain_text() {
    // Given: An encrypted file with credentials
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "my_secret_key", "my_secret_value").unwrap();
    save_credential(&file_path, password, "another_key", "another_value").unwrap();

    // When: Getting display output
    let display = get_list_display(&file_path, password).unwrap();

    // Then: Keys should be shown in plain text
    assert!(display.contains("my_secret_key"));
    assert!(display.contains("another_key"));
}

#[test]
fn test_list_display_hides_values_with_exactly_10_asterisks() {
    // Given: An encrypted file with credentials
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "key1", "secret_value_123").unwrap();
    save_credential(&file_path, password, "key2", "x").unwrap();  // single char
    save_credential(&file_path, password, "key3", &"a".repeat(1000)).unwrap();  // very long

    // When: Getting display output
    let display = get_list_display(&file_path, password).unwrap();

    // Then: All values should be hidden with exactly 10 asterisks each
    // Count occurrences of 10 consecutive asterisks
    let asterisk_groups: Vec<&str> = display.split_whitespace()
        .filter(|s| s.chars().all(|c| c == '*'))
        .collect();

    // Should have 3 groups of asterisks (one per credential)
    assert_eq!(asterisk_groups.len(), 3);

    // Each group should be exactly 10 asterisks
    for group in asterisk_groups {
        assert_eq!(group.len(), 10, "Each value should be masked with exactly 10 asterisks");
    }
}

#[test]
fn test_list_display_does_not_show_actual_values() {
    // Given: An encrypted file with a credential
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";
    let secret_value = "ThisIsMySecretValue123";

    save_credential(&file_path, password, "api_key", secret_value).unwrap();

    // When: Getting display output
    let display = get_list_display(&file_path, password).unwrap();

    // Then: Should not contain the actual secret value
    assert!(!display.contains(secret_value));
}

#[test]
fn test_list_preserves_order_of_credentials() {
    // Given: An encrypted file with credentials added in specific order
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "first", "value1").unwrap();
    save_credential(&file_path, password, "second", "value2").unwrap();
    save_credential(&file_path, password, "third", "value3").unwrap();

    // When: Listing credentials
    let credentials = list_credentials(&file_path, password).unwrap();

    // Then: Should return credentials in the same order they were added
    assert_eq!(credentials[0].key, "first");
    assert_eq!(credentials[1].key, "second");
    assert_eq!(credentials[2].key, "third");
}

// ============================================================================
// Error Cases
// ============================================================================

#[test]
fn test_list_fails_with_wrong_password() {
    // Given: An encrypted file created with one password
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let correct_password = "correct123";
    let wrong_password = "wrong456789";

    save_credential(&file_path, correct_password, "key", "value").unwrap();

    // When: Attempting to list with wrong password
    let result = list_credentials(&file_path, wrong_password);

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("password")
            || error_msg.contains("decrypt")
            || error_msg.contains("authentication"));
}

#[test]
fn test_list_fails_when_file_does_not_exist() {
    // Given: A non-existent file path
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "nonexistent.enc");

    assert!(!file_path.exists());

    // When: Attempting to list credentials
    let result = list_credentials(&file_path, "password123");

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("not found") || error_msg.contains("does not exist"));
}

#[test]
fn test_list_fails_with_corrupted_file() {
    // Given: A corrupted/invalid encrypted file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "corrupted.enc");

    // Write invalid data
    fs::write(&file_path, "this is not encrypted data").expect("Failed to write corrupted file");

    // When: Attempting to list credentials
    let result = list_credentials(&file_path, "password123");

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("decrypt")
            || error_msg.contains("invalid")
            || error_msg.contains("corrupted"));
}

#[test]
fn test_list_fails_with_empty_file() {
    // Given: An empty file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "empty.enc");

    fs::write(&file_path, "").expect("Failed to create empty file");

    // When: Attempting to list credentials
    let result = list_credentials(&file_path, "password123");

    // Then: Should return an error
    assert!(result.is_err());
}

#[test]
fn test_list_fails_with_truncated_encrypted_file() {
    // Given: A valid encrypted file that has been truncated
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "key", "value").unwrap();

    // Truncate the file
    let mut file_content = fs::read(&file_path).unwrap();
    file_content.truncate(file_content.len() / 2);
    fs::write(&file_path, file_content).unwrap();

    // When: Attempting to list credentials
    let result = list_credentials(&file_path, "password123");

    // Then: Should return an error
    assert!(result.is_err());
}

// ============================================================================
// Password Validation Tests
// ============================================================================

#[test]
fn test_list_with_empty_password() {
    // Given: An encrypted file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "key", "value").unwrap();

    // When: Attempting to list with empty password
    let result = list_credentials(&file_path, "");

    // Then: Should return an error (password validation)
    assert!(result.is_err());
}

#[test]
fn test_list_with_password_less_than_8_characters() {
    // Given: An encrypted file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "key", "value").unwrap();

    // When: Attempting to list with 7-character password
    let result = list_credentials(&file_path, "pass123");

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("8") || error_msg.contains("length") || error_msg.contains("characters"));
}

#[test]
fn test_list_with_password_only_whitespace() {
    // Given: An encrypted file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "key", "value").unwrap();

    // When: Attempting to list with whitespace-only password
    let result = list_credentials(&file_path, "        ");

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("whitespace") || error_msg.contains("invalid"));
}

#[test]
fn test_list_with_password_starting_with_whitespace() {
    // Given: An encrypted file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "key", "value").unwrap();

    // When: Attempting to list with password starting with whitespace
    let result = list_credentials(&file_path, " password123");

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("whitespace") || error_msg.contains("space"));
}

#[test]
fn test_list_with_password_ending_with_whitespace() {
    // Given: An encrypted file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "key", "value").unwrap();

    // When: Attempting to list with password ending with whitespace
    let result = list_credentials(&file_path, "password123 ");

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("whitespace") || error_msg.contains("space"));
}

// ============================================================================
// Special Characters and Encoding Tests
// ============================================================================

#[test]
fn test_list_with_special_characters_in_key_and_value() {
    // Given: An encrypted file with special characters
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let special_key = "key@#$%^&*()";
    let special_value = "value!@#$%^&*(){}[]|\\:;\"'<>,.?/~`";

    save_credential(&file_path, "password123", special_key, special_value).unwrap();

    // When: Listing credentials
    let credentials = list_credentials(&file_path, "password123").unwrap();

    // Then: Should retrieve special characters correctly
    assert_eq!(credentials[0].key, special_key);
    assert_eq!(credentials[0].value, special_value);

    // And: Display should show key in plain text but not value
    let display = get_list_display(&file_path, "password123").unwrap();
    assert!(display.contains(special_key));
    assert!(!display.contains(special_value)); // Value should be masked
}

#[test]
fn test_list_with_unicode_characters() {
    // Given: An encrypted file with unicode characters
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let unicode_key = "日本語キー";
    let unicode_value = "中文值🔑";

    save_credential(&file_path, "password123", unicode_key, unicode_value).unwrap();

    // When: Listing credentials
    let credentials = list_credentials(&file_path, "password123").unwrap();

    // Then: Should retrieve unicode correctly
    assert_eq!(credentials[0].key, unicode_key);
    assert_eq!(credentials[0].value, unicode_value);

    // And: Display should show key in plain text
    let display = get_list_display(&file_path, "password123").unwrap();
    assert!(display.contains(unicode_key));
    assert!(!display.contains(unicode_value)); // Value should be masked
}

#[test]
fn test_list_with_newlines_in_key_and_value() {
    // Given: An encrypted file with newlines
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let key_with_newline = "multi\nline\nkey";
    let value_with_newline = "multi\nline\nvalue";

    save_credential(&file_path, "password123", key_with_newline, value_with_newline).unwrap();

    // When: Listing credentials
    let credentials = list_credentials(&file_path, "password123").unwrap();

    // Then: Should retrieve newlines correctly
    assert_eq!(credentials[0].key, key_with_newline);
    assert_eq!(credentials[0].value, value_with_newline);
}

#[test]
fn test_list_with_very_long_key() {
    // Given: An encrypted file with very long key
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let long_key = "key_".to_string() + &"a".repeat(10000);

    save_credential(&file_path, "password123", &long_key, "value").unwrap();

    // When: Listing credentials
    let credentials = list_credentials(&file_path, "password123").unwrap();

    // Then: Should retrieve correctly
    assert_eq!(credentials[0].key, long_key);
}

// ============================================================================
// Position ID Tests
// ============================================================================

#[test]
fn test_list_display_position_ids_are_sequential() {
    // Given: An encrypted file with 5 credentials
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    for i in 1..=5 {
        save_credential(&file_path, password, &format!("key{}", i), &format!("value{}", i)).unwrap();
    }

    // When: Getting display output
    let display = get_list_display(&file_path, password).unwrap();

    // Then: Should show sequential position IDs from 1 to 5 in [n] format
    for i in 1..=5 {
        let id_pattern = format!("[{}]", i);
        assert!(
            display.contains(&id_pattern),
            "Display should contain position ID [{}]", i
        );
    }
}

#[test]
fn test_list_display_single_credential_shows_id_1() {
    // Given: An encrypted file with one credential
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "only_key", "only_value").unwrap();

    // When: Getting display output
    let display = get_list_display(&file_path, "password123").unwrap();

    // Then: Should show position ID [1]
    assert!(display.contains("[1]"), "Display should contain [1]");
    // And: Should not show position ID [2]
    assert!(!display.contains("[2]"), "Display should not contain [2]");
}

#[test]
fn test_list_display_format_specification() {
    // Given: An encrypted file with credentials
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "api_key", "secret123").unwrap();
    save_credential(&file_path, password, "db_pass", "dbsecret456").unwrap();

    // When: Getting display output
    let display = get_list_display(&file_path, password).unwrap();

    // Then: Should follow format: [ID] Key: <key> Value: **********
    // Position IDs should be in [n] format
    assert!(display.contains("[1]"), "First credential should have position ID [1]");
    assert!(display.contains("[2]"), "Second credential should have position ID [2]");

    // Keys should be visible
    assert!(display.contains("api_key"));
    assert!(display.contains("db_pass"));

    // Values should be masked
    assert!(!display.contains("secret123"));
    assert!(!display.contains("dbsecret456"));
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_list_retrieves_exactly_what_was_saved() {
    // Given: Various test cases with different data
    let test_cases = vec![
        ("simple_key", "simple_value"),
        ("key with spaces", "value with spaces"),
        ("key!@#$%", "value^&*()"),
        ("キー", "値"),
    ];

    let temp_dir = setup_test_dir();

    for (i, (key, value)) in test_cases.iter().enumerate() {
        let file_path = get_test_file_path(&temp_dir, &format!("test_{}.enc", i));

        // When: Saving and then listing
        save_credential(&file_path, "password123", key, value).unwrap();
        let credentials = list_credentials(&file_path, "password123").unwrap();

        // Then: Should retrieve exactly what was saved
        assert_eq!(credentials.len(), 1);
        assert_eq!(credentials[0].key, *key);
        assert_eq!(credentials[0].value, *value);
    }
}

#[test]
fn test_list_after_save_with_different_passwords() {
    // Given: Files encrypted with different passwords
    let temp_dir = setup_test_dir();
    let file_path1 = get_test_file_path(&temp_dir, "file1.enc");
    let file_path2 = get_test_file_path(&temp_dir, "file2.enc");

    save_credential(&file_path1, "password123", "key1", "value1").unwrap();
    save_credential(&file_path2, "different456", "key2", "value2").unwrap();

    // When: Listing with correct passwords
    let creds1 = list_credentials(&file_path1, "password123").unwrap();
    let creds2 = list_credentials(&file_path2, "different456").unwrap();

    // Then: Should retrieve correct credentials
    assert_eq!(creds1[0].key, "key1");
    assert_eq!(creds1[0].value, "value1");
    assert_eq!(creds2[0].key, "key2");
    assert_eq!(creds2[0].value, "value2");

    // And: Should fail with wrong passwords
    assert!(list_credentials(&file_path1, "different456").is_err());
    assert!(list_credentials(&file_path2, "password123").is_err());
}

#[test]
fn test_list_display_formatting_consistency() {
    // Given: An encrypted file with multiple credentials
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "key1", "value1").unwrap();
    save_credential(&file_path, password, "key2", "value2").unwrap();

    // When: Getting display output multiple times
    let display1 = get_list_display(&file_path, password).unwrap();
    let display2 = get_list_display(&file_path, password).unwrap();

    // Then: Displays should be identical (consistent formatting)
    assert_eq!(display1, display2);
}
