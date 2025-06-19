use crate::bot::{Context, Error};
use crate::database::queries;
use crate::utils::time::get_current_date_jst;
use crate::utils::format::{format_work_sessions_summary, format_error_message};
use chrono::{Datelike, Days};

/// 今日の勤務レポートを表示します
#[poise::command(slash_command)]
pub async fn daily(ctx: Context<'_>) -> Result<(), Error> {
    let user_id = ctx.author().id.to_string();
    let username = ctx.author().name.clone();
    let pool = &ctx.data().pool;

    // Create or get user
    let user = match queries::create_or_get_user(pool, &user_id, &username).await {
        Ok(user) => user,
        Err(e) => {
            let msg = format_error_message(&format!("ユーザー情報の取得に失敗しました: {}", e));
            ctx.say(msg).await?;
            return Ok(());
        }
    };

    let today = get_current_date_jst();

    match queries::get_work_sessions_by_date_range(pool, user.id, today, today).await {
        Ok(sessions) => {
            let report = format_work_sessions_summary(&sessions);
            let header = format!("📅 **{}の日次レポート** ({})\n\n", username, today.format("%Y-%m-%d"));
            let full_message = format!("{}{}", header, report);
            
            ctx.say(full_message).await?;
        }
        Err(e) => {
            let msg = format_error_message(&format!("勤務記録の取得に失敗しました: {}", e));
            ctx.say(msg).await?;
        }
    }

    Ok(())
}

/// 今週の勤務レポートを表示します
#[poise::command(slash_command)]
pub async fn weekly(ctx: Context<'_>) -> Result<(), Error> {
    let user_id = ctx.author().id.to_string();
    let username = ctx.author().name.clone();
    let pool = &ctx.data().pool;

    // Create or get user
    let user = match queries::create_or_get_user(pool, &user_id, &username).await {
        Ok(user) => user,
        Err(e) => {
            let msg = format_error_message(&format!("ユーザー情報の取得に失敗しました: {}", e));
            ctx.say(msg).await?;
            return Ok(());
        }
    };

    let today = get_current_date_jst();
    let days_since_monday = today.weekday().num_days_from_monday() as u64;
    let start_of_week = today.checked_sub_days(Days::new(days_since_monday)).unwrap_or(today);

    match queries::get_work_sessions_by_date_range(pool, user.id, start_of_week, today).await {
        Ok(sessions) => {
            let report = format_work_sessions_summary(&sessions);
            let header = format!(
                "📅 **{}の週次レポート** ({} ～ {})\n\n", 
                username, 
                start_of_week.format("%Y-%m-%d"),
                today.format("%Y-%m-%d")
            );
            let full_message = format!("{}{}", header, report);
            
            ctx.say(full_message).await?;
        }
        Err(e) => {
            let msg = format_error_message(&format!("勤務記録の取得に失敗しました: {}", e));
            ctx.say(msg).await?;
        }
    }

    Ok(())
}

/// 今月の勤務レポートを表示します
#[poise::command(slash_command)]
pub async fn monthly(ctx: Context<'_>) -> Result<(), Error> {
    let user_id = ctx.author().id.to_string();
    let username = ctx.author().name.clone();
    let pool = &ctx.data().pool;

    // Create or get user
    let user = match queries::create_or_get_user(pool, &user_id, &username).await {
        Ok(user) => user,
        Err(e) => {
            let msg = format_error_message(&format!("ユーザー情報の取得に失敗しました: {}", e));
            ctx.say(msg).await?;
            return Ok(());
        }
    };

    let today = get_current_date_jst();
    let start_of_month = chrono::NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
        .unwrap_or(today);

    match queries::get_work_sessions_by_date_range(pool, user.id, start_of_month, today).await {
        Ok(sessions) => {
            let report = format_work_sessions_summary(&sessions);
            let header = format!(
                "📅 **{}の月次レポート** ({} ～ {})\n\n", 
                username, 
                start_of_month.format("%Y-%m-%d"),
                today.format("%Y-%m-%d")
            );
            let full_message = format!("{}{}", header, report);
            
            ctx.say(full_message).await?;
        }
        Err(e) => {
            let msg = format_error_message(&format!("勤務記録の取得に失敗しました: {}", e));
            ctx.say(msg).await?;
        }
    }

    Ok(())
}