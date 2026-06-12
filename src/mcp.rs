use crate::storage::{
    generate_env_vars, get_list_display, read_credential, remove_credential, save_credential,
    update_credential,
};
use serde_json::{json, Value};
use std::path::Path;

/// Processes a single JSON-RPC 2.0 message and returns the response string.
/// Returns an empty string for notifications (requests without an `id`).
pub fn handle_mcp_message(input: &str) -> String {
    let request: Value = match serde_json::from_str(input) {
        Ok(v) => v,
        Err(_) => {
            return json!({
                "jsonrpc": "2.0",
                "id": null,
                "error": { "code": -32700, "message": "Parse error" }
            })
            .to_string();
        }
    };

    // Notifications have no "id" key — must not receive a response
    let has_id = request
        .as_object()
        .map(|o| o.contains_key("id"))
        .unwrap_or(false);
    if !has_id {
        return String::new();
    }

    let id = &request["id"];

    let method = match request["method"].as_str() {
        Some(m) => m,
        None => return error_response(id, -32600, "Invalid Request: missing method"),
    };

    match method {
        "initialize" => handle_initialize(id),
        "notifications/initialized" => String::new(),
        "tools/list" => handle_tools_list(id),
        "tools/call" => handle_tools_call(id, &request["params"]),
        _ => error_response(id, -32601, "Method not found"),
    }
}

// ============================================================================
// Method handlers
// ============================================================================

fn handle_initialize(id: &Value) -> String {
    success_response(
        id,
        json!({
            "protocolVersion": "2024-11-05",
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "lootbox", "version": env!("CARGO_PKG_VERSION") }
        }),
    )
}

fn handle_tools_list(id: &Value) -> String {
    success_response(
        id,
        json!({
            "tools": [
                {
                    "name": "save_credential",
                    "description": "Save a new credential (key-value pair) to an encrypted file. Creates a new file or appends to an existing one when the password matches.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "filename": { "type": "string", "description": "Path to the encrypted credentials file" },
                            "password": { "type": "string", "description": "File password (min 8 chars, no leading/trailing whitespace)" },
                            "key":      { "type": "string", "description": "Name of the secret (e.g. 'API_KEY')" },
                            "value":    { "type": "string", "description": "Secret value to store" }
                        },
                        "required": ["filename", "password", "key", "value"]
                    }
                },
                {
                    "name": "list_credentials",
                    "description": "List all credentials in an encrypted file. Keys are shown in plain text; values are masked as **********.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "filename": { "type": "string", "description": "Path to the encrypted credentials file" },
                            "password": { "type": "string", "description": "File password" }
                        },
                        "required": ["filename", "password"]
                    }
                },
                {
                    "name": "read_credential",
                    "description": "Read a specific credential by its 1-based position ID. Returns the key and value in plain text.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "filename": { "type": "string",  "description": "Path to the encrypted credentials file" },
                            "password": { "type": "string",  "description": "File password" },
                            "id":       { "type": "integer", "description": "1-based position ID of the credential to read" }
                        },
                        "required": ["filename", "password", "id"]
                    }
                },
                {
                    "name": "update_credential",
                    "description": "Update an existing credential by its 1-based position ID. Omit 'key' or 'value' to keep the current value.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "filename": { "type": "string",  "description": "Path to the encrypted credentials file" },
                            "password": { "type": "string",  "description": "File password" },
                            "id":       { "type": "integer", "description": "1-based position ID of the credential to update" },
                            "key":      { "type": "string",  "description": "New key name (optional)" },
                            "value":    { "type": "string",  "description": "New secret value (optional)" }
                        },
                        "required": ["filename", "password", "id"]
                    }
                },
                {
                    "name": "remove_credential",
                    "description": "Remove a credential by its 1-based position ID. Subsequent credentials shift down by one position.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "filename": { "type": "string",  "description": "Path to the encrypted credentials file" },
                            "password": { "type": "string",  "description": "File password" },
                            "id":       { "type": "integer", "description": "1-based position ID of the credential to remove" }
                        },
                        "required": ["filename", "password", "id"]
                    }
                },
                {
                    "name": "generate_env_vars",
                    "description": "Generate shell export statements from all credentials. Keys are uppercased with spaces replaced by underscores. Invalid keys and null-byte values are reported as skipped.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "filename": { "type": "string", "description": "Path to the encrypted credentials file" },
                            "password": { "type": "string", "description": "File password" }
                        },
                        "required": ["filename", "password"]
                    }
                }
            ]
        }),
    )
}

fn handle_tools_call(id: &Value, params: &Value) -> String {
    let tool_name = match params["name"].as_str() {
        Some(n) => n,
        None => return tool_error_response(id, "Missing required field: name"),
    };

    let args = &params["arguments"];

    match tool_name {
        "save_credential"  => tool_save_credential(id, args),
        "list_credentials" => tool_list_credentials(id, args),
        "read_credential"  => tool_read_credential(id, args),
        "update_credential" => tool_update_credential(id, args),
        "remove_credential" => tool_remove_credential(id, args),
        "generate_env_vars" => tool_generate_env_vars(id, args),
        _ => tool_error_response(id, &format!("Unknown tool: {}", tool_name)),
    }
}

// ============================================================================
// Tool implementations
// ============================================================================

fn tool_save_credential(id: &Value, args: &Value) -> String {
    let filename = match args["filename"].as_str() {
        Some(f) => f,
        None => return tool_error_response(id, "Missing required argument: filename"),
    };
    let password = match args["password"].as_str() {
        Some(p) => p,
        None => return tool_error_response(id, "Missing required argument: password"),
    };
    let key = match args["key"].as_str() {
        Some(k) => k,
        None => return tool_error_response(id, "Missing required argument: key"),
    };
    let value = match args["value"].as_str() {
        Some(v) => v,
        None => return tool_error_response(id, "Missing required argument: value"),
    };

    match save_credential(Path::new(filename), password, key, value) {
        Ok(()) => tool_success_response(id, "Credential saved successfully."),
        Err(e) => tool_error_response(id, &e.to_string()),
    }
}

fn tool_list_credentials(id: &Value, args: &Value) -> String {
    let filename = match args["filename"].as_str() {
        Some(f) => f,
        None => return tool_error_response(id, "Missing required argument: filename"),
    };
    let password = match args["password"].as_str() {
        Some(p) => p,
        None => return tool_error_response(id, "Missing required argument: password"),
    };

    match get_list_display(Path::new(filename), password) {
        Ok(display) => tool_success_response(id, &display),
        Err(e) => tool_error_response(id, &e.to_string()),
    }
}

fn tool_read_credential(id: &Value, args: &Value) -> String {
    let filename = match args["filename"].as_str() {
        Some(f) => f,
        None => return tool_error_response(id, "Missing required argument: filename"),
    };
    let password = match args["password"].as_str() {
        Some(p) => p,
        None => return tool_error_response(id, "Missing required argument: password"),
    };
    let position_id = match args["id"].as_u64() {
        Some(n) => n as usize,
        None => {
            return tool_error_response(
                id,
                "Missing required argument: id (must be a positive integer)",
            )
        }
    };

    match read_credential(Path::new(filename), password, position_id) {
        Ok(cred) => tool_success_response(
            id,
            &format!("[{}] Key: {}\nValue: {}", position_id, cred.key, cred.value),
        ),
        Err(e) => tool_error_response(id, &e.to_string()),
    }
}

fn tool_update_credential(id: &Value, args: &Value) -> String {
    let filename = match args["filename"].as_str() {
        Some(f) => f,
        None => return tool_error_response(id, "Missing required argument: filename"),
    };
    let password = match args["password"].as_str() {
        Some(p) => p,
        None => return tool_error_response(id, "Missing required argument: password"),
    };
    let position_id = match args["id"].as_u64() {
        Some(n) => n as usize,
        None => {
            return tool_error_response(
                id,
                "Missing required argument: id (must be a positive integer)",
            )
        }
    };

    let new_key = args["key"].as_str();
    let new_value = args["value"].as_str();

    match update_credential(Path::new(filename), password, position_id, new_key, new_value) {
        Ok(()) => tool_success_response(id, "Credential updated successfully."),
        Err(e) => tool_error_response(id, &e.to_string()),
    }
}

fn tool_remove_credential(id: &Value, args: &Value) -> String {
    let filename = match args["filename"].as_str() {
        Some(f) => f,
        None => return tool_error_response(id, "Missing required argument: filename"),
    };
    let password = match args["password"].as_str() {
        Some(p) => p,
        None => return tool_error_response(id, "Missing required argument: password"),
    };
    let position_id = match args["id"].as_u64() {
        Some(n) => n as usize,
        None => {
            return tool_error_response(
                id,
                "Missing required argument: id (must be a positive integer)",
            )
        }
    };

    match remove_credential(Path::new(filename), password, position_id) {
        Ok(()) => tool_success_response(id, "Credential removed successfully."),
        Err(e) => tool_error_response(id, &e.to_string()),
    }
}

fn tool_generate_env_vars(id: &Value, args: &Value) -> String {
    let filename = match args["filename"].as_str() {
        Some(f) => f,
        None => return tool_error_response(id, "Missing required argument: filename"),
    };
    let password = match args["password"].as_str() {
        Some(p) => p,
        None => return tool_error_response(id, "Missing required argument: password"),
    };

    match generate_env_vars(Path::new(filename), password) {
        Ok(result) => {
            let mut output = String::new();

            for entry in &result.created {
                let escaped = entry.value.replace('\'', "'\\''");
                output.push_str(&format!("export {}='{}'\n", entry.env_name, escaped));
            }

            if !result.invalid.is_empty() {
                output.push_str("\n# Skipped:\n");
                for entry in &result.invalid {
                    output.push_str(&format!("#   {} - {}\n", entry.original_key, entry.reason));
                }
            }

            tool_success_response(id, output.trim_end())
        }
        Err(e) => tool_error_response(id, &e.to_string()),
    }
}

// ============================================================================
// Response helpers
// ============================================================================

fn success_response(id: &Value, result: Value) -> String {
    json!({ "jsonrpc": "2.0", "id": id, "result": result }).to_string()
}

fn error_response(id: &Value, code: i32, message: &str) -> String {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message }
    })
    .to_string()
}

fn tool_success_response(id: &Value, text: &str) -> String {
    success_response(
        id,
        json!({
            "content": [{ "type": "text", "text": text }],
            "isError": false
        }),
    )
}

fn tool_error_response(id: &Value, message: &str) -> String {
    success_response(
        id,
        json!({
            "content": [{ "type": "text", "text": message }],
            "isError": true
        }),
    )
}
