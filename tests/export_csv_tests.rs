use lootbox::{
    export_credentials_to_csv, import_credentials_from_csv, list_credentials, remove_credential,
    save_credential,
};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn setup_test_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp dir")
}

fn get_test_file_path(dir: &TempDir, filename: &str) -> PathBuf {
    dir.path().join(filename)
}

// ============================================================================
// Basic output
// ============================================================================

#[test]
fn test_export_creates_csv_file() {
    let temp_dir = setup_test_dir();
    let enc_file = get_test_file_path(&temp_dir, "credentials.enc");
    let csv_file = get_test_file_path(&temp_dir, "export.csv");

    save_credential(&enc_file, "password123", "api_key", "secret").unwrap();
    export_credentials_to_csv(&enc_file, "password123", &csv_file).unwrap();

    assert!(csv_file.exists());
}

#[test]
fn test_export_csv_has_header_row() {
    let temp_dir = setup_test_dir();
    let enc_file = get_test_file_path(&temp_dir, "credentials.enc");
    let csv_file = get_test_file_path(&temp_dir, "export.csv");

    save_credential(&enc_file, "password123", "api_key", "secret").unwrap();
    export_credentials_to_csv(&enc_file, "password123", &csv_file).unwrap();

    let content = fs::read_to_string(&csv_file).unwrap();
    let first_line = content.lines().next().unwrap();
    assert_eq!(first_line, "key,value");
}

#[test]
fn test_export_single_credential() {
    let temp_dir = setup_test_dir();
    let enc_file = get_test_file_path(&temp_dir, "credentials.enc");
    let csv_file = get_test_file_path(&temp_dir, "export.csv");

    save_credential(&enc_file, "password123", "API_KEY", "sk-12345").unwrap();
    export_credentials_to_csv(&enc_file, "password123", &csv_file).unwrap();

    let content = fs::read_to_string(&csv_file).unwrap();
    assert!(content.contains("API_KEY,sk-12345"));
}

#[test]
fn test_export_multiple_credentials() {
    let temp_dir = setup_test_dir();
    let enc_file = get_test_file_path(&temp_dir, "credentials.enc");
    let csv_file = get_test_file_path(&temp_dir, "export.csv");

    save_credential(&enc_file, "password123", "key1", "value1").unwrap();
    save_credential(&enc_file, "password123", "key2", "value2").unwrap();
    save_credential(&enc_file, "password123", "key3", "value3").unwrap();
    export_credentials_to_csv(&enc_file, "password123", &csv_file).unwrap();

    let content = fs::read_to_string(&csv_file).unwrap();
    assert!(content.contains("key1,value1"));
    assert!(content.contains("key2,value2"));
    assert!(content.contains("key3,value3"));
}

#[test]
fn test_export_empty_vault_has_only_header() {
    let temp_dir = setup_test_dir();
    let enc_file = get_test_file_path(&temp_dir, "credentials.enc");
    let csv_file = get_test_file_path(&temp_dir, "export.csv");

    save_credential(&enc_file, "password123", "key", "value").unwrap();
    remove_credential(&enc_file, "password123", 1).unwrap();

    export_credentials_to_csv(&enc_file, "password123", &csv_file).unwrap();

    let content = fs::read_to_string(&csv_file).unwrap();
    assert_eq!(content.trim(), "key,value");
}

#[test]
fn test_export_values_in_plain_text() {
    let temp_dir = setup_test_dir();
    let enc_file = get_test_file_path(&temp_dir, "credentials.enc");
    let csv_file = get_test_file_path(&temp_dir, "export.csv");

    save_credential(&enc_file, "password123", "db_pass", "super_secret_password").unwrap();
    export_credentials_to_csv(&enc_file, "password123", &csv_file).unwrap();

    let content = fs::read_to_string(&csv_file).unwrap();
    assert!(content.contains("super_secret_password"));
    assert!(!content.contains("**********"));
}

// ============================================================================
// CSV escaping
// ============================================================================

#[test]
fn test_export_key_with_comma_is_quoted() {
    let temp_dir = setup_test_dir();
    let enc_file = get_test_file_path(&temp_dir, "credentials.enc");
    let csv_file = get_test_file_path(&temp_dir, "export.csv");

    save_credential(&enc_file, "password123", "hello,world", "myvalue").unwrap();
    export_credentials_to_csv(&enc_file, "password123", &csv_file).unwrap();

    let content = fs::read_to_string(&csv_file).unwrap();
    assert!(content.contains("\"hello,world\""));
}

#[test]
fn test_export_value_with_comma_is_quoted() {
    let temp_dir = setup_test_dir();
    let enc_file = get_test_file_path(&temp_dir, "credentials.enc");
    let csv_file = get_test_file_path(&temp_dir, "export.csv");

    save_credential(&enc_file, "password123", "mykey", "val,ue").unwrap();
    export_credentials_to_csv(&enc_file, "password123", &csv_file).unwrap();

    let content = fs::read_to_string(&csv_file).unwrap();
    assert!(content.contains("\"val,ue\""));
}

#[test]
fn test_export_key_with_double_quote_is_escaped() {
    let temp_dir = setup_test_dir();
    let enc_file = get_test_file_path(&temp_dir, "credentials.enc");
    let csv_file = get_test_file_path(&temp_dir, "export.csv");

    let key_with_quote = "say \"hi\"";
    save_credential(&enc_file, "password123", key_with_quote, "myvalue").unwrap();
    export_credentials_to_csv(&enc_file, "password123", &csv_file).unwrap();

    let content = fs::read_to_string(&csv_file).unwrap();
    // RFC-4180: field is wrapped in " and internal " is doubled
    assert!(content.contains("\"say \"\"hi\"\"\""));
}

#[test]
fn test_export_value_with_double_quote_is_escaped() {
    let temp_dir = setup_test_dir();
    let enc_file = get_test_file_path(&temp_dir, "credentials.enc");
    let csv_file = get_test_file_path(&temp_dir, "export.csv");

    let value_with_quote = "pass\"word";
    save_credential(&enc_file, "password123", "mykey", value_with_quote).unwrap();
    export_credentials_to_csv(&enc_file, "password123", &csv_file).unwrap();

    let content = fs::read_to_string(&csv_file).unwrap();
    assert!(content.contains("\"pass\"\"word\""));
}

#[test]
fn test_export_value_with_newline_is_quoted() {
    let temp_dir = setup_test_dir();
    let enc_file = get_test_file_path(&temp_dir, "credentials.enc");
    let csv_file = get_test_file_path(&temp_dir, "export.csv");

    save_credential(&enc_file, "password123", "cert", "line1\nline2").unwrap();
    export_credentials_to_csv(&enc_file, "password123", &csv_file).unwrap();

    let content = fs::read_to_string(&csv_file).unwrap();
    assert!(content.contains("\"line1\nline2\""));
}

// ============================================================================
// File behaviour
// ============================================================================

#[test]
fn test_export_overwrites_existing_csv_file() {
    let temp_dir = setup_test_dir();
    let enc_file = get_test_file_path(&temp_dir, "credentials.enc");
    let csv_file = get_test_file_path(&temp_dir, "export.csv");

    save_credential(&enc_file, "password123", "key1", "value1").unwrap();
    export_credentials_to_csv(&enc_file, "password123", &csv_file).unwrap();

    save_credential(&enc_file, "password123", "key2", "value2").unwrap();
    export_credentials_to_csv(&enc_file, "password123", &csv_file).unwrap();

    let content = fs::read_to_string(&csv_file).unwrap();
    assert!(content.contains("key1,value1"));
    assert!(content.contains("key2,value2"));
    // Only 2 data rows (not 3 from a doubled export)
    assert_eq!(content.lines().count(), 3); // header + 2 rows
}

#[test]
fn test_export_does_not_modify_encrypted_file() {
    let temp_dir = setup_test_dir();
    let enc_file = get_test_file_path(&temp_dir, "credentials.enc");
    let csv_file = get_test_file_path(&temp_dir, "export.csv");

    save_credential(&enc_file, "password123", "key", "value").unwrap();
    let bytes_before = fs::read(&enc_file).unwrap();

    export_credentials_to_csv(&enc_file, "password123", &csv_file).unwrap();

    let bytes_after = fs::read(&enc_file).unwrap();
    assert_eq!(bytes_before, bytes_after);
}

// ============================================================================
// Error cases
// ============================================================================

#[test]
fn test_export_fails_with_wrong_password() {
    let temp_dir = setup_test_dir();
    let enc_file = get_test_file_path(&temp_dir, "credentials.enc");
    let csv_file = get_test_file_path(&temp_dir, "export.csv");

    save_credential(&enc_file, "correct_pass", "key", "value").unwrap();

    let result = export_credentials_to_csv(&enc_file, "wrong_passwrd", &csv_file);
    assert!(result.is_err());
}

#[test]
fn test_export_fails_when_enc_file_not_found() {
    let temp_dir = setup_test_dir();
    let enc_file = get_test_file_path(&temp_dir, "nonexistent.enc");
    let csv_file = get_test_file_path(&temp_dir, "export.csv");

    let result = export_credentials_to_csv(&enc_file, "password123", &csv_file);
    assert!(result.is_err());
}

#[test]
fn test_export_fails_with_corrupted_file() {
    let temp_dir = setup_test_dir();
    let enc_file = get_test_file_path(&temp_dir, "credentials.enc");
    let csv_file = get_test_file_path(&temp_dir, "export.csv");

    fs::write(&enc_file, b"this is not a valid encrypted file").unwrap();

    let result = export_credentials_to_csv(&enc_file, "password123", &csv_file);
    assert!(result.is_err());
}

// ============================================================================
// Round-trip, order, encoding
// ============================================================================

#[test]
fn test_export_round_trip_with_import() {
    let temp_dir = setup_test_dir();
    let enc_file = get_test_file_path(&temp_dir, "credentials.enc");
    let enc_file2 = get_test_file_path(&temp_dir, "credentials2.enc");
    let csv_file = get_test_file_path(&temp_dir, "export.csv");

    save_credential(&enc_file, "password123", "alpha", "aaa").unwrap();
    save_credential(&enc_file, "password123", "beta", "bbb").unwrap();
    save_credential(&enc_file, "password123", "gamma", "ccc").unwrap();

    export_credentials_to_csv(&enc_file, "password123", &csv_file).unwrap();
    import_credentials_from_csv(&enc_file2, "password123", &csv_file).unwrap();

    let creds = list_credentials(&enc_file2, "password123").unwrap();
    assert_eq!(creds.len(), 3);
    assert_eq!(creds[0].key, "alpha");
    assert_eq!(creds[0].value, "aaa");
    assert_eq!(creds[1].key, "beta");
    assert_eq!(creds[1].value, "bbb");
    assert_eq!(creds[2].key, "gamma");
    assert_eq!(creds[2].value, "ccc");
}

#[test]
fn test_export_preserves_credential_order() {
    let temp_dir = setup_test_dir();
    let enc_file = get_test_file_path(&temp_dir, "credentials.enc");
    let csv_file = get_test_file_path(&temp_dir, "export.csv");

    save_credential(&enc_file, "password123", "first", "1").unwrap();
    save_credential(&enc_file, "password123", "second", "2").unwrap();
    save_credential(&enc_file, "password123", "third", "3").unwrap();

    export_credentials_to_csv(&enc_file, "password123", &csv_file).unwrap();

    let content = fs::read_to_string(&csv_file).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines[0], "key,value");
    assert_eq!(lines[1], "first,1");
    assert_eq!(lines[2], "second,2");
    assert_eq!(lines[3], "third,3");
}

#[test]
fn test_export_key_with_unicode() {
    let temp_dir = setup_test_dir();
    let enc_file = get_test_file_path(&temp_dir, "credentials.enc");
    let csv_file = get_test_file_path(&temp_dir, "export.csv");

    save_credential(&enc_file, "password123", "clé", "valeur").unwrap();
    export_credentials_to_csv(&enc_file, "password123", &csv_file).unwrap();

    let content = fs::read_to_string(&csv_file).unwrap();
    assert!(content.contains("clé,valeur"));
}

#[test]
fn test_export_value_with_special_characters() {
    let temp_dir = setup_test_dir();
    let enc_file = get_test_file_path(&temp_dir, "credentials.enc");
    let csv_file = get_test_file_path(&temp_dir, "export.csv");

    save_credential(&enc_file, "password123", "token", "a&b<c>d=e!f").unwrap();
    export_credentials_to_csv(&enc_file, "password123", &csv_file).unwrap();

    let content = fs::read_to_string(&csv_file).unwrap();
    assert!(content.contains("a&b<c>d=e!f"));
}
