use aes_gcm::aead::AeadMut;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use argon2::PasswordHasher;
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordVerifier},
};
use base64::{Engine as _, engine::general_purpose};
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub fn hash_password(password_plain: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    Ok(argon2
        .hash_password(password_plain.as_bytes(), &salt)?
        .to_string())
}

pub fn verify_password(hash: &str, password_plain: &str) -> bool {
    let parsed_hash = match PasswordHash::new(hash) {
        Ok(h) => h,
        Err(_) => return false,
    };

    Argon2::default()
        .verify_password(password_plain.as_bytes(), &parsed_hash)
        .is_ok()
}

pub fn generate_jwt(user_id: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET not set");

    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("Error calculating timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_owned(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

fn get_encryption_key() -> [u8; 32] {
    let key_str = env::var("ENCRYPTION_KEY")
        .unwrap_or_else(|_| "01234567890123456789012345678901".to_string());

    let mut key = [0u8; 32];
    let bytes = key_str.as_bytes();
    let len = std::cmp::min(bytes.len(), 32);
    key[..len].copy_from_slice(&bytes[..len]);
    key
}

pub fn encrypt_deterministic(plaintext: &str) -> Result<String, String> {
    let key = get_encryption_key();
    let mut cipher = Aes256Gcm::new(&key.into());

    let mut hasher = Sha256::new();
    hasher.update(plaintext.as_bytes());
    let hash = hasher.finalize();

    let nonce_bytes = &hash[..12];
    let nonce = Nonce::from_slice(nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut combined = nonce_bytes.to_vec();
    combined.extend(ciphertext);

    Ok(general_purpose::STANDARD.encode(combined))
}

pub fn decrypt_deterministic(encrypted_b64: &str) -> Result<String, String> {
    let key = get_encryption_key();
    let mut cipher = Aes256Gcm::new(&key.into());

    let combined = general_purpose::STANDARD
        .decode(encrypted_b64)
        .map_err(|_| "Invalid base64".to_string())?;

    if combined.len() < 12 {
        return Err("Invalid encrypted data length".to_string());
    }

    let nonce = Nonce::from_slice(&combined[..12]);
    let ciphertext = &combined[12..];

    let decrypted = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    String::from_utf8(decrypted).map_err(|_| "Invalid UTF-8 in decrypted data".to_string())
}
