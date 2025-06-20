use crate::utils::time::{parse_time_string, parse_time_with_day_info};
use anyhow::Result;
use chrono::{NaiveDate, NaiveTime};

pub fn validate_time_format(time_str: &str) -> Result<NaiveTime> {
    parse_time_string(time_str)
}

/// Validate time format and return time with next-day information for night shifts
pub fn validate_time_format_with_day_info(time_str: &str) -> Result<(NaiveTime, bool)> {
    parse_time_with_day_info(time_str)
}

pub fn validate_time_order(start_time: NaiveTime, end_time: NaiveTime) -> Result<()> {
    if end_time <= start_time {
        return Err(anyhow::anyhow!(
            "終了時間は開始時間より後である必要があります"
        ));
    }
    Ok(())
}

pub fn validate_reasonable_work_hours(start_time: NaiveTime, end_time: NaiveTime) -> Result<()> {
    let duration = end_time.signed_duration_since(start_time);
    let hours = duration.num_hours();

    if hours > 24 {
        return Err(anyhow::anyhow!(
            "勤務時間が24時間を超えています。正しい時間を入力してください"
        ));
    }

    if hours > 16 {
        return Err(anyhow::anyhow!(
            "勤務時間が16時間を超えています。本当に正しいですか？"
        ));
    }

    Ok(())
}

pub fn validate_date_not_future(date: NaiveDate) -> Result<()> {
    let today = chrono::Local::now().date_naive();

    if date > today {
        return Err(anyhow::anyhow!("未来の日付を指定することはできません"));
    }

    Ok(())
}

pub fn validate_reasonable_past_date(date: NaiveDate) -> Result<()> {
    let today = chrono::Local::now().date_naive();
    let days_ago = today.signed_duration_since(date).num_days();

    if days_ago > 365 {
        return Err(anyhow::anyhow!("1年以上前の日付は指定できません"));
    }

    Ok(())
}
