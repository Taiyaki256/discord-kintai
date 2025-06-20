use crate::database::models::{AttendanceRecord, WorkSession};
use crate::utils::time::{format_duration_minutes, format_time_jst};
use chrono::{DateTime, Utc};
use poise::serenity_prelude as serenity;

pub fn format_attendance_status(records: &[AttendanceRecord]) -> String {
    if records.is_empty() {
        return "今日はまだ勤務記録がありません".to_string();
    }

    let mut status = String::new();
    let mut start_time: Option<DateTime<Utc>> = None;
    let mut total_minutes = 0i32;
    let mut session_count = 0;

    status.push_str("**本日の勤務記録:**\n");

    for record in records {
        match record.record_type.as_str() {
            "start" => {
                if start_time.is_some() {
                    // 前のセッションが未終了
                    status.push_str("  ⚠️ 前回の終了記録なし\n");
                }
                session_count += 1;
                status.push_str(&format!(
                    "#{} 🟢 **開始**: {} {}\n",
                    session_count,
                    format_time_jst(record.timestamp),
                    if record.is_modified {
                        "(修正済み)"
                    } else {
                        ""
                    }
                ));
                start_time = Some(record.timestamp);
            }
            "end" => {
                status.push_str(&format!(
                    "#{} 🔴 **終了**: {} {}\n",
                    session_count,
                    format_time_jst(record.timestamp),
                    if record.is_modified {
                        "(修正済み)"
                    } else {
                        ""
                    }
                ));

                if let Some(start) = start_time {
                    let duration =
                        record.timestamp.signed_duration_since(start).num_minutes() as i32;
                    total_minutes += duration;
                    status.push_str(&format!(
                        "#{} ⏱️ 勤務時間: {}\n",
                        session_count,
                        format_duration_minutes(duration)
                    ));
                } else {
                    status.push_str(&format!("#{} ⚠️ 対応する開始記録なし\n", session_count));
                }
                start_time = None;
                status.push('\n');
            }
            _ => {}
        }
    }

    // If still working
    if start_time.is_some() {
        status.push_str(&format!("#{} ⚠️ **現在勤務中**\n\n", session_count));
    }

    if total_minutes > 0 {
        status.push_str(&format!(
            "📊 **本日の合計勤務時間**: {}",
            format_duration_minutes(total_minutes)
        ));
    }

    if session_count > 1 {
        status.push_str(&format!("\n🔄 **セッション数**: {}", session_count));
    }

    status
}

pub fn format_work_sessions_summary(sessions: &[WorkSession]) -> String {
    if sessions.is_empty() {
        return "指定期間に勤務記録がありません".to_string();
    }

    let mut summary = String::new();
    let mut total_minutes = 0i32;
    let mut current_date: Option<chrono::NaiveDate> = None;
    let mut daily_minutes = 0i32;

    for session in sessions {
        // 日付が変わった場合の処理
        if current_date != Some(session.date) {
            // 前の日の合計を表示
            if let Some(prev_date) = current_date {
                if daily_minutes > 0 {
                    summary.push_str(&format!(
                        "   📊 **{}合計**: {}\n\n",
                        prev_date.format("%m/%d"),
                        format_duration_minutes(daily_minutes)
                    ));
                }
            }

            // 新しい日のヘッダー
            current_date = Some(session.date);
            daily_minutes = 0;
            summary.push_str(&format!(
                "📅 **{}**\n",
                session.date.format("%Y-%m-%d (%a)")
            ));
        }

        summary.push_str(&format!(
            "   🟢 開始: {}",
            format_time_jst(session.start_time)
        ));

        if let Some(end_time) = session.end_time {
            summary.push_str(&format!(" → 🔴 終了: {}", format_time_jst(end_time)));

            if let Some(minutes) = session.total_minutes {
                summary.push_str(&format!(" ({})", format_duration_minutes(minutes)));
                total_minutes += minutes;
                daily_minutes += minutes;
            }
            summary.push('\n');
        } else {
            summary.push_str(" → ⚠️ **未終了**\n");
        }
    }

    // 最後の日の合計を表示
    if let Some(last_date) = current_date {
        if daily_minutes > 0 {
            summary.push_str(&format!(
                "   📊 **{}合計**: {}\n\n",
                last_date.format("%m/%d"),
                format_duration_minutes(daily_minutes)
            ));
        }
    }

    if total_minutes > 0 {
        summary.push_str(&format!(
            "🎯 **総合計勤務時間**: {}",
            format_duration_minutes(total_minutes)
        ));
    }

    summary
}

pub fn format_error_message(error: &str) -> String {
    format!("❌ **エラー**: {}", error)
}

pub fn format_success_message(message: &str) -> String {
    format!("✅ {}", message)
}

pub fn format_info_message(message: &str) -> String {
    format!("ℹ️ {}", message)
}

// Embed utility functions
pub fn create_success_embed(title: &str, description: &str) -> serenity::CreateEmbed {
    serenity::CreateEmbed::new()
        .title(title)
        .description(description)
        .color(0x00ff00) // Green
        .timestamp(chrono::Utc::now())
}

pub fn create_error_embed(title: &str, description: &str) -> serenity::CreateEmbed {
    serenity::CreateEmbed::new()
        .title(title)
        .description(description)
        .color(0xff0000) // Red
        .timestamp(chrono::Utc::now())
}

pub fn create_info_embed(title: &str, description: &str) -> serenity::CreateEmbed {
    serenity::CreateEmbed::new()
        .title(title)
        .description(description)
        .color(0x3498db) // Blue
        .timestamp(chrono::Utc::now())
}

pub fn create_status_embed(
    username: &str,
    date: chrono::NaiveDate,
    records: &[AttendanceRecord],
) -> serenity::CreateEmbed {
    let status_text = format_attendance_status(records);
    serenity::CreateEmbed::new()
        .title("📊 勤務状況")
        .description(status_text)
        .color(0x3498db) // Blue
        .author(serenity::CreateEmbedAuthor::new(format!(
            "{} の勤務状況",
            username
        )))
        .footer(serenity::CreateEmbedFooter::new(
            date.format("%Y年%m月%d日").to_string(),
        ))
        .timestamp(chrono::Utc::now())
}

pub fn create_report_embed(
    username: &str,
    title: &str,
    date_range: &str,
    sessions: &[WorkSession],
) -> serenity::CreateEmbed {
    let report_text = format_work_sessions_summary(sessions);
    serenity::CreateEmbed::new()
        .title(format!("📅 {}", title))
        .description(report_text)
        .color(0x9b59b6) // Purple
        .author(serenity::CreateEmbedAuthor::new(format!(
            "{} のレポート",
            username
        )))
        .footer(serenity::CreateEmbedFooter::new(date_range))
        .timestamp(chrono::Utc::now())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, TimeZone};

    fn create_test_record(
        id: i64,
        record_type: &str,
        hour: u32,
        minute: u32,
        is_modified: bool,
    ) -> AttendanceRecord {
        let jst_offset = chrono::FixedOffset::east_opt(9 * 3600).unwrap();
        let date = NaiveDate::from_ymd_opt(2023, 12, 15).unwrap();
        let time = chrono::NaiveTime::from_hms_opt(hour, minute, 0).unwrap();
        let datetime = jst_offset
            .from_local_datetime(&date.and_time(time))
            .unwrap()
            .to_utc();

        AttendanceRecord {
            id,
            user_id: 1,
            record_type: record_type.to_string(),
            timestamp: datetime,
            is_modified,
            original_timestamp: None,
            created_at: datetime,
            updated_at: datetime,
        }
    }

    fn create_test_session(
        id: i64,
        start_hour: u32,
        start_minute: u32,
        end_hour: Option<u32>,
        end_minute: Option<u32>,
        date: NaiveDate,
    ) -> WorkSession {
        let jst_offset = chrono::FixedOffset::east_opt(9 * 3600).unwrap();
        let start_time = chrono::NaiveTime::from_hms_opt(start_hour, start_minute, 0).unwrap();
        let start_datetime = jst_offset
            .from_local_datetime(&date.and_time(start_time))
            .unwrap()
            .to_utc();

        let (end_time, total_minutes, is_completed) =
            if let (Some(eh), Some(em)) = (end_hour, end_minute) {
                let end_time = chrono::NaiveTime::from_hms_opt(eh, em, 0).unwrap();
                let end_datetime = jst_offset
                    .from_local_datetime(&date.and_time(end_time))
                    .unwrap()
                    .to_utc();
                let duration = end_datetime
                    .signed_duration_since(start_datetime)
                    .num_minutes() as i32;
                (Some(end_datetime), Some(duration), true)
            } else {
                (None, None, false)
            };

        WorkSession {
            id,
            user_id: 1,
            start_time: start_datetime,
            end_time,
            total_minutes,
            date,
            is_completed,
            created_at: start_datetime,
            updated_at: start_datetime,
        }
    }

    #[test]
    fn test_format_attendance_status_empty() {
        let records = vec![];
        let result = format_attendance_status(&records);
        assert_eq!(result, "今日はまだ勤務記録がありません");
    }

    #[test]
    fn test_format_attendance_status_single_complete_session() {
        let records = vec![
            create_test_record(1, "start", 9, 0, false),
            create_test_record(2, "end", 17, 30, false),
        ];
        let result = format_attendance_status(&records);

        assert!(result.contains("**本日の勤務記録:**"));
        assert!(result.contains("#1 🟢 **開始**: 09:00"));
        assert!(result.contains("#1 🔴 **終了**: 17:30"));
        assert!(result.contains("#1 ⏱️ 勤務時間: 8時間30分"));
        assert!(result.contains("📊 **本日の合計勤務時間**: 8時間30分"));
        assert!(!result.contains("(修正済み)"));
    }

    #[test]
    fn test_format_attendance_status_modified_records() {
        let records = vec![
            create_test_record(1, "start", 9, 0, true),
            create_test_record(2, "end", 17, 30, true),
        ];
        let result = format_attendance_status(&records);

        assert!(result.contains("#1 🟢 **開始**: 09:00 (修正済み)"));
        assert!(result.contains("#1 🔴 **終了**: 17:30 (修正済み)"));
    }

    #[test]
    fn test_format_attendance_status_currently_working() {
        let records = vec![create_test_record(1, "start", 9, 0, false)];
        let result = format_attendance_status(&records);

        assert!(result.contains("#1 🟢 **開始**: 09:00"));
        assert!(result.contains("#1 ⚠️ **現在勤務中**"));
        assert!(!result.contains("📊 **本日の合計勤務時間**"));
    }

    #[test]
    fn test_format_attendance_status_multiple_sessions() {
        let records = vec![
            create_test_record(1, "start", 9, 0, false),
            create_test_record(2, "end", 12, 0, false),
            create_test_record(3, "start", 13, 0, false),
            create_test_record(4, "end", 17, 30, false),
        ];
        let result = format_attendance_status(&records);

        assert!(result.contains("#1 🟢 **開始**: 09:00"));
        assert!(result.contains("#1 🔴 **終了**: 12:00"));
        assert!(result.contains("#1 ⏱️ 勤務時間: 3時間0分"));
        assert!(result.contains("#2 🟢 **開始**: 13:00"));
        assert!(result.contains("#2 🔴 **終了**: 17:30"));
        assert!(result.contains("#2 ⏱️ 勤務時間: 4時間30分"));
        assert!(result.contains("📊 **本日の合計勤務時間**: 7時間30分"));
        assert!(result.contains("🔄 **セッション数**: 2"));
    }

    #[test]
    fn test_format_attendance_status_end_without_start() {
        let records = vec![create_test_record(1, "end", 17, 30, false)];
        let result = format_attendance_status(&records);

        assert!(result.contains("#0 🔴 **終了**: 17:30"));
        assert!(result.contains("#0 ⚠️ 対応する開始記録なし"));
    }

    #[test]
    fn test_format_attendance_status_missing_end() {
        let records = vec![
            create_test_record(1, "start", 9, 0, false),
            create_test_record(2, "start", 13, 0, false),
        ];
        let result = format_attendance_status(&records);

        assert!(result.contains("#1 🟢 **開始**: 09:00"));
        assert!(result.contains("⚠️ 前回の終了記録なし"));
        assert!(result.contains("#2 🟢 **開始**: 13:00"));
        assert!(result.contains("#2 ⚠️ **現在勤務中**"));
    }

    #[test]
    fn test_format_work_sessions_summary_empty() {
        let sessions = vec![];
        let result = format_work_sessions_summary(&sessions);
        assert_eq!(result, "指定期間に勤務記録がありません");
    }

    #[test]
    fn test_format_work_sessions_summary_single_day() {
        let date = NaiveDate::from_ymd_opt(2023, 12, 15).unwrap();
        let sessions = vec![create_test_session(1, 9, 0, Some(17), Some(30), date)];
        let result = format_work_sessions_summary(&sessions);

        assert!(result.contains("📅 **2023-12-15 (Fri)**"));
        assert!(result.contains("🟢 開始: 09:00 → 🔴 終了: 17:30 (8時間30分)"));
        assert!(result.contains("📊 **12/15合計**: 8時間30分"));
        assert!(result.contains("🎯 **総合計勤務時間**: 8時間30分"));
    }

    #[test]
    fn test_format_work_sessions_summary_multiple_days() {
        let date1 = NaiveDate::from_ymd_opt(2023, 12, 15).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2023, 12, 16).unwrap();
        let sessions = vec![
            create_test_session(1, 9, 0, Some(17), Some(0), date1),
            create_test_session(2, 10, 0, Some(18), Some(30), date2),
        ];
        let result = format_work_sessions_summary(&sessions);

        assert!(result.contains("📅 **2023-12-15 (Fri)**"));
        assert!(result.contains("🟢 開始: 09:00 → 🔴 終了: 17:00 (8時間0分)"));
        assert!(result.contains("📊 **12/15合計**: 8時間0分"));

        assert!(result.contains("📅 **2023-12-16 (Sat)**"));
        assert!(result.contains("🟢 開始: 10:00 → 🔴 終了: 18:30 (8時間30分)"));
        assert!(result.contains("📊 **12/16合計**: 8時間30分"));

        assert!(result.contains("🎯 **総合計勤務時間**: 16時間30分"));
    }

    #[test]
    fn test_format_work_sessions_summary_with_incomplete_session() {
        let date = NaiveDate::from_ymd_opt(2023, 12, 15).unwrap();
        let sessions = vec![
            create_test_session(1, 9, 0, Some(12), Some(0), date),
            create_test_session(2, 13, 0, None, None, date),
        ];
        let result = format_work_sessions_summary(&sessions);

        assert!(result.contains("📅 **2023-12-15 (Fri)**"));
        assert!(result.contains("🟢 開始: 09:00 → 🔴 終了: 12:00 (3時間0分)"));
        assert!(result.contains("🟢 開始: 13:00 → ⚠️ **未終了**"));
        assert!(result.contains("📊 **12/15合計**: 3時間0分"));
        assert!(result.contains("🎯 **総合計勤務時間**: 3時間0分"));
    }

    #[test]
    fn test_format_work_sessions_summary_multiple_sessions_same_day() {
        let date = NaiveDate::from_ymd_opt(2023, 12, 15).unwrap();
        let sessions = vec![
            create_test_session(1, 9, 0, Some(12), Some(0), date),
            create_test_session(2, 13, 0, Some(17), Some(30), date),
        ];
        let result = format_work_sessions_summary(&sessions);

        assert!(result.contains("📅 **2023-12-15 (Fri)**"));
        assert!(result.contains("🟢 開始: 09:00 → 🔴 終了: 12:00 (3時間0分)"));
        assert!(result.contains("🟢 開始: 13:00 → 🔴 終了: 17:30 (4時間30分)"));
        assert!(result.contains("📊 **12/15合計**: 7時間30分"));
        assert!(result.contains("🎯 **総合計勤務時間**: 7時間30分"));
    }

    #[test]
    fn test_format_error_message() {
        let result = format_error_message("テストエラー");
        assert_eq!(result, "❌ **エラー**: テストエラー");
    }

    #[test]
    fn test_format_success_message() {
        let result = format_success_message("テスト成功");
        assert_eq!(result, "✅ テスト成功");
    }

    #[test]
    fn test_format_info_message() {
        let result = format_info_message("テスト情報");
        assert_eq!(result, "ℹ️ テスト情報");
    }

    // Note: Testing CreateEmbed directly is not straightforward in Serenity v0.12
    // as the internal data structure is not publicly accessible.
    // These tests verify that the functions don't panic and return CreateEmbed instances.

    #[test]
    fn test_create_success_embed() {
        let _embed = create_success_embed("テストタイトル", "テスト説明");
        // Embed creation successful (no panic)
    }

    #[test]
    fn test_create_error_embed() {
        let _embed = create_error_embed("エラータイトル", "エラー説明");
        // Embed creation successful (no panic)
    }

    #[test]
    fn test_create_info_embed() {
        let _embed = create_info_embed("情報タイトル", "情報説明");
        // Embed creation successful (no panic)
    }

    #[test]
    fn test_create_status_embed() {
        let date = NaiveDate::from_ymd_opt(2023, 12, 15).unwrap();
        let records = vec![
            create_test_record(1, "start", 9, 0, false),
            create_test_record(2, "end", 17, 30, false),
        ];
        let _embed = create_status_embed("テストユーザー", date, &records);
        // Embed creation successful (no panic)
    }

    #[test]
    fn test_create_report_embed() {
        let date = NaiveDate::from_ymd_opt(2023, 12, 15).unwrap();
        let sessions = vec![create_test_session(1, 9, 0, Some(17), Some(30), date)];
        let _embed = create_report_embed("テストユーザー", "日次レポート", "2023-12-15", &sessions);
        // Embed creation successful (no panic)
    }
}
