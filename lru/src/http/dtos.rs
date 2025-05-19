use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadResponse {
    pub key: String,
    pub size: usize,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadRequest {
    pub key: String,
}