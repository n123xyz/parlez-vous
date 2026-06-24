use tauri::{AppHandle, command, Runtime, State, Manager};
use std::sync::Mutex;
use crate::models::*;
use crate::Result;
use crate::SupertonicExt;
use crate::helper::{load_text_to_speech, load_voice_style, TextToSpeech};

pub struct SupertonicState {
    pub tts: std::sync::Arc<Mutex<Option<TextToSpeech>>>,
}

impl Default for SupertonicState {
    fn default() -> Self {
        Self { tts: std::sync::Arc::new(Mutex::new(None)) }
    }
}

#[command]
pub(crate) async fn is_supertonic_ready<R: Runtime>(
    app: AppHandle<R>,
) -> Result<IsSupertonicReadyResponse> {
    let app_data_dir = app.path().app_local_data_dir().map_err(|e| crate::Error::Io(std::io::Error::new(std::io::ErrorKind::NotFound, e.to_string())))?;
    let req = IsSupertonicReadyRequest {
        app_data_dir: app_data_dir.to_string_lossy().to_string(),
    };
    app.supertonic().is_supertonic_ready(req)
}

#[command]
pub(crate) async fn download_supertonic_models<R: Runtime>(
    app: AppHandle<R>,
    mut payload: DownloadSupertonicRequest,
) -> Result<DownloadSupertonicResponse> {
    let app_data_dir = app.path().app_local_data_dir().map_err(|e| crate::Error::Io(std::io::Error::new(std::io::ErrorKind::NotFound, e.to_string())))?;
    payload.model_path = app_data_dir.to_string_lossy().to_string();
    app.supertonic().download_supertonic_models(payload)
}

#[command]
pub async fn generate_supertonic_tts<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, SupertonicState>,
    payload: GenerateTtsRequest,
) -> Result<GenerateTtsResponse> {
    let tts_arc = state.tts.clone();
    let app_data_dir = app.path().app_local_data_dir().map_err(|e| crate::Error::Io(std::io::Error::new(std::io::ErrorKind::NotFound, e.to_string())))?;
    
    tokio::task::spawn_blocking(move || {
        let mut tts_guard = tts_arc.lock().unwrap();

        let model_dir = app_data_dir.join("onnx");
        let style_path = app_data_dir.join("voice_styles").join("F1.json");

        if tts_guard.is_none() {
            let _ = ort::init().with_name("supertonic-tts").commit();

            let model_dir_str = model_dir.to_string_lossy().to_string();
            
            let tts = load_text_to_speech(&model_dir_str, true, true, 4, 1)
                .map_err(|e| crate::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
                
            *tts_guard = Some(tts);
        }

        let tts = tts_guard.as_mut().unwrap();
        let style = load_voice_style(&[style_path.to_string_lossy().to_string()], false)
            .map_err(|e| crate::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        let mut final_lang = crate::helper::normalize_lang_code(&payload.lang);
        if let Some(info) = whatlang::detect(&payload.text) {
            let code = info.lang().code();
            if crate::helper::is_valid_lang(code) {
                final_lang = code.to_string();
            }
        }

        let (wav_data, _dur) = tts.call(
            &payload.text,
            &final_lang,
            &style,
            payload.steps as usize,
            payload.speed,
            0.1,
            |_curr, _total, _chunk| true,
        ).map_err(|e| crate::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        let mut pcm_data = Vec::with_capacity(wav_data.len() * 2);
        for &sample in &wav_data {
            let clamped = sample.max(-1.0).min(1.0);
            let val = (clamped * 32767.0) as i16;
            pcm_data.extend_from_slice(&val.to_le_bytes());
        }

        Ok(GenerateTtsResponse {
            audio_bytes: pcm_data,
            sample_rate: tts.sample_rate as u32,
        })
    }).await.unwrap_or_else(|e| Err(crate::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))))
}

#[command]
pub async fn purge_supertonic_models<R: Runtime>(
    app: AppHandle<R>,
) -> Result<()> {
    let app_data_dir = app.path().app_local_data_dir().map_err(|e| crate::Error::Io(std::io::Error::new(std::io::ErrorKind::NotFound, e.to_string())))?;
    let req = IsSupertonicReadyRequest {
        app_data_dir: app_data_dir.to_string_lossy().to_string(),
    };
    app.supertonic().purge_supertonic_models(req)
}
