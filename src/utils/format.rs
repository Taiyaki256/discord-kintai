use crate::database::models::{AttendanceRecord, WorkSession};
use crate::utils::time::{format_time_jst, format_duration_minutes};
use chrono::{DateTime, Utc};

pub fn format_attendance_status(records: &[AttendanceRecord]) -> String {
    if records.is_empty() {
        return "今日はまだ勤務記録がありません".to_string();
    }

    let mut status = String::new();
    let mut start_time: Option<DateTime<Utc>> = None;
    let mut total_minutes = 0i32;

    for record in records {
        match record.record_type.as_str() {
            "start" => {
                status.push_str(&format!(
                    "🟢 **開始**: {} {}\n",
                    format_time_jst(record.timestamp),
                    if record.is_modified { "(修正済み)" } else { "" }
                ));
                start_time = Some(record.timestamp);
            }
            "end" => {
                status.push_str(&format!(
                    "🔴 **終了**: {} {}\n",
                    format_time_jst(record.timestamp),
                    if record.is_modified { "(修正済み)" } else { "" }
                ));
                
                if let Some(start) = start_time {
                    let duration = record.timestamp.signed_duration_since(start).num_minutes() as i32;
                    total_minutes += duration;
                    status.push_str(&format!("  ⏱️ 勤務時間: {}\n", format_duration_minutes(duration)));
                }
                start_time = None;
            }
            _ => {}
        }
    }

    // If still working
    if start_time.is_some() {
        status.push_str("⚠️ **現在勤務中**\n");
    }

    if total_minutes > 0 {
        status.push_str(&format!("\n📊 **本日の合計勤務時間**: {}", format_duration_minutes(total_minutes)));
    }

    status
}

pub fn format_work_sessions_summary(sessions: &[WorkSession]) -> String {
    if sessions.is_empty() {
        return "指定期間に勤務記録がありません".to_string();
    }

    let mut summary = String::new();
    let mut total_minutes = 0i32;

    for session in sessions {
        summary.push_str(&format!(
            "📅 **{}**\n",
            session.date.format("%Y-%m-%d (%a)")
        ));
        
        summary.push_str(&format!(
            "   🟢 開始: {}\n",
            format_time_jst(session.start_time)
        ));

        if let Some(end_time) = session.end_time {
            summary.push_str(&format!(
                "   🔴 終了: {}\n",
                format_time_jst(end_time)
            ));
            
            if let Some(minutes) = session.total_minutes {
                summary.push_str(&format!(
                    "   ⏱️ 勤務時間: {}\n",
                    format_duration_minutes(minutes)
                ));
                total_minutes += minutes;
            }
        } else {
            summary.push_str("   ⚠️ 未終了\n");
        }
        
        summary.push('\n');
    }

    if total_minutes > 0 {
        summary.push_str(&format!("📊 **合計勤務時間**: {}", format_duration_minutes(total_minutes)));
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