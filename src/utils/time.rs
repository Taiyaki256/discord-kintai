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
    
    if let Ok(time) = NaiveTime::parse_from_str(time_str, "%H:%M") {
        return Ok(time);
    }
    
    if let Ok(time) = NaiveTime::parse_from_str(time_str, "%H:%M:%S") {
        return Ok(time);
    }
    
    Err(anyhow::anyhow!("Invalid time format. Use HH:MM or HH:MM:SS"))
}

pub fn combine_date_time_jst(date: NaiveDate, time: NaiveTime) -> DateTime<Utc> {
    let jst_offset = chrono::FixedOffset::east_opt(9 * 3600).unwrap();
    let naive_datetime = date.and_time(time);
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