use crate::database::models::{AttendanceRecord, WorkSession};
use crate::utils::time::{format_time_jst, format_duration_minutes};
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
                    if record.is_modified { "(修正済み)" } else { "" }
                ));
                start_time = Some(record.timestamp);
            }
            "end" => {
                status.push_str(&format!(
                    "#{} 🔴 **終了**: {} {}\n",
                    session_count,
                    format_time_jst(record.timestamp),
                    if record.is_modified { "(修正済み)" } else { "" }
                ));
                
                if let Some(start) = start_time {
                    let duration = record.timestamp.signed_duration_since(start).num_minutes() as i32;
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
        status.push_str(&format!("📊 **本日の合計勤務時間**: {}", format_duration_minutes(total_minutes)));
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
            summary.push_str(&format!(
                " → 🔴 終了: {}",
                format_time_jst(end_time)
            ));
            
            if let Some(minutes) = session.total_minutes {
                summary.push_str(&format!(
                    " ({})",
                    format_duration_minutes(minutes)
                ));
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
        summary.push_str(&format!("🎯 **総合計勤務時間**: {}", format_duration_minutes(total_minutes)));
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

pub fn create_status_embed(username: &str, date: chrono::NaiveDate, records: &[AttendanceRecord]) -> serenity::CreateEmbed {
    let status_text = format_attendance_status(records);
    serenity::CreateEmbed::new()
        .title("📊 勤務状況")
        .description(status_text)
        .color(0x3498db) // Blue
        .author(serenity::CreateEmbedAuthor::new(format!("{} の勤務状況", username)))
        .footer(serenity::CreateEmbedFooter::new(date.format("%Y年%m月%d日").to_string()))
        .timestamp(chrono::Utc::now())
}

pub fn create_report_embed(username: &str, title: &str, date_range: &str, sessions: &[WorkSession]) -> serenity::CreateEmbed {
    let report_text = format_work_sessions_summary(sessions);
    serenity::CreateEmbed::new()
        .title(format!("📅 {}", title))
        .description(report_text)
        .color(0x9b59b6) // Purple
        .author(serenity::CreateEmbedAuthor::new(format!("{} のレポート", username)))
        .footer(serenity::CreateEmbedFooter::new(date_range))
        .timestamp(chrono::Utc::now())
}