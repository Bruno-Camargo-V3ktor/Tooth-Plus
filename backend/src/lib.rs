pub mod db;
pub mod documents_pdf;
pub mod email;
pub mod error;
pub mod evolution;
pub mod migrations;
pub mod routes;
pub mod security;
pub mod storage;

use std::env;

pub fn resolve_uploads_dir() -> String {
    if let Ok(bucket) = env::var("STORAGE_BUCKET") {
        if std::path::Path::new(&bucket).is_dir() {
            return bucket;
        }
        let alt = format!("../{}", bucket.trim_start_matches("./"));
        if std::path::Path::new(&alt).is_dir() {
            return alt;
        }
    }
    if std::path::Path::new("uploads").is_dir() {
        return "uploads".to_string();
    }
    if std::path::Path::new("../uploads").is_dir() {
        return "../uploads".to_string();
    }
    "uploads".to_string()
}
