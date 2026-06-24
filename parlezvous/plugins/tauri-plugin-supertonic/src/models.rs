use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateTtsRequest {
  pub text: String,
  pub lang: String,
  pub speed: f32,
  pub steps: u32,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateTtsResponse {
  pub audio_bytes: Vec<u8>,
  pub sample_rate: u32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadSupertonicRequest {
  pub model_path: String,
  pub download_url: String,
  pub token: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadSupertonicResponse {
  pub success: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IsSupertonicReadyRequest {
  pub app_data_dir: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IsSupertonicReadyResponse {
  pub exists: bool,
  pub is_downloading: bool,
}
