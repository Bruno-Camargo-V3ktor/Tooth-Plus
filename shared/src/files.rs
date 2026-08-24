use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct FileUploadRequest {
    pub filename: String,
    pub mime_type: String,
    pub base64_content: String,
    #[serde(default)]
    pub clinic_id: Option<String>,
    #[serde(default)]
    pub module: Option<String>,
}

impl FileUploadRequest {
    pub fn new(filename: String, mime_type: String, base64_content: String) -> Self {
        Self {
            filename,
            mime_type,
            base64_content,
            clinic_id: None,
            module: None,
        }
    }
}
