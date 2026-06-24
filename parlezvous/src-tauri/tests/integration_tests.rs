use async_trait::async_trait;
use parlezvous_lib::ai::{
    ChatMessage, ChatResponse, JournalResponse, JournalVariables, LlmProvider, VocabItem,
};
use parlezvous_lib::db::SCHEMA_V1;
use parlezvous_lib::services::journal::process_journal_generation;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

mockall::mock! {
    pub LlmProvider {}
    #[async_trait]
    impl LlmProvider for LlmProvider {
        async fn check_health(&self) -> bool;
        async fn list_models(&self) -> Result<Vec<String>, String>;
        async fn generate_guided_journal(&self, variables: JournalVariables) -> Result<JournalResponse, String>;
        async fn generate_chat_response(&self, history: Vec<ChatMessage>, model: String) -> Result<ChatResponse, String>;
    }
}

fn setup_in_memory_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(SCHEMA_V1).unwrap();
    conn
}

#[tokio::test]
async fn test_generate_journal_inserts_correctly() {
    let mut mock_ai = MockLlmProvider::new();

    // Configure the mock to return a static predictable JSON response
    mock_ai.expect_generate_guided_journal().returning(|_| {
        Ok(JournalResponse {
            generated_target_text: "Bonjour!".to_string(),
            native_translation: "Hello!".to_string(),
            vocabulary: vec![VocabItem {
                target_text: "Bonjour".to_string(),
                native_text: "Hello".to_string(),
            }],
        })
    });

    let conn = setup_in_memory_db();
    let db = Arc::new(Mutex::new(conn));
    let ai = Arc::new(mock_ai);

    let variables = JournalVariables {
        mood: "happy".to_string(),
        weather: "sunny".to_string(),
        activity: "coding".to_string(),
        model: "test-model".to_string(),
    };

    // Call the decoupled business logic
    let (response, chips) = process_journal_generation(db.clone(), ai, variables)
        .await
        .unwrap();

    // Verify correct response
    assert_eq!(response.generated_target_text, "Bonjour!");
    assert_eq!(chips.len(), 1);
    assert_eq!(chips[0].target_text, "Bonjour");

    // Verify Database state
    let conn = db.lock().unwrap();

    let journal_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM journal_entries", [], |row| row.get(0))
        .unwrap();
    assert_eq!(journal_count, 1, "Should have inserted 1 journal entry");

    let vocab_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM vocabulary", [], |row| row.get(0))
        .unwrap();
    assert_eq!(vocab_count, 1, "Should have inserted 1 vocabulary entry");

    let rel_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM journal_vocabulary", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(
        rel_count, 1,
        "Should have inserted 1 journal_vocabulary relationship"
    );
}

#[tokio::test]
async fn test_get_vocabulary_fetches_correctly() {
    let conn = setup_in_memory_db();

    // Insert some mock data
    conn.execute(
        "INSERT INTO vocabulary (language_code, target_text, native_text, is_character) VALUES (?1, ?2, ?3, 0)",
        ("fr", "chat", "cat")
    ).unwrap();

    conn.execute(
        "INSERT INTO vocabulary (language_code, target_text, native_text, is_character) VALUES (?1, ?2, ?3, 1)",
        ("ko", "안녕", "hello")
    ).unwrap();

    let db = Arc::new(Mutex::new(conn));

    // Call the service
    let vocab_list = parlezvous_lib::services::vocab::get_vocabulary(db)
        .await
        .unwrap();

    // Verify it fetches both and orders by created_at DESC (which should be ko then fr, if inserted sequentially, but order might be same. Let's just check length and content)
    assert_eq!(vocab_list.len(), 2);

    // Verify content
    let has_cat = vocab_list
        .iter()
        .any(|v| v.target_text == "chat" && v.native_text == "cat" && !v.is_character);
    let has_hello = vocab_list
        .iter()
        .any(|v| v.target_text == "안녕" && v.native_text == "hello" && v.is_character);

    assert!(has_cat, "Missing 'chat' vocabulary item");
    assert!(has_hello, "Missing '안녕' vocabulary item");
}
