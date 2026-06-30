pub mod ai;
pub mod db;
pub mod model;
pub mod services;

use ai::ollama_adapter::OllamaAdapter;
use ai::LlmProvider;
use ai::{ChatMessage, ChatResponse, GradingVariables, JournalResponse, JournalVariables};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager, State};

use db::VocabId;
use services::journal::{process_journal_generation, process_journal_grading, DbVocabItem};
use services::srs::calculate_new_srs_state;

struct AppState {
    db: Arc<Mutex<Connection>>,
    ai: Arc<dyn LlmProvider + Send + Sync>,
    skill_level: std::sync::RwLock<String>,
    conjugation_task: tokio::sync::Mutex<Option<tokio::task::AbortHandle>>,
}

#[tauri::command]
async fn check_ollama_health(state: State<'_, AppState>) -> Result<bool, String> {
    println!("[IPC] check_ollama_health called");
    Ok(state.ai.check_health().await)
}

#[tauri::command]
async fn upload_and_ingest_textbook(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    file_path: String,
    model: String,
) -> Result<(), String> {
    // Copy the PDF into app_data_dir/textbooks/ so the webview can access it
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let textbooks_dir = app_dir.join("textbooks");
    std::fs::create_dir_all(&textbooks_dir).map_err(|e| e.to_string())?;

    let source = std::path::Path::new(&file_path);
    let file_name = source
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let dest = textbooks_dir.join(&file_name);

    if !dest.exists() {
        std::fs::copy(&source, &dest).map_err(|e| format!("Failed to copy PDF: {}", e))?;
        println!("[RAG] Copied PDF to: {:?}", dest);
    }

    crate::services::rag::ingest_pdf(state.db.clone(), state.ai.clone(), file_path, model).await
}

#[tauri::command]
async fn get_textbook_dir(app_handle: tauri::AppHandle) -> Result<String, String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let textbooks_dir = app_dir.join("textbooks");
    std::fs::create_dir_all(&textbooks_dir).map_err(|e| e.to_string())?;
    Ok(textbooks_dir.to_string_lossy().to_string())
}

#[tauri::command]
async fn get_user_skill_level(state: State<'_, AppState>) -> Result<String, String> {
    let skill = state
        .skill_level
        .read()
        .map_err(|_| "Failed to lock skill_level")?
        .clone();
    Ok(skill)
}

#[tauri::command]
async fn set_user_skill_level(state: State<'_, AppState>, level: String) -> Result<(), String> {
    crate::services::profile::update_skill_level(state.db.clone(), level.clone())?;
    let mut skill = state
        .skill_level
        .write()
        .map_err(|_| "Failed to lock skill_level")?;
    *skill = level;
    Ok(())
}

#[tauri::command]
async fn list_ollama_models(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    println!("[IPC] list_ollama_models called");
    state.ai.list_models().await
}

#[tauri::command]
async fn add_active_seconds(state: State<'_, AppState>, seconds: i32) -> Result<(), String> {
    crate::services::profile::add_active_seconds(state.db.clone(), seconds)
}

#[tauri::command]
async fn get_curriculum(
    state: State<'_, AppState>,
    language: String,
) -> Result<crate::services::curriculum::LanguageCurriculum, String> {
    crate::services::curriculum::get_curriculum(state.db.clone(), language)
}

#[tauri::command]
async fn add_time_xp(
    state: State<'_, AppState>,
    language: String,
    seconds: i32,
) -> Result<(), String> {
    crate::services::curriculum::add_time_xp(state.db.clone(), language, seconds)
}

#[tauri::command]
async fn set_active_theme(
    state: State<'_, AppState>,
    language: String,
    theme_id: String,
) -> Result<(), String> {
    crate::services::curriculum::set_active_theme(state.db.clone(), language, theme_id)
}

#[tauri::command]
async fn get_profile(
    state: State<'_, AppState>,
) -> Result<crate::services::profile::UserProfile, String> {
    crate::services::profile::get_profile(state.db.clone())
}

#[tauri::command]
async fn generate_journal(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    mood: String,
    weather: String,
    activity: String,
    model: String,
    language: String,
    active_theme: Option<String>,
) -> Result<JournalResponse, String> {
    println!(
        "[IPC] generate_journal called with mood: {}, weather: {}, activity: {}, language: {}",
        mood, weather, activity, language
    );
    let skill_level = state
        .skill_level
        .read()
        .map_err(|_| "Failed to lock skill_level")?
        .clone();
    let variables = JournalVariables {
        mood,
        weather,
        activity,
        model,
        language,
        skill_level,
        active_theme,
    };

    let (response, db_vocab_chips) =
        process_journal_generation(state.db.clone(), state.ai.clone(), variables).await?;

    app_handle
        .emit("vocabulary_extracted", &db_vocab_chips)
        .map_err(|e| e.to_string())?;

    Ok(response)
}

#[tauri::command]
async fn grade_journal(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    entry: String,
    model: String,
    language: String,
) -> Result<JournalResponse, String> {
    println!("[IPC] grade_journal called for language: {}", language);
    let skill_level = state
        .skill_level
        .read()
        .map_err(|_| "Failed to lock skill_level")?
        .clone();
    let variables = GradingVariables {
        entry,
        model,
        language,
        skill_level,
    };

    let (response, db_vocab_chips) =
        process_journal_grading(state.db.clone(), state.ai.clone(), variables).await?;

    app_handle
        .emit("vocabulary_extracted", &db_vocab_chips)
        .map_err(|e| e.to_string())?;

    Ok(response)
}

#[tauri::command]
async fn generate_tts_audio<R: tauri::Runtime>(
    app_handle: tauri::AppHandle<R>,
    text: String,
    language: String,
    voice: String,
    speed: f32,
    url: String,
) -> Result<Vec<u8>, String> {
    // Strip angle brackets from translated words, keeping valid TTS tags intact.
    let re = regex::Regex::new(r"<([^>]+)>").map_err(|e| e.to_string())?;
    let cleaned_text = re.replace_all(&text, |caps: &regex::Captures| {
        let inner = &caps[1];
        if inner == "laugh" || inner == "breath" || inner == "cough" || inner == "surprise" 
            || inner == "sad" || inner == "sigh" || inner.starts_with("lang:") || inner.starts_with("emotion:") {
            caps[0].to_string()
        } else {
            inner.to_string()
        }
    }).to_string();

    // 2. Strip parenthetical text (e.g., romanizations like "(annyeonghaseyo)" or translations like "(hello)")
    // so they are not spoken out loud by the TTS engine.
    let re_paren = regex::Regex::new(r"\s*\([^)]+\)").map_err(|e| e.to_string())?;
    let cleaned_text = re_paren.replace_all(&cleaned_text, "").to_string();

    use crate::ai::tts_adapter::TtsProvider;
    let provider: Box<dyn TtsProvider> = if url == "supertonic" || url.starts_with("supertonic://") {
        Box::new(crate::ai::tts_adapter::SupertonicTtsAdapter::new(app_handle.clone()))
    } else {
        Box::new(crate::ai::tts_adapter::OpenAiTtsAdapter::new(url))
    };

    provider.generate_tts(&cleaned_text, &language, &voice, speed).await
}

#[tauri::command]
async fn add_to_srs(state: State<'_, AppState>, vocab_id: VocabId) -> Result<(), String> {
    println!("[IPC] add_to_srs called with vocab_id: {:?}", vocab_id);
    let db = state.db.clone();

    tokio::task::spawn_blocking(move || {
        let conn = db.lock().map_err(|_| "DB lock failed")?;
        conn.execute(
            "INSERT OR IGNORE INTO srs_state (vocab_id, review_level, next_review_date, ease_factor, interval_days) VALUES (?1, 1, date('now', '+1 day'), 2.5, 1)",
            [vocab_id]
        ).map_err(|e| e.to_string())?;
        Ok::<(), String>(())
    }).await.map_err(|e| e.to_string())??;

    Ok(())
}

#[tauri::command]
async fn infer_character(
    state: State<'_, AppState>,
    pixels: Vec<u8>,
    vocab_id: VocabId,
    target_text: String,
) -> Result<String, String> {
    println!(
        "[IPC] infer_character called for target_text: {}",
        target_text
    );

    // Run ONNX inference via burn-generated model
    let (predicted, confidence) = tokio::task::spawn_blocking(move || model::infer(&pixels))
        .await
        .map_err(|e| e.to_string())??;

    println!(
        "[IPC] Predicted: {} (confidence: {:.2}%)",
        predicted,
        confidence * 100.0
    );

    if predicted == target_text {
        let db = state.db.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.lock().map_err(|_| "DB lock failed")?;

            // Fetch current state
            let (current_level, current_ease): (i32, f32) = conn
                .query_row(
                    "SELECT review_level, ease_factor FROM srs_state WHERE vocab_id = ?1",
                    [vocab_id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .unwrap_or((0, 2.5));

            // Use decoupled business logic for fuzzable SRS math
            let (new_level, new_ease) = calculate_new_srs_state(current_level, current_ease)?;

            conn.execute(
                "UPDATE srs_state SET review_level = ?1, ease_factor = ?2 WHERE vocab_id = ?3",
                rusqlite::params![new_level, new_ease, vocab_id],
            )
            .map_err(|e| e.to_string())?;

            Ok::<(), String>(())
        })
        .await
        .map_err(|e| e.to_string())??;
    }

    Ok(predicted)
}

#[tauri::command]
fn get_all_jamo() -> Vec<String> {
    model::ALL_JAMO.iter().map(|s| s.to_string()).collect()
}

#[tauri::command]
async fn get_all_vocabulary(state: State<'_, AppState>) -> Result<Vec<DbVocabItem>, String> {
    println!("[IPC] get_all_vocabulary called");
    crate::services::vocab::get_vocabulary(state.db.clone()).await
}

#[tauri::command]
async fn get_settings(
    state: State<'_, AppState>,
) -> Result<crate::services::settings::AppSettings, String> {
    crate::services::settings::get_settings(state.db.clone())
}

#[tauri::command]
async fn update_settings(
    state: State<'_, AppState>,
    settings: crate::services::settings::AppSettings,
) -> Result<(), String> {
    crate::services::settings::update_settings(state.db.clone(), settings)
}

#[tauri::command]
async fn chat_with_avatar(
    state: State<'_, AppState>,
    history: Vec<ChatMessage>,
    model: String,
    language: String,
    active_textbook: Option<String>,
    active_page: Option<i32>,
    active_theme: Option<String>,
    audio_base64: Option<String>,
    image_uri: Option<String>,
) -> Result<ChatResponse, String> {
    println!(
        "[IPC] chat_with_avatar called for model {} in {}",
        model, language
    );

    let context_str = if let Some(book) = active_textbook {
        let last_user_msg = history
            .iter()
            .rev()
            .find(|m| m.role == "user")
            .map(|m| m.content.clone())
            .unwrap_or_default();
        if !last_user_msg.is_empty() {
            let settings = crate::services::settings::get_settings(state.db.clone())?;
            let chunks = crate::services::rag::query_context(
                state.db.clone(),
                state.ai.clone(),
                book,
                last_user_msg,
                settings.embedding_model,
                active_page,
                active_theme.clone(),
            )
            .await?;
            chunks.join("\n\n")
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let skill_level = state
        .skill_level
        .read()
        .map_err(|_| "Failed to lock skill_level")?
        .clone();
    state
        .ai
        .generate_chat_response(
            history,
            model,
            language,
            skill_level,
            context_str,
            active_theme,
            audio_base64,
            image_uri,
        )
        .await
}

#[derive(serde::Serialize)]
struct ConjugationResponse {
    exercise: crate::ai::ConjugationExercise,
    history_id: i64,
}

#[tauri::command]
async fn generate_conjugation_exercise(
    state: State<'_, AppState>,
    language: String,
    model: String,
) -> Result<ConjugationResponse, String> {
    println!("[IPC] generate_conjugation_exercise called");

    let db = state.db.clone();
    let ai = state.ai.clone();
    
    let handle = tokio::spawn(async move {
        crate::services::conjugator::process_conjugation_generation(
            db,
            ai,
            language,
            model,
        )
        .await
    });

    {
        let mut task_guard = state.conjugation_task.lock().await;
        if let Some(old_abort) = task_guard.replace(handle.abort_handle()) {
            old_abort.abort();
        }
    }

    match handle.await {
        Ok(Ok((exercise, history_id))) => Ok(ConjugationResponse { exercise, history_id }),
        Ok(Err(e)) => Err(e),
        Err(e) if e.is_cancelled() => Err("Cancelled".to_string()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn cancel_conjugation_generation(state: State<'_, AppState>) -> Result<(), String> {
    println!("[IPC] cancel_conjugation_generation called");
    let mut task_guard = state.conjugation_task.lock().await;
    if let Some(abort_handle) = task_guard.take() {
        abort_handle.abort();
    }
    Ok(())
}

#[tauri::command]
async fn record_conjugation_result(
    state: State<'_, AppState>,
    history_id: i64,
    correct: bool,
) -> Result<(), String> {
    println!(
        "[IPC] record_conjugation_result called (id={}, correct={})",
        history_id, correct
    );
    crate::services::conjugator::record_conjugation_result(state.db.clone(), history_id, correct)
        .await
}

#[tauri::command]
async fn get_journal_entries(
    state: State<'_, AppState>,
) -> Result<Vec<crate::services::calendar::JournalEntryDTO>, String> {
    crate::services::calendar::get_journal_entries(state.db.clone()).await
}

#[tauri::command]
async fn list_textbooks(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    crate::services::rag::list_textbooks(state.db.clone()).await
}

#[tauri::command]
async fn get_all_tense_stats(
    state: State<'_, AppState>,
    language: String,
) -> Result<Vec<crate::ai::TenseStat>, String> {
    crate::services::conjugator::get_all_tense_stats(state.db.clone(), language)
}

#[tauri::command]
async fn generate_supertonic_tts(
    app: tauri::AppHandle,
    state: tauri::State<'_, tauri_plugin_supertonic::commands::SupertonicState>,
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

#[tauri::command]
async fn check_tokenizer_exists(app_handle: tauri::AppHandle) -> Result<bool, String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    
    let tokenizer_path = app_dir.join("tokenizer.json");
    let config_path = app_dir.join("tokenizer_config.json");
    
    Ok(tokenizer_path.exists() && config_path.exists())
}

#[tauri::command]
async fn download_tokenizer(app_handle: tauri::AppHandle) -> Result<(), String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    
    std::fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;

    let tokenizer_path = app_dir.join("tokenizer.json");
    let config_path = app_dir.join("tokenizer_config.json");

    let client = reqwest::Client::new();
    
    let res = client.get("https://huggingface.co/google/gemma-4-E2B/resolve/main/tokenizer.json")
        .send().await.map_err(|e| format!("Failed to fetch tokenizer.json: {}", e))?;
    let bytes = res.bytes().await.map_err(|e| e.to_string())?;
    std::fs::write(&tokenizer_path, bytes).map_err(|e| e.to_string())?;

    let res = client.get("https://huggingface.co/google/gemma-4-E2B/resolve/main/tokenizer_config.json")
        .send().await.map_err(|e| format!("Failed to fetch tokenizer_config.json: {}", e))?;
    let bytes = res.bytes().await.map_err(|e| e.to_string())?;
    std::fs::write(&config_path, bytes).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn delete_tokenizer(app_handle: tauri::AppHandle) -> Result<(), String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    
    let tokenizer_path = app_dir.join("tokenizer.json");
    let config_path = app_dir.join("tokenizer_config.json");
    
    let _ = std::fs::remove_file(tokenizer_path);
    let _ = std::fs::remove_file(config_path);
    
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_litert::init())
        .plugin(tauri_plugin_supertonic::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let conn = db::init_db(app.handle()).map_err(|e| {
                eprintln!("❌ CRITICAL: Failed to initialize database: {}", e);
                e
            })?;
            let db_arc = Arc::new(Mutex::new(conn));
            let initial_skill_level = crate::services::profile::get_skill_level(db_arc.clone())
                .unwrap_or_else(|_| "Beginner".to_string());
                
            #[cfg(target_os = "android")]
            let ai_adapter: Arc<dyn LlmProvider + Send + Sync> = {
                let settings = crate::services::settings::get_settings(db_arc.clone())
                    .unwrap_or_else(|_| crate::services::settings::AppSettings {
                        target_language: "French".to_string(),
                        tts_server_url: "".to_string(),
                        asr_server_url: "".to_string(),
                        ollama_server_url: "".to_string(),
                        embedding_model: "".to_string(),
                        active_model: "".to_string(),
                        huggingface_token: None,
                        litert_accelerator: "Auto".to_string(),
                        litert_max_tokens: 5000,
                    });
                match crate::ai::litert_adapter::LiteRtAdapter::new(app.handle().clone(), "gemma-4-E2B-it.litertlm".to_string(), settings.litert_accelerator, settings.litert_max_tokens) {
                    Ok(adapter) => Arc::new(adapter),
                    Err(e) => {
                        println!("[LiteRT] Failed to load, falling back to Ollama: {}", e);
                        Arc::new(OllamaAdapter::new(db_arc.clone()))
                    }
                }
            };

            #[cfg(not(target_os = "android"))]
            let ai_adapter: Arc<dyn LlmProvider + Send + Sync> = Arc::new(OllamaAdapter::new(db_arc.clone()));

            app.manage(AppState {
                db: db_arc,
                ai: ai_adapter,
                skill_level: std::sync::RwLock::new(initial_skill_level),
                conjugation_task: tokio::sync::Mutex::new(None),
            });

            #[cfg(all(target_os = "linux", not(target_os = "android")))]
            {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.with_webview(|webview| {
                        use webkit2gtk::PermissionRequestExt;
                        use webkit2gtk::WebViewExt;
                        webview
                            .inner()
                            .connect_permission_request(move |_, request| {
                                request.allow();
                                true
                            });
                    });
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            check_ollama_health,
            list_ollama_models,
            add_active_seconds,
            get_curriculum,
            add_time_xp,
            set_active_theme,
            get_profile,
            generate_journal,
            chat_with_avatar,
            add_to_srs,
            get_all_vocabulary,
            get_settings,
            update_settings,
            infer_character,
            get_all_jamo,
            generate_conjugation_exercise,
            record_conjugation_result,
            get_journal_entries,
            upload_and_ingest_textbook,
            get_user_skill_level,
            set_user_skill_level,
            list_textbooks,
            get_textbook_dir,
            grade_journal,
            add_active_seconds,
            get_profile,
            generate_supertonic_tts,
            generate_tts_audio,
            check_tokenizer_exists,
            download_tokenizer,
            delete_tokenizer,
            get_all_tense_stats,
            cancel_conjugation_generation
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
