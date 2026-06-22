use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use anyhow::Result;
use argon2::{
    password_hash::{rand_core::RngCore, SaltString},
    Argon2, PasswordHasher,
};

/// Derives a 32-byte encryption key from a password using Argon2
pub fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; 32]> {
    let argon2 = Argon2::default();

    // Convert salt bytes to SaltString format
    let salt_string = SaltString::encode_b64(salt)
        .map_err(|e| anyhow::anyhow!("Failed to encode salt: {}", e))?;

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?;

    let hash_bytes = password_hash
        .hash
        .ok_or_else(|| anyhow::anyhow!("Failed to get hash bytes"))?;

    let hash_slice = hash_bytes.as_bytes();

    // Take first 32 bytes for AES-256 key
    let mut key = [0u8; 32];
    key.copy_from_slice(&hash_slice[..32]);

    Ok(key)
}

/// Generates a random salt for key derivation
pub fn generate_salt() -> [u8; 16] {
    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);
    salt
}

/// Generates a random nonce for AES-GCM encryption
pub fn generate_nonce() -> [u8; 12] {
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

/// Encrypts data using AES-256-GCM
pub fn encrypt(data: &[u8], key: &[u8; 32], nonce: &[u8; 12]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from_slice(nonce);

    cipher
        .encrypt(nonce, data)
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))
}

/// Decrypts data using AES-256-GCM
pub fn decrypt(encrypted_data: &[u8], key: &[u8; 32], nonce: &[u8; 12]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from_slice(nonce);

    cipher
        .decrypt(nonce, encrypted_data)
        .map_err(|_| anyhow::anyhow!("Decryption failed: Invalid password or corrupted data"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_derivation() {
        let password = "test_password_123";
        let salt = [0u8; 16];

        let key1 = derive_key(password, &salt).unwrap();
        let key2 = derive_key(password, &salt).unwrap();

        // Same password and salt should produce same key
        assert_eq!(key1, key2);
        assert_eq!(key1.len(), 32);
    }

    #[test]
    fn test_different_salts_produce_different_keys() {
        let password = "test_password_123";
        let salt1 = [0u8; 16];
        let salt2 = [1u8; 16];

        let key1 = derive_key(password, &salt1).unwrap();
        let key2 = derive_key(password, &salt2).unwrap();

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_encryption_decryption() {
        let data = b"Hello, World!";
        let key = [0u8; 32];
        let nonce = [0u8; 12];

        let encrypted = encrypt(data, &key, &nonce).unwrap();
        assert_ne!(encrypted.as_slice(), data);

        let decrypted = decrypt(&encrypted, &key, &nonce).unwrap();
        assert_eq!(decrypted.as_slice(), data);
    }

    #[test]
    fn test_decryption_with_wrong_key_fails() {
        let data = b"Secret data";
        let key1 = [0u8; 32];
        let key2 = [1u8; 32];
        let nonce = [0u8; 12];

        let encrypted = encrypt(data, &key1, &nonce).unwrap();
        let result = decrypt(&encrypted, &key2, &nonce);

        assert!(result.is_err());
    }

    #[test]
    fn test_generate_salt() {
        let salt1 = generate_salt();
        let salt2 = generate_salt();

        assert_eq!(salt1.len(), 16);
        assert_ne!(salt1, salt2); // Should be random
    }

    #[test]
    fn test_generate_nonce() {
        let nonce1 = generate_nonce();
        let nonce2 = generate_nonce();

        assert_eq!(nonce1.len(), 12);
        assert_ne!(nonce1, nonce2); // Should be random
    }
}
