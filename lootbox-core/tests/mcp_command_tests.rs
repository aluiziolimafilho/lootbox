use lootbox::mcp::handle_mcp_message;
use lootbox::save_credential;
use serde_json::{json, Value};
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

fn parse_response(response: &str) -> Value {
    serde_json::from_str(response).expect("Response should be valid JSON")
}

fn build_request(id: u64, method: &str, params: Value) -> String {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params
    })
    .to_string()
}

fn build_tool_call(id: u64, tool: &str, arguments: Value) -> String {
    build_request(
        id,
        "tools/call",
        json!({ "name": tool, "arguments": arguments }),
    )
}

// ============================================================================
// Protocol Handshake Tests
// ============================================================================

#[test]
fn test_mcp_initialize_returns_valid_jsonrpc_response() {
    let request = build_request(
        1,
        "initialize",
        json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "test-client", "version": "1.0" }
        }),
    );

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    assert_eq!(parsed["jsonrpc"], "2.0");
    assert_eq!(parsed["id"], 1);
    assert!(parsed["result"].is_object());
    assert!(parsed["error"].is_null());
}

#[test]
fn test_mcp_initialize_returns_protocol_version() {
    let request = build_request(
        1,
        "initialize",
        json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "test-client", "version": "1.0" }
        }),
    );

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    assert!(parsed["result"]["protocolVersion"].is_string());
}

#[test]
fn test_mcp_initialize_returns_server_info_with_name() {
    let request = build_request(
        1,
        "initialize",
        json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "test-client", "version": "1.0" }
        }),
    );

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    let server_info = &parsed["result"]["serverInfo"];
    assert!(server_info.is_object());
    assert!(server_info["name"].is_string());
}

#[test]
fn test_mcp_initialize_declares_tools_capability() {
    let request = build_request(
        1,
        "initialize",
        json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "test-client", "version": "1.0" }
        }),
    );

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    assert!(parsed["result"]["capabilities"]["tools"].is_object());
}

#[test]
fn test_mcp_notification_initialized_returns_no_response() {
    // Notifications have no `id` field and must not receive a response
    let notification = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized",
        "params": {}
    })
    .to_string();

    let response = handle_mcp_message(&notification);

    assert!(response.is_empty());
}

#[test]
fn test_mcp_unknown_method_returns_method_not_found_error() {
    let request = build_request(99, "nonexistent/method", json!({}));

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    assert_eq!(parsed["id"], 99);
    assert!(parsed["error"].is_object());
    // JSON-RPC method-not-found code is -32601
    assert_eq!(parsed["error"]["code"], -32601);
}

#[test]
fn test_mcp_invalid_json_returns_parse_error() {
    let response = handle_mcp_message("this is not json {{{");
    let parsed = parse_response(&response);

    assert!(parsed["error"].is_object());
    // JSON-RPC parse-error code is -32700
    assert_eq!(parsed["error"]["code"], -32700);
}

#[test]
fn test_mcp_response_id_matches_request_id() {
    let request = build_request(
        42,
        "initialize",
        json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "test-client", "version": "1.0" }
        }),
    );

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    assert_eq!(parsed["id"], 42);
}

// ============================================================================
// Tools Discovery Tests
// ============================================================================

#[test]
fn test_mcp_tools_list_returns_array_of_tools() {
    let request = build_request(2, "tools/list", json!({}));

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    assert!(parsed["result"]["tools"].is_array());
    assert!(parsed["result"]["tools"].as_array().unwrap().len() >= 6);
}

#[test]
fn test_mcp_tools_list_includes_save_credential() {
    let request = build_request(2, "tools/list", json!({}));

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    let tools = parsed["result"]["tools"].as_array().unwrap();
    let names: Vec<&str> = tools
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();
    assert!(names.contains(&"save_credential"));
}

#[test]
fn test_mcp_tools_list_includes_list_credentials() {
    let request = build_request(2, "tools/list", json!({}));

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    let tools = parsed["result"]["tools"].as_array().unwrap();
    let names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
    assert!(names.contains(&"list_credentials"));
}

#[test]
fn test_mcp_tools_list_includes_read_credential() {
    let request = build_request(2, "tools/list", json!({}));

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    let tools = parsed["result"]["tools"].as_array().unwrap();
    let names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
    assert!(names.contains(&"read_credential"));
}

#[test]
fn test_mcp_tools_list_includes_update_credential() {
    let request = build_request(2, "tools/list", json!({}));

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    let tools = parsed["result"]["tools"].as_array().unwrap();
    let names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
    assert!(names.contains(&"update_credential"));
}

#[test]
fn test_mcp_tools_list_includes_remove_credential() {
    let request = build_request(2, "tools/list", json!({}));

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    let tools = parsed["result"]["tools"].as_array().unwrap();
    let names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
    assert!(names.contains(&"remove_credential"));
}

#[test]
fn test_mcp_tools_list_includes_generate_env_vars() {
    let request = build_request(2, "tools/list", json!({}));

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    let tools = parsed["result"]["tools"].as_array().unwrap();
    let names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
    assert!(names.contains(&"generate_env_vars"));
}

#[test]
fn test_mcp_tools_list_each_tool_has_description() {
    let request = build_request(2, "tools/list", json!({}));

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    let tools = parsed["result"]["tools"].as_array().unwrap();
    for tool in tools {
        assert!(
            tool["description"].is_string(),
            "Tool '{}' is missing a description",
            tool["name"]
        );
        assert!(
            !tool["description"].as_str().unwrap().is_empty(),
            "Tool '{}' has an empty description",
            tool["name"]
        );
    }
}

#[test]
fn test_mcp_tools_list_each_tool_has_input_schema() {
    let request = build_request(2, "tools/list", json!({}));

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    let tools = parsed["result"]["tools"].as_array().unwrap();
    for tool in tools {
        assert!(
            tool["inputSchema"].is_object(),
            "Tool '{}' is missing an inputSchema",
            tool["name"]
        );
    }
}

// ============================================================================
// save_credential Tool Tests
// ============================================================================

#[test]
fn test_mcp_tool_save_credential_creates_file_and_returns_success() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    let request = build_tool_call(
        3,
        "save_credential",
        json!({
            "filename": file_path.to_str().unwrap(),
            "password": "password123",
            "key": "api_key",
            "value": "secret_value"
        }),
    );

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    assert!(parsed["result"]["isError"] == false || parsed["result"]["isError"].is_null());
    assert!(file_path.exists());
}

#[test]
fn test_mcp_tool_save_credential_result_contains_text_content() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    let request = build_tool_call(
        3,
        "save_credential",
        json!({
            "filename": file_path.to_str().unwrap(),
            "password": "password123",
            "key": "my_key",
            "value": "my_value"
        }),
    );

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    let content = &parsed["result"]["content"];
    assert!(content.is_array());
    assert!(!content.as_array().unwrap().is_empty());
    assert_eq!(content[0]["type"], "text");
}

#[test]
fn test_mcp_tool_save_credential_fails_with_wrong_password_on_existing_file() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "correct123", "key", "value", None, None, None).unwrap();

    let request = build_tool_call(
        3,
        "save_credential",
        json!({
            "filename": file_path.to_str().unwrap(),
            "password": "wrong_password",
            "key": "key2",
            "value": "value2"
        }),
    );

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    assert_eq!(parsed["result"]["isError"], true);
}

#[test]
fn test_mcp_tool_save_credential_fails_with_invalid_password() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    let request = build_tool_call(
        3,
        "save_credential",
        json!({
            "filename": file_path.to_str().unwrap(),
            "password": "short",
            "key": "key",
            "value": "value"
        }),
    );

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    assert_eq!(parsed["result"]["isError"], true);
}

#[test]
fn test_mcp_tool_save_credential_missing_required_param_returns_error() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    // Missing "value" parameter
    let request = build_tool_call(
        3,
        "save_credential",
        json!({
            "filename": file_path.to_str().unwrap(),
            "password": "password123",
            "key": "my_key"
        }),
    );

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    assert_eq!(parsed["result"]["isError"], true);
}

// ============================================================================
// list_credentials Tool Tests
// ============================================================================

#[test]
fn test_mcp_tool_list_credentials_returns_masked_values() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "api_key", "super_secret", None, None, None).unwrap();

    let request = build_tool_call(
        4,
        "list_credentials",
        json!({
            "filename": file_path.to_str().unwrap(),
            "password": "password123"
        }),
    );

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    let text = parsed["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("api_key"));
    assert!(!text.contains("super_secret"));
    assert!(text.contains("**********"));
}

#[test]
fn test_mcp_tool_list_credentials_shows_position_ids() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "first_key", "val1", None, None, None).unwrap();
    save_credential(&file_path, "password123", "second_key", "val2", None, None, None).unwrap();

    let request = build_tool_call(
        4,
        "list_credentials",
        json!({
            "filename": file_path.to_str().unwrap(),
            "password": "password123"
        }),
    );

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    let text = parsed["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("[1]"));
    assert!(text.contains("[2]"));
}

#[test]
fn test_mcp_tool_list_credentials_fails_with_wrong_password() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "correct123", "key", "value", None, None, None).unwrap();

    let request = build_tool_call(
        4,
        "list_credentials",
        json!({
            "filename": file_path.to_str().unwrap(),
            "password": "wrong456789"
        }),
    );

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    assert_eq!(parsed["result"]["isError"], true);
}

#[test]
fn test_mcp_tool_list_credentials_fails_with_nonexistent_file() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "nonexistent.enc");

    let request = build_tool_call(
        4,
        "list_credentials",
        json!({
            "filename": file_path.to_str().unwrap(),
            "password": "password123"
        }),
    );

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    assert_eq!(parsed["result"]["isError"], true);
}

// ============================================================================
// read_credential Tool Tests
// ============================================================================

#[test]
fn test_mcp_tool_read_credential_returns_plain_text_key_and_value() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "api_key", "super_secret", None, None, None).unwrap();

    let request = build_tool_call(
        5,
        "read_credential",
        json!({
            "filename": file_path.to_str().unwrap(),
            "password": "password123",
            "id": 1
        }),
    );

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    let text = parsed["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("api_key"));
    assert!(text.contains("super_secret"));
}

#[test]
fn test_mcp_tool_read_credential_reads_correct_entry_by_id() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "first", "value1", None, None, None).unwrap();
    save_credential(&file_path, "password123", "second", "value2", None, None, None).unwrap();
    save_credential(&file_path, "password123", "third", "value3", None, None, None).unwrap();

    let request = build_tool_call(
        5,
        "read_credential",
        json!({
            "filename": file_path.to_str().unwrap(),
            "password": "password123",
            "id": 2
        }),
    );

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    let text = parsed["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("second"));
    assert!(text.contains("value2"));
    assert!(!text.contains("value1"));
    assert!(!text.contains("value3"));
}

#[test]
fn test_mcp_tool_read_credential_fails_with_invalid_id() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "key", "value", None, None, None).unwrap();

    let request = build_tool_call(
        5,
        "read_credential",
        json!({
            "filename": file_path.to_str().unwrap(),
            "password": "password123",
            "id": 99
        }),
    );

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    assert_eq!(parsed["result"]["isError"], true);
}

#[test]
fn test_mcp_tool_read_credential_fails_with_wrong_password() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "correct123", "key", "value", None, None, None).unwrap();

    let request = build_tool_call(
        5,
        "read_credential",
        json!({
            "filename": file_path.to_str().unwrap(),
            "password": "wrong456789",
            "id": 1
        }),
    );

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    assert_eq!(parsed["result"]["isError"], true);
}

// ============================================================================
// update_credential Tool Tests
// ============================================================================

#[test]
fn test_mcp_tool_update_credential_changes_key_and_value() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "old_key", "old_value", None, None, None).unwrap();

    let request = build_tool_call(
        6,
        "update_credential",
        json!({
            "filename": file_path.to_str().unwrap(),
            "password": "password123",
            "id": 1,
            "key": "new_key",
            "value": "new_value"
        }),
    );

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    assert!(parsed["result"]["isError"] == false || parsed["result"]["isError"].is_null());

    // Verify by reading back
    let read_request = build_tool_call(
        7,
        "read_credential",
        json!({
            "filename": file_path.to_str().unwrap(),
            "password": "password123",
            "id": 1
        }),
    );
    let read_response = handle_mcp_message(&read_request);
    let read_parsed = parse_response(&read_response);
    let text = read_parsed["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("new_key"));
    assert!(text.contains("new_value"));
}

#[test]
fn test_mcp_tool_update_credential_with_only_key_keeps_value() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "old_key", "original_value", None, None, None).unwrap();

    let request = build_tool_call(
        6,
        "update_credential",
        json!({
            "filename": file_path.to_str().unwrap(),
            "password": "password123",
            "id": 1,
            "key": "renamed_key"
        }),
    );

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    assert!(parsed["result"]["isError"] == false || parsed["result"]["isError"].is_null());

    let read_request = build_tool_call(
        7,
        "read_credential",
        json!({
            "filename": file_path.to_str().unwrap(),
            "password": "password123",
            "id": 1
        }),
    );
    let read_response = handle_mcp_message(&read_request);
    let read_parsed = parse_response(&read_response);
    let text = read_parsed["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("renamed_key"));
    assert!(text.contains("original_value"));
}

#[test]
fn test_mcp_tool_update_credential_fails_with_invalid_id() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "key", "value", None, None, None).unwrap();

    let request = build_tool_call(
        6,
        "update_credential",
        json!({
            "filename": file_path.to_str().unwrap(),
            "password": "password123",
            "id": 99,
            "key": "new_key"
        }),
    );

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    assert_eq!(parsed["result"]["isError"], true);
}

#[test]
fn test_mcp_tool_update_credential_fails_with_wrong_password() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "correct123", "key", "value", None, None, None).unwrap();

    let request = build_tool_call(
        6,
        "update_credential",
        json!({
            "filename": file_path.to_str().unwrap(),
            "password": "wrong456789",
            "id": 1,
            "key": "new_key"
        }),
    );

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    assert_eq!(parsed["result"]["isError"], true);
}

// ============================================================================
// remove_credential Tool Tests
// ============================================================================

#[test]
fn test_mcp_tool_remove_credential_removes_the_entry() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "key1", "value1", None, None, None).unwrap();
    save_credential(&file_path, "password123", "key2", "value2", None, None, None).unwrap();

    let request = build_tool_call(
        7,
        "remove_credential",
        json!({
            "filename": file_path.to_str().unwrap(),
            "password": "password123",
            "id": 1
        }),
    );

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    assert!(parsed["result"]["isError"] == false || parsed["result"]["isError"].is_null());

    // Verify removal — only key2 should remain at position 1
    let list_request = build_tool_call(
        8,
        "list_credentials",
        json!({
            "filename": file_path.to_str().unwrap(),
            "password": "password123"
        }),
    );
    let list_response = handle_mcp_message(&list_request);
    let list_parsed = parse_response(&list_response);
    let text = list_parsed["result"]["content"][0]["text"].as_str().unwrap();
    assert!(!text.contains("key1"));
    assert!(text.contains("key2"));
}

#[test]
fn test_mcp_tool_remove_credential_fails_with_invalid_id() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "key", "value", None, None, None).unwrap();

    let request = build_tool_call(
        7,
        "remove_credential",
        json!({
            "filename": file_path.to_str().unwrap(),
            "password": "password123",
            "id": 99
        }),
    );

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    assert_eq!(parsed["result"]["isError"], true);
}

#[test]
fn test_mcp_tool_remove_credential_fails_with_wrong_password() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "correct123", "key", "value", None, None, None).unwrap();

    let request = build_tool_call(
        7,
        "remove_credential",
        json!({
            "filename": file_path.to_str().unwrap(),
            "password": "wrong456789",
            "id": 1
        }),
    );

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    assert_eq!(parsed["result"]["isError"], true);
}

// ============================================================================
// generate_env_vars Tool Tests
// ============================================================================

#[test]
fn test_mcp_tool_generate_env_vars_returns_export_statements() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "api key", "secret123", None, None, None).unwrap();

    let request = build_tool_call(
        8,
        "generate_env_vars",
        json!({
            "filename": file_path.to_str().unwrap(),
            "password": "password123",
            "id": 1
        }),
    );

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    assert!(parsed["result"]["isError"] == false || parsed["result"]["isError"].is_null());
    let text = parsed["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("export API_KEY="));
}

#[test]
fn test_mcp_tool_generate_env_vars_reports_invalid_keys_separately() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "valid key", "ok", None, None, None).unwrap();
    save_credential(&file_path, "password123", "bad-key", "ok", None, None, None).unwrap();

    // Select the invalid key credential (ID 2)
    let request = build_tool_call(
        8,
        "generate_env_vars",
        json!({
            "filename": file_path.to_str().unwrap(),
            "password": "password123",
            "id": 2
        }),
    );

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    let text = parsed["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("bad-key"));
}

#[test]
fn test_mcp_tool_generate_env_vars_fails_with_wrong_password() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "correct123", "key", "value", None, None, None).unwrap();

    let request = build_tool_call(
        8,
        "generate_env_vars",
        json!({
            "filename": file_path.to_str().unwrap(),
            "password": "wrong456789",
            "id": 1
        }),
    );

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    assert_eq!(parsed["result"]["isError"], true);
}

#[test]
fn test_mcp_tool_generate_env_vars_fails_without_id() {
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "password123", "key", "value", None, None, None).unwrap();

    let request = build_tool_call(
        8,
        "generate_env_vars",
        json!({
            "filename": file_path.to_str().unwrap(),
            "password": "password123"
        }),
    );

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    assert_eq!(parsed["result"]["isError"], true);
}

// ============================================================================
// General Tool Error Handling Tests
// ============================================================================

#[test]
fn test_mcp_unknown_tool_name_returns_error_result() {
    let request = build_tool_call(
        9,
        "nonexistent_tool",
        json!({ "filename": "/some/file", "password": "password123" }),
    );

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    assert_eq!(parsed["result"]["isError"], true);
}

#[test]
fn test_mcp_tool_call_with_empty_arguments_returns_error() {
    let request = build_tool_call(9, "save_credential", json!({}));

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    assert_eq!(parsed["result"]["isError"], true);
}

#[test]
fn test_mcp_all_error_results_contain_text_content() {
    // A wrong-password call should still return a well-formed result with text content
    let temp_dir = setup_test_dir();
    let file_path = get_test_file_path(&temp_dir, "credentials.enc");

    save_credential(&file_path, "correct123", "key", "value", None, None, None).unwrap();

    let request = build_tool_call(
        9,
        "list_credentials",
        json!({
            "filename": file_path.to_str().unwrap(),
            "password": "wrong456789"
        }),
    );

    let response = handle_mcp_message(&request);
    let parsed = parse_response(&response);

    assert_eq!(parsed["result"]["isError"], true);
    let content = &parsed["result"]["content"];
    assert!(content.is_array());
    assert!(!content.as_array().unwrap().is_empty());
    assert_eq!(content[0]["type"], "text");
    assert!(!content[0]["text"].as_str().unwrap().is_empty());
}
