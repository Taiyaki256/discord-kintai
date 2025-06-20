use crate::bot::{Context, Error};
use crate::database::queries;
use crate::utils::format::{create_error_embed, create_report_embed};
use crate::utils::time::get_current_date_jst;
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
            let embed = create_error_embed(
                "エラー",
                &format!("ユーザー情報の取得に失敗しました: {}", e),
            );
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            return Ok(());
        }
    };

    let today = get_current_date_jst();

    match queries::get_work_sessions_by_date_range(pool, user.id, today, today).await {
        Ok(sessions) => {
            let embed = create_report_embed(
                &username,
                "日次レポート",
                &today.format("%Y年%m月%d日").to_string(),
                &sessions,
            );

            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        }
        Err(e) => {
            let embed =
                create_error_embed("エラー", &format!("勤務記録の取得に失敗しました: {}", e));
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
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
            let embed = create_error_embed(
                "エラー",
                &format!("ユーザー情報の取得に失敗しました: {}", e),
            );
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            return Ok(());
        }
    };

    let today = get_current_date_jst();
    let days_since_monday = today.weekday().num_days_from_monday() as u64;
    let start_of_week = today
        .checked_sub_days(Days::new(days_since_monday))
        .unwrap_or(today);

    match queries::get_work_sessions_by_date_range(pool, user.id, start_of_week, today).await {
        Ok(sessions) => {
            let date_range = format!(
                "{} ～ {}",
                start_of_week.format("%Y年%m月%d日"),
                today.format("%Y年%m月%d日")
            );

            let embed = create_report_embed(&username, "週次レポート", &date_range, &sessions);

            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        }
        Err(e) => {
            let embed =
                create_error_embed("エラー", &format!("勤務記録の取得に失敗しました: {}", e));
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
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
            let embed = create_error_embed(
                "エラー",
                &format!("ユーザー情報の取得に失敗しました: {}", e),
            );
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            return Ok(());
        }
    };

    let today = get_current_date_jst();
    let start_of_month =
        chrono::NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap_or(today);

    match queries::get_work_sessions_by_date_range(pool, user.id, start_of_month, today).await {
        Ok(sessions) => {
            let date_range = format!(
                "{} ～ {}",
                start_of_month.format("%Y年%m月%d日"),
                today.format("%Y年%m月%d日")
            );

            let embed = create_report_embed(&username, "月次レポート", &date_range, &sessions);

            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        }
        Err(e) => {
            let embed =
                create_error_embed("エラー", &format!("勤務記録の取得に失敗しました: {}", e));
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        }
    }

    Ok(())
}
