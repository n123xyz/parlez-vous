use tauri::{AppHandle, command, Runtime};

use crate::models::*;
use crate::Result;
use crate::LitertExt;

#[command]
pub(crate) async fn init_model<R: Runtime>(
    app: AppHandle<R>,
    payload: InitModelRequest,
) -> Result<InitModelResponse> {
    app.litert().init_model(payload)
}

#[command]
pub(crate) async fn check_model_exists<R: Runtime>(
    app: AppHandle<R>,
    payload: CheckModelRequest,
) -> Result<CheckModelResponse> {
    app.litert().check_model_exists(payload)
}

#[command]
pub(crate) async fn download_model<R: Runtime>(
    app: AppHandle<R>,
    payload: DownloadModelRequest,
) -> Result<DownloadModelResponse> {
    app.litert().download_model(payload)
}

#[command]
pub(crate) async fn purge_model<R: Runtime>(
    app: AppHandle<R>,
    payload: PurgeModelRequest,
) -> Result<()> {
    app.litert().purge_model(payload)
}

#[command]
pub(crate) async fn generate_chat<R: Runtime>(
    app: AppHandle<R>,
    payload: GenerateChatRequest,
) -> Result<GenerateChatResponse> {
    app.litert().generate_chat(payload)
}

#[command]
pub(crate) async fn close_model<R: Runtime>(
    app: AppHandle<R>,
    payload: CloseModelRequest,
) -> Result<CloseModelResponse> {
    app.litert().close_model(payload)
}

#[command]
pub(crate) async fn pick_gallery_image<R: Runtime>(
    app: AppHandle<R>,
) -> Result<PickGalleryImageResponse> {
    app.litert().pick_gallery_image()
}
