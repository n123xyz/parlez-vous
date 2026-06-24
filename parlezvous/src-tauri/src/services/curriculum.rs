use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize)]
pub struct LanguageCurriculum {
    pub language: String,
    pub current_tier: i32,
    pub active_theme_id: String,
    pub total_xp: i32,
    pub active_seconds: i32,
}

pub fn get_curriculum(
    db: Arc<Mutex<Connection>>,
    language: String,
) -> Result<LanguageCurriculum, String> {
    let conn = db.lock().map_err(|_| "Failed to lock database")?;

    // Create default if not exists
    let mut stmt = conn.prepare("SELECT current_tier, active_theme_id, total_xp, active_seconds FROM language_curriculum WHERE language = ?1").map_err(|e| e.to_string())?;

    let result = stmt.query_row(rusqlite::params![language], |row| {
        Ok(LanguageCurriculum {
            language: language.clone(),
            current_tier: row.get(0)?,
            active_theme_id: row.get(1)?,
            total_xp: row.get(2)?,
            active_seconds: row.get(3)?,
        })
    });

    match result {
        Ok(curr) => Ok(curr),
        Err(_) => {
            // Insert default
            conn.execute(
                "INSERT INTO language_curriculum (language, current_tier, active_theme_id, total_xp, active_seconds) VALUES (?1, 1, 'greetings', 0, 0)",
                rusqlite::params![language],
            ).map_err(|e| e.to_string())?;
            Ok(LanguageCurriculum {
                language,
                current_tier: 1,
                active_theme_id: "greetings".to_string(),
                total_xp: 0,
                active_seconds: 0,
            })
        }
    }
}

pub fn add_time_xp(
    db: Arc<Mutex<Connection>>,
    language: String,
    seconds: i32,
) -> Result<(), String> {
    let conn = db.lock().map_err(|_| "Failed to lock database")?;

    // 1. Calculate Conjugation Multiplier (7 days)
    let mut accuracy_stmt = conn.prepare(
        "SELECT COUNT(*) as total, SUM(CASE WHEN answered_correct = 1 THEN 1 ELSE 0 END) as correct 
         FROM conjugation_history 
         WHERE language_code = ?1 AND created_at > datetime('now', '-7 days')"
    ).map_err(|e| e.to_string())?;

    let (total, correct): (i32, i32) = accuracy_stmt
        .query_row(rusqlite::params![language], |row| {
            Ok((row.get(0).unwrap_or(0), row.get(1).unwrap_or(0)))
        })
        .unwrap_or((0, 0));

    let mut multiplier = 1.0;
    if total > 0 {
        let accuracy = (correct as f64) / (total as f64);
        if accuracy > 0.85 {
            multiplier = 1.2;
        } else if accuracy < 0.50 {
            multiplier = 0.5;
        }
    }

    let xp_earned = (seconds as f64 * multiplier).round() as i32;

    // 2. Update DB. We use ON CONFLICT DO UPDATE just in case but it should exist because it's fetched usually.
    // However, if the user hasn't opened the map page, it might not exist. Let's ensure it exists.
    conn.execute(
        "INSERT INTO language_curriculum (language, current_tier, active_theme_id, total_xp, active_seconds) 
         VALUES (?3, 1, 'greetings', ?2, ?1)
         ON CONFLICT(language) DO UPDATE SET 
             active_seconds = active_seconds + ?1, 
             total_xp = total_xp + ?2,
             last_practiced = CURRENT_TIMESTAMP",
        rusqlite::params![seconds, xp_earned, language],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

pub fn set_active_theme(
    db: Arc<Mutex<Connection>>,
    language: String,
    theme_id: String,
) -> Result<(), String> {
    let conn = db.lock().map_err(|_| "Failed to lock database")?;
    conn.execute(
        "UPDATE language_curriculum SET active_theme_id = ?1, last_practiced = CURRENT_TIMESTAMP WHERE language = ?2",
        rusqlite::params![theme_id, language],
    ).map_err(|e| e.to_string())?;
    Ok(())
}
