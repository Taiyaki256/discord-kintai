use crate::bot::{Context, Error};
use crate::database::queries;
use crate::utils::time::get_current_date_jst;
use crate::utils::format::{format_attendance_status, format_error_message};
use poise::serenity_prelude as serenity;

/// 現在の勤務状況を確認します
#[poise::command(slash_command)]
pub async fn status(ctx: Context<'_>) -> Result<(), Error> {
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

    // Get today's records
    match queries::get_today_records(pool, user.id, current_date).await {
        Ok(records) => {
            let status_text = format_attendance_status(&records);
            let header = format!("📊 **{}の勤務状況** ({})\n\n", username, current_date.format("%Y-%m-%d"));
            let full_message = format!("{}{}", header, status_text);
            
            // Create interactive buttons
            let components = vec![serenity::CreateActionRow::Buttons(vec![
                serenity::CreateButton::new("time_edit")
                    .label("🕐 時間修正")
                    .style(serenity::ButtonStyle::Primary),
                serenity::CreateButton::new("end_register")
                    .label("✅ 終了登録")
                    .style(serenity::ButtonStyle::Success),
                serenity::CreateButton::new("delete_record")
                    .label("🗑️ 削除")
                    .style(serenity::ButtonStyle::Danger),
                serenity::CreateButton::new("history_view")
                    .label("📋 履歴")
                    .style(serenity::ButtonStyle::Secondary),
            ])];
            
            let builder = poise::CreateReply::default()
                .content(full_message)
                .components(components);
            
            ctx.send(builder).await?;
        }
        Err(e) => {
            let msg = format_error_message(&format!("勤務記録の取得に失敗しました: {}", e));
            ctx.say(msg).await?;
        }
    }

    Ok(())
}