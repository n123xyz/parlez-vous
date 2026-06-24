use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub async fn generate_supertonic_tts(
    app: AppHandle,
    state: State<'_, tauri_plugin_supertonic::commands::SupertonicState>,
    text: String,
    lang: String,
    speed: f32,
    steps: u32,
) -> Result<tauri_plugin_supertonic::GenerateTtsResponse, String> {
    tauri_plugin_supertonic::commands::generate_supertonic_tts(
        app,
        state,
        tauri_plugin_supertonic::GenerateTtsRequest {
            text,
            lang,
            speed,
            steps,
        },
    )
    .await
    .map_err(|e| e.to_string())
}
