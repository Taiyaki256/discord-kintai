use crate::database::models::{User, AttendanceRecord, WorkSession, RecordType};
use sqlx::{SqlitePool, Row};
use chrono::{DateTime, Utc, NaiveDate};
use anyhow::Result;

// User queries using simpler API without macros
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
    let result = sqlx::query(
        "INSERT INTO users (discord_id, username) VALUES (?, ?)"
    )
    .bind(discord_id)
    .bind(username)
    .execute(pool)
    .await?;

    let user_id = result.last_insert_rowid();
    get_user_by_id(pool, user_id).await
}

pub async fn get_user_by_discord_id(pool: &SqlitePool, discord_id: &str) -> Result<User> {
    let row = sqlx::query(
        "SELECT id, discord_id, username, created_at FROM users WHERE discord_id = ?"
    )
    .bind(discord_id)
    .fetch_one(pool)
    .await?;

    Ok(User {
        id: row.get("id"),
        discord_id: row.get("discord_id"),
        username: row.get("username"),
        created_at: row.get("created_at"),
    })
}

pub async fn get_user_by_id(pool: &SqlitePool, user_id: i64) -> Result<User> {
    let row = sqlx::query(
        "SELECT id, discord_id, username, created_at FROM users WHERE id = ?"
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(User {
        id: row.get("id"),
        discord_id: row.get("discord_id"),
        username: row.get("username"),
        created_at: row.get("created_at"),
    })
}

// Attendance record queries
pub async fn create_attendance_record(
    pool: &SqlitePool,
    user_id: i64,
    record_type: RecordType,
    timestamp: DateTime<Utc>,
) -> Result<AttendanceRecord> {
    let record_type_str = record_type.as_str();
    
    let result = sqlx::query(
        "INSERT INTO attendance_records (user_id, record_type, timestamp) VALUES (?, ?, ?)"
    )
    .bind(user_id)
    .bind(record_type_str)
    .bind(timestamp)
    .execute(pool)
    .await?;

    let record_id = result.last_insert_rowid();
    get_attendance_record_by_id(pool, record_id).await
}

pub async fn get_attendance_record_by_id(pool: &SqlitePool, record_id: i64) -> Result<AttendanceRecord> {
    let row = sqlx::query(
        "SELECT id, user_id, record_type, timestamp, is_modified, original_timestamp, created_at, updated_at 
         FROM attendance_records WHERE id = ?"
    )
    .bind(record_id)
    .fetch_one(pool)
    .await?;

    Ok(AttendanceRecord {
        id: row.get("id"),
        user_id: row.get("user_id"),
        record_type: row.get("record_type"),
        timestamp: row.get("timestamp"),
        is_modified: row.get("is_modified"),
        original_timestamp: row.get("original_timestamp"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

pub async fn get_today_records(
    pool: &SqlitePool,
    user_id: i64,
    date: NaiveDate,
) -> Result<Vec<AttendanceRecord>> {
    let start_of_day = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
    let end_of_day = date.and_hms_opt(23, 59, 59).unwrap().and_utc();

    let rows = sqlx::query(
        "SELECT id, user_id, record_type, timestamp, is_modified, original_timestamp, created_at, updated_at 
         FROM attendance_records 
         WHERE user_id = ? AND timestamp >= ? AND timestamp <= ?
         ORDER BY timestamp ASC"
    )
    .bind(user_id)
    .bind(start_of_day)
    .bind(end_of_day)
    .fetch_all(pool)
    .await?;

    let records = rows.into_iter().map(|row| {
        AttendanceRecord {
            id: row.get("id"),
            user_id: row.get("user_id"),
            record_type: row.get("record_type"),
            timestamp: row.get("timestamp"),
            is_modified: row.get("is_modified"),
            original_timestamp: row.get("original_timestamp"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }).collect();

    Ok(records)
}

// Work session queries
pub async fn create_work_session(
    pool: &SqlitePool,
    user_id: i64,
    start_time: DateTime<Utc>,
    date: NaiveDate,
) -> Result<WorkSession> {
    let result = sqlx::query(
        "INSERT INTO work_sessions (user_id, start_time, date) VALUES (?, ?, ?)"
    )
    .bind(user_id)
    .bind(start_time)
    .bind(date)
    .execute(pool)
    .await?;

    let session_id = result.last_insert_rowid();
    get_work_session_by_id(pool, session_id).await
}

pub async fn get_work_session_by_id(pool: &SqlitePool, session_id: i64) -> Result<WorkSession> {
    let row = sqlx::query(
        "SELECT id, user_id, start_time, end_time, total_minutes, date, is_completed, created_at, updated_at 
         FROM work_sessions WHERE id = ?"
    )
    .bind(session_id)
    .fetch_one(pool)
    .await?;

    Ok(WorkSession {
        id: row.get("id"),
        user_id: row.get("user_id"),
        start_time: row.get("start_time"),
        end_time: row.get("end_time"),
        total_minutes: row.get("total_minutes"),
        date: row.get("date"),
        is_completed: row.get("is_completed"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

pub async fn get_active_work_session(pool: &SqlitePool, user_id: i64) -> Result<Option<WorkSession>> {
    let row_opt = sqlx::query(
        "SELECT id, user_id, start_time, end_time, total_minutes, date, is_completed, created_at, updated_at 
         FROM work_sessions 
         WHERE user_id = ? AND is_completed = FALSE 
         ORDER BY start_time DESC 
         LIMIT 1"
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    match row_opt {
        Some(row) => Ok(Some(WorkSession {
            id: row.get("id"),
            user_id: row.get("user_id"),
            start_time: row.get("start_time"),
            end_time: row.get("end_time"),
            total_minutes: row.get("total_minutes"),
            date: row.get("date"),
            is_completed: row.get("is_completed"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })),
        None => Ok(None),
    }
}

pub async fn complete_work_session(
    pool: &SqlitePool,
    session_id: i64,
    end_time: DateTime<Utc>,
) -> Result<()> {
    // Get current session to calculate duration
    let session = get_work_session_by_id(pool, session_id).await?;
    let duration = end_time.signed_duration_since(session.start_time);
    let total_minutes = duration.num_minutes() as i32;

    sqlx::query(
        "UPDATE work_sessions 
         SET end_time = ?, total_minutes = ?, is_completed = TRUE, updated_at = CURRENT_TIMESTAMP 
         WHERE id = ?"
    )
    .bind(end_time)
    .bind(total_minutes)
    .bind(session_id)
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
    let rows = sqlx::query(
        "SELECT id, user_id, start_time, end_time, total_minutes, date, is_completed, created_at, updated_at 
         FROM work_sessions 
         WHERE user_id = ? AND date >= ? AND date <= ?
         ORDER BY date ASC, start_time ASC"
    )
    .bind(user_id)
    .bind(start_date)
    .bind(end_date)
    .fetch_all(pool)
    .await?;

    let sessions = rows.into_iter().map(|row| {
        WorkSession {
            id: row.get("id"),
            user_id: row.get("user_id"),
            start_time: row.get("start_time"),
            end_time: row.get("end_time"),
            total_minutes: row.get("total_minutes"),
            date: row.get("date"),
            is_completed: row.get("is_completed"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }).collect();

    Ok(sessions)
}