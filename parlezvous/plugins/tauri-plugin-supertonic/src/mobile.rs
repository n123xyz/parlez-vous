use serde::de::DeserializeOwned;
use tauri::{
  plugin::{PluginApi, PluginHandle},
  AppHandle, Runtime,
};

use crate::models::*;

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_supertonic);

// initializes the Kotlin or Swift plugin classes
pub fn init<R: Runtime, C: DeserializeOwned>(
  _app: &AppHandle<R>,
  api: PluginApi<R, C>,
) -> crate::Result<Supertonic<R>> {
  #[cfg(target_os = "android")]
  let handle = api.register_android_plugin("com.plugin.supertonic", "SupertonicPlugin")?;
  #[cfg(target_os = "ios")]
  let handle = api.register_ios_plugin(init_plugin_supertonic)?;
  Ok(Supertonic(handle))
}

/// Access to the Supertonic APIs.
pub struct Supertonic<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> Supertonic<R> {
  pub fn is_supertonic_ready(&self, payload: IsSupertonicReadyRequest) -> crate::Result<IsSupertonicReadyResponse> {
    self
      .0
      .run_mobile_plugin("isSupertonicReady", payload)
      .map_err(Into::into)
  }

  pub fn purge_supertonic_models(&self, payload: IsSupertonicReadyRequest) -> crate::Result<()> {
    self
      .0
      .run_mobile_plugin("purgeSupertonicModels", payload)
      .map_err(Into::into)
  }

  pub fn download_supertonic_models(&self, payload: DownloadSupertonicRequest) -> crate::Result<DownloadSupertonicResponse> {
    self.0
      .run_mobile_plugin("downloadSupertonicModels", payload)
      .map_err(Into::into)
  }
}
