use chrono::{DateTime, Utc, NaiveDate};
use serde::{Serialize, Deserialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub discord_id: String,
    pub username: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AttendanceRecord {
    pub id: i64,
    pub user_id: i64,
    pub record_type: String, // "start" or "end"
    pub timestamp: DateTime<Utc>,
    pub is_modified: bool,
    pub original_timestamp: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct WorkSession {
    pub id: i64,
    pub user_id: i64,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub total_minutes: Option<i32>,
    pub date: NaiveDate,
    pub is_completed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy)]
pub enum RecordType {
    Start,
    End,
}

impl RecordType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RecordType::Start => "start",
            RecordType::End => "end",
        }
    }
}

impl From<String> for RecordType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "start" => RecordType::Start,
            "end" => RecordType::End,
            _ => panic!("Invalid record type: {}", s),
        }
    }
}