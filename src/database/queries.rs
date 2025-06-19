use crate::database::models::{User, AttendanceRecord, WorkSession, RecordType};
use sqlx::SqlitePool;
use chrono::{DateTime, Utc, NaiveDate};
use anyhow::Result;

// User queries
pub async fn create_or_get_user(
    pool: &SqlitePool,
    discord_id: &str,
    username: &str,
) -> Result<User> {
    // Try to get existing user first
    if let Ok(user) = get_user_by_discord_id(pool, discord_id).await {
        return Ok(user);
    }

    // Create new user if not exists
    let user_id = sqlx::query!(
        "INSERT INTO users (discord_id, username) VALUES (?, ?)",
        discord_id,
        username
    )
    .execute(pool)
    .await?
    .last_insert_rowid();

    get_user_by_id(pool, user_id).await
}

pub async fn get_user_by_discord_id(pool: &SqlitePool, discord_id: &str) -> Result<User> {
    let user = sqlx::query_as!(
        User,
        "SELECT id, discord_id, username, created_at FROM users WHERE discord_id = ?",
        discord_id
    )
    .fetch_one(pool)
    .await?;

    Ok(user)
}

pub async fn get_user_by_id(pool: &SqlitePool, user_id: i64) -> Result<User> {
    let user = sqlx::query_as!(
        User,
        "SELECT id, discord_id, username, created_at FROM users WHERE id = ?",
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(user)
}

// Attendance record queries
pub async fn create_attendance_record(
    pool: &SqlitePool,
    user_id: i64,
    record_type: RecordType,
    timestamp: DateTime<Utc>,
) -> Result<AttendanceRecord> {
    let record_type_str = record_type.as_str();
    
    let record_id = sqlx::query!(
        "INSERT INTO attendance_records (user_id, record_type, timestamp) VALUES (?, ?, ?)",
        user_id,
        record_type_str,
        timestamp
    )
    .execute(pool)
    .await?
    .last_insert_rowid();

    get_attendance_record_by_id(pool, record_id).await
}

pub async fn get_attendance_record_by_id(pool: &SqlitePool, record_id: i64) -> Result<AttendanceRecord> {
    let record = sqlx::query_as!(
        AttendanceRecord,
        "SELECT id, user_id, record_type, timestamp, is_modified, original_timestamp, created_at, updated_at 
         FROM attendance_records WHERE id = ?",
        record_id
    )
    .fetch_one(pool)
    .await?;

    Ok(record)
}

pub async fn get_today_records(
    pool: &SqlitePool,
    user_id: i64,
    date: NaiveDate,
) -> Result<Vec<AttendanceRecord>> {
    let start_of_day = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
    let end_of_day = date.and_hms_opt(23, 59, 59).unwrap().and_utc();

    let records = sqlx::query_as!(
        AttendanceRecord,
        "SELECT id, user_id, record_type, timestamp, is_modified, original_timestamp, created_at, updated_at 
         FROM attendance_records 
         WHERE user_id = ? AND timestamp >= ? AND timestamp <= ?
         ORDER BY timestamp ASC",
        user_id,
        start_of_day,
        end_of_day
    )
    .fetch_all(pool)
    .await?;

    Ok(records)
}

pub async fn update_attendance_record_time(
    pool: &SqlitePool,
    record_id: i64,
    new_timestamp: DateTime<Utc>,
) -> Result<()> {
    // First get the current record to preserve original timestamp
    let current_record = get_attendance_record_by_id(pool, record_id).await?;
    let original_timestamp = if current_record.is_modified {
        current_record.original_timestamp
    } else {
        Some(current_record.timestamp)
    };

    sqlx::query!(
        "UPDATE attendance_records 
         SET timestamp = ?, is_modified = TRUE, original_timestamp = ?, updated_at = CURRENT_TIMESTAMP 
         WHERE id = ?",
        new_timestamp,
        original_timestamp,
        record_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_attendance_record(pool: &SqlitePool, record_id: i64) -> Result<()> {
    sqlx::query!("DELETE FROM attendance_records WHERE id = ?", record_id)
        .execute(pool)
        .await?;

    Ok(())
}

// Work session queries
pub async fn create_work_session(
    pool: &SqlitePool,
    user_id: i64,
    start_time: DateTime<Utc>,
    date: NaiveDate,
) -> Result<WorkSession> {
    let session_id = sqlx::query!(
        "INSERT INTO work_sessions (user_id, start_time, date) VALUES (?, ?, ?)",
        user_id,
        start_time,
        date
    )
    .execute(pool)
    .await?
    .last_insert_rowid();

    get_work_session_by_id(pool, session_id).await
}

pub async fn get_work_session_by_id(pool: &SqlitePool, session_id: i64) -> Result<WorkSession> {
    let session = sqlx::query_as!(
        WorkSession,
        "SELECT id, user_id, start_time, end_time, total_minutes, date, is_completed, created_at, updated_at 
         FROM work_sessions WHERE id = ?",
        session_id
    )
    .fetch_one(pool)
    .await?;

    Ok(session)
}

pub async fn get_active_work_session(pool: &SqlitePool, user_id: i64) -> Result<Option<WorkSession>> {
    let session = sqlx::query_as!(
        WorkSession,
        "SELECT id, user_id, start_time, end_time, total_minutes, date, is_completed, created_at, updated_at 
         FROM work_sessions 
         WHERE user_id = ? AND is_completed = FALSE 
         ORDER BY start_time DESC 
         LIMIT 1",
        user_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(session)
}

pub async fn complete_work_session(
    pool: &SqlitePool,
    session_id: i64,
    end_time: DateTime<Utc>,
) -> Result<()> {
    let total_minutes = sqlx::query_scalar!(
        "SELECT (julianday(?) - julianday(start_time)) * 24 * 60 
         FROM work_sessions WHERE id = ?",
        end_time,
        session_id
    )
    .fetch_one(pool)
    .await? as i32;

    sqlx::query!(
        "UPDATE work_sessions 
         SET end_time = ?, total_minutes = ?, is_completed = TRUE, updated_at = CURRENT_TIMESTAMP 
         WHERE id = ?",
        end_time,
        total_minutes,
        session_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_work_sessions_by_date_range(
    pool: &SqlitePool,
    user_id: i64,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Vec<WorkSession>> {
    let sessions = sqlx::query_as!(
        WorkSession,
        "SELECT id, user_id, start_time, end_time, total_minutes, date, is_completed, created_at, updated_at 
         FROM work_sessions 
         WHERE user_id = ? AND date >= ? AND date <= ?
         ORDER BY date ASC, start_time ASC",
        user_id,
        start_date,
        end_date
    )
    .fetch_all(pool)
    .await?;

    Ok(sessions)
}