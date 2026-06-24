use crate::db::VocabId;
use crate::services::journal::DbVocabItem;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

pub async fn get_vocabulary(db: Arc<Mutex<Connection>>) -> Result<Vec<DbVocabItem>, String> {
    tokio::task::spawn_blocking(move || {
        let conn = db.lock().map_err(|_| "DB lock failed")?;
        
        let mut stmt = conn.prepare(
            "SELECT id, language_code, target_text, native_text, is_character FROM vocabulary ORDER BY created_at DESC"
        ).map_err(|e| e.to_string())?;
        
        let rows = stmt.query_map([], |row| {
            Ok(DbVocabItem {
                id: VocabId(row.get(0)?),
                target_text: row.get(2)?,
                native_text: row.get(3)?,
                is_character: row.get(4)?,
            })
        }).map_err(|e| e.to_string())?;

        let mut vocab_list = Vec::new();
        for item in rows {
            vocab_list.push(item.map_err(|e| e.to_string())?);
        }
        
        Ok::<Vec<DbVocabItem>, String>(vocab_list)
    }).await.map_err(|e| e.to_string())?
}
