use rusqlite::Connection;
use std::sync::{Arc, Mutex};

pub fn get_skill_level(db: Arc<Mutex<Connection>>) -> Result<String, String> {
    let conn = db.lock().map_err(|_| "Failed to lock database")?;
    let level: String = conn
        .query_row(
            "SELECT skill_level FROM user_profile WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| "Beginner".to_string());
    Ok(level)
}

pub fn update_skill_level(db: Arc<Mutex<Connection>>, level: String) -> Result<(), String> {
    let conn = db.lock().map_err(|_| "Failed to lock database")?;
    conn.execute(
        "UPDATE user_profile SET skill_level = ?1 WHERE id = 1",
        rusqlite::params![level],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(serde::Serialize)]
pub struct UserProfile {
    pub skill_level: String,
    pub tier: i32,
    pub active_seconds: i32,
}

pub fn get_profile(db: Arc<Mutex<Connection>>) -> Result<UserProfile, String> {
    let conn = db.lock().map_err(|_| "Failed to lock database")?;
    let profile = conn
        .query_row(
            "SELECT skill_level, tier, active_seconds FROM user_profile WHERE id = 1",
            [],
            |row| {
                Ok(UserProfile {
                    skill_level: row.get(0)?,
                    tier: row.get(1)?,
                    active_seconds: row.get(2)?,
                })
            },
        )
        .unwrap_or_else(|_| UserProfile {
            skill_level: "Beginner".to_string(),
            tier: 1,
            active_seconds: 0,
        });
    Ok(profile)
}

pub fn add_active_seconds(db: Arc<Mutex<Connection>>, seconds: i32) -> Result<(), String> {
    let conn = db.lock().map_err(|_| "Failed to lock database")?;
    conn.execute(
        "UPDATE user_profile SET active_seconds = active_seconds + ?1 WHERE id = 1",
        rusqlite::params![seconds],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
