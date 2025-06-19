use crate::bot::{Context, Error};
use crate::database::queries;
use crate::utils::time::get_current_date_jst;
use crate::utils::format::{create_status_embed, create_error_embed};
use crate::utils::record_selector::RecordSelector;
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
            let embed = create_error_embed("エラー", &format!("ユーザー情報の取得に失敗しました: {}", e));
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            return Ok(());
        }
    };

    let current_date = get_current_date_jst();

    // Get today's records
    match queries::get_today_records(pool, user.id, current_date).await {
        Ok(records) => {
            // Create record selector for available actions
            let record_selector = RecordSelector::new(records.clone());
            
            // Create interactive buttons with user ID embedded
            let mut buttons = vec![
                serenity::CreateButton::new(&format!("record_add:{}", user_id))
                    .label("✅ 記録追加")
                    .style(serenity::ButtonStyle::Success),
                serenity::CreateButton::new(&format!("history_view:{}", user_id))
                    .label("📋 履歴")
                    .style(serenity::ButtonStyle::Secondary),
            ];

            // Add edit and delete buttons only if there are records
            if !record_selector.is_empty() {
                buttons.insert(0, serenity::CreateButton::new(&format!("time_edit:{}", user_id))
                    .label("🕐 時間修正")
                    .style(serenity::ButtonStyle::Primary));
                buttons.insert(2, serenity::CreateButton::new(&format!("delete_record:{}", user_id))
                    .label("🗑️ 削除")
                    .style(serenity::ButtonStyle::Danger));
            }

            let components = vec![serenity::CreateActionRow::Buttons(buttons)];
            
            let embed = create_status_embed(&username, current_date, &records);
            
            let builder = poise::CreateReply::default()
                .embed(embed)
                .components(components);
            
            ctx.send(builder).await?;
        }
        Err(e) => {
            let embed = create_error_embed("エラー", &format!("勤務記録の取得に失敗しました: {}", e));
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        }
    }

    Ok(())
}