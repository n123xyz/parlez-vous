use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::models::*;

pub fn init<R: Runtime, C: DeserializeOwned>(
  app: &AppHandle<R>,
  _api: PluginApi<R, C>,
) -> crate::Result<Supertonic<R>> {
  Ok(Supertonic(app.clone()))
}

/// Access to the supertonic APIs.
pub struct Supertonic<R: Runtime>(AppHandle<R>);

impl<R: Runtime> Supertonic<R> {
  pub fn is_supertonic_ready(&self, _payload: IsSupertonicReadyRequest) -> crate::Result<IsSupertonicReadyResponse> {
    Ok(IsSupertonicReadyResponse { exists: false, is_downloading: false })
  }

  pub fn purge_supertonic_models(&self, payload: IsSupertonicReadyRequest) -> crate::Result<()> {
    let app_data_dir = std::path::PathBuf::from(payload.app_data_dir);
    let _ = std::fs::remove_dir_all(app_data_dir.join("onnx"));
    let _ = std::fs::remove_dir_all(app_data_dir.join("voice_styles"));
    Ok(())
  }

  pub fn download_supertonic_models(&self, _payload: DownloadSupertonicRequest) -> crate::Result<DownloadSupertonicResponse> {
    Ok(DownloadSupertonicResponse { success: false })
  }
}
