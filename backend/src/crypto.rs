use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::path::Path;
use std::os::unix::fs::OpenOptionsExt;
use std::io::Write;

use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::Aead;
use rand::rand_core::UnwrapErr;
use rand::rngs::SysRng;
use rand::Rng;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;

use crate::error::AppError;

/// Retrieves the encryption key from `ENCRYPTION_KEY` env var,
/// or reads/generates a key at `data/keys/encryption_key.txt`.
pub fn get_or_create_key() -> Result<Vec<u8>, AppError> {
    if let Ok(env_key) = env::var("ENCRYPTION_KEY") {
        let trimmed = env_key.trim();
        let key_bytes = STANDARD.decode(trimmed)
            .map_err(|_| AppError::internal("Invalid Base64 in ENCRYPTION_KEY environment variable"))?;
        
        if key_bytes.len() != 32 {
            return Err(AppError::internal("ENCRYPTION_KEY must be a 32-byte key (Base64 encoded)"));
        }

        return Ok(key_bytes);
    }

    let key_dir = Path::new("data/keys");
    let key_path = key_dir.join("encryption_key.txt");

    if key_path.exists() {
        let content = fs::read_to_string(&key_path)
            .map_err(|_| AppError::internal("Failed to read encryption key file"))?;
        
        let key_bytes = STANDARD.decode(content.trim())
            .map_err(|_| AppError::internal("Invalid Base64 content in encryption key file"))?;

        if key_bytes.len() != 32 {
            return Err(AppError::internal("Key file does not contain a valid 32-byte key"));
        }

        Ok(key_bytes)
    } else {
        tracing::info!("Encryption key not found. Generating...");
        fs::create_dir_all(key_dir)
            .map_err(|_| AppError::internal("Failed to create keys directory"))?;

        let mut key_bytes = [0u8; 32];
        let mut sys_rng = UnwrapErr(SysRng);
        sys_rng.fill_bytes(&mut key_bytes);

        let encoded_key = STANDARD.encode(key_bytes);

        let mut options = OpenOptions::new();
        options.write(true).create(true).truncate(true).mode(0o600);
        options.open(key_path).map_err(|_| AppError::internal("Failed to open encryption key file"))?
            .write_all(encoded_key.as_bytes()).map_err(|_| AppError::internal("Failed to write encryption key file"))?;

        Ok(key_bytes.to_vec())
    }
}

pub fn encrypt_data(plaintext: &str, key: &[u8]) -> Result<String, AppError> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| AppError::internal("Invalid key length"))?;
    
    let mut nonce_bytes = [0u8; 12];
    rand::fill(&mut nonce_bytes);
    let nonce = Nonce::from(nonce_bytes);

    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|_| AppError::internal("Encryption failed"))?; 

    let mut combined_data = nonce_bytes.to_vec();
    combined_data.extend_from_slice(&ciphertext);

    Ok(STANDARD.encode(combined_data))
}

pub fn decrypt_data(encrypted_base64: &str, key: &[u8]) -> Result<String, AppError> {
    let combined_data = STANDARD.decode(encrypted_base64)
        .map_err(|_| AppError::internal("Invalid base64 payload"))?;
    
    if combined_data.len() < 12 {
        return Err(AppError::internal("Invalid encrypted payload length"));
    }

    let (nonce_bytes, ciphertext) = combined_data.split_at(12);
    
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| AppError::internal("Invalid key length"))?;
    
    let nonce = Nonce::try_from(nonce_bytes)
        .map_err(|_| AppError::internal("Failed to parse nonce from bytes"))?;

    let plaintext_bytes = cipher
        .decrypt(&nonce, ciphertext)
        .map_err(|_| AppError::internal("Decryption failed"))?;

    String::from_utf8(plaintext_bytes).map_err(|_| AppError::internal("Invalid UTF-8"))
}