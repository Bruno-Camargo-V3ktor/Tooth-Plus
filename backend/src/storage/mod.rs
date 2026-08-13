use async_trait::async_trait;
use aws_config::meta::region::RegionProviderChain;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::Client;
use aws_sdk_s3::config::Credentials;
use std::sync::Arc;

pub mod local;
pub mod s3;

#[async_trait]
pub trait StorageProvider: Send + Sync {
    async fn upload_file(
        &self,
        prefix: &str,
        extension: &str,
        base64_data: &str,
    ) -> Result<String, String>;
    async fn delete_file(&self, path: &str) -> Result<(), String>;
}

pub struct StorageConfig {
    pub provider_type: String,
    pub bucket_name: String,
    pub region: String,
    pub endpoint_url: String,
    pub access_key: String,
    pub secret_key: String,
    pub public_url: String,
}

pub async fn build_storage_provider(config: StorageConfig) -> Arc<dyn StorageProvider> {
    if config.provider_type == "local" {
        return Arc::new(local::LocalStorage {
            base_path: config.bucket_name,
            public_url: config.public_url,
        });
    }

    let credentials = Credentials::new(config.access_key, config.secret_key, None, None, "Static");

    let region_provider =
        RegionProviderChain::default_provider().or_else(Region::new(config.region));

    let aws_config = aws_config::defaults(BehaviorVersion::latest())
        .credentials_provider(credentials)
        .region(region_provider)
        .endpoint_url(config.endpoint_url)
        .load()
        .await;

    let s3_client = Client::new(&aws_config);

    Arc::new(s3::S3Storage {
        client: s3_client,
        bucket_name: config.bucket_name,
        public_url: config.public_url,
    })
}
