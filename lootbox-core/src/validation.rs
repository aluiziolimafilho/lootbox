use anyhow::{bail, Result};

pub const PASSWORD_MAX_LENGTH: usize = 32;
pub const SECRET_KEY_MAX_LENGTH: usize = 64;
pub const SECRET_VALUE_MAX_LENGTH: usize = 5000;
pub const NAME_MAX_LENGTH: usize = 255;
pub const DESCRIPTION_MAX_LENGTH: usize = 5000;
pub const URL_MAX_LENGTH: usize = 3000;

/// Validates password according to requirements:
/// - At least 8 characters
/// - At most PASSWORD_MAX_LENGTH characters
/// - Cannot contain only whitespace
/// - Cannot start or end with whitespace
pub fn validate_password(password: &str) -> Result<()> {
    if password.len() < 8 {
        bail!("Password must be at least 8 characters long");
    }

    if password.trim().is_empty() {
        bail!("Password cannot contain only whitespace");
    }

    if password.starts_with(char::is_whitespace) {
        bail!("Password cannot start with whitespace");
    }

    if password.ends_with(char::is_whitespace) {
        bail!("Password cannot end with whitespace");
    }

    if password.len() > PASSWORD_MAX_LENGTH {
        bail!("Password must be at most {PASSWORD_MAX_LENGTH} characters long");
    }

    Ok(())
}

/// Validates secret_key (required field, cannot be empty or only whitespace)
pub fn validate_secret_key(key: &str) -> Result<()> {
    if key.is_empty() {
        bail!("Secret key cannot be empty");
    }

    if key.trim().is_empty() {
        bail!("Secret key cannot contain only whitespace");
    }

    if key.len() > SECRET_KEY_MAX_LENGTH {
        bail!("Secret key must be at most {SECRET_KEY_MAX_LENGTH} characters long");
    }

    Ok(())
}

/// Validates secret_value (required field, cannot be empty or only whitespace)
pub fn validate_secret_value(value: &str) -> Result<()> {
    if value.is_empty() {
        bail!("Secret value cannot be empty");
    }

    if value.trim().is_empty() {
        bail!("Secret value cannot contain only whitespace");
    }

    if value.len() > SECRET_VALUE_MAX_LENGTH {
        bail!("Secret value must be at most {SECRET_VALUE_MAX_LENGTH} characters long");
    }

    Ok(())
}

/// Validates credential name (required when provided: non-empty, max 255 chars)
pub fn validate_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("Name cannot be empty");
    }

    if name.len() > NAME_MAX_LENGTH {
        bail!("Name must be at most {NAME_MAX_LENGTH} characters long");
    }

    Ok(())
}

/// Validates description (optional: max 5000 chars; empty is accepted)
pub fn validate_description(desc: &str) -> Result<()> {
    if desc.len() > DESCRIPTION_MAX_LENGTH {
        bail!("Description must be at most {DESCRIPTION_MAX_LENGTH} characters long");
    }
    Ok(())
}

/// Validates URL (optional: max 3000 chars; empty is accepted)
pub fn validate_url(url: &str) -> Result<()> {
    if url.len() > URL_MAX_LENGTH {
        bail!("URL must be at most {URL_MAX_LENGTH} characters long");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_validation() {
        // Valid passwords
        assert!(validate_password("password").is_ok());
        assert!(validate_password("12345678").is_ok());
        assert!(validate_password("pass word").is_ok()); // space in middle is ok

        // Invalid passwords
        assert!(validate_password("").is_err());
        assert!(validate_password("short").is_err());
        assert!(validate_password("1234567").is_err()); // 7 chars
        assert!(validate_password("        ").is_err()); // only whitespace
        assert!(validate_password(" password").is_err()); // starts with space
        assert!(validate_password("password ").is_err()); // ends with space
    }

    #[test]
    fn test_password_max_length() {
        assert!(validate_password(&"a".repeat(PASSWORD_MAX_LENGTH)).is_ok());
        assert!(validate_password(&"a".repeat(PASSWORD_MAX_LENGTH + 1)).is_err());
    }

    #[test]
    fn test_secret_key_validation() {
        // Valid keys
        assert!(validate_secret_key("key").is_ok());
        assert!(validate_secret_key("my_key").is_ok());

        // Invalid keys
        assert!(validate_secret_key("").is_err());
        assert!(validate_secret_key("   ").is_err());
    }

    #[test]
    fn test_secret_key_max_length() {
        assert!(validate_secret_key(&"a".repeat(SECRET_KEY_MAX_LENGTH)).is_ok());
        assert!(validate_secret_key(&"a".repeat(SECRET_KEY_MAX_LENGTH + 1)).is_err());
    }

    #[test]
    fn test_secret_value_validation() {
        // Valid values
        assert!(validate_secret_value("value").is_ok());
        assert!(validate_secret_value("my_value").is_ok());

        // Invalid values
        assert!(validate_secret_value("").is_err());
        assert!(validate_secret_value("   ").is_err());
    }

    #[test]
    fn test_secret_value_max_length() {
        assert!(validate_secret_value(&"a".repeat(SECRET_VALUE_MAX_LENGTH)).is_ok());
        assert!(validate_secret_value(&"a".repeat(SECRET_VALUE_MAX_LENGTH + 1)).is_err());
    }

    #[test]
    fn test_name_validation() {
        assert!(validate_name("My API Key").is_ok());
        assert!(validate_name("x").is_ok());
        assert!(validate_name("").is_err());
        assert!(validate_name(&"a".repeat(NAME_MAX_LENGTH)).is_ok());
        assert!(validate_name(&"a".repeat(NAME_MAX_LENGTH + 1)).is_err());
    }

    #[test]
    fn test_description_validation() {
        assert!(validate_description("").is_ok());
        assert!(validate_description("some description").is_ok());
        assert!(validate_description(&"a".repeat(DESCRIPTION_MAX_LENGTH)).is_ok());
        assert!(validate_description(&"a".repeat(DESCRIPTION_MAX_LENGTH + 1)).is_err());
    }

    #[test]
    fn test_url_validation() {
        assert!(validate_url("").is_ok());
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url(&"a".repeat(URL_MAX_LENGTH)).is_ok());
        assert!(validate_url(&"a".repeat(URL_MAX_LENGTH + 1)).is_err());
    }
}
