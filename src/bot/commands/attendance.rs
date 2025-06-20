use crate::bot::{Context, Error};
use crate::database::models::RecordType;
use crate::database::queries;
use crate::utils::format::{create_error_embed, create_success_embed};
use crate::utils::session_manager::SessionManager;
use crate::utils::time::{
    get_current_date_jst, get_current_datetime_jst, get_date_from_utc_timestamp,
};

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
            let embed = create_error_embed(
                "エラー",
                &format!("ユーザー情報の取得に失敗しました: {}", e),
            );
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            return Ok(());
        }
    };

    let current_datetime = get_current_datetime_jst().to_utc();
    // Use the date from the actual timestamp being stored
    let current_date = get_date_from_utc_timestamp(current_datetime);

    tracing::info!(
        "Start command - User ID: {}, Date from timestamp: {}, UTC Timestamp: {:?}",
        user.id,
        current_date,
        current_datetime
    );

    // Check if there's already an unpaired start record
    let today_records = match queries::get_today_records(pool, user.id, current_date).await {
        Ok(records) => records,
        Err(e) => {
            let embed =
                create_error_embed("エラー", &format!("勤務記録の取得に失敗しました: {}", e));
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            return Ok(());
        }
    };

    // Debug: Log all records
    tracing::info!(
        "Start command - Today's records count: {}",
        today_records.len()
    );
    for (i, record) in today_records.iter().enumerate() {
        tracing::info!(
            "Record {}: type={}, timestamp={:?}",
            i,
            record.record_type,
            record.timestamp
        );
    }

    // Check if the last record is an unpaired start
    if let Some(last_record) = today_records.last() {
        tracing::info!("Last record type: {}", last_record.record_type);
        if last_record.record_type == "start" {
            let embed = create_error_embed(
                "既に勤務中です",
                &format!(
                    "開始時刻: {}\n先に `/end` で終了してください。",
                    crate::utils::time::format_time_jst(last_record.timestamp)
                ),
            );
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            return Ok(());
        }
    } else {
        tracing::info!("No records found for today");
    }

    // Create attendance record
    tracing::info!("Creating start record for user {}", user.id);
    match queries::create_attendance_record(pool, user.id, RecordType::Start, current_datetime)
        .await
    {
        Ok(_) => {
            tracing::info!("Start record created successfully");
            // Recalculate sessions after adding start record
            let session_manager = SessionManager::new(pool.clone());
            if let Err(e) = session_manager
                .trigger_recalculation(user.id, current_date)
                .await
            {
                tracing::error!("Failed to recalculate sessions: {}", e);
            }

            let embed = create_success_embed(
                "勤務開始",
                &format!(
                    "勤務を開始しました\n開始時刻: {}",
                    crate::utils::time::format_time_jst(current_datetime)
                ),
            );
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        }
        Err(e) => {
            let embed =
                create_error_embed("エラー", &format!("勤務記録の作成に失敗しました: {}", e));
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
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
            let embed = create_error_embed(
                "エラー",
                &format!("ユーザー情報の取得に失敗しました: {}", e),
            );
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            return Ok(());
        }
    };

    let current_datetime = get_current_datetime_jst().to_utc();

    // Check if there's an unpaired start record
    let current_date = get_date_from_utc_timestamp(current_datetime);

    tracing::info!(
        "End command - User ID: {}, Date from timestamp: {}, UTC Timestamp: {:?}",
        user.id,
        current_date,
        current_datetime
    );

    let today_records = match queries::get_today_records(pool, user.id, current_date).await {
        Ok(records) => records,
        Err(e) => {
            let embed =
                create_error_embed("エラー", &format!("勤務記録の取得に失敗しました: {}", e));
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            return Ok(());
        }
    };

    // Debug: Log all records
    tracing::info!(
        "End command - Today's records count: {}",
        today_records.len()
    );
    for (i, record) in today_records.iter().enumerate() {
        tracing::info!(
            "Record {}: type={}, timestamp={:?}",
            i,
            record.record_type,
            record.timestamp
        );
    }

    // Check if the last record is an unpaired start
    let start_record = match today_records.last() {
        Some(record) if record.record_type == "start" => {
            tracing::info!("Found unpaired start record");
            record
        }
        Some(record) => {
            tracing::info!("Last record is not start, it's: {}", record.record_type);
            let embed =
                create_error_embed("勤務中ではありません", "先に `/start` で開始してください。");
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            return Ok(());
        }
        None => {
            tracing::info!("No records found for today");
            let embed =
                create_error_embed("勤務中ではありません", "先に `/start` で開始してください。");
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            return Ok(());
        }
    };

    // Create attendance record
    match queries::create_attendance_record(pool, user.id, RecordType::End, current_datetime).await
    {
        Ok(_) => {
            // Recalculate sessions after adding end record
            let session_manager = SessionManager::new(pool.clone());
            if let Err(e) = session_manager
                .trigger_recalculation(user.id, current_date)
                .await
            {
                tracing::error!("Failed to recalculate sessions: {}", e);
            }

            let duration = current_datetime.signed_duration_since(start_record.timestamp);
            let duration_str =
                crate::utils::time::format_duration_minutes(duration.num_minutes() as i32);

            let embed = create_success_embed(
                "勤務終了",
                &format!(
                    "勤務を終了しました\n終了時刻: {}\n勤務時間: {}",
                    crate::utils::time::format_time_jst(current_datetime),
                    duration_str
                ),
            );
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        }
        Err(e) => {
            let embed =
                create_error_embed("エラー", &format!("勤務記録の作成に失敗しました: {}", e));
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        }
    }

    Ok(())
}
