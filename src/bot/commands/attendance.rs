use crate::bot::{Context, Error};
use crate::database::queries;
use crate::database::models::RecordType;
use crate::utils::time::{get_current_date_jst, get_current_datetime_jst};
use crate::utils::format::{format_success_message, format_error_message};

/// 勤務を開始します
#[poise::command(slash_command)]
pub async fn start(ctx: Context<'_>) -> Result<(), Error> {
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

    let current_date = get_current_date_jst();
    let current_datetime = get_current_datetime_jst().to_utc();

    // Check if there's already an active session
    if let Ok(Some(active_session)) = queries::get_active_work_session(pool, user.id).await {
        let msg = format_error_message(&format!(
            "既に勤務中です（開始時刻: {}）。先に `/end` で終了してください。",
            crate::utils::time::format_time_jst(active_session.start_time)
        ));
        ctx.say(msg).await?;
        return Ok(());
    }

    // Create attendance record
    match queries::create_attendance_record(pool, user.id, RecordType::Start, current_datetime).await {
        Ok(_) => {
            // Create work session
            match queries::create_work_session(pool, user.id, current_datetime, current_date).await {
                Ok(_) => {
                    let msg = format_success_message(&format!(
                        "勤務を開始しました（{}）",
                        crate::utils::time::format_time_jst(current_datetime)
                    ));
                    ctx.say(msg).await?;
                }
                Err(e) => {
                    let msg = format_error_message(&format!("勤務セッションの作成に失敗しました: {}", e));
                    ctx.say(msg).await?;
                }
            }
        }
        Err(e) => {
            let msg = format_error_message(&format!("勤務記録の作成に失敗しました: {}", e));
            ctx.say(msg).await?;
        }
    }

    Ok(())
}

/// 勤務を終了します
#[poise::command(slash_command)]
pub async fn end(ctx: Context<'_>) -> Result<(), Error> {
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

    let current_datetime = get_current_datetime_jst().to_utc();

    // Check if there's an active session
    let active_session = match queries::get_active_work_session(pool, user.id).await {
        Ok(Some(session)) => session,
        Ok(None) => {
            let msg = format_error_message("勤務中ではありません。先に `/start` で開始してください。");
            ctx.say(msg).await?;
            return Ok(());
        }
        Err(e) => {
            let msg = format_error_message(&format!("勤務セッション情報の取得に失敗しました: {}", e));
            ctx.say(msg).await?;
            return Ok(());
        }
    };

    // Create attendance record
    match queries::create_attendance_record(pool, user.id, RecordType::End, current_datetime).await {
        Ok(_) => {
            // Complete work session
            match queries::complete_work_session(pool, active_session.id, current_datetime).await {
                Ok(_) => {
                    let duration = current_datetime.signed_duration_since(active_session.start_time);
                    let duration_str = crate::utils::time::format_duration_minutes(duration.num_minutes() as i32);
                    
                    let msg = format_success_message(&format!(
                        "勤務を終了しました（{}）\n勤務時間: {}",
                        crate::utils::time::format_time_jst(current_datetime),
                        duration_str
                    ));
                    ctx.say(msg).await?;
                }
                Err(e) => {
                    let msg = format_error_message(&format!("勤務セッションの完了に失敗しました: {}", e));
                    ctx.say(msg).await?;
                }
            }
        }
        Err(e) => {
            let msg = format_error_message(&format!("勤務記録の作成に失敗しました: {}", e));
            ctx.say(msg).await?;
        }
    }

    Ok(())
}