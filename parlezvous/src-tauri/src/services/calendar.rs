use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Debug)]
pub struct JournalEntryDTO {
    pub id: i64,
    pub language_code: String,
    pub date: String,
    pub mood_input: String,
    pub weather_input: String,
    pub activity_input: String,
    pub generated_target_text: String,
    pub native_translation: String,
}

pub async fn get_journal_entries(
    db: Arc<Mutex<Connection>>,
) -> Result<Vec<JournalEntryDTO>, String> {
    tokio::task::spawn_blocking(move || {
        let conn = db.lock().map_err(|_| "DB lock failed")?;
        
        let mut stmt = conn.prepare(
            "SELECT id, language_code, date, mood_input, weather_input, activity_input, generated_target_text, native_translation FROM journal_entries ORDER BY date DESC, id DESC"
        ).map_err(|e| e.to_string())?;
        
        let rows = stmt.query_map([], |row| {
            Ok(JournalEntryDTO {
                id: row.get(0)?,
                language_code: row.get(1)?,
                date: row.get(2)?,
                mood_input: row.get(3)?,
                weather_input: row.get(4)?,
                activity_input: row.get(5)?,
                generated_target_text: row.get(6)?,
                native_translation: row.get(7)?,
            })
        }).map_err(|e| e.to_string())?;

        let mut entries = Vec::new();
        for item in rows {
            entries.push(item.map_err(|e| e.to_string())?);
        }
        
        Ok::<Vec<JournalEntryDTO>, String>(entries)
    }).await.map_err(|e| e.to_string())?
}
