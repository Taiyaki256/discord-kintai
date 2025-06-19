use chrono::{DateTime, Utc, NaiveDate, NaiveTime, TimeZone};
use anyhow::Result;

pub fn get_current_date_jst() -> NaiveDate {
    let jst_offset = chrono::FixedOffset::east_opt(9 * 3600).unwrap();
    let now_jst = Utc::now().with_timezone(&jst_offset);
    now_jst.date_naive()
}

pub fn get_current_datetime_jst() -> DateTime<chrono::FixedOffset> {
    let jst_offset = chrono::FixedOffset::east_opt(9 * 3600).unwrap();
    Utc::now().with_timezone(&jst_offset)
}

pub fn parse_time_string(time_str: &str) -> Result<NaiveTime> {
    let time_str = time_str.trim();
    
    // Try standard time format first
    if let Ok(time) = NaiveTime::parse_from_str(time_str, "%H:%M") {
        return Ok(time);
    }
    
    if let Ok(time) = NaiveTime::parse_from_str(time_str, "%H:%M:%S") {
        return Ok(time);
    }
    
    // Handle night shift format (25:10 = 01:10 next day)
    if let Some(colon_pos) = time_str.find(':') {
        let hour_str = &time_str[..colon_pos];
        let minute_str = &time_str[colon_pos + 1..];
        
        if let (Ok(hour), Ok(minute)) = (hour_str.parse::<u32>(), minute_str.parse::<u32>()) {
            if hour >= 24 && hour < 48 && minute < 60 {
                // Convert 24+ hour to 0-23 hour for next day
                let adjusted_hour = hour - 24;
                if let Some(time) = NaiveTime::from_hms_opt(adjusted_hour, minute, 0) {
                    return Ok(time);
                }
            }
        }
    }
    
    Err(anyhow::anyhow!("Invalid time format. Use HH:MM (supports 00:00-47:59 for night shifts)"))
}

pub fn combine_date_time_jst(date: NaiveDate, time: NaiveTime) -> DateTime<Utc> {
    let jst_offset = chrono::FixedOffset::east_opt(9 * 3600).unwrap();
    let naive_datetime = date.and_time(time);
    jst_offset.from_local_datetime(&naive_datetime).unwrap().to_utc()
}

/// Parse time string and return both the NaiveTime and whether it represents next day
/// Returns (time, is_next_day) where is_next_day=true for 24:00-47:59 input
pub fn parse_time_with_day_info(time_str: &str) -> Result<(NaiveTime, bool)> {
    let time_str = time_str.trim();
    
    // Try standard time format first
    if let Ok(time) = NaiveTime::parse_from_str(time_str, "%H:%M") {
        return Ok((time, false));
    }
    
    if let Ok(time) = NaiveTime::parse_from_str(time_str, "%H:%M:%S") {
        return Ok((time, false));
    }
    
    // Handle night shift format (25:10 = 01:10 next day)
    if let Some(colon_pos) = time_str.find(':') {
        let hour_str = &time_str[..colon_pos];
        let minute_str = &time_str[colon_pos + 1..];
        
        if let (Ok(hour), Ok(minute)) = (hour_str.parse::<u32>(), minute_str.parse::<u32>()) {
            if hour >= 24 && hour < 48 && minute < 60 {
                // Convert 24+ hour to 0-23 hour for next day
                let adjusted_hour = hour - 24;
                if let Some(time) = NaiveTime::from_hms_opt(adjusted_hour, minute, 0) {
                    return Ok((time, true)); // is_next_day = true
                }
            }
        }
    }
    
    Err(anyhow::anyhow!("Invalid time format. Use HH:MM (supports 00:00-47:59 for night shifts)"))
}

/// Combine date and time with proper next-day handling for night shifts
pub fn combine_date_time_jst_with_day_offset(date: NaiveDate, time: NaiveTime, is_next_day: bool) -> DateTime<Utc> {
    let actual_date = if is_next_day {
        date.succ_opt().unwrap_or(date) // Add one day
    } else {
        date
    };
    
    let jst_offset = chrono::FixedOffset::east_opt(9 * 3600).unwrap();
    let naive_datetime = actual_date.and_time(time);
    jst_offset.from_local_datetime(&naive_datetime).unwrap().to_utc()
}

pub fn calculate_work_duration(start: DateTime<Utc>, end: DateTime<Utc>) -> i32 {
    let duration = end.signed_duration_since(start);
    duration.num_minutes() as i32
}

pub fn format_duration_minutes(minutes: i32) -> String {
    let hours = minutes / 60;
    let mins = minutes % 60;
    
    if hours > 0 {
        format!("{}時間{}分", hours, mins)
    } else {
        format!("{}分", mins)
    }
}

pub fn format_datetime_jst(datetime: DateTime<Utc>) -> String {
    let jst_offset = chrono::FixedOffset::east_opt(9 * 3600).unwrap();
    let jst_time = datetime.with_timezone(&jst_offset);
    jst_time.format("%Y-%m-%d %H:%M:%S JST").to_string()
}

pub fn format_time_jst(datetime: DateTime<Utc>) -> String {
    let jst_offset = chrono::FixedOffset::east_opt(9 * 3600).unwrap();
    let jst_time = datetime.with_timezone(&jst_offset);
    jst_time.format("%H:%M").to_string()
}

pub fn get_date_from_utc_timestamp(timestamp: DateTime<Utc>) -> NaiveDate {
    let jst_offset = chrono::FixedOffset::east_opt(9 * 3600).unwrap();
    let jst_time = timestamp.with_timezone(&jst_offset);
    let date = jst_time.date_naive();
    tracing::info!("get_date_from_utc_timestamp: UTC={:?}, JST={:?}, Date={}", 
                   timestamp, jst_time, date);
    date
}