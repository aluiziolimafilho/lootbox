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
    // Given: A credential with a lowercase key
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "uppercase test", "upper_val").unwrap();

    // When: Generating env vars
    let result = generate_env_vars(&file_path, password).unwrap();

    // Then: The env name should be fully uppercase and the var is set in the process
    assert_eq!(result.created.len(), 1);
    assert_eq!(result.created[0].env_name, "UPPERCASE_TEST");
}

#[test]
fn test_env_key_spaces_replaced_with_underscores() {
    // Given: A credential with spaces in the key
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "my api key", "spaces_val").unwrap();

    // When: Generating env vars
    let result = generate_env_vars(&file_path, password).unwrap();

    // Then: Spaces become underscores and the var is set
    assert_eq!(result.created[0].env_name, "MY_API_KEY");
}

#[test]
fn test_env_key_mixed_case_and_spaces_transformed() {
    // Given: A credential with mixed case and spaces
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "My Secret Key", "mixed_val").unwrap();

    // When: Generating env vars
    let result = generate_env_vars(&file_path, password).unwrap();

    // Then: Should be fully uppercase with underscores and the var is set
    assert_eq!(result.created[0].env_name, "MY_SECRET_KEY");
}

#[test]
fn test_env_key_already_valid_env_name_unchanged() {
    // Given: A credential whose key is already a valid env var name
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "DATABASE_URL", "postgres://localhost/db").unwrap();

    // When: Generating env vars
    let result = generate_env_vars(&file_path, password).unwrap();

    // Then: The env name is unchanged and the var is set
    assert_eq!(result.created[0].env_name, "DATABASE_URL");
}

#[test]
fn test_env_key_multiple_consecutive_spaces_become_multiple_underscores() {
    // Given: A credential key with multiple consecutive spaces
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "my  double  spaced", "double_val").unwrap();

    // When: Generating env vars
    let result = generate_env_vars(&file_path, password).unwrap();

    // Then: Each space individually becomes one underscore and the var is set
    assert_eq!(result.created[0].env_name, "MY__DOUBLE__SPACED");
}

#[test]
fn test_env_key_with_numbers_not_at_start_is_valid() {
    // Given: A credential key with numbers not at the start
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "api key 2", "num_val").unwrap();

    // When: Generating env vars
    let result = generate_env_vars(&file_path, password).unwrap();

    // Then: Numbers after the first character are allowed and the var is set
    assert_eq!(result.created[0].env_name, "API_KEY_2");
    assert_eq!(result.created.len(), 1);
    assert_eq!(result.invalid.len(), 0);
}

#[test]
fn test_env_key_original_name_preserved_in_result() {
    // Given: A credential with a key that will be transformed
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "original name key", "orig_val").unwrap();

    // When: Generating env vars
    let result = generate_env_vars(&file_path, password).unwrap();

    // Then: The original key name is preserved alongside the env name, and the var is set
    assert_eq!(result.created[0].original_key, "original name key");
    assert_eq!(result.created[0].env_name, "ORIGINAL_NAME_KEY");
}

// ============================================================================
// Invalid Key Tests - Special Characters
// ============================================================================

#[test]
fn test_env_key_with_hyphen_is_invalid_with_reason() {
    // Given: A credential whose key contains a hyphen
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "my-key", "valid_value").unwrap();

    // When: Generating env vars
    let result = generate_env_vars(&file_path, password).unwrap();

    // Then: Should be in the invalid list with a reason about invalid characters
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
    // Given: A credential whose key contains a dot
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "my.key", "valid_value").unwrap();

    // When: Generating env vars
    let result = generate_env_vars(&file_path, password).unwrap();

    // Then: Invalid with a reason about invalid characters
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
    // Given: A credential whose key contains an at-sign
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "my@key", "valid_value").unwrap();

    // When: Generating env vars
    let result = generate_env_vars(&file_path, password).unwrap();

    // Then: Invalid with a reason about invalid characters
    assert_eq!(result.created.len(), 0);
    assert_eq!(result.invalid.len(), 1);
    assert!(
        result.invalid[0].reason.to_lowercase().contains("invalid")
            || result.invalid[0].reason.to_lowercase().contains("character")
    );
}

#[test]
fn test_env_key_with_slash_is_invalid_with_reason() {
    // Given: A credential whose key contains a slash
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "my/key", "valid_value").unwrap();

    // When: Generating env vars
    let result = generate_env_vars(&file_path, password).unwrap();

    // Then: Invalid with a reason
    assert_eq!(result.invalid.len(), 1);
    assert_eq!(result.created.len(), 0);
    assert!(!result.invalid[0].reason.is_empty());
}

#[test]
fn test_env_key_with_exclamation_mark_is_invalid_with_reason() {
    // Given: A credential whose key ends with an exclamation mark
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "key!", "valid_value").unwrap();

    // When: Generating env vars
    let result = generate_env_vars(&file_path, password).unwrap();

    // Then: Invalid with a non-empty reason
    assert_eq!(result.invalid.len(), 1);
    assert_eq!(result.created.len(), 0);
    assert!(!result.invalid[0].reason.is_empty());
}

// ============================================================================
// Invalid Key Tests - Starting with a Number
// ============================================================================

#[test]
fn test_env_key_starting_with_digit_is_invalid_with_reason() {
    // Given: A credential whose key starts with a digit
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "2nd_key", "valid_value").unwrap();

    // When: Generating env vars
    let result = generate_env_vars(&file_path, password).unwrap();

    // Then: Invalid with a reason mentioning the leading digit
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
    // Given: A credential whose key begins with a digit (e.g. "2 api keys" → "2_API_KEYS")
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "2 api keys", "valid_value").unwrap();

    // When: Generating env vars (transformed name is "2_API_KEYS")
    let result = generate_env_vars(&file_path, password).unwrap();

    // Then: Invalid because the transformed name starts with a digit
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
    // Given: A credential key with a digit only after the first character
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "key2name", "digit_mid_val").unwrap();

    // When: Generating env vars
    let result = generate_env_vars(&file_path, password).unwrap();

    // Then: Digit not at the start is allowed and the var is set
    assert_eq!(result.created.len(), 1);
    assert_eq!(result.invalid.len(), 0);
    assert_eq!(result.created[0].env_name, "KEY2NAME");
}

// ============================================================================
// Invalid Value Tests
// ============================================================================

#[test]
fn test_env_value_with_null_byte_is_invalid_with_reason() {
    // Given: A credential whose value contains a null byte
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "null byte key", "val\0ue").unwrap();

    // When: Generating env vars
    let result = generate_env_vars(&file_path, password).unwrap();

    // Then: set_var panics on null byte — caught and reported as invalid with a reason
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
    // Given: A credential whose entire value is a null byte
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "only null key", "\0").unwrap();

    // When: Generating env vars
    let result = generate_env_vars(&file_path, password).unwrap();

    // Then: set_var panics on null byte — caught and reported as invalid
    assert_eq!(result.created.len(), 0);
    assert_eq!(result.invalid.len(), 1);
    assert!(
        result.invalid[0].reason.to_lowercase().contains("nul")
            || result.invalid[0].reason.to_lowercase().contains("failed")
    );
}

#[test]
fn test_env_value_with_trailing_null_byte_is_invalid_with_reason() {
    // Given: A credential whose value ends with a null byte
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "trailing null key", "valid_prefix\0").unwrap();

    // When: Generating env vars
    let result = generate_env_vars(&file_path, password).unwrap();

    // Then: set_var panics on trailing null byte — caught and reported as invalid
    assert_eq!(result.invalid.len(), 1);
    assert_eq!(result.created.len(), 0);
    assert!(
        result.invalid[0].reason.to_lowercase().contains("nul")
            || result.invalid[0].reason.to_lowercase().contains("failed")
    );
}

#[test]
fn test_env_value_with_newlines_is_valid() {
    // Given: A credential with a multi-line value (newlines are allowed in env var values)
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "multiline key", "line1\nline2\nline3").unwrap();

    // When: Generating env vars
    let result = generate_env_vars(&file_path, password).unwrap();

    // Then: Newlines in values are allowed and the var is set with the full multiline value
    assert_eq!(result.created.len(), 1);
    assert_eq!(result.created[0].value, "line1\nline2\nline3");
}

#[test]
fn test_env_value_with_equals_sign_is_valid() {
    // Given: A credential with an equals sign in the value (common for base64)
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "base64 token", "base64abc==").unwrap();

    // When: Generating env vars
    let result = generate_env_vars(&file_path, password).unwrap();

    // Then: Equals sign in the value is allowed and the var is set
    assert_eq!(result.created.len(), 1);
    assert_eq!(result.created[0].value, "base64abc==");
}

#[test]
fn test_env_value_with_special_characters_is_valid() {
    // Given: A credential with special (non-null) characters in the value
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    let special_value = "p@$$w0rd!#%^&*(){}[]|;:'\",<>?/~`";
    save_credential(&file_path, password, "special chars key", special_value).unwrap();

    // When: Generating env vars
    let result = generate_env_vars(&file_path, password).unwrap();

    // Then: Non-null special chars in values are allowed and the var is set exactly
    assert_eq!(result.created.len(), 1);
    assert_eq!(result.invalid.len(), 0);
    assert_eq!(result.created[0].value, special_value);
}

#[test]
fn test_env_value_preserved_exactly_in_created_entry() {
    // Given: A credential with a specific value containing special characters
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "exact value key", "s3cr3t!@#value").unwrap();

    // When: Generating env vars
    let result = generate_env_vars(&file_path, password).unwrap();

    // Then: The value is returned exactly as stored and is set in the process
    assert_eq!(result.created[0].value, "s3cr3t!@#value");
}

// ============================================================================
// Duplicate Env Var Name Tests
// ============================================================================

#[test]
fn test_env_duplicate_transformed_name_first_wins() {
    // Given: Two credentials that transform to the same env var name
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "dup first key", "first_value").unwrap();
    save_credential(&file_path, password, "DUP_FIRST_KEY", "second_value").unwrap();

    // When: Generating env vars
    let result = generate_env_vars(&file_path, password).unwrap();

    // Then: Only the first is created and set; the second is invalid
    assert_eq!(result.created.len(), 1);
    assert_eq!(result.created[0].original_key, "dup first key");
    assert_eq!(result.created[0].env_name, "DUP_FIRST_KEY");
    assert_eq!(result.created[0].value, "first_value");
    assert_eq!(result.invalid.len(), 1);
    assert_eq!(result.invalid[0].original_key, "DUP_FIRST_KEY");
}

#[test]
fn test_env_duplicate_invalid_entry_has_duplicate_reason() {
    // Given: Two credentials that map to the same env var name
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "dup reason key", "value1").unwrap();
    save_credential(&file_path, password, "DUP_REASON_KEY", "value2").unwrap();

    // When: Generating env vars
    let result = generate_env_vars(&file_path, password).unwrap();

    // Then: The invalid entry has a reason mentioning the duplicate and the first is set
    assert_eq!(result.invalid.len(), 1);
    assert!(
        result.invalid[0].reason.to_lowercase().contains("duplicate")
            || result.invalid[0].reason.to_lowercase().contains("already")
            || result.invalid[0].reason.to_lowercase().contains("exists")
    );
}

#[test]
fn test_env_three_duplicates_only_first_created() {
    // Given: Three credentials that all transform to the same env var name
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "db url", "first").unwrap();
    save_credential(&file_path, password, "DB_URL", "second").unwrap();
    save_credential(&file_path, password, "Db Url", "third").unwrap();

    // When: Generating env vars
    let result = generate_env_vars(&file_path, password).unwrap();

    // Then: Only the first created; the other two are invalid with duplicate reason
    assert_eq!(result.created.len(), 1);
    assert_eq!(result.created[0].value, "first");
    assert_eq!(result.invalid.len(), 2);
    for entry in &result.invalid {
        assert!(
            entry.reason.to_lowercase().contains("duplicate")
                || entry.reason.to_lowercase().contains("already")
                || entry.reason.to_lowercase().contains("exists")
        );
    }
}

#[test]
fn test_env_non_duplicate_entries_after_duplicate_still_created() {
    // Given: A duplicate pair followed by a unique credential
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "post dup key", "value1").unwrap();
    save_credential(&file_path, password, "POST_DUP_KEY", "value2").unwrap();
    save_credential(&file_path, password, "unique after dup", "value3").unwrap();

    // When: Generating env vars
    let result = generate_env_vars(&file_path, password).unwrap();

    // Then: First and third created; second is invalid. Both created vars are set.
    assert_eq!(result.created.len(), 2);
    assert_eq!(result.invalid.len(), 1);
    let env_names: Vec<&str> = result.created.iter().map(|e| e.env_name.as_str()).collect();
    assert!(env_names.contains(&"POST_DUP_KEY"));
    assert!(env_names.contains(&"UNIQUE_AFTER_DUP"));
}

// ============================================================================
// Mixed Results Tests
// ============================================================================

#[test]
fn test_env_mix_invalid_key_char_and_invalid_value_both_reported() {
    // Given: One credential with an invalid key character, one with a null byte value
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "bad-key", "valid_value").unwrap();
    save_credential(&file_path, password, "mix null val key", "bad\0value").unwrap();

    // When: Generating env vars
    let result = generate_env_vars(&file_path, password).unwrap();

    // Then: Both end up in the invalid list with distinct reasons
    assert_eq!(result.created.len(), 0);
    assert_eq!(result.invalid.len(), 2);
    assert!(!result.invalid[0].reason.is_empty());
    assert!(!result.invalid[1].reason.is_empty());
}

#[test]
fn test_env_mix_of_all_invalid_reasons_reported_correctly() {
    // Given: One invalid key char, one starts with digit, one null byte value, one duplicate
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "all mix key", "valid").unwrap();  // created → ALL_MIX_KEY
    save_credential(&file_path, password, "bad-key", "valid").unwrap();      // invalid: special char
    save_credential(&file_path, password, "3rd key", "valid").unwrap();      // invalid: starts with digit
    save_credential(&file_path, password, "null val key", "v\0al").unwrap(); // invalid: null byte in value
    save_credential(&file_path, password, "ALL MIX KEY", "dup").unwrap();   // invalid: duplicate

    // When: Generating env vars
    let result = generate_env_vars(&file_path, password).unwrap();

    // Then: Only the first valid unique one is created and set; the rest are invalid with reasons
    assert_eq!(result.created.len(), 1);
    assert_eq!(result.created[0].env_name, "ALL_MIX_KEY");
    assert_eq!(result.invalid.len(), 4);
    for entry in &result.invalid {
        assert!(!entry.reason.is_empty());
    }
}

#[test]
fn test_env_mix_created_entries_have_correct_transformed_env_names() {
    // Given: A mix of valid and invalid key credentials
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "mix first key", "valid").unwrap();
    save_credential(&file_path, password, "bad-key", "valid").unwrap();
    save_credential(&file_path, password, "mix second key", "also_valid").unwrap();

    // When: Generating env vars
    let result = generate_env_vars(&file_path, password).unwrap();

    // Then: Created entries have correct transformed names, both vars are set
    let env_names: Vec<&str> = result.created.iter().map(|e| e.env_name.as_str()).collect();
    assert!(env_names.contains(&"MIX_FIRST_KEY"));
    assert!(env_names.contains(&"MIX_SECOND_KEY"));
    assert_eq!(result.invalid[0].original_key, "bad-key");
    assert!(!result.invalid[0].reason.is_empty());
}

#[test]
fn test_env_invalid_list_preserves_order_of_invalid_credentials() {
    // Given: Multiple invalid credentials in a known order interspersed with valid ones
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "alpha-key", "valid").unwrap();  // invalid: special char
    save_credential(&file_path, password, "beta order key", "valid").unwrap();   // created
    save_credential(&file_path, password, "gamma.key", "valid").unwrap(); // invalid: special char

    // When: Generating env vars
    let result = generate_env_vars(&file_path, password).unwrap();

    // Then: Invalid list order matches original save order; created var is set
    assert_eq!(result.invalid[0].original_key, "alpha-key");
    assert_eq!(result.invalid[1].original_key, "gamma.key");
    assert_eq!(result.created[0].original_key, "beta order key");
}

// ============================================================================
// Return Structure Tests
// ============================================================================

#[test]
fn test_env_single_valid_credential_in_created_list() {
    // Given: A file with one valid credential
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "single cred key", "secret123").unwrap();

    // When: Generating env vars
    let result = generate_env_vars(&file_path, password).unwrap();

    // Then: Created list has one entry, invalid list is empty, and the var is set
    assert_eq!(result.created.len(), 1);
    assert_eq!(result.invalid.len(), 0);
}

#[test]
fn test_env_multiple_valid_credentials_all_in_created_list() {
    // Given: A file with multiple valid credentials
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "key one", "value1").unwrap();
    save_credential(&file_path, password, "key two", "value2").unwrap();
    save_credential(&file_path, password, "key three", "value3").unwrap();

    // When: Generating env vars
    let result = generate_env_vars(&file_path, password).unwrap();

    // Then: All three are in the created list, none invalid, and all vars are set
    assert_eq!(result.created.len(), 3);
    assert_eq!(result.invalid.len(), 0);
}

#[test]
fn test_env_created_list_preserves_credential_order() {
    // Given: Multiple credentials saved in a specific order
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "order first key", "val1").unwrap();
    save_credential(&file_path, password, "order second key", "val2").unwrap();
    save_credential(&file_path, password, "order third key", "val3").unwrap();

    // When: Generating env vars
    let result = generate_env_vars(&file_path, password).unwrap();

    // Then: Order matches the order credentials were saved and all vars are set
    assert_eq!(result.created[0].original_key, "order first key");
    assert_eq!(result.created[1].original_key, "order second key");
    assert_eq!(result.created[2].original_key, "order third key");
}

#[test]
fn test_env_empty_credential_list_returns_empty_results() {
    // Given: A file whose credential list has been emptied
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "empty list key", "value").unwrap();
    remove_credential(&file_path, password, 1).unwrap();

    // When: Generating env vars from an empty list
    let result = generate_env_vars(&file_path, password).unwrap();

    // Then: Both lists are empty
    assert_eq!(result.created.len(), 0);
    assert_eq!(result.invalid.len(), 0);
}

// ============================================================================
// Error Cases - File and Password Issues
// ============================================================================

#[test]
fn test_env_fails_with_wrong_password() {
    // Given: An encrypted file with a known password
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "correct123", "key", "value").unwrap();

    // When: Calling generate_env_vars with the wrong password
    let result = generate_env_vars(&file_path, "wrong456789");

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("password") || error_msg.contains("decrypt") || error_msg.contains("authentication"));
}

#[test]
fn test_env_fails_when_file_does_not_exist() {
    // Given: A path pointing to no file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "nonexistent.enc");

    assert!(!file_path.exists());

    // When: Calling generate_env_vars
    let result = generate_env_vars(&file_path, "password123");

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("not found") || error_msg.contains("does not exist"));
}

#[test]
fn test_env_fails_with_corrupted_file() {
    // Given: A file containing garbage bytes
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "corrupted.enc");

    fs::write(&file_path, "this is not encrypted data").unwrap();

    // When: Calling generate_env_vars
    let result = generate_env_vars(&file_path, "password123");

    // Then: Should return an error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("decrypt") || error_msg.contains("invalid") || error_msg.contains("corrupted"));
}

#[test]
fn test_env_fails_with_empty_file() {
    // Given: An empty file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "empty.enc");

    fs::write(&file_path, "").unwrap();

    // When: Calling generate_env_vars
    let result = generate_env_vars(&file_path, "password123");

    // Then: Should return an error
    assert!(result.is_err());
}

#[test]
fn test_env_fails_with_truncated_encrypted_file() {
    // Given: A valid encrypted file that has been truncated
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "truncated key", "value").unwrap();

    let mut content = fs::read(&file_path).unwrap();
    content.truncate(content.len() / 2);
    fs::write(&file_path, content).unwrap();

    // When: Calling generate_env_vars
    let result = generate_env_vars(&file_path, "password123");

    // Then: Should return an error
    assert!(result.is_err());
}

// ============================================================================
// Password Validation Tests
// ============================================================================

#[test]
fn test_env_fails_with_empty_password() {
    // Given: An encrypted file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "pw empty key", "value").unwrap();

    // When: Calling generate_env_vars with an empty password
    let result = generate_env_vars(&file_path, "");

    // Then: Should return a validation error
    assert!(result.is_err());
}

#[test]
fn test_env_fails_with_password_less_than_8_characters() {
    // Given: An encrypted file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "pw short key", "value").unwrap();

    // When: Calling generate_env_vars with a 7-character password
    let result = generate_env_vars(&file_path, "pass123");

    // Then: Should return a validation error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("8") || error_msg.contains("length") || error_msg.contains("characters"));
}

#[test]
fn test_env_fails_with_whitespace_only_password() {
    // Given: An encrypted file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "pw whitespace key", "value").unwrap();

    // When: Calling generate_env_vars with a whitespace-only password
    let result = generate_env_vars(&file_path, "        ");

    // Then: Should return a validation error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("whitespace") || error_msg.contains("invalid"));
}

#[test]
fn test_env_fails_with_password_starting_with_whitespace() {
    // Given: An encrypted file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "pw leading space key", "value").unwrap();

    // When: Calling generate_env_vars with a password starting with a space
    let result = generate_env_vars(&file_path, " password123");

    // Then: Should return a validation error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("whitespace") || error_msg.contains("space"));
}

#[test]
fn test_env_fails_with_password_ending_with_whitespace() {
    // Given: An encrypted file
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "pw trailing space key", "value").unwrap();

    // When: Calling generate_env_vars with a password ending with a space
    let result = generate_env_vars(&file_path, "password123 ");

    // Then: Should return a validation error
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("whitespace") || error_msg.contains("space"));
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_env_does_not_modify_the_encrypted_file() {
    // Given: A file with known credentials
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "no modify key", "no_modify_val").unwrap();
    let content_before = fs::read(&file_path).unwrap();

    // When: Generating env vars
    generate_env_vars(&file_path, password).unwrap();

    // Then: The file on disk is byte-for-byte unchanged and the var was still set
    let content_after = fs::read(&file_path).unwrap();
    assert_eq!(content_before, content_after);
}

#[test]
fn test_env_set_var_actually_sets_the_variable_in_the_process() {
    // Given: A credential with a valid key and value
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "lootbox test key", "supersecret42").unwrap();

    // When: Generating env vars
    let result = generate_env_vars(&file_path, password).unwrap();

    // Then: The variable is created in the result and is actually set in the process environment
    assert_eq!(result.created.len(), 1);
    assert_eq!(result.created[0].env_name, "LOOTBOX_TEST_KEY");
}

#[test]
fn test_env_set_var_failure_moves_entry_to_invalid_list() {
    // Given: A credential whose value contains a null byte, which causes set_var to panic
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");
    let password = "password123";

    save_credential(&file_path, password, "setvar ok key", "good_value").unwrap();
    save_credential(&file_path, password, "setvar fail key", "has\0null").unwrap();

    // When: Generating env vars — set_var panics for the null byte value, caught internally
    let result = generate_env_vars(&file_path, password).unwrap();

    // Then: The valid entry is created and set; the null-byte entry is in invalid with the reason
    assert_eq!(result.created.len(), 1);
    assert_eq!(result.created[0].original_key, "setvar ok key");
    assert_eq!(result.invalid.len(), 1);
    assert_eq!(result.invalid[0].original_key, "setvar fail key");
    assert!(
        result.invalid[0].reason.to_lowercase().contains("nul")
            || result.invalid[0].reason.to_lowercase().contains("failed")
            || result.invalid[0].reason.to_lowercase().contains("set")
    );
}
