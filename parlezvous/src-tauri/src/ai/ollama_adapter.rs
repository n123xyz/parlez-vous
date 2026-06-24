use super::{
    ChatMessage, ChatResponse, GradingVariables, JournalResponse, JournalVariables, LlmProvider,
};
use async_trait::async_trait;
use ollama_rs::{
    generation::{
        chat::{request::ChatMessageRequest, ChatMessage as OllamaChatMessage},
        completion::request::GenerationRequest,
        parameters::FormatType,
    },
    Ollama,
};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

pub struct OllamaAdapter {
    db: Arc<Mutex<Connection>>,
}

impl OllamaAdapter {
    pub fn new(db: Arc<Mutex<Connection>>) -> Self {
        Self { db }
    }

    fn get_client(&self) -> Result<Ollama, String> {
        let conn = self.db.lock().map_err(|_| "Failed to lock DB")?;
        let url: String = conn
            .query_row(
                "SELECT ollama_server_url FROM settings WHERE id = 1",
                [],
                |r| r.get(0),
            )
            .unwrap_or_else(|_| "http://localhost".to_string());

        let url = url.trim_end_matches('/');
        let mut host = url;
        let mut port = if url.starts_with("https://") { 443 } else { 11434 };

        // Very basic host/port parsing. Ollama::new expects (host, port)
        if url.starts_with("http://") || url.starts_with("https://") {
            if let Some((h, p)) = url.rsplit_once(':') {
                if let Ok(parsed_port) = p.parse::<u16>() {
                    // Make sure the split wasn't the https:// protocol part
                    if h != "http" && h != "https" {
                        host = h;
                        port = parsed_port;
                    }
                }
            }
        }

        Ok(Ollama::new(host.to_string(), port))
    }

    async fn execute_journal_generation(
        &self,
        model: &str,
        prompt: String,
    ) -> Result<JournalResponse, String> {
        let max_attempts = 3;
        let mut attempts = 0;

        while attempts < max_attempts {
            let request =
                GenerationRequest::new(model.to_string(), prompt.clone()).format(FormatType::Json);

            let client = self.get_client()?;
            let response = client.generate(request).await.map_err(|e| e.to_string())?;
            let content = response.response;

            let json_content =
                if let (Some(start), Some(end)) = (content.find('{'), content.rfind('}')) {
                    &content[start..=end]
                } else {
                    &content
                };

            match serde_json::from_str::<JournalResponse>(json_content) {
                Ok(parsed) => return Ok(parsed),
                Err(e) => {
                    attempts += 1;
                    println!(
                        "LLM JSON Parse Error on attempt {}: {} \nRaw output: {}",
                        attempts, e, content
                    );
                    if attempts >= max_attempts {
                        return Err(format!(
                            "The AI model failed to produce valid JSON after {} attempts.",
                            max_attempts
                        ));
                    }
                }
            }
        }

        Err("Unknown error in generation loop".to_string())
    }
}

#[async_trait]
impl LlmProvider for OllamaAdapter {
    async fn check_health(&self) -> bool {
        if let Ok(client) = self.get_client() {
            client.list_local_models().await.is_ok()
        } else {
            false
        }
    }

    async fn list_models(&self) -> Result<Vec<String>, String> {
        let client = self.get_client()?;
        let models = client
            .list_local_models()
            .await
            .map_err(|e| e.to_string())?;
        Ok(models.into_iter().map(|m| m.name).collect())
    }

    async fn generate_guided_journal(
        &self,
        variables: JournalVariables,
    ) -> Result<JournalResponse, String> {
        let prompt = super::build_journal_prompt(
            &variables.language,
            &variables.skill_level,
            &variables.mood,
            &variables.weather,
            &variables.activity,
            &variables.active_theme,
        );

        self.execute_journal_generation(&variables.model, prompt)
            .await
    }

    async fn grade_custom_journal(
        &self,
        variables: GradingVariables,
    ) -> Result<JournalResponse, String> {
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

        self.execute_journal_generation(&variables.model, prompt)
            .await
    }
    async fn generate_chat_response(
        &self,
        history: Vec<ChatMessage>,
        model: String,
        language: String,
        skill_level: String,
        context: String,
        active_theme: Option<String>,
        _audio_base64: Option<String>,
        _image_uri: Option<String>,
    ) -> Result<ChatResponse, String> {
        let is_vision = _image_uri.is_some();
        let system_prompt_str = super::build_chat_system_prompt(
            &language,
            &skill_level,
            &context,
            &active_theme,
            !is_vision,
            false,
            is_vision,
        );

        let mut messages = Vec::new();

        let system_prompt = OllamaChatMessage::system(system_prompt_str.clone());
        messages.push(system_prompt);

        let history_len = history.len();

        for (i, msg) in history.into_iter().enumerate() {
            let mut o_msg = if msg.role == "user" {
                OllamaChatMessage::user(msg.content)
            } else {
                OllamaChatMessage::assistant(msg.content)
            };
            
            // Add image to the last user message if vision is enabled
            if is_vision && i == history_len - 1 && msg.role == "user" {
                if let Some(uri) = &_image_uri {
                    if uri.starts_with("data:image/") {
                        let base64_data = uri.splitn(2, ',').nth(1).unwrap_or("").to_string();
                        if !base64_data.is_empty() {
                            o_msg = o_msg.add_image(ollama_rs::generation::images::Image::from_base64(&base64_data));
                        }
                    }
                }
            }
            messages.push(o_msg);
        }

        let max_gen_attempts = 3;
        let mut gen_attempts = 0;

        loop {
            let request = ChatMessageRequest::new(model.clone(), messages.clone());
            let client = self.get_client()?;
            let response = client
                .send_chat_messages(request)
                .await
                .map_err(|e| e.to_string())?;
            let content = response.message.content.clone();

            if is_vision {
                return Ok(ChatResponse {
                    response: content.clone(),
                    idealized_correction: None,
                    context_summary: None,
                });
            }

            let json_content =
                if let (Some(start), Some(end)) = (content.find('{'), content.rfind('}')) {
                    &content[start..=end]
                } else {
                    &content
                };

            match serde_json::from_str::<ChatResponse>(json_content) {
                Ok(mut parsed) => {
                    // Strip hallucinated bracket tags like [expression:smile] that aren't [anim:...]
                    let re = regex::Regex::new(r"\[([^\]]+)\]").unwrap();
                    parsed.response = re.replace_all(&parsed.response, |caps: &regex::Captures| {
                        let tag = &caps[1];
                        if tag.starts_with("anim:") {
                            caps[0].to_string()
                        } else {
                            "".to_string()
                        }
                    }).to_string();
                    
                    return Ok(parsed);
                },
                Err(e) => {
                    gen_attempts += 1;
                    println!(
                        "LLM Chat JSON Parse Error on attempt {}: {} \nRaw output: {}",
                        gen_attempts, e, content
                    );
                    if gen_attempts >= max_gen_attempts {
                        return Err(format!(
                            "The AI model failed to produce valid JSON after {} attempts.",
                            max_gen_attempts
                        ));
                    }
                }
            }
        }
    }

    async fn generate_conjugation_exercise(
        &self,
        language: String,
        model: String,
        previously_used: Vec<String>,
        tense_stats: Vec<crate::ai::TenseStat>,
        active_theme: String,
    ) -> Result<crate::ai::ConjugationExercise, String> {
        let prompt = super::build_conjugation_prompt(
            &language,
            &previously_used,
            &tense_stats,
            &active_theme,
        );

        let max_attempts = 3;
        let mut attempts = 0;

        while attempts < max_attempts {
            let request = GenerationRequest::new(model.clone(), prompt.clone()).format(FormatType::Json);

            let client = self.get_client()?;
            let response = client.generate(request).await.map_err(|e| e.to_string())?;
            let content = response.response;

            let json_content =
                if let (Some(start), Some(end)) = (content.find('{'), content.rfind('}')) {
                    &content[start..=end]
                } else {
                    &content
                };

            match serde_json::from_str::<crate::ai::ConjugationExercise>(json_content) {
                Ok(mut parsed) => {
                    // One-pass revision/verification
                    let verification_prompt = super::build_conjugation_verification_prompt(&language, &parsed);
                    let verify_request = GenerationRequest::new(model.clone(), verification_prompt).format(FormatType::Json);
                    
                    println!("[Conjugation] Running 1-pass verification on candidate...");
                    if let Ok(client) = self.get_client() {
                        if let Ok(response) = client.generate(verify_request).await {
                            let content = response.response;
                            let v_json_content = if let (Some(start), Some(end)) = (content.find('{'), content.rfind('}')) {
                                &content[start..=end]
                            } else {
                                &content
                            };
                            
                            if let Ok(revised) = serde_json::from_str::<crate::ai::ConjugationExercise>(v_json_content) {
                                println!("[Conjugation] Verification successfully parsed, unconditionally accepting revised candidate.");
                                parsed = revised;
                            } else {
                                println!("[Conjugation] Verification JSON parse failed, falling back to original candidate.");
                            }
                        } else {
                            println!("[Conjugation] Verification LLM call failed, falling back to original candidate.");
                        }
                    }

                    return Ok(parsed);
                },
                Err(e) => {
                    attempts += 1;
                    println!(
                        "LLM Conjugation JSON Parse Error on attempt {}: {} \nRaw output: {}",
                        attempts, e, content
                    );
                    if attempts >= max_attempts {
                        return Err(format!(
                            "The AI model failed to produce valid JSON after {} attempts.",
                            max_attempts
                        ));
                    }
                }
            }
        }

        Err("Unknown error in conjugation generation loop".to_string())
    }

    async fn generate_embedding(&self, text: String, model: String) -> Result<Vec<f64>, String> {
        let max_attempts = 3;
        let mut attempts = 0;

        while attempts < max_attempts {
            // Note: ollama-rs 0.3 handles embeddings via generate_embeddings.
            let request =
                ollama_rs::generation::embeddings::request::GenerateEmbeddingsRequest::new(
                    model.clone(),
                    ollama_rs::generation::embeddings::request::EmbeddingsInput::Single(
                        text.clone(),
                    ),
                );

            let client = self.get_client()?;
            match client.generate_embeddings(request).await {
                Ok(response) => {
                    return Ok(response
                        .embeddings
                        .into_iter()
                        .flatten()
                        .map(|v| v as f64)
                        .collect());
                }
                Err(e) => {
                    attempts += 1;
                    println!("LLM Embedding Error on attempt {}: {}", attempts, e);
                    if attempts >= max_attempts {
                        return Err(format!(
                            "The AI model failed to generate embeddings after {} attempts.",
                            max_attempts
                        ));
                    }
                }
            }
        }

        Err("Unknown error in embedding generation loop".to_string())
    }
}

