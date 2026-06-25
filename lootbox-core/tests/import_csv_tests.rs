use lootbox::{
    export_credentials_to_csv, import_credentials_from_csv, list_credentials, save_credential,
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
// Basic import behaviour
// ============================================================================

#[test]
fn test_import_creates_encrypted_file_if_not_exists() {
    let temp_dir = setup_test_dir();
    let enc_file = get_test_file_path(&temp_dir, "credentials.enc");
    let csv_file = get_test_file_path(&temp_dir, "import.csv");

    fs::write(&csv_file, "key,value\nAPI_KEY,secret123\n").unwrap();

    assert!(!enc_file.exists());
    import_credentials_from_csv(&enc_file, "password123", &csv_file).unwrap();
    assert!(enc_file.exists());
}

#[test]
fn test_import_single_row() {
    let temp_dir = setup_test_dir();
    let enc_file = get_test_file_path(&temp_dir, "credentials.enc");
    let csv_file = get_test_file_path(&temp_dir, "import.csv");

    fs::write(&csv_file, "key,value\nmy_key,my_value\n").unwrap();
    import_credentials_from_csv(&enc_file, "password123", &csv_file).unwrap();

    let creds = list_credentials(&enc_file, "password123").unwrap();
    assert_eq!(creds.len(), 1);
    assert_eq!(creds[0].key, "my_key");
    assert_eq!(creds[0].value, "my_value");
}

#[test]
fn test_import_multiple_rows() {
    let temp_dir = setup_test_dir();
    let enc_file = get_test_file_path(&temp_dir, "credentials.enc");
    let csv_file = get_test_file_path(&temp_dir, "import.csv");

    fs::write(&csv_file, "key,value\nkey1,val1\nkey2,val2\nkey3,val3\n").unwrap();
    import_credentials_from_csv(&enc_file, "password123", &csv_file).unwrap();

    let creds = list_credentials(&enc_file, "password123").unwrap();
    assert_eq!(creds.len(), 3);
    assert_eq!(creds[0].key, "key1");
    assert_eq!(creds[1].key, "key2");
    assert_eq!(creds[2].key, "key3");
}

#[test]
fn test_import_returns_correct_count() {
    let temp_dir = setup_test_dir();
    let enc_file = get_test_file_path(&temp_dir, "credentials.enc");
    let csv_file = get_test_file_path(&temp_dir, "import.csv");

    fs::write(&csv_file, "key,value\nk1,v1\nk2,v2\nk3,v3\nk4,v4\n").unwrap();
    let count = import_credentials_from_csv(&enc_file, "password123", &csv_file).unwrap();

    assert_eq!(count, 4);
}

#[test]
fn test_import_appends_credentials_to_existing_file() {
    let temp_dir = setup_test_dir();
    let enc_file = get_test_file_path(&temp_dir, "credentials.enc");
    let csv_file = get_test_file_path(&temp_dir, "import.csv");

    save_credential(&enc_file, "password123", "existing_key", "existing_val", None, None, None).unwrap();

    fs::write(&csv_file, "key,value\nimported_key,imported_val\n").unwrap();
    import_credentials_from_csv(&enc_file, "password123", &csv_file).unwrap();

    let creds = list_credentials(&enc_file, "password123").unwrap();
    assert_eq!(creds.len(), 2);
    assert_eq!(creds[0].key, "existing_key");
    assert_eq!(creds[1].key, "imported_key");
}

#[test]
fn test_import_empty_csv_only_header() {
    let temp_dir = setup_test_dir();
    let enc_file = get_test_file_path(&temp_dir, "credentials.enc");
    let csv_file = get_test_file_path(&temp_dir, "import.csv");

    fs::write(&csv_file, "key,value\n").unwrap();
    save_credential(&enc_file, "password123", "preexisting", "value", None, None, None).unwrap();

    let count = import_credentials_from_csv(&enc_file, "password123", &csv_file).unwrap();

    assert_eq!(count, 0);
    let creds = list_credentials(&enc_file, "password123").unwrap();
    assert_eq!(creds.len(), 1); // preexisting credential untouched
}

// ============================================================================
// Blank lines
// ============================================================================

#[test]
fn test_import_ignores_blank_lines() {
    let temp_dir = setup_test_dir();
    let enc_file = get_test_file_path(&temp_dir, "credentials.enc");
    let csv_file = get_test_file_path(&temp_dir, "import.csv");

    fs::write(&csv_file, "key,value\n\nfoo,bar\n\nbaz,qux\n").unwrap();
    let count = import_credentials_from_csv(&enc_file, "password123", &csv_file).unwrap();

    assert_eq!(count, 2);
    let creds = list_credentials(&enc_file, "password123").unwrap();
    assert_eq!(creds[0].key, "foo");
    assert_eq!(creds[1].key, "baz");
}

// ============================================================================
// Quoted fields (RFC-4180)
// ============================================================================

#[test]
fn test_import_quoted_field_with_comma() {
    let temp_dir = setup_test_dir();
    let enc_file = get_test_file_path(&temp_dir, "credentials.enc");
    let csv_file = get_test_file_path(&temp_dir, "import.csv");

    fs::write(&csv_file, "key,value\n\"hello,world\",myvalue\n").unwrap();
    import_credentials_from_csv(&enc_file, "password123", &csv_file).unwrap();

    let creds = list_credentials(&enc_file, "password123").unwrap();
    assert_eq!(creds[0].key, "hello,world");
    assert_eq!(creds[0].value, "myvalue");
}

#[test]
fn test_import_quoted_field_with_double_quote() {
    let temp_dir = setup_test_dir();
    let enc_file = get_test_file_path(&temp_dir, "credentials.enc");
    let csv_file = get_test_file_path(&temp_dir, "import.csv");

    // CSV: "say ""hi""",myvalue  → key = say "hi"
    fs::write(&csv_file, "key,value\n\"say \"\"hi\"\"\",myvalue\n").unwrap();
    import_credentials_from_csv(&enc_file, "password123", &csv_file).unwrap();

    let creds = list_credentials(&enc_file, "password123").unwrap();
    assert_eq!(creds[0].key, "say \"hi\"");
    assert_eq!(creds[0].value, "myvalue");
}

#[test]
fn test_import_quoted_field_with_newline() {
    let temp_dir = setup_test_dir();
    let enc_file = get_test_file_path(&temp_dir, "credentials.enc");
    let csv_file = get_test_file_path(&temp_dir, "import.csv");

    // value contains a newline inside double quotes
    fs::write(&csv_file, "key,value\ncert,\"line1\nline2\"\n").unwrap();
    import_credentials_from_csv(&enc_file, "password123", &csv_file).unwrap();

    let creds = list_credentials(&enc_file, "password123").unwrap();
    assert_eq!(creds[0].key, "cert");
    assert_eq!(creds[0].value, "line1\nline2");
}

// ============================================================================
// Error cases
// ============================================================================

#[test]
fn test_import_fails_when_csv_not_found() {
    let temp_dir = setup_test_dir();
    let enc_file = get_test_file_path(&temp_dir, "credentials.enc");
    let csv_file = get_test_file_path(&temp_dir, "nonexistent.csv");

    let result = import_credentials_from_csv(&enc_file, "password123", &csv_file);
    assert!(result.is_err());
}

#[test]
fn test_import_fails_with_wrong_header() {
    let temp_dir = setup_test_dir();
    let enc_file = get_test_file_path(&temp_dir, "credentials.enc");
    let csv_file = get_test_file_path(&temp_dir, "import.csv");

    fs::write(&csv_file, "Key,Value\nfoo,bar\n").unwrap();

    let result = import_credentials_from_csv(&enc_file, "password123", &csv_file);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("header"));
}

#[test]
fn test_import_fails_with_wrong_password() {
    let temp_dir = setup_test_dir();
    let enc_file = get_test_file_path(&temp_dir, "credentials.enc");
    let csv_file = get_test_file_path(&temp_dir, "import.csv");

    save_credential(&enc_file, "correct_password", "existing", "data", None, None, None).unwrap();
    fs::write(&csv_file, "key,value\nnew_key,new_val\n").unwrap();

    let result = import_credentials_from_csv(&enc_file, "wrong_password!", &csv_file);
    assert!(result.is_err());
}

#[test]
fn test_import_fails_with_empty_password() {
    let temp_dir = setup_test_dir();
    let enc_file = get_test_file_path(&temp_dir, "credentials.enc");
    let csv_file = get_test_file_path(&temp_dir, "import.csv");

    fs::write(&csv_file, "key,value\nfoo,bar\n").unwrap();

    let result = import_credentials_from_csv(&enc_file, "", &csv_file);
    assert!(result.is_err());
}

#[test]
fn test_import_whitespace_only_key_fails() {
    let temp_dir = setup_test_dir();
    let enc_file = get_test_file_path(&temp_dir, "credentials.enc");
    let csv_file = get_test_file_path(&temp_dir, "import.csv");

    fs::write(&csv_file, "key,value\n   ,somevalue\n").unwrap();

    let result = import_credentials_from_csv(&enc_file, "password123", &csv_file);
    assert!(result.is_err());
}

#[test]
fn test_import_whitespace_only_value_fails() {
    let temp_dir = setup_test_dir();
    let enc_file = get_test_file_path(&temp_dir, "credentials.enc");
    let csv_file = get_test_file_path(&temp_dir, "import.csv");

    fs::write(&csv_file, "key,value\nsomekey,   \n").unwrap();

    let result = import_credentials_from_csv(&enc_file, "password123", &csv_file);
    assert!(result.is_err());
}

#[test]
fn test_import_key_exceeding_max_length_fails() {
    let temp_dir = setup_test_dir();
    let enc_file = get_test_file_path(&temp_dir, "credentials.enc");
    let csv_file = get_test_file_path(&temp_dir, "import.csv");

    let long_key = "a".repeat(65); // max allowed is 64
    let csv_content = format!("key,value\n{},somevalue\n", long_key);
    fs::write(&csv_file, &csv_content).unwrap();

    let result = import_credentials_from_csv(&enc_file, "password123", &csv_file);
    assert!(result.is_err());
}

#[test]
fn test_import_value_exceeding_max_length_fails() {
    let temp_dir = setup_test_dir();
    let enc_file = get_test_file_path(&temp_dir, "credentials.enc");
    let csv_file = get_test_file_path(&temp_dir, "import.csv");

    let long_value = "x".repeat(5001); // max allowed is 5000
    let csv_content = format!("key,value\nsome_key,{}\n", long_value);
    fs::write(&csv_file, &csv_content).unwrap();

    let result = import_credentials_from_csv(&enc_file, "password123", &csv_file);
    assert!(result.is_err());
}

// ============================================================================
// Round-trip and encoding
// ============================================================================

#[test]
fn test_import_round_trip_with_export() {
    let temp_dir = setup_test_dir();
    let enc_file = get_test_file_path(&temp_dir, "credentials.enc");
    let csv_file = get_test_file_path(&temp_dir, "import.csv");
    let csv_file2 = get_test_file_path(&temp_dir, "export.csv");

    fs::write(&csv_file, "key,value\nAPI_KEY,secret123\nDB_PASS,db_password\n").unwrap();
    import_credentials_from_csv(&enc_file, "password123", &csv_file).unwrap();

    export_credentials_to_csv(&enc_file, "password123", &csv_file2).unwrap();

    let content = fs::read_to_string(&csv_file2).unwrap();
    assert!(content.contains("API_KEY,secret123"));
    assert!(content.contains("DB_PASS,db_password"));
}

#[test]
fn test_import_unicode_characters_preserved() {
    let temp_dir = setup_test_dir();
    let enc_file = get_test_file_path(&temp_dir, "credentials.enc");
    let csv_file = get_test_file_path(&temp_dir, "import.csv");

    fs::write(&csv_file, "key,value\nclé,日本語の値\n").unwrap();
    import_credentials_from_csv(&enc_file, "password123", &csv_file).unwrap();

    let creds = list_credentials(&enc_file, "password123").unwrap();
    assert_eq!(creds[0].key, "clé");
    assert_eq!(creds[0].value, "日本語の値");
}
