use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokenizers::Tokenizer;

pub mod ollama_adapter;
#[cfg(target_os = "android")]
pub mod litert_adapter;
pub mod tts_adapter;

#[derive(Serialize, Deserialize, Debug)]
pub struct JournalVariables {
    pub mood: String,
    pub weather: String,
    pub activity: String,
    pub model: String,
    pub language: String,
    pub skill_level: String,
    pub active_theme: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GradingVariables {
    pub entry: String,
    pub model: String,
    pub language: String,
    pub skill_level: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, schemars::JsonSchema)]
pub struct JournalResponse {
    pub generated_target_text: String,
    pub native_translation: String,
    pub vocabulary: Vec<VocabItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, schemars::JsonSchema)]
pub struct VocabItem {
    pub target_text: String,
    pub native_text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, schemars::JsonSchema)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    #[serde(default)]
    pub audio_base64: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, schemars::JsonSchema)]
pub struct ChatResponse {
    pub response: String,
    pub idealized_correction: Option<String>,
    #[serde(default)]
    pub context_summary: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, schemars::JsonSchema)]
pub struct ConjugationExercise {
    pub subject: String,
    pub tense: String,
    pub sentence: String,
    pub verb: String,
    pub answer: String,
    pub translation: String,
}

/// Per-tense accuracy stats derived from the last N answered exercises.
#[derive(Serialize, Deserialize, Debug, Clone, schemars::JsonSchema)]
pub struct TenseStat {
    pub tense: String,
    pub total: u32,
    pub correct: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, schemars::JsonSchema)]
pub struct KeystonePuzzle {
    pub language: String,
    pub code_with_blank: String,
    pub exact_answer: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, schemars::JsonSchema)]
pub struct SpeedrunPuzzle {
    pub language: String,
    pub code: String,
    pub correct_answer: String,
    pub distractors: Vec<String>,
}

#[async_trait]
pub trait LlmProvider {
    async fn check_health(&self) -> bool;
    async fn list_models(&self) -> Result<Vec<String>, String>;
    async fn generate_guided_journal(
        &self,
        variables: JournalVariables,
    ) -> Result<JournalResponse, String>;
    async fn grade_custom_journal(
        &self,
        variables: GradingVariables,
    ) -> Result<JournalResponse, String>;
    async fn generate_chat_response(
        &self,
        history: Vec<ChatMessage>,
        model: String,
        language: String,
        skill_level: String,
        context: String,
        active_theme: Option<String>,
        active_subtheme: Option<String>,
        audio_base64: Option<String>,
        image_uri: Option<String>,
    ) -> Result<ChatResponse, String>;
    async fn generate_conjugation_exercise(
        &self,
        language: String,
        model: String,
        previously_used: Vec<String>,
        tense_stats: Vec<TenseStat>,
        active_theme: String,
    ) -> Result<ConjugationExercise, String>;
    async fn generate_embedding(&self, text: String, model: String) -> Result<Vec<f64>, String>;
    async fn generate_coding_puzzle(
        &self,
        language: String,
        model: String,
        theme: String,
        puzzle_type: String,
        previously_used: Vec<String>,
    ) -> Result<String, String>;
    async fn generate_language_puzzle(
        &self,
        language: String,
        model: String,
        skill_level: String,
        active_theme: String,
        active_subtheme: String,
        puzzle_type: String,
        previously_used: Vec<String>,
    ) -> Result<String, String>;
}

// -----------------------------------------------------------------------------
// Shared Prompt Builders & Context Management
// -----------------------------------------------------------------------------

pub fn build_journal_prompt(
    language: &str,
    skill_level: &str,
    mood: &str,
    weather: &str,
    activity: &str,
    active_theme: &Option<String>,
) -> String {
    let mut prompt = format!(
        "Generate a journal entry in {lang} for a {skill_level} learner based on these inputs: \
        Mood: {mood}, Weather: {weather}, Activity: {activity}. \
        Adapt your vocabulary and grammar complexity to the student's {skill_level} level.\n",
        lang=language, skill_level=skill_level, mood=mood, weather=weather, activity=activity
    );

    if let Some(theme) = active_theme {
        prompt.push_str(&format!("The user is currently studying the curriculum theme: {}. Please gently incorporate ideas or vocabulary related to this theme into the journal scaffold.\n", theme));
    }

    prompt.push_str("Also extract 3-5 useful dictionary-form vocabulary words from the entry. \
        Output STRICTLY valid JSON with the following structure, nothing else: \
        {\"generated_target_text\": \"...\", \"native_translation\": \"...\", \"vocabulary\": [{\"target_text\": \"...\", \"native_text\": \"...\"}], \"feedback\": null}");

    prompt
}

pub fn build_coding_puzzle_prompt(language: &str, theme: &str, puzzle_type: &str, previously_used: &[String]) -> (String, String) {
    let example_keystone_code = match language.to_lowercase().as_str() {
        "rust" => "fn do_work(a: i32, b: i32) -> i32 {\\n    return ___BLANK___;\\n}",
        "javascript" | "typescript" | "js" | "ts" => "function doWork(a, b) {\\n    return ___BLANK___;\\n}",
        "go" => "func doWork(a int, b int) int {\\n    return ___BLANK___\\n}",
        "c++" | "cpp" | "c" => "int do_work(int a, int b) {\\n    return ___BLANK___;\\n}",
        "java" => "public int doWork(int a, int b) {\\n    return ___BLANK___;\\n}",
        "kotlin" | "kt" => "fun doWork(a: Int, b: Int): Int {\\n    return ___BLANK___\\n}",
        _ => "def do_work(a, b):\\n    return ___BLANK___", // Default to python-like
    };

    let example_speedrun_code = match language.to_lowercase().as_str() {
        "rust" => "fn do_work(arr: &[i32]) -> i32 {\\n    return arr[arr.len() - 1];\\n}",
        "javascript" | "typescript" | "js" | "ts" => "function doWork(arr) {\\n    return arr[arr.length - 1];\\n}",
        "go" => "func doWork(arr []int) int {\\n    return arr[len(arr)-1]\\n}",
        "c++" | "cpp" | "c" => "int do_work(std::vector<int>& arr) {\\n    return arr.back();\\n}",
        "java" => "public int doWork(int[] arr) {\\n    return arr[arr.length - 1];\\n}",
        "kotlin" | "kt" => "fun doWork(arr: IntArray): Int {\\n    return arr.last()\\n}",
        _ => "def do_work(arr):\\n    return arr[-1]", // Default to python-like
    };

    let result = if puzzle_type == "keystone" {
        (
            format!(
                "You are a strict JSON code generator for a programming game.\nYou will be given a programming topic. Generate a concise, functional snippet of code (10-15 lines max) that solves it.\n\nCRITICAL GAMEPLAY RULES:\n1. You MUST replace the single most important line of algorithmic logic (the 'keystone' of the function) with the exact string \"___BLANK___\".\n2. The code MUST contain exactly one instance of \"___BLANK___\".\n3. Do not blank out function definitions, imports, or basic brackets. Only blank out the core logical step.\n4. CRITICAL: ALL generated code MUST be written strictly in the {language} programming language. Do NOT use any other language.\n\nCRITICAL JSON RULES:\n1. You MUST escape all internal double quotes inside your code strings (e.g., use \\\" instead of \").\n2. You MUST use \\n for newlines inside your code strings. Do NOT use actual line breaks.\n\nOutput strictly in this JSON format:\n{{\n  \"language\": \"{language}\",\n  \"code_with_blank\": \"{example_keystone_code}\",\n  \"exact_answer\": \"a + b\"\n}}"
            ),
            theme.to_string()
        )
    } else {
        (
            format!(
                "You are a strict JSON code generator for a programming game.\nYou will be given a programming topic. Generate a concise, functional snippet of code (10-15 lines max) that solves it.\n\nCRITICAL GAMEPLAY RULES:\n1. You MUST name the main function generically, exactly `doWork` or `do_work`, so the name does NOT give away what the code does.\n2. Do NOT include any comments, docstrings, or documentation in the code. It must be completely uncommented.\n3. Provide a short, 1-sentence correct description of what the code does, and 3 plausible but incorrect distractor descriptions. The distractors should sound like real programming operations.\n4. CRITICAL: ALL generated code MUST be written strictly in the {language} programming language. Do NOT use any other language.\n\nCRITICAL JSON RULES:\n1. You MUST escape all internal double quotes inside your code strings (e.g., use \\\" instead of \").\n2. You MUST use \\n for newlines inside your code strings. Do NOT use actual line breaks.\n\nOutput strictly in this JSON format:\n{{\n  \"language\": \"{language}\",\n  \"code\": \"{example_speedrun_code}\",\n  \"correct_answer\": \"Returns the last element of an array.\",\n  \"distractors\": [\"Removes the first element of an array.\", \"Reverses the array in place.\", \"Returns the length of the array.\"]\n}}"
            ),
            theme.to_string()
        )
    };

    if !previously_used.is_empty() {
        let mut appended_prompt = result.0;
        appended_prompt.push_str("\n\nFORBIDDEN PREVIOUS GAMES: The user has recently played the following games. CRITICAL INSTRUCTION: You MUST NOT generate a puzzle that matches these recent puzzles. Create something completely NEW and DIFFERENT:\n");
        for game in previously_used {
            appended_prompt.push_str(&format!("- {}\n", game));
        }
        (appended_prompt, result.1)
    } else {
        result
    }
}

pub fn build_language_puzzle_prompt(language: &str, skill_level: &str, active_theme: &str, active_subtheme: &str, puzzle_type: &str, previously_used: &[String]) -> String {

    let example_keystone = match language.to_lowercase().as_str() {
        "french" => "Je ___BLANK___ un croissant.",
        "spanish" => "Yo ___BLANK___ una manzana.",
        _ => "I ___BLANK___ a book.",
    };

    let example_speedrun = match language.to_lowercase().as_str() {
        "french" => "Je mange un croissant.",
        "spanish" => "Yo como una manzana.",
        _ => "I am reading a book.",
    };

    let prompt = if puzzle_type == "keystone" {
        format!(
            "You are a strict JSON generator for a language learning game. \
            Generate a single sentence in {language} suitable for a {skill_level} learner. \
            The sentence should be related to the theme: '{active_theme}', specifically focusing on the context of: '{active_subtheme}'. \
            \nCRITICAL GAMEPLAY RULES:\n\
            1. You MUST replace a key word in the sentence (like a conjugated verb or an important noun) with the exact string \"___BLANK___\".\n\
            2. The sentence MUST contain exactly one instance of \"___BLANK___\".\n\
            3. CRITICAL: ALL generated text MUST be grammatically correct {language}, except for the blank.\n\
            \nOutput strictly in this JSON format:\n\
            {{\n  \"language\": \"{language}\",\n  \"code_with_blank\": \"{example_keystone}\",\n  \"exact_answer\": \"mange\"\n}}"
        )
    } else {
        format!(
            "You are a strict JSON generator for a language learning game. \
            Generate a single sentence in {language} suitable for a {skill_level} learner. \
            The sentence should be related to the theme: '{active_theme}', specifically focusing on the context of: '{active_subtheme}'. \
            \nCRITICAL GAMEPLAY RULES:\n\
            1. Provide a correct English translation of the sentence, and 3 plausible but incorrect distractor English translations.\n\
            2. CRITICAL: ALL generated text MUST be grammatically correct {language}.\n\
            \nOutput strictly in this JSON format:\n\
            {{\n  \"language\": \"{language}\",\n  \"code\": \"{example_speedrun}\",\n  \"correct_answer\": \"I eat a croissant.\",\n  \"distractors\": [\"I want a croissant.\", \"I make a croissant.\", \"You eat a croissant.\"]\n}}"
        )
    };

    if !previously_used.is_empty() {
        let mut appended_prompt = prompt;
        appended_prompt.push_str("\n\nFORBIDDEN PREVIOUS GAMES: The user has recently played the following games. CRITICAL INSTRUCTION: You MUST NOT generate a puzzle that matches these recent puzzles. Create something completely NEW and DIFFERENT:\n");
        for game in previously_used {
            appended_prompt.push_str(&format!("- {}\n", game));
        }
        appended_prompt
    } else {
        prompt
    }
}

pub fn build_chat_system_prompt(
    language: &str,
    skill_level: &str,
    context: &str,
    active_theme: &Option<String>,
    active_subtheme: &Option<String>,
    use_json: bool,
    use_expression_tags: bool,
    is_vision_judge: bool,
) -> String {
    let mut prompt = String::with_capacity(2048);

    // move to own prompt, not sure why here.
    if is_vision_judge {
        prompt.push_str(&format!(
            "# ROLE\n\
            You are a helpful language learning vision judge for {language}. \
            Your job is to describe the attached image and grade the user's description of it.\n\
            Point out anything they missed, and correct their grammar based on their {skill_level} level.\n\n",
            language=language, skill_level=skill_level
        ));
    } else {
        prompt.push_str(&format!(
            "Role: {} language partner. Keep conversation flowing naturally. Adapt to user's level. Ignore minor errors. End with a short follow-up question.\n\n\
            Rules:\n\
            - Max 3 sentences.\n\
            - No random greetings mid-chat.\n\
            - You control an avatar. Include exact animation tags: [anim:shrug], [anim:greet], [anim:peace], [anim:shoot], [anim:spin], [anim:pose], [anim:squat], [anim:full]. No other tags.\n",
            language
        ));

        if use_expression_tags {
            prompt.push_str(
                "- Expression tags (use exactly as shown):\n\
                  * <laugh> : END of sentence (x3)\n\
                  * <breath> : START of sentence (x3)\n\
                  * <sad> : BOTH ends of sentence (x3)\n"
            );
        }

        if !cfg!(target_os = "android") && !context.trim().is_empty() {
            prompt.push_str("Textbook Context (use to guide conversation):\n");
            prompt.push_str(context);
            prompt.push_str("\n\n");
        }

        if let Some(theme) = active_theme {
            prompt.push_str(&format!("Theme: {}. Steer chat towards this and use related words.\n", theme));
            if let Some(subtheme) = active_subtheme {
                prompt.push_str(&format!("Specific Context/Subtheme: {}.\n", subtheme));
            }
            prompt.push_str("\n");
        }
    }

    if skill_level.trim().eq_ignore_ascii_case("beginner") {
        prompt.push_str(&format!(
            "User level: Beginner. Act as strict tutor. Speak mostly English. Introduce 1-2 new {} words/phrases per turn with meaning. Use native script for {} words.\n",
            language, language
        ));
    } else {
        prompt.push_str(&format!(
            "User level: {}. Speak almost entirely in {}. Adapt vocabulary to their level.\n",
            skill_level, language
        ));
    }
    
    prompt.push_str(
        "No romanization/phonetics. Use perfect orthography/punctuation in native script. No parentheses.\n\n"
    );

    // 9. Output Formatting Constraint (Crucial this goes at the end)
    prompt.push_str("# OUTPUT FORMAT\n");
    if use_json {
        prompt.push_str(
            "Output STRICTLY valid JSON with the following structure, nothing else:\n\
            {\n  \
                \"response\": \"Your conversational response here (including your animation/expression tags)\",\n  \
                \"idealized_correction\": \"If the user's last message had grammar errors, provide the correction here. If perfect, return null.\"\n\
            }\n"
        );
    } else {
        prompt.push_str("Do not use JSON. Provide your response directly.\n");
    }

    prompt
}

pub fn build_conjugation_prompt(
    language: &str,
    previously_used: &[String],
    _tense_stats: &[TenseStat],
    active_theme: &str,
) -> String {
    let lenses = [
        "an emotional perspective", "a historical or time-based context", "a technical or analytical angle", 
        "a visual or descriptive focus", "a social or relational viewpoint", "an action-oriented or energetic tone", 
        "a mysterious or curious angle", "a humorous or lighthearted perspective", "a formal or professional tone", 
        "a mundane or everyday slice-of-life angle"
    ];
    let time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let lens = lenses[(time as usize) % lenses.len()];

    let mut prompt = format!(
        "Generate a conjugation exercise for a student learning {lang}. \
        First, pick a completely random, natural, and unique infinitive verb. \
        Pick a specific subject, and construct a natural sentence around it. \
        CRITICAL: The sentence MUST strictly relate to the theme of '{theme}', specifically focusing on '{lens}'. \
        CRITICAL: You MUST conjugate your chosen verb DIRECTLY. DO NOT use modal or auxiliary verbs (like pouvoir, vouloir, devoir, aller) followed by the infinitive of your chosen verb. Your chosen verb must be the primary conjugated action in the sentence. \
        CRITICAL: In languages like French, object pronouns (like 'vous', 'te', 'me') often sit right before the verb (e.g. 'Je vous accueille'). Do NOT mistake the object for the subject. The 'subject' field MUST accurately reflect the true entity performing the action, and the verb MUST be conjugated to match that subject. \
        Output STRICTLY valid JSON with the following structure, nothing else, IN THIS EXACT ORDER: \n\
        {{ \n  \
            \"verb\": \"Pick a random infinitive verb (e.g. manger)\", \n  \
            \"subject\": \"Pick a random subject pronoun or noun (the true doer of the action)\", \n  \
            \"sentence\": \"Write a complete, natural sentence in {lang} using your chosen verb and subject directly conjugated.\", \n  \
            \"tense\": \"Identify the exact tense of the primary conjugated verb you used in the sentence (e.g. passé composé)\", \n  \
            \"answer\": \"Extract just the exact conjugated verb from the sentence.\", \n  \
            \"translation\": \"english translation of the infinitive verb\" \n\
        }}",
        lang=language,
        theme=active_theme,
        lens=lens
    );


    if !previously_used.is_empty() {
        prompt.push_str("\n\nFORBIDDEN VERBS: The student has recently practiced the following verbs. \
            CRITICAL INSTRUCTION: You MUST select a completely NEW and DIFFERENT VERB. Under NO CIRCUMSTANCES may your sentence use any of these verbs as the primary conjugated verb. If you do, the system will crash:\n");
        for entry in previously_used {
            prompt.push_str(&format!("- {}\n", entry));
        }
    }

    prompt
}

pub fn build_conjugation_verification_prompt(
    language: &str,
    exercise: &ConjugationExercise,
) -> String {
    format!(
        "Please verify the following conjugation exercise for a student learning {lang}. \n\
        Subject: {subject} \n\
        Tense: {tense} \n\
        Sentence: {sentence} \n\
        Verb (infinitive): {verb} \n\
        Current Answer: {answer} \n\
        Translation of Infinitive: {translation} \n\n\
        If the current answer and sentence are completely correct, output the same JSON exactly. \
        If there is any error in the conjugation (e.g. wrong tense, wrong subject agreement, misspelled), correct the 'answer', 'subject', and/or 'sentence' field and output the fixed JSON. \
        CRITICAL: The 'answer' MUST be the conjugated form of the exact verb '{verb}'. Do NOT change the answer to a completely different verb.\n\
        CRITICAL BUG FIX RULE 1: If the sentence uses a modal or auxiliary verb (like pouvoir, vouloir, devoir, aller) followed by the infinitive '{verb}', THIS IS WRONG. You MUST completely rewrite the sentence so that '{verb}' itself is directly conjugated as the primary verb of the sentence, and then update the 'answer' to match it.\n\
        CRITICAL BUG FIX RULE 2: Object pronouns (e.g. 'vous', 'te', 'me') placed before the verb do NOT determine conjugation. The verb MUST match the true subject. If the 'subject' field does not match the actual grammatical subject of the sentence, correct the 'subject' field and ensure the 'answer' matches the true subject.\n\
        Output STRICTLY valid JSON with the following structure, nothing else, IN THIS EXACT ORDER: \n\
        {{\n  \"verb\": \"{verb}\",\n  \"subject\": \"{subject}\",\n  \"sentence\": \"{sentence}\",\n  \"tense\": \"{tense}\",\n  \"answer\": \"corrected or original conjugated form\",\n  \"translation\": \"{translation}\"\n}}",
        lang=language, subject=exercise.subject, tense=exercise.tense, sentence=exercise.sentence, verb=exercise.verb, answer=exercise.answer, translation=exercise.translation
    )
}

/// Truncate chat history using a character-based token budget window.
/// Ensures we do not exceed safe token limits, dropping oldest messages first.
pub fn truncate_history_by_tokens(
    system_prompt: &str,
    history: &[ChatMessage],
    max_tokens_budget: usize,
) -> Vec<ChatMessage> {
    // FIX 1: Use .chars().count() to prevent multi-byte characters (like Hangul/Kanji) 
    // from artificially inflating the token estimate.
    let estimate_tokens = |text: &str| -> usize {
        (text.chars().count() as f64 / 3.5).ceil() as usize
    };

    let sys_tokens = estimate_tokens(system_prompt);
    
    // FIX 2: Bump the reserved output budget to 250
    let safe_budget = if max_tokens_budget > sys_tokens + 250 {
        max_tokens_budget - sys_tokens - 250
    } else {
        0
    };

    let mut current_budget = 0;
    let mut keep_count = 0;

    // Traverse from newest to oldest
    for (i, msg) in history.iter().rev().enumerate() {
        let mut msg_tokens = estimate_tokens(&msg.content);
        if msg.content.contains("🎤 [Audio Message]") {
            msg_tokens += 1200; // Native audio footprint penalty
        }
        let total_msg_tokens = msg_tokens + 5; 
        
        // FIX 3: Guarantee we ALWAYS keep the very last message (i == 0)
        if i > 0 && current_budget + total_msg_tokens > safe_budget {
            break;
        }
        
        current_budget += total_msg_tokens;
        keep_count += 1;
    }

    let start_idx = history.len().saturating_sub(keep_count);
    history[start_idx..].to_vec()
}

pub fn truncate_history_exact(
    system_prompt: &str,
    history: &[ChatMessage],
    max_tokens_budget: usize,
    tokenizer: &Tokenizer,
) -> Vec<ChatMessage> {
    let sys_tokens = tokenizer.encode(system_prompt, true).unwrap_or_default().get_ids().len();
    
    let safe_budget = max_tokens_budget.saturating_sub(sys_tokens + 250);
    
    let mut current_budget = 0;
    let mut keep_count = 0;

    for (i, msg) in history.iter().rev().enumerate() {
        let mut msg_tokens = tokenizer.encode(msg.content.clone(), true).unwrap_or_default().get_ids().len();
        if msg.content.contains("🎤 [Audio Message]") {
            msg_tokens += 1200; // Native audio footprint penalty
        }
        let total_msg_tokens = msg_tokens + 5; // chat template overhead

        if i > 0 && current_budget + total_msg_tokens > safe_budget {
            break;
        }

        current_budget += total_msg_tokens;
        keep_count += 1;
    }

    let start_idx = history.len().saturating_sub(keep_count);
    history[start_idx..].to_vec()
}
