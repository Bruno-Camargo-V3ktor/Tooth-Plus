use super::StorageProvider;
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose};
use std::fs;
use std::io::Write;
use std::path::Path;
use uuid::Uuid;

pub struct LocalStorage {
    pub base_path: String,
    pub public_url: String,
}

#[async_trait]
impl StorageProvider for LocalStorage {
    async fn upload_file(
        &self,
        prefix: &str,
        extension: &str,
        base64_data: &str,
    ) -> Result<String, String> {
        let decoded = general_purpose::STANDARD
            .decode(base64_data)
            .map_err(|_| "Invalid base64 payload".to_string())?;

        let safe_filename = format!("{}.{}", Uuid::new_v4(), extension.trim_start_matches('.'));
        let relative_path = format!("{}/{}", prefix.trim_matches('/'), safe_filename);
        let full_path = format!("{}/{}", self.base_path, relative_path);

        let path_obj = Path::new(&full_path);

        if let Some(parent) = path_obj.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        let mut file =
            fs::File::create(&full_path).map_err(|e| format!("Failed to create file: {}", e))?;
        file.write_all(&decoded)
            .map_err(|e| format!("Failed to write file: {}", e))?;

        Ok(format!(
            "{}/{}",
            self.public_url.trim_end_matches('/'),
            relative_path
        ))
    }

    async fn delete_file(&self, path: &str) -> Result<(), String> {
        if path.contains("..") {
            return Err("Invalid path".to_string());
        }

        let full_path = format!("{}/{}", self.base_path, path.trim_start_matches('/'));

        if Path::new(&full_path).exists() {
            fs::remove_file(full_path).map_err(|e| format!("Failed to delete file: {}", e))?;
        }

        Ok(())
    }
}
