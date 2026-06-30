use rusqlite::ffi::sqlite3_auto_extension;
use rusqlite::types::{FromSql, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use sqlite_vec::sqlite3_vec_init;
use tauri::Manager;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VocabId(pub i64);

impl ToSql for VocabId {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        self.0.to_sql()
    }
}
impl FromSql for VocabId {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_i64().map(VocabId)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct JournalId(pub i64);

impl ToSql for JournalId {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        self.0.to_sql()
    }
}
impl FromSql for JournalId {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_i64().map(JournalId)
    }
}

#[cfg(target_os = "android")]
pub const SCHEMA_V1: &str = "
    CREATE TABLE vocabulary (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        language_code TEXT NOT NULL,
        target_text TEXT NOT NULL,
        native_text TEXT NOT NULL,
        is_character BOOLEAN DEFAULT 0,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP
    );

    CREATE TABLE srs_state (
        vocab_id INTEGER PRIMARY KEY,
        review_level INTEGER DEFAULT 0,
        next_review_date DATETIME,
        ease_factor REAL DEFAULT 2.5,
        interval_days INTEGER DEFAULT 0,
        FOREIGN KEY(vocab_id) REFERENCES vocabulary(id) ON DELETE CASCADE
    );

    CREATE TABLE journal_entries (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        language_code TEXT NOT NULL,
        date DATE DEFAULT CURRENT_TIMESTAMP,
        mood_input TEXT,
        weather_input TEXT,
        activity_input TEXT,
        generated_target_text TEXT,
        native_translation TEXT
    );

    CREATE TABLE journal_vocabulary (
        journal_id INTEGER,
        vocab_id INTEGER,
        PRIMARY KEY (journal_id, vocab_id),
        FOREIGN KEY(journal_id) REFERENCES journal_entries(id) ON DELETE CASCADE,
        FOREIGN KEY(vocab_id) REFERENCES vocabulary(id) ON DELETE CASCADE
    );

    CREATE TABLE settings (
        id INTEGER PRIMARY KEY CHECK (id = 1),
        target_language TEXT NOT NULL DEFAULT 'French',
        tts_server_url TEXT NOT NULL DEFAULT 'http://localhost:5050/v1/audio/speech',
        asr_server_url TEXT NOT NULL DEFAULT 'http://127.0.0.1:8000/v1/audio/transcriptions',
        ollama_server_url TEXT NOT NULL DEFAULT 'http://localhost:11434',
        embedding_model TEXT NOT NULL DEFAULT 'nomic-embed-text',
        active_model TEXT NOT NULL DEFAULT 'gemma-4-E2B-it.litertlm',
        huggingface_token TEXT DEFAULT '',
        litert_accelerator TEXT NOT NULL DEFAULT 'Auto',
        litert_max_tokens INTEGER NOT NULL DEFAULT 5000
    );
    INSERT INTO settings (id) VALUES (1);

    CREATE TABLE user_profile (
        id INTEGER PRIMARY KEY CHECK (id = 1),
        skill_level TEXT NOT NULL DEFAULT 'Beginner',
        tier INTEGER NOT NULL DEFAULT 1,
        active_seconds INTEGER NOT NULL DEFAULT 0
    );
    INSERT INTO user_profile (id) VALUES (1);

    CREATE TABLE document_chunks (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        document_name TEXT,
        chunk_text TEXT,
        page_number INTEGER NOT NULL DEFAULT 1
    );

    CREATE VIRTUAL TABLE vec_chunks USING vec0(
        embedding float[768]
    );

    CREATE TABLE conjugation_history (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        language_code TEXT NOT NULL,
        verb TEXT NOT NULL,
        tense TEXT NOT NULL,
        subject TEXT NOT NULL,
        answer TEXT NOT NULL,
        translation TEXT NOT NULL,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
        answered_correct BOOLEAN DEFAULT NULL,
        sentence TEXT NOT NULL DEFAULT ''
    );

    CREATE TABLE IF NOT EXISTS language_curriculum (
        language TEXT PRIMARY KEY,
        current_tier INTEGER DEFAULT 1,
        active_theme_id TEXT NOT NULL,
        total_xp INTEGER DEFAULT 0,
        active_seconds INTEGER DEFAULT 0,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
        last_practiced DATETIME DEFAULT CURRENT_TIMESTAMP
    );

    PRAGMA user_version = 1;
";

#[cfg(not(target_os = "android"))]
pub const SCHEMA_V1: &str = "
    CREATE TABLE vocabulary (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        language_code TEXT NOT NULL,
        target_text TEXT NOT NULL,
        native_text TEXT NOT NULL,
        is_character BOOLEAN DEFAULT 0,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP
    );

    CREATE TABLE srs_state (
        vocab_id INTEGER PRIMARY KEY,
        review_level INTEGER DEFAULT 0,
        next_review_date DATETIME,
        ease_factor REAL DEFAULT 2.5,
        interval_days INTEGER DEFAULT 0,
        FOREIGN KEY(vocab_id) REFERENCES vocabulary(id) ON DELETE CASCADE
    );

    CREATE TABLE journal_entries (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        language_code TEXT NOT NULL,
        date DATE DEFAULT CURRENT_TIMESTAMP,
        mood_input TEXT,
        weather_input TEXT,
        activity_input TEXT,
        generated_target_text TEXT,
        native_translation TEXT
    );

    CREATE TABLE journal_vocabulary (
        journal_id INTEGER,
        vocab_id INTEGER,
        PRIMARY KEY (journal_id, vocab_id),
        FOREIGN KEY(journal_id) REFERENCES journal_entries(id) ON DELETE CASCADE,
        FOREIGN KEY(vocab_id) REFERENCES vocabulary(id) ON DELETE CASCADE
    );

    CREATE TABLE settings (
        id INTEGER PRIMARY KEY CHECK (id = 1),
        target_language TEXT NOT NULL DEFAULT 'French',
        tts_server_url TEXT NOT NULL DEFAULT 'http://localhost:5050/v1/audio/speech',
        asr_server_url TEXT NOT NULL DEFAULT 'http://127.0.0.1:8000/v1/audio/transcriptions',
        ollama_server_url TEXT NOT NULL DEFAULT 'http://localhost:11434',
        embedding_model TEXT NOT NULL DEFAULT 'nomic-embed-text',
        active_model TEXT NOT NULL DEFAULT '',
        huggingface_token TEXT DEFAULT '',
        litert_accelerator TEXT NOT NULL DEFAULT 'Auto',
        litert_max_tokens INTEGER NOT NULL DEFAULT 5000
    );
    INSERT INTO settings (id) VALUES (1);

    CREATE TABLE user_profile (
        id INTEGER PRIMARY KEY CHECK (id = 1),
        skill_level TEXT NOT NULL DEFAULT 'Beginner',
        tier INTEGER NOT NULL DEFAULT 1,
        active_seconds INTEGER NOT NULL DEFAULT 0
    );
    INSERT INTO user_profile (id) VALUES (1);

    CREATE TABLE document_chunks (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        document_name TEXT,
        chunk_text TEXT,
        page_number INTEGER NOT NULL DEFAULT 1
    );

    CREATE VIRTUAL TABLE vec_chunks USING vec0(
        embedding float[768]
    );

    CREATE TABLE conjugation_history (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        language_code TEXT NOT NULL,
        verb TEXT NOT NULL,
        tense TEXT NOT NULL,
        subject TEXT NOT NULL,
        answer TEXT NOT NULL,
        translation TEXT NOT NULL,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
        answered_correct BOOLEAN DEFAULT NULL,
        sentence TEXT NOT NULL DEFAULT ''
    );

    CREATE TABLE IF NOT EXISTS language_curriculum (
        language TEXT PRIMARY KEY,
        current_tier INTEGER DEFAULT 1,
        active_theme_id TEXT NOT NULL,
        total_xp INTEGER DEFAULT 0,
        active_seconds INTEGER DEFAULT 0,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
        last_practiced DATETIME DEFAULT CURRENT_TIMESTAMP
    );

    PRAGMA user_version = 1;
";

const DB_VERSION_NUM: usize = 1;

pub fn init_db(app_handle: &tauri::AppHandle) -> Result<Connection, String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app_data_dir: {}", e))?;

    eprintln!("📂 DEBUG: app_data_dir resolved to: {:?}", app_dir);
    eprintln!("📂 DEBUG: app_data_dir exists: {}", app_dir.exists());

    // CRITICAL: On some Android devices the base app_data_dir doesn't exist yet
    // at onActivityCreate time. We must create it explicitly before any I/O.
    if !app_dir.exists() {
        eprintln!("📂 DEBUG: Creating app_data_dir...");
        std::fs::create_dir_all(&app_dir)
            .map_err(|e| format!("Failed to create base app_data_dir {:?}: {}", app_dir, e))?;
    }

    let db_path = app_dir.join("sqlite.db");
    eprintln!("📂 DEBUG: Opening database at: {:?}", db_path);

    // Load sqlite-vec extension before opening the connection
    unsafe {
        sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_vec_init as *const ())));
    }

    let conn = Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database at {:?}: {}", db_path, e))?;

    let schemas = [SCHEMA_V1];

    for i in 0..DB_VERSION_NUM {
        // Migration: set user_version
        let user_version: i32 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .map_err(|e| e.to_string())?;

        if user_version == i as i32 {
            conn.execute_batch(schemas[i]).map_err(|e| e.to_string())?;
        }
    }

    Ok(conn)
}
