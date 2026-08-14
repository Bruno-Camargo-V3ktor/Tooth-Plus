use base64::{Engine as _, engine::general_purpose};
use rand::RngExt;
use sha2::{Digest, Sha256};

pub fn generate_otp_code() -> String {
    let mut rng = rand::rng();
    let code: u32 = rng.random_range(100_000..=999_999);
    code.to_string()
}

pub fn hash_otp(code: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(code.as_bytes());
    let result = hasher.finalize();
    general_purpose::STANDARD.encode(result)
}

pub fn verify_otp(input_code: &str, saved_hash: &str) -> bool {
    let input_hash = hash_otp(input_code);
    input_hash == saved_hash
}
