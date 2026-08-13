use super::StorageProvider;
use async_trait::async_trait;
use aws_sdk_s3::{Client, primitives::ByteStream};
use base64::{Engine as _, engine::general_purpose};
use uuid::Uuid;

pub struct S3Storage {
    pub client: Client,
    pub bucket_name: String,
    pub public_url: String,
}

#[async_trait]
impl StorageProvider for S3Storage {
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

        let stream = ByteStream::from(decoded);

        self.client
            .put_object()
            .bucket(&self.bucket_name)
            .key(&relative_path)
            .body(stream)
            .send()
            .await
            .map_err(|e| format!("S3 upload failed: {}", e))?;

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

        self.client
            .delete_object()
            .bucket(&self.bucket_name)
            .key(path.trim_start_matches('/'))
            .send()
            .await
            .map_err(|e| format!("S3 delete failed: {}", e))?;

        Ok(())
    }
}
