use tauri::{
  plugin::{Builder, TauriPlugin},
  Manager, Runtime,
};

pub use models::*;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

pub mod commands;
mod error;
mod models;
pub mod helper;
pub mod thermal;

pub use error::{Error, Result};

#[cfg(desktop)]
use desktop::Supertonic;
#[cfg(mobile)]
use mobile::Supertonic;

pub trait SupertonicExt<R: Runtime> {
  fn supertonic(&self) -> &Supertonic<R>;
}

impl<R: Runtime, T: Manager<R>> crate::SupertonicExt<R> for T {
  fn supertonic(&self) -> &Supertonic<R> {
    self.state::<Supertonic<R>>().inner()
  }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("supertonic")
    .invoke_handler(tauri::generate_handler![
        commands::generate_supertonic_tts,
        commands::is_supertonic_ready,
        commands::download_supertonic_models,
        commands::purge_supertonic_models
    ])
    .setup(|app, api| {
      #[cfg(mobile)]
      let supertonic = mobile::init(app, api)?;
      #[cfg(desktop)]
      let supertonic = desktop::init(app, api)?;
      app.manage(supertonic);
      app.manage(commands::SupertonicState::default());
      Ok(())
    })
    .build()
}
