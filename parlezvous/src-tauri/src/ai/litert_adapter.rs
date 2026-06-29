use crate::ai::{
    ChatMessage, ChatResponse, ConjugationExercise, GradingVariables, JournalResponse,
    JournalVariables, LlmProvider, TenseStat,
};
use async_trait::async_trait;
use tauri::{AppHandle, Manager};
use tauri_plugin_litert::LitertExt;
use tokenizers::Tokenizer;

pub struct LiteRtAdapter {
    app_handle: AppHandle,
    max_tokens: u32,
    model_path: String,
    accelerator: String,
    last_history: std::sync::Mutex<Option<Vec<crate::ai::ChatMessage>>>,
    is_initialized: std::sync::Arc<tokio::sync::Mutex<bool>>,
    tokenizer: std::sync::Mutex<Option<Tokenizer>>,
}

impl LiteRtAdapter {
    pub fn new(app_handle: AppHandle, model_path: String, accelerator: String, max_tokens: u32) -> Result<Self, String> {
        let is_initialized = std::sync::Arc::new(tokio::sync::Mutex::new(false));
        let instance = Self {
            app_handle: app_handle.clone(),
            max_tokens,
            model_path: model_path.clone(),
            accelerator: accelerator.clone(),
            last_history: std::sync::Mutex::new(None),
            is_initialized: is_initialized.clone(),
            tokenizer: std::sync::Mutex::new(None),
        };

        // PRE-LOAD: Fire off the initialization immediately in the background 
        // to bypass the ANR while still having the model ready for the first chat!
        let payload = tauri_plugin_litert::InitModelRequest {
            model_path,
            accelerator,
            max_tokens,
        };

        let is_initialized_clone = is_initialized.clone();
        tauri::async_runtime::spawn(async move {
            // Lock the mutex so that if the user chats before this finishes, 
            // ensure_initialized() will wait for this lock instead of spawning a duplicate!
            let mut init_lock = is_initialized_clone.lock().await;
            if *init_lock {
                return;
            }

            let success = tokio::task::spawn_blocking(move || {
                match app_handle.litert().init_model(payload) {
                    Ok(response) => response.success,
                    Err(e) => {
                        println!("[LiteRT Preload] Init Error: {}", e);
                        false
                    }
                }
            }).await.unwrap_or(false);

            if success {
                *init_lock = true;
            }
        });

        Ok(instance)
    }

    fn ensure_tokenizer_loaded(&self) {
        let mut t = self.tokenizer.lock().unwrap();
        if t.is_none() {
            if let Ok(app_dir) = self.app_handle.path().app_data_dir() {
                let tokenizer_path = app_dir.join("tokenizer.json");
                if tokenizer_path.exists() {
                    if let Ok(tokenizer) = Tokenizer::from_file(tokenizer_path) {
                        *t = Some(tokenizer);
                    }
                }
            }
        }
    }

    async fn ensure_initialized(&self) -> Result<(), String> {
        let mut initialized = self.is_initialized.lock().await;
        if *initialized {
            return Ok(());
        }

        let payload = tauri_plugin_litert::InitModelRequest {
            model_path: self.model_path.clone(),
            accelerator: self.accelerator.clone(),
            max_tokens: self.max_tokens,
        };

        // Call init_model in spawn_blocking to prevent run_mobile_plugin from blocking the tokio worker thread
        let app_handle = self.app_handle.clone();
        let success_or_err = tokio::task::spawn_blocking(move || {
            match app_handle.litert().init_model(payload) {
                Ok(response) => {
                    if response.success {
                        Ok(())
                    } else {
                        Err("Plugin returned success=false".to_string())
                    }
                },
                Err(e) => {
                    println!("[LiteRT] Init Error: {}", e);
                    Err(e.to_string())
                }
            }
        })
        .await
        .map_err(|e| format!("Task failed: {}", e))?;

        match success_or_err {
            Ok(_) => {
                *initialized = true;
                Ok(())
            }
            Err(e) => {
                Err(format!("LiteRT Init Error: {}", e))
            }
        }
    }
    
    fn unsupported(&self, feature: &str) -> String {
        format!("{} is currently unsupported on the LiteRT local adapter.", feature)
    }

    fn execute_json_generation<T: serde::de::DeserializeOwned>(
        &self,
        prompt: String,
        error_prefix: &str,
    ) -> Result<T, String> {
        let max_attempts = 3;
        let mut attempts = 0;

        // Wrap the prompt in Gemma tags
        let gemma_prompt = format!("<start_of_turn>user\n{}<end_of_turn>\n<start_of_turn>model\n", prompt);

        while attempts < max_attempts {
            let payload = tauri_plugin_litert::GenerateChatRequest { prompt: gemma_prompt.clone(), reset: true, audio_base64: None, image_uri: None };
            
            let response = match self.app_handle.litert().generate_chat(payload) {
                Ok(res) => res.response,
                Err(e) => {
                    attempts += 1;
                    println!("{} Native Error on attempt {}: {}", error_prefix, attempts, e);
                    if attempts >= max_attempts {
                        return Err(format!("Android inference error: {}", e));
                    }
                    continue;
                }
            };
            
            let content = response;
            let json_content = if let (Some(start), Some(end)) = (content.find('{'), content.rfind('}')) {
                &content[start..=end]
            } else {
                &content
            };
            
            match serde_json::from_str::<T>(json_content) {
                Ok(parsed) => return Ok(parsed),
                Err(e) => {
                    attempts += 1;
                    println!(
                        "{} JSON Parse Error on attempt {}: {} \nRaw output: {}",
                        error_prefix, attempts, e, content
                    );
                    if attempts >= max_attempts {
                        return Err(format!("The AI model failed to produce valid JSON after {} attempts.", max_attempts));
                    }
                }
            }
        }
        Err(format!("The AI model failed to produce valid JSON after {} attempts.", max_attempts))
    }
}

#[async_trait]
impl LlmProvider for LiteRtAdapter {
    async fn check_health(&self) -> bool {
        true
    }

    async fn list_models(&self) -> Result<Vec<String>, String> {
        Ok(vec!["gemma-4-E2B-it.litertlm".to_string()])
    }

    async fn generate_guided_journal(
        &self,
        variables: JournalVariables,
    ) -> Result<JournalResponse, String> {
        self.ensure_initialized().await?;
        let prompt = super::build_journal_prompt(
            &variables.language,
            &variables.skill_level,
            &variables.mood,
            &variables.weather,
            &variables.activity,
            &variables.active_theme,
        );

        self.execute_json_generation(prompt, "[Journal]")
    }

    async fn grade_custom_journal(
        &self,
        variables: GradingVariables,
    ) -> Result<JournalResponse, String> {
        self.ensure_initialized().await?;
        let prompt = format!(
            "You are a strict language teacher. Evaluate the following custom journal entry written in {lang} by a {skill_level} learner:\n\n\
            ENTRY: \"{custom}\"\n\n\
            1. Place your corrected, idealized version of the user's text into the `generated_target_text` field.\n\
            2. Provide a native English translation of your CORRECTED text in the `native_translation` field.\n\
            3. Place your grammatical explanations and constructive feedback into the `feedback` field.\n\
            4. Extract 3-5 useful dictionary-form vocabulary words from your CORRECTED text.\n\
            Output STRICTLY valid JSON with the following structure, nothing else: \
            {{\"generated_target_text\": \"...\", \"native_translation\": \"...\", \"vocabulary\": [{{\"target_text\": \"...\", \"native_text\": \"...\"}}], \"feedback\": \"...\"}}",
            lang=variables.language, skill_level=variables.skill_level, custom=variables.entry
        );

        self.execute_json_generation(prompt, "Grade")
    }

    async fn generate_chat_response(
        &self,
        history: Vec<ChatMessage>,
        _model: String,
        language: String,
        skill_level: String,
        context: String,
        active_theme: Option<String>,
        audio_base64: Option<String>,
        image_uri: Option<String>,
    ) -> Result<ChatResponse, String> {
        self.ensure_initialized().await?;
        let system_prompt_str = super::build_chat_system_prompt(
            &language,
            &skill_level,
            &context,
            &active_theme,
            false,
            true, // use_expression_tags
            image_uri.is_some(),
        );

        self.ensure_tokenizer_loaded();

        let mut context_summary: Option<String> = None;
        let mut truncated_history = {
            let t = self.tokenizer.lock().unwrap();
            if let Some(tokenizer) = &*t {
                super::truncate_history_exact(&system_prompt_str, &history, self.max_tokens as usize, tokenizer)
            } else {
                super::truncate_history_by_tokens(&system_prompt_str, &history, self.max_tokens as usize)
            }
        };

        let compression_needed = truncated_history.len() < history.len();

        let mut is_continuation = false;
        
        if compression_needed {
            println!("[LiteRT] Context maxed or loops predicted. Compressing history...");
            let mut summary_prompt = format!(
                "You are an internal system compressing memory for a language tutor avatar. Summarize the following conversation in 1-2 sentences in English. Focus strictly on the current topic of conversation and note any specific {} vocabulary the user was struggling with or practicing:\n\n",
                language
            );
            
            // Summarize everything except the very last user message
            let history_to_summarize = &history[0..history.len().saturating_sub(1)];
            for msg in history_to_summarize {
                summary_prompt.push_str(&format!("{}: {}\n", msg.role, msg.content));
            }
            summary_prompt.push_str("\n\nSummary:");
            
            let gemma_summary_prompt = format!("<start_of_turn>user\n{}<end_of_turn>\n<start_of_turn>model\n", summary_prompt);
            let payload = tauri_plugin_litert::GenerateChatRequest { prompt: gemma_summary_prompt, reset: true, audio_base64: None, image_uri: None };
            
            let summary_text = match self.app_handle.litert().generate_chat(payload) {
                Ok(res) => res.response.trim().to_string(),
                Err(_) => format!("(Context compressed due to length)")
            };
            
            context_summary = Some(summary_text.clone());
            
            // Replace the truncated history with just the summary injected into the last user message
            let mut new_history = Vec::new();
            if let Some(last_msg) = history.last() {
                let mut modified_last_msg = last_msg.clone();
                modified_last_msg.content = format!("[Previous Conversation Summary: {}]\n\n{}", summary_text, modified_last_msg.content);
                new_history.push(modified_last_msg);
            }
            truncated_history = new_history;
            is_continuation = false; // Force a full reset with the new summarized context
            
            // Also reset last_history so we don't think it's a continuation next time if it doesn't match
            if let Ok(mut last_hist_guard) = self.last_history.lock() {
                *last_hist_guard = None;
            }
        } else {
            // Normal continuation logic
            let mut last_hist_guard = self.last_history.lock().unwrap();
            if let Some(last_hist) = &*last_hist_guard {
                if !history.is_empty() && history.len() >= last_hist.len() && truncated_history.len() == history.len() {
                    let mut matches = true;
                    for (i, msg) in last_hist.iter().enumerate() {
                        if history[i].role != msg.role || history[i].content != msg.content {
                            matches = false;
                            break;
                        }
                    }
                    if matches {
                        is_continuation = true;
                    }
                }
            }
            *last_hist_guard = Some(truncated_history.clone());
        }

        let is_multimodal = audio_base64.is_some() || image_uri.is_some();
        let gemma_prompt = if is_multimodal {
            // MULTIMODAL: Do not wrap in <start_of_turn>. Let Kotlin/LiteRT handle it natively.
            let mut text = truncated_history.last().unwrap_or(history.last().unwrap()).content.clone();
            if text.contains("🎤 [Audio Message]") || text.trim().is_empty() {
                text = text.replace("🎤 [Audio Message]", "Please evaluate this media and respond appropriately.");
                if text.trim().is_empty() {
                    text = "Please evaluate this media and respond appropriately.".to_string();
                }
            }
            
            // Gemma 4 vision requires the <image> token in the prompt text
            if image_uri.is_some() {
                text = format!("<image>\n{}", text);
            }
            
            if !is_continuation {
                format!("{}\n\n{}", system_prompt_str, text)
            } else {
                text
            }
        } else if is_continuation {
            let last_msg = history.last().unwrap();
            format!("<start_of_turn>{}\n{}<end_of_turn>\n", last_msg.role, last_msg.content)
        } else {
            let mut prompt = String::new();

            if truncated_history.is_empty() {
                prompt.push_str(&format!("<start_of_turn>user\n{}<end_of_turn>\n", system_prompt_str));
            } else {
                let mut first_user_found = false;
                for msg in truncated_history {
                    prompt.push_str(&format!("<start_of_turn>{}\n", msg.role));
                    if !first_user_found && msg.role == "user" {
                        prompt.push_str(&format!("{}\n\n", system_prompt_str));
                        first_user_found = true;
                    }
                    prompt.push_str(&msg.content);
                    prompt.push_str("<end_of_turn>\n");
                }
            }
            prompt
        };

        let max_attempts = 3;
        let mut attempts = 0;

        while attempts < max_attempts {
            let payload = tauri_plugin_litert::GenerateChatRequest {
                prompt: if is_multimodal { gemma_prompt.clone() } else { gemma_prompt.clone() + "<start_of_turn>model\n" },
                reset: !is_continuation,
                audio_base64: audio_base64.clone(),
                image_uri: image_uri.clone(),
            };
            
            let response_text = match self.app_handle.litert().generate_chat(payload) {
                Ok(res) => res.response,
                Err(e) => {
                    attempts += 1;
                    if attempts >= max_attempts {
                        return Err(format!("Android inference error: {}", e));
                    }
                    continue;
                }
            };

            return Ok(ChatResponse {
                response: response_text,
                idealized_correction: None,
                context_summary: context_summary.clone(),
            });
        }
        Err("Unknown error in generation loop".to_string())
    }

    async fn generate_conjugation_exercise(
        &self,
        language: String,
        _model: String,
        previously_used: Vec<String>,
        tense_stats: Vec<TenseStat>,
        active_theme: String,
    ) -> Result<ConjugationExercise, String> {
        self.ensure_initialized().await?;
        let prompt = super::build_conjugation_prompt(
            &language,
            &previously_used,
            &tense_stats,
            &active_theme,
        );

        let initial_exercise: ConjugationExercise = self.execute_json_generation(prompt, "Conjugation")?;
        
        let verify_prompt = super::build_conjugation_verification_prompt(&language, &initial_exercise);
        self.execute_json_generation(verify_prompt, "Conjugation Verification")
    }

    async fn generate_embedding(&self, _text: String, _model: String) -> Result<Vec<f64>, String> {
        Err(self.unsupported("Embeddings"))
    }
}
