use tauri::{
  plugin::{Builder, TauriPlugin},
  Manager, Runtime,
};

pub use models::*;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod commands;
mod error;
mod models;

pub use error::{Error, Result};

#[cfg(desktop)]
use desktop::Litert;
#[cfg(mobile)]
use mobile::Litert;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the litert APIs.
pub trait LitertExt<R: Runtime> {
  fn litert(&self) -> &Litert<R>;
}

impl<R: Runtime, T: Manager<R>> crate::LitertExt<R> for T {
  fn litert(&self) -> &Litert<R> {
    self.state::<Litert<R>>().inner()
  }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("litert")
    .invoke_handler(tauri::generate_handler![
        commands::init_model,
        commands::check_model_exists,
        commands::download_model,
        commands::purge_model,
        commands::generate_chat,
        commands::close_model,
        commands::pick_gallery_image
    ])
    .setup(|app, api| {
      #[cfg(mobile)]
      let litert = mobile::init(app, api)?;
      #[cfg(desktop)]
      let litert = desktop::init(app, api)?;
      app.manage(litert);
      Ok(())
    })
    .build()
}
