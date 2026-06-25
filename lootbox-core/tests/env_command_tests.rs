use lootbox::{generate_env_vars, remove_credential, save_credential};
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
// Key Transformation Tests - Valid Results
// ============================================================================

#[test]
fn test_env_key_converted_to_uppercase() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "uppercase test", "upper_val").unwrap();

    let result = generate_env_vars(&file_path, password, 1).unwrap();

    assert_eq!(result.created.len(), 1);
    assert_eq!(result.created[0].env_name, "UPPERCASE_TEST");
}

#[test]
fn test_env_key_spaces_replaced_with_underscores() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "my api key", "spaces_val").unwrap();

    let result = generate_env_vars(&file_path, password, 1).unwrap();

    assert_eq!(result.created[0].env_name, "MY_API_KEY");
}

#[test]
fn test_env_key_mixed_case_and_spaces_transformed() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "My Secret Key", "mixed_val").unwrap();

    let result = generate_env_vars(&file_path, password, 1).unwrap();

    assert_eq!(result.created[0].env_name, "MY_SECRET_KEY");
}

#[test]
fn test_env_key_already_valid_env_name_unchanged() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "DATABASE_URL", "postgres://localhost/db").unwrap();

    let result = generate_env_vars(&file_path, password, 1).unwrap();

    assert_eq!(result.created[0].env_name, "DATABASE_URL");
}

#[test]
fn test_env_key_multiple_consecutive_spaces_become_multiple_underscores() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "my  double  spaced", "double_val").unwrap();

    let result = generate_env_vars(&file_path, password, 1).unwrap();

    assert_eq!(result.created[0].env_name, "MY__DOUBLE__SPACED");
}

#[test]
fn test_env_key_with_numbers_not_at_start_is_valid() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "api key 2", "num_val").unwrap();

    let result = generate_env_vars(&file_path, password, 1).unwrap();

    assert_eq!(result.created[0].env_name, "API_KEY_2");
    assert_eq!(result.created.len(), 1);
    assert_eq!(result.invalid.len(), 0);
}

#[test]
fn test_env_key_original_name_preserved_in_result() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "original name key", "orig_val").unwrap();

    let result = generate_env_vars(&file_path, password, 1).unwrap();

    assert_eq!(result.created[0].original_key, "original name key");
    assert_eq!(result.created[0].env_name, "ORIGINAL_NAME_KEY");
}

// ============================================================================
// Invalid Key Tests - Special Characters
// ============================================================================

#[test]
fn test_env_key_with_hyphen_is_invalid_with_reason() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "my-key", "valid_value").unwrap();

    let result = generate_env_vars(&file_path, password, 1).unwrap();

    assert_eq!(result.created.len(), 0);
    assert_eq!(result.invalid.len(), 1);
    assert_eq!(result.invalid[0].original_key, "my-key");
    assert!(
        result.invalid[0].reason.to_lowercase().contains("invalid")
            || result.invalid[0].reason.to_lowercase().contains("character")
            || result.invalid[0].reason.to_lowercase().contains("special")
    );
}

#[test]
fn test_env_key_with_dot_is_invalid_with_reason() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "my.key", "valid_value").unwrap();

    let result = generate_env_vars(&file_path, password, 1).unwrap();

    assert_eq!(result.created.len(), 0);
    assert_eq!(result.invalid.len(), 1);
    assert_eq!(result.invalid[0].original_key, "my.key");
    assert!(
        result.invalid[0].reason.to_lowercase().contains("invalid")
            || result.invalid[0].reason.to_lowercase().contains("character")
            || result.invalid[0].reason.to_lowercase().contains("special")
    );
}

#[test]
fn test_env_key_with_at_sign_is_invalid_with_reason() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "my@key", "valid_value").unwrap();

    let result = generate_env_vars(&file_path, password, 1).unwrap();

    assert_eq!(result.created.len(), 0);
    assert_eq!(result.invalid.len(), 1);
    assert!(
        result.invalid[0].reason.to_lowercase().contains("invalid")
            || result.invalid[0].reason.to_lowercase().contains("character")
    );
}

#[test]
fn test_env_key_with_slash_is_invalid_with_reason() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "my/key", "valid_value").unwrap();

    let result = generate_env_vars(&file_path, password, 1).unwrap();

    assert_eq!(result.invalid.len(), 1);
    assert_eq!(result.created.len(), 0);
    assert!(!result.invalid[0].reason.is_empty());
}

#[test]
fn test_env_key_with_exclamation_mark_is_invalid_with_reason() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "key!", "valid_value").unwrap();

    let result = generate_env_vars(&file_path, password, 1).unwrap();

    assert_eq!(result.invalid.len(), 1);
    assert_eq!(result.created.len(), 0);
    assert!(!result.invalid[0].reason.is_empty());
}

// ============================================================================
// Invalid Key Tests - Starting with a Number
// ============================================================================

#[test]
fn test_env_key_starting_with_digit_is_invalid_with_reason() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "2nd_key", "valid_value").unwrap();

    let result = generate_env_vars(&file_path, password, 1).unwrap();

    assert_eq!(result.created.len(), 0);
    assert_eq!(result.invalid.len(), 1);
    assert_eq!(result.invalid[0].original_key, "2nd_key");
    assert!(
        result.invalid[0].reason.to_lowercase().contains("digit")
            || result.invalid[0].reason.to_lowercase().contains("number")
            || result.invalid[0].reason.to_lowercase().contains("start")
    );
}

#[test]
fn test_env_key_starting_with_digit_after_transformation_is_invalid() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "2 api keys", "valid_value").unwrap();

    let result = generate_env_vars(&file_path, password, 1).unwrap();

    assert_eq!(result.created.len(), 0);
    assert_eq!(result.invalid.len(), 1);
    assert!(
        result.invalid[0].reason.to_lowercase().contains("digit")
            || result.invalid[0].reason.to_lowercase().contains("number")
            || result.invalid[0].reason.to_lowercase().contains("start")
    );
}

#[test]
fn test_env_key_with_digit_not_at_start_is_valid() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "key2name", "digit_mid_val").unwrap();

    let result = generate_env_vars(&file_path, password, 1).unwrap();

    assert_eq!(result.created.len(), 1);
    assert_eq!(result.invalid.len(), 0);
    assert_eq!(result.created[0].env_name, "KEY2NAME");
}

// ============================================================================
// Invalid Value Tests
// ============================================================================

#[test]
fn test_env_value_with_null_byte_is_invalid_with_reason() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "null byte key", "val\0ue").unwrap();

    let result = generate_env_vars(&file_path, password, 1).unwrap();

    assert_eq!(result.created.len(), 0);
    assert_eq!(result.invalid.len(), 1);
    assert_eq!(result.invalid[0].original_key, "null byte key");
    assert!(
        result.invalid[0].reason.to_lowercase().contains("nul")
            || result.invalid[0].reason.to_lowercase().contains("failed")
            || result.invalid[0].reason.to_lowercase().contains("set")
    );
}

#[test]
fn test_env_value_with_only_null_byte_is_invalid_with_reason() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "only null key", "\0").unwrap();

    let result = generate_env_vars(&file_path, password, 1).unwrap();

    assert_eq!(result.created.len(), 0);
    assert_eq!(result.invalid.len(), 1);
    assert!(
        result.invalid[0].reason.to_lowercase().contains("nul")
            || result.invalid[0].reason.to_lowercase().contains("failed")
    );
}

#[test]
fn test_env_value_with_trailing_null_byte_is_invalid_with_reason() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "trailing null key", "valid_prefix\0").unwrap();

    let result = generate_env_vars(&file_path, password, 1).unwrap();

    assert_eq!(result.invalid.len(), 1);
    assert_eq!(result.created.len(), 0);
    assert!(
        result.invalid[0].reason.to_lowercase().contains("nul")
            || result.invalid[0].reason.to_lowercase().contains("failed")
    );
}

#[test]
fn test_env_value_with_newlines_is_valid() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "multiline key", "line1\nline2\nline3").unwrap();

    let result = generate_env_vars(&file_path, password, 1).unwrap();

    assert_eq!(result.created.len(), 1);
    assert_eq!(result.created[0].value, "line1\nline2\nline3");
}

#[test]
fn test_env_value_with_equals_sign_is_valid() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "base64 token", "base64abc==").unwrap();

    let result = generate_env_vars(&file_path, password, 1).unwrap();

    assert_eq!(result.created.len(), 1);
    assert_eq!(result.created[0].value, "base64abc==");
}

#[test]
fn test_env_value_with_special_characters_is_valid() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    let special_value = "p@$$w0rd!#%^&*(){}[]|;:'\",<>?/~`";
    save_credential(&file_path, password, "special chars key", special_value).unwrap();

    let result = generate_env_vars(&file_path, password, 1).unwrap();

    assert_eq!(result.created.len(), 1);
    assert_eq!(result.invalid.len(), 0);
    assert_eq!(result.created[0].value, special_value);
}

#[test]
fn test_env_value_preserved_exactly_in_created_entry() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "exact value key", "s3cr3t!@#value").unwrap();

    let result = generate_env_vars(&file_path, password, 1).unwrap();

    assert_eq!(result.created[0].value, "s3cr3t!@#value");
}

// ============================================================================
// ID Selection Tests
// ============================================================================

#[test]
fn test_env_single_valid_credential_in_created_list() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "single cred key", "secret123").unwrap();

    let result = generate_env_vars(&file_path, password, 1).unwrap();

    assert_eq!(result.created.len(), 1);
    assert_eq!(result.invalid.len(), 0);
}

#[test]
fn test_env_selects_first_credential_by_id() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "first key", "value1").unwrap();
    save_credential(&file_path, password, "second key", "value2").unwrap();
    save_credential(&file_path, password, "third key", "value3").unwrap();

    let result = generate_env_vars(&file_path, password, 1).unwrap();

    assert_eq!(result.created.len(), 1);
    assert_eq!(result.created[0].original_key, "first key");
    assert_eq!(result.created[0].env_name, "FIRST_KEY");
    assert_eq!(result.created[0].value, "value1");
}

#[test]
fn test_env_selects_middle_credential_by_id() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "first key", "value1").unwrap();
    save_credential(&file_path, password, "second key", "value2").unwrap();
    save_credential(&file_path, password, "third key", "value3").unwrap();

    let result = generate_env_vars(&file_path, password, 2).unwrap();

    assert_eq!(result.created.len(), 1);
    assert_eq!(result.created[0].original_key, "second key");
    assert_eq!(result.created[0].env_name, "SECOND_KEY");
    assert_eq!(result.created[0].value, "value2");
}

#[test]
fn test_env_selects_last_credential_by_id() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "first key", "value1").unwrap();
    save_credential(&file_path, password, "second key", "value2").unwrap();
    save_credential(&file_path, password, "third key", "value3").unwrap();

    let result = generate_env_vars(&file_path, password, 3).unwrap();

    assert_eq!(result.created.len(), 1);
    assert_eq!(result.created[0].original_key, "third key");
    assert_eq!(result.created[0].env_name, "THIRD_KEY");
    assert_eq!(result.created[0].value, "value3");
}

#[test]
fn test_env_invalid_key_credential_selected_by_id() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "valid key", "ok").unwrap();
    save_credential(&file_path, password, "bad-key", "ok").unwrap();

    // Select the second credential (invalid key)
    let result = generate_env_vars(&file_path, password, 2).unwrap();

    assert_eq!(result.created.len(), 0);
    assert_eq!(result.invalid.len(), 1);
    assert_eq!(result.invalid[0].original_key, "bad-key");
    assert!(!result.invalid[0].reason.is_empty());
}

// ============================================================================
// ID Error Cases
// ============================================================================

#[test]
fn test_env_fails_with_id_zero() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "some key", "value").unwrap();

    let result = generate_env_vars(&file_path, password, 0);

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.to_lowercase().contains("invalid")
            || error_msg.to_lowercase().contains("least 1")
    );
}

#[test]
fn test_env_fails_with_id_greater_than_count() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "only key", "value").unwrap();

    let result = generate_env_vars(&file_path, password, 5);

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.to_lowercase().contains("invalid")
            || error_msg.to_lowercase().contains("5")
    );
}

// ============================================================================
// Error Cases - File and Password Issues
// ============================================================================

#[test]
fn test_env_fails_with_wrong_password() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "correct123", "key", "value").unwrap();

    let result = generate_env_vars(&file_path, "wrong456789", 1);

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("password")
            || error_msg.contains("decrypt")
            || error_msg.contains("authentication")
    );
}

#[test]
fn test_env_fails_when_file_does_not_exist() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "nonexistent.enc");

    assert!(!file_path.exists());

    let result = generate_env_vars(&file_path, "password123", 1);

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("not found") || error_msg.contains("does not exist"));
}

#[test]
fn test_env_fails_with_corrupted_file() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "corrupted.enc");

    fs::write(&file_path, "this is not encrypted data").unwrap();

    let result = generate_env_vars(&file_path, "password123", 1);

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("decrypt")
            || error_msg.contains("invalid")
            || error_msg.contains("corrupted")
            || error_msg.contains("unsupported")
    );
}

#[test]
fn test_env_fails_with_empty_file() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "empty.enc");

    fs::write(&file_path, "").unwrap();

    let result = generate_env_vars(&file_path, "password123", 1);

    assert!(result.is_err());
}

#[test]
fn test_env_fails_with_truncated_encrypted_file() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "truncated key", "value").unwrap();

    let mut content = fs::read(&file_path).unwrap();
    content.truncate(content.len() / 2);
    fs::write(&file_path, content).unwrap();

    let result = generate_env_vars(&file_path, "password123", 1);

    assert!(result.is_err());
}

// ============================================================================
// Password Validation Tests
// ============================================================================

#[test]
fn test_env_fails_with_empty_password() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "pw empty key", "value").unwrap();

    let result = generate_env_vars(&file_path, "", 1);

    assert!(result.is_err());
}

#[test]
fn test_env_fails_with_password_less_than_8_characters() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "pw short key", "value").unwrap();

    let result = generate_env_vars(&file_path, "pass123", 1);

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("8")
            || error_msg.contains("length")
            || error_msg.contains("characters")
    );
}

#[test]
fn test_env_fails_with_whitespace_only_password() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "pw whitespace key", "value").unwrap();

    let result = generate_env_vars(&file_path, "        ", 1);

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("whitespace") || error_msg.contains("invalid"));
}

#[test]
fn test_env_fails_with_password_starting_with_whitespace() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "pw leading space key", "value").unwrap();

    let result = generate_env_vars(&file_path, " password123", 1);

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("whitespace") || error_msg.contains("space"));
}

#[test]
fn test_env_fails_with_password_ending_with_whitespace() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "pw trailing space key", "value").unwrap();

    let result = generate_env_vars(&file_path, "password123 ", 1);

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("whitespace") || error_msg.contains("space"));
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_env_does_not_modify_the_encrypted_file() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "no modify key", "no_modify_val").unwrap();
    let content_before = fs::read(&file_path).unwrap();

    generate_env_vars(&file_path, password, 1).unwrap();

    let content_after = fs::read(&file_path).unwrap();
    assert_eq!(content_before, content_after);
}

#[test]
fn test_env_set_var_actually_sets_the_variable_in_the_process() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "lootbox test key", "supersecret42").unwrap();

    let result = generate_env_vars(&file_path, password, 1).unwrap();

    assert_eq!(result.created.len(), 1);
    assert_eq!(result.created[0].env_name, "LOOTBOX_TEST_KEY");
}

#[test]
fn test_env_null_byte_value_credential_is_in_invalid_list() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "null val key", "has\0null").unwrap();

    let result = generate_env_vars(&file_path, password, 1).unwrap();

    assert_eq!(result.created.len(), 0);
    assert_eq!(result.invalid.len(), 1);
    assert_eq!(result.invalid[0].original_key, "null val key");
    assert!(
        result.invalid[0].reason.to_lowercase().contains("nul")
            || result.invalid[0].reason.to_lowercase().contains("failed")
            || result.invalid[0].reason.to_lowercase().contains("set")
    );
}

#[test]
fn test_env_fails_after_all_credentials_removed() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "to remove", "value").unwrap();
    remove_credential(&file_path, password, 1).unwrap();

    // File is empty now — any ID should fail
    let result = generate_env_vars(&file_path, password, 1);

    assert!(result.is_err());
}
