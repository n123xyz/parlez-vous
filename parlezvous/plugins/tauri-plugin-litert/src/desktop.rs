use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::models::*;

pub fn init<R: Runtime, C: DeserializeOwned>(
  app: &AppHandle<R>,
  _api: PluginApi<R, C>,
) -> crate::Result<Litert<R>> {
  Ok(Litert(app.clone()))
}

/// Access to the litert APIs.
pub struct Litert<R: Runtime>(AppHandle<R>);

impl<R: Runtime> Litert<R> {
  pub fn init_model(&self, _payload: InitModelRequest) -> crate::Result<InitModelResponse> {
    Ok(InitModelResponse { success: false })
  }

  pub fn check_model_exists(&self, _payload: CheckModelRequest) -> crate::Result<CheckModelResponse> {
    Ok(CheckModelResponse { exists: false })
  }

  pub fn download_model(&self, _payload: DownloadModelRequest) -> crate::Result<DownloadModelResponse> {
    Ok(DownloadModelResponse { success: false })
  }

  pub fn purge_model(&self, _payload: PurgeModelRequest) -> crate::Result<()> {
    Ok(())
  }

  pub fn generate_chat(&self, _payload: GenerateChatRequest) -> crate::Result<GenerateChatResponse> {
    Ok(GenerateChatResponse { response: String::new() })
  }

  pub fn close_model(&self, _payload: CloseModelRequest) -> crate::Result<CloseModelResponse> {
    Ok(CloseModelResponse { success: false })
  }

  pub fn pick_gallery_image(&self) -> crate::Result<PickGalleryImageResponse> {
    Ok(PickGalleryImageResponse { path: String::new() })
  }
}
