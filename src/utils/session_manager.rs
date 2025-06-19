use crate::database::models::{AttendanceRecord, RecordType};
use crate::database::queries;
use chrono::{DateTime, Utc, NaiveDate};
use sqlx::SqlitePool;
use anyhow::Result;

pub struct SessionManager {
    pool: SqlitePool,
}

impl SessionManager {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 指定ユーザーの指定日のセッションを再計算
    pub async fn recalculate_sessions(&self, user_id: i64, date: NaiveDate) -> Result<()> {
        // 1. 既存のセッションをすべて削除
        self.delete_existing_sessions(user_id, date).await?;

        // 2. その日の記録を取得（時系列順）
        let records = queries::get_today_records(&self.pool, user_id, date).await?;

        // 3. 記録からセッションを再構築
        let sessions = self.build_sessions_from_records(records)?;

        // 4. 新しいセッションをデータベースに保存
        for session_data in sessions {
            self.create_session(user_id, session_data, date).await?;
        }

        Ok(())
    }

    /// 既存のセッションを削除
    async fn delete_existing_sessions(&self, user_id: i64, date: NaiveDate) -> Result<()> {
        sqlx::query(
            "DELETE FROM work_sessions WHERE user_id = ? AND date = ?"
        )
        .bind(user_id)
        .bind(date)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 記録からセッションデータを構築
    fn build_sessions_from_records(&self, records: Vec<AttendanceRecord>) -> Result<Vec<SessionData>> {
        let mut sessions = Vec::new();
        let mut current_start: Option<DateTime<Utc>> = None;

        for record in records {
            match RecordType::from(record.record_type) {
                RecordType::Start => {
                    // 既に開始済みの場合は警告（後で検証機能で対応）
                    if current_start.is_some() {
                        tracing::warn!("Multiple start records without end: user_id={}, record_id={}", record.user_id, record.id);
                    }
                    current_start = Some(record.timestamp);
                }
                RecordType::End => {
                    if let Some(start_time) = current_start.take() {
                        // ペア完成
                        let total_minutes = record.timestamp
                            .signed_duration_since(start_time)
                            .num_minutes() as i32;

                        sessions.push(SessionData {
                            start_time,
                            end_time: Some(record.timestamp),
                            total_minutes: Some(total_minutes),
                            is_completed: true,
                        });
                    } else {
                        // 開始なしの終了記録（後で検証機能で対応）
                        tracing::warn!("End record without start: user_id={}, record_id={}", record.user_id, record.id);
                    }
                }
            }
        }

        // 未完了のセッション（開始のみ）
        if let Some(start_time) = current_start {
            sessions.push(SessionData {
                start_time,
                end_time: None,
                total_minutes: None,
                is_completed: false,
            });
        }

        Ok(sessions)
    }

    /// セッションをデータベースに作成
    async fn create_session(&self, user_id: i64, session_data: SessionData, date: NaiveDate) -> Result<()> {
        sqlx::query(
            "INSERT INTO work_sessions (user_id, start_time, end_time, total_minutes, date, is_completed)
             VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(user_id)
        .bind(session_data.start_time)
        .bind(session_data.end_time)
        .bind(session_data.total_minutes)
        .bind(date)
        .bind(session_data.is_completed)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 記録追加・修正・削除後のセッション再計算のトリガー
    pub async fn trigger_recalculation(&self, user_id: i64, affected_date: NaiveDate) -> Result<()> {
        tracing::info!("Triggering session recalculation for user_id={}, date={}", user_id, affected_date);
        self.recalculate_sessions(user_id, affected_date).await
    }
}

#[derive(Debug)]
struct SessionData {
    start_time: DateTime<Utc>,
    end_time: Option<DateTime<Utc>>,
    total_minutes: Option<i32>,
    is_completed: bool,
}