use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileUploadRequest {
    pub filename: String,
    pub mime_type: String,
    pub base64_content: String,
}
