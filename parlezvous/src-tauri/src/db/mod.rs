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
        date DATE DEFAULT CURRENT_DATE,
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

    PRAGMA user_version = 1;
";

pub const SCHEMA_V2: &str = "
    CREATE TABLE settings (
        id INTEGER PRIMARY KEY CHECK (id = 1),
        target_language TEXT NOT NULL DEFAULT 'French',
        tts_server_url TEXT NOT NULL DEFAULT 'http://localhost:5050/v1/audio/speech'
    );
    INSERT INTO settings (id) VALUES (1);
    PRAGMA user_version = 2;
";

pub const SCHEMA_V3: &str = "
    CREATE TABLE user_profile (
        id INTEGER PRIMARY KEY CHECK (id = 1),
        skill_level TEXT NOT NULL DEFAULT 'Beginner'
    );
    INSERT INTO user_profile (id) VALUES (1);

    CREATE TABLE document_chunks (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        document_name TEXT,
        chunk_text TEXT
    );

    CREATE VIRTUAL TABLE vec_chunks USING vec0(
        embedding float[768]
    );

    PRAGMA user_version = 3;
";

pub const SCHEMA_V4: &str = "
    CREATE TABLE conjugation_history (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        language_code TEXT NOT NULL,
        verb TEXT NOT NULL,
        tense TEXT NOT NULL,
        pronoun TEXT NOT NULL,
        answer TEXT NOT NULL,
        english_translation TEXT NOT NULL,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP
    );

    PRAGMA user_version = 4;
";

pub const SCHEMA_V5: &str = "
    ALTER TABLE conjugation_history ADD COLUMN answered_correct BOOLEAN DEFAULT NULL;

    PRAGMA user_version = 5;
";

pub const SCHEMA_V6: &str = "
    ALTER TABLE settings ADD COLUMN asr_server_url TEXT NOT NULL DEFAULT 'http://127.0.0.1:8000/v1/audio/transcriptions';

    PRAGMA user_version = 6;
";

pub const SCHEMA_V7: &str = "
    ALTER TABLE settings ADD COLUMN ollama_server_url TEXT NOT NULL DEFAULT 'http://localhost:11434';

    PRAGMA user_version = 7;
";

pub const SCHEMA_V8: &str = "
    ALTER TABLE user_profile ADD COLUMN tier INTEGER NOT NULL DEFAULT 1;
    ALTER TABLE user_profile ADD COLUMN active_seconds INTEGER NOT NULL DEFAULT 0;

    PRAGMA user_version = 8;
";

pub const SCHEMA_V9: &str = "
    ALTER TABLE document_chunks ADD COLUMN page_number INTEGER NOT NULL DEFAULT 1;

    PRAGMA user_version = 9;
";

pub const SCHEMA_V10: &str = "
    ALTER TABLE settings ADD COLUMN embedding_model TEXT NOT NULL DEFAULT 'nomic-embed-text';

    PRAGMA user_version = 10;
";

#[cfg(target_os = "android")]
pub const SCHEMA_V11: &str = "
    ALTER TABLE settings ADD COLUMN active_model TEXT NOT NULL DEFAULT 'gemma-4-E2B-it.litertlm';

    PRAGMA user_version = 11;
";

#[cfg(not(target_os = "android"))]
pub const SCHEMA_V11: &str = "
    ALTER TABLE settings ADD COLUMN active_model TEXT NOT NULL DEFAULT '';

    PRAGMA user_version = 11;
";

pub const SCHEMA_V12: &str = "
    CREATE TABLE IF NOT EXISTS language_curriculum (
        language TEXT PRIMARY KEY,
        current_tier INTEGER DEFAULT 1,
        active_theme_id TEXT NOT NULL,
        total_xp INTEGER DEFAULT 0,
        active_seconds INTEGER DEFAULT 0,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
        last_practiced DATETIME DEFAULT CURRENT_TIMESTAMP
    );

    PRAGMA user_version = 12;
";

pub const SCHEMA_V13: &str = "
    ALTER TABLE settings ADD COLUMN huggingface_token TEXT DEFAULT '';
    PRAGMA user_version = 13;
";

pub const SCHEMA_V14: &str = "
    ALTER TABLE settings ADD COLUMN litert_accelerator TEXT NOT NULL DEFAULT 'Auto';
    PRAGMA user_version = 14;
";

pub const SCHEMA_V15: &str = "
    ALTER TABLE settings ADD COLUMN litert_max_tokens INTEGER NOT NULL DEFAULT 5000;
    PRAGMA user_version = 15;
";

pub const SCHEMA_V16: &str = "
    ALTER TABLE conjugation_history RENAME COLUMN pronoun TO subject;
    ALTER TABLE conjugation_history RENAME COLUMN english_translation TO translation;
    ALTER TABLE conjugation_history ADD COLUMN sentence TEXT NOT NULL DEFAULT '';
    PRAGMA user_version = 16;
";

pub const SCHEMA_V17: &str = "
    PRAGMA user_version = 17;
";

const DB_VERSION_NUM: usize = 17;

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

    let schemas = [
        SCHEMA_V1, SCHEMA_V2, SCHEMA_V3, SCHEMA_V4, SCHEMA_V5, SCHEMA_V6, SCHEMA_V7, SCHEMA_V8,
        SCHEMA_V9, SCHEMA_V10, SCHEMA_V11, SCHEMA_V12, SCHEMA_V13, SCHEMA_V14, SCHEMA_V15, SCHEMA_V16, SCHEMA_V17,
    ];

    for i in 0..DB_VERSION_NUM {
        // Migration: set user_version
        let user_version: i32 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .map_err(|e| e.to_string())?;

        if user_version == i as i32 {
            conn.execute_batch(schemas[i]).map_err(|e| e.to_string())?;

            // Post-schema migrations
            if i == 11 {
                // Migrating from V11 to V12
                let _ = conn.execute(
                    "INSERT INTO language_curriculum (language, current_tier, active_theme_id, total_xp, active_seconds)
                     SELECT s.target_language, u.tier, 'greetings', 0, u.active_seconds
                     FROM settings s, user_profile u
                     WHERE u.id = 1 AND s.id = 1 AND u.active_seconds > 0
                     ON CONFLICT DO NOTHING",
                    []
                );
            } else if i == 16 {
                // Migrating from V16 to V17 (Normalize conjugation history tenses)
                if let Ok(mut stmt) = conn.prepare("SELECT DISTINCT tense FROM conjugation_history") {
                    if let Ok(rows) = stmt.query_map([], |r| r.get::<_, String>(0)) {
                        for tense_res in rows {
                            if let Ok(tense) = tense_res {
                                let mut new_tense = tense.clone();
                                if let Some(idx) = new_tense.find('(') {
                                    new_tense = new_tense[..idx].trim().to_string();
                                }
                                new_tense = new_tense.to_lowercase();
                                if new_tense != tense {
                                    let _ = conn.execute("UPDATE conjugation_history SET tense = ?1 WHERE tense = ?2", rusqlite::params![new_tense, tense]);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(conn)
}
