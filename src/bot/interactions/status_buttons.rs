use crate::bot::{Data, Error};
use crate::database::models::RecordType;
use crate::database::queries;
use crate::utils::format::{create_error_embed, create_success_embed, format_error_message};
use crate::utils::record_selector::RecordSelector;
use crate::utils::record_validator::RecordValidator;
use crate::utils::session_manager::SessionManager;
use crate::utils::time::{combine_date_time_jst, get_current_date_jst};
use crate::utils::validation::validate_time_format;
use chrono::{Datelike, NaiveDate};
use poise::serenity_prelude as serenity;

pub async fn handle_status_interaction(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    data: &Data,
) -> Result<(), Error> {
    let custom_id = &interaction.data.custom_id;

    // Extract action and user ID from custom_id (format: "action:user_id" or "action:user_id:extra")
    let parts: Vec<&str> = custom_id.split(':').collect();
    if parts.len() >= 2 {
        let action = parts[0];
        let original_user_id = parts[1];

        // Verify user has permission to interact with this status message
        if interaction.user.id.to_string() != original_user_id {
            let embed =
                create_error_embed("アクセス拒否", "他のユーザーの勤務状況は操作できません");
            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .embed(embed)
                            .ephemeral(true),
                    ),
                )
                .await?;
            return Ok(());
        }

        match action {
            "time_edit" => handle_time_edit_selection(ctx, interaction, data).await,
            "record_add" => handle_record_add(ctx, interaction, data).await,
            "delete_record" => handle_delete_record_selection(ctx, interaction, data).await,
            "history_view" => handle_history_view(ctx, interaction, data).await,
            "add_start_record" => handle_add_start_record(ctx, interaction, data).await,
            "add_end_record" => handle_add_end_record(ctx, interaction, data).await,
            "cancel_add" => handle_cancel_action(ctx, interaction, data).await,
            "confirm_delete_single" => handle_confirm_delete_single(ctx, interaction, data).await,
            "confirm_delete_all" => handle_confirm_delete_all(ctx, interaction, data).await,
            "cancel_delete" => handle_cancel_action(ctx, interaction, data).await,
            _ => {
                interaction
                    .create_response(
                        &ctx.http,
                        serenity::CreateInteractionResponse::Message(
                            serenity::CreateInteractionResponseMessage::new()
                                .content("未実装の機能です")
                                .ephemeral(true),
                        ),
                    )
                    .await?;
                Ok(())
            }
        }
    } else {
        // Handle cases without user ID (select menus, etc.)
        match custom_id.as_str() {
            // Select menu interactions
            "edit_record_select" => handle_edit_record_selected(ctx, interaction, data).await,
            "delete_record_select" => handle_delete_record_selected(ctx, interaction, data).await,
            "history_date_select" => handle_history_date_selected(ctx, interaction, data).await,
            _ => {
                interaction
                    .create_response(
                        &ctx.http,
                        serenity::CreateInteractionResponse::Message(
                            serenity::CreateInteractionResponseMessage::new()
                                .content("未実装の機能です")
                                .ephemeral(true),
                        ),
                    )
                    .await?;
                Ok(())
            }
        }
    }
}

async fn handle_time_edit_selection(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    data: &Data,
) -> Result<(), Error> {
    // Get user information
    let user_id = interaction.user.id.to_string();
    let username = interaction.user.name.clone();
    let pool = &data.pool;

    // Get user from database
    let user = match queries::create_or_get_user(pool, &user_id, &username).await {
        Ok(user) => user,
        Err(e) => {
            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content(format_error_message(&format!(
                                "ユーザー情報の取得に失敗しました: {}",
                                e
                            )))
                            .ephemeral(true),
                    ),
                )
                .await?;
            return Ok(());
        }
    };

    let current_date = get_current_date_jst();

    // Get today's records
    let records = match queries::get_today_records(pool, user.id, current_date).await {
        Ok(records) => records,
        Err(e) => {
            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content(format_error_message(&format!(
                                "勤務記録の取得に失敗しました: {}",
                                e
                            )))
                            .ephemeral(true),
                    ),
                )
                .await?;
            return Ok(());
        }
    };

    let record_selector = RecordSelector::new(records);

    if record_selector.is_empty() {
        interaction
            .create_response(
                &ctx.http,
                serenity::CreateInteractionResponse::Message(
                    serenity::CreateInteractionResponseMessage::new()
                        .content("修正できる記録がありません")
                        .ephemeral(true),
                ),
            )
            .await?;
        return Ok(());
    }

    // Create select menu for record selection
    if let Some(select_menu) =
        record_selector.create_select_menu("edit_record_select", "修正する記録を選択してください")
    {
        let components = vec![serenity::CreateActionRow::SelectMenu(select_menu)];

        interaction
            .create_response(
                &ctx.http,
                serenity::CreateInteractionResponse::UpdateMessage(
                    serenity::CreateInteractionResponseMessage::new()
                        .content("🕐 **時間修正**: 修正する記録を選択してください")
                        .components(components),
                ),
            )
            .await?;
    } else {
        interaction
            .create_response(
                &ctx.http,
                serenity::CreateInteractionResponse::Message(
                    serenity::CreateInteractionResponseMessage::new()
                        .content("記録の選択メニューの作成に失敗しました")
                        .ephemeral(true),
                ),
            )
            .await?;
    }

    Ok(())
}

async fn handle_record_add(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    _data: &Data,
) -> Result<(), Error> {
    let user_id = interaction.user.id.to_string();

    // Create buttons for start/end selection with user ID included
    let components = vec![serenity::CreateActionRow::Buttons(vec![
        serenity::CreateButton::new(format!("add_start_record:{}", user_id))
            .label("🟢 開始記録を追加")
            .style(serenity::ButtonStyle::Success),
        serenity::CreateButton::new(format!("add_end_record:{}", user_id))
            .label("🔴 終了記録を追加")
            .style(serenity::ButtonStyle::Danger),
        serenity::CreateButton::new(format!("cancel_add:{}", user_id))
            .label("❌ キャンセル")
            .style(serenity::ButtonStyle::Secondary),
    ])];

    interaction
        .create_response(
            &ctx.http,
            serenity::CreateInteractionResponse::UpdateMessage(
                serenity::CreateInteractionResponseMessage::new()
                    .content("✅ **記録追加**: 追加する記録の種類を選択してください")
                    .components(components),
            ),
        )
        .await?;

    Ok(())
}

async fn handle_delete_record_selection(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    data: &Data,
) -> Result<(), Error> {
    // Get user information
    let user_id = interaction.user.id.to_string();
    let username = interaction.user.name.clone();
    let pool = &data.pool;

    // Get user from database
    let user = match queries::create_or_get_user(pool, &user_id, &username).await {
        Ok(user) => user,
        Err(e) => {
            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content(format_error_message(&format!(
                                "ユーザー情報の取得に失敗しました: {}",
                                e
                            )))
                            .ephemeral(true),
                    ),
                )
                .await?;
            return Ok(());
        }
    };

    let current_date = get_current_date_jst();

    // Get today's records
    let records = match queries::get_today_records(pool, user.id, current_date).await {
        Ok(records) => records,
        Err(e) => {
            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content(format_error_message(&format!(
                                "勤務記録の取得に失敗しました: {}",
                                e
                            )))
                            .ephemeral(true),
                    ),
                )
                .await?;
            return Ok(());
        }
    };

    let record_selector = RecordSelector::new(records);

    if record_selector.is_empty() {
        interaction
            .create_response(
                &ctx.http,
                serenity::CreateInteractionResponse::Message(
                    serenity::CreateInteractionResponseMessage::new()
                        .content("削除できる記録がありません")
                        .ephemeral(true),
                ),
            )
            .await?;
        return Ok(());
    }

    // Create select menu for record deletion
    if let Some(select_menu) = record_selector.create_delete_select_menu("delete_record_select") {
        let components = vec![serenity::CreateActionRow::SelectMenu(select_menu)];

        interaction
            .create_response(
                &ctx.http,
                serenity::CreateInteractionResponse::UpdateMessage(
                    serenity::CreateInteractionResponseMessage::new()
                        .content("🗑️ **記録削除**: 削除する記録を選択してください")
                        .components(components),
                ),
            )
            .await?;
    } else {
        interaction
            .create_response(
                &ctx.http,
                serenity::CreateInteractionResponse::Message(
                    serenity::CreateInteractionResponseMessage::new()
                        .content("削除選択メニューの作成に失敗しました")
                        .ephemeral(true),
                ),
            )
            .await?;
    }

    Ok(())
}

async fn handle_history_view(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    data: &Data,
) -> Result<(), Error> {
    // Get user information
    let user_id = interaction.user.id.to_string();
    let username = interaction.user.name.clone();
    let pool = &data.pool;

    // Get user from database
    let user = match queries::create_or_get_user(pool, &user_id, &username).await {
        Ok(user) => user,
        Err(e) => {
            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content(format_error_message(&format!(
                                "ユーザー情報の取得に失敗しました: {}",
                                e
                            )))
                            .ephemeral(true),
                    ),
                )
                .await?;
            return Ok(());
        }
    };

    // Get available dates for history
    let available_dates = match queries::get_user_available_dates(pool, user.id).await {
        Ok(dates) => dates,
        Err(e) => {
            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content(format_error_message(&format!(
                                "履歴データの取得に失敗しました: {}",
                                e
                            )))
                            .ephemeral(true),
                    ),
                )
                .await?;
            return Ok(());
        }
    };

    if available_dates.is_empty() {
        interaction
            .create_response(
                &ctx.http,
                serenity::CreateInteractionResponse::Message(
                    serenity::CreateInteractionResponseMessage::new()
                        .content("📋 過去30日間に勤務記録がありません")
                        .ephemeral(true),
                ),
            )
            .await?;
        return Ok(());
    }

    // Create date selection menu
    let mut options = Vec::new();

    for date in available_dates.iter().take(20) {
        // Limit to 20 dates to avoid Discord limits
        let date_str = date.format("%Y-%m-%d").to_string();
        let display_str = format!("{} ({})", date.format("%Y/%m/%d"), get_weekday_jp(*date));
        options.push(serenity::CreateSelectMenuOption::new(display_str, date_str));
    }

    let select_menu = serenity::CreateSelectMenu::new(
        "history_date_select",
        serenity::CreateSelectMenuKind::String { options },
    )
    .placeholder("日付を選択してください");

    let components = vec![serenity::CreateActionRow::SelectMenu(select_menu)];

    interaction
        .create_response(
            &ctx.http,
            serenity::CreateInteractionResponse::UpdateMessage(
                serenity::CreateInteractionResponseMessage::new()
                    .content("📋 **履歴表示**: 表示する日付を選択してください")
                    .components(components),
            ),
        )
        .await?;

    Ok(())
}

async fn handle_add_start_record(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    _data: &Data,
) -> Result<(), Error> {
    let modal = serenity::CreateModal::new("add_start_modal", "開始記録追加").components(vec![
        serenity::CreateActionRow::InputText(
            serenity::CreateInputText::new(
                serenity::InputTextStyle::Short,
                "開始時間",
                "start_time",
            )
            .placeholder("HH:MM 形式で入力 (例: 09:00)")
            .required(true)
            .max_length(5),
        ),
    ]);

    interaction
        .create_response(&ctx.http, serenity::CreateInteractionResponse::Modal(modal))
        .await?;

    Ok(())
}

async fn handle_add_end_record(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    _data: &Data,
) -> Result<(), Error> {
    let modal = serenity::CreateModal::new("add_end_modal", "終了記録追加").components(vec![
        serenity::CreateActionRow::InputText(
            serenity::CreateInputText::new(serenity::InputTextStyle::Short, "終了時間", "end_time")
                .placeholder("HH:MM 形式で入力 (例: 18:00)")
                .required(true)
                .max_length(5),
        ),
    ]);

    interaction
        .create_response(&ctx.http, serenity::CreateInteractionResponse::Modal(modal))
        .await?;

    Ok(())
}

async fn handle_cancel_action(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    _data: &Data,
) -> Result<(), Error> {
    interaction
        .create_response(
            &ctx.http,
            serenity::CreateInteractionResponse::UpdateMessage(
                serenity::CreateInteractionResponseMessage::new()
                    .content("操作をキャンセルしました")
                    .components(vec![]),
            ),
        )
        .await?;

    Ok(())
}

async fn handle_edit_record_selected(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    _data: &Data,
) -> Result<(), Error> {
    let selected_record_id =
        if let serenity::ComponentInteractionDataKind::StringSelect { values } =
            &interaction.data.kind
        {
            values.first().cloned().unwrap_or_default()
        } else {
            String::new()
        };

    let modal = serenity::CreateModal::new("time_edit_modal", "時間修正").components(vec![
        serenity::CreateActionRow::InputText(
            serenity::CreateInputText::new(
                serenity::InputTextStyle::Short,
                "新しい時間",
                "new_time",
            )
            .placeholder("HH:MM 形式で入力 (例: 09:30)")
            .required(true)
            .max_length(5),
        ),
        serenity::CreateActionRow::InputText(
            serenity::CreateInputText::new(
                serenity::InputTextStyle::Short,
                "記録ID (変更不可)",
                "record_id",
            )
            .value(&selected_record_id)
            .required(false)
            .max_length(10),
        ),
    ]);

    interaction
        .create_response(&ctx.http, serenity::CreateInteractionResponse::Modal(modal))
        .await?;

    Ok(())
}

async fn handle_delete_record_selected(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    _data: &Data,
) -> Result<(), Error> {
    let user_id = interaction.user.id.to_string();

    let selected_value = if let serenity::ComponentInteractionDataKind::StringSelect { values } =
        &interaction.data.kind
    {
        values.first().cloned().unwrap_or_default()
    } else {
        String::new()
    };

    let (content, button_id) = if selected_value == "delete_all" {
        (
            "すべての記録を削除しますか？",
            format!("confirm_delete_all:{}", user_id),
        )
    } else {
        // Include the record_id in the button for individual deletion
        (
            "選択した記録を削除しますか？",
            format!("confirm_delete_single:{}:{}", user_id, selected_value),
        )
    };

    let components = vec![serenity::CreateActionRow::Buttons(vec![
        serenity::CreateButton::new(&button_id)
            .label("🗑️ 削除する")
            .style(serenity::ButtonStyle::Danger),
        serenity::CreateButton::new(format!("cancel_delete:{}", user_id))
            .label("❌ キャンセル")
            .style(serenity::ButtonStyle::Secondary),
    ])];

    interaction
        .create_response(
            &ctx.http,
            serenity::CreateInteractionResponse::UpdateMessage(
                serenity::CreateInteractionResponseMessage::new()
                    .content(format!("⚠️ **確認**: {}", content))
                    .components(components),
            ),
        )
        .await?;

    Ok(())
}

pub async fn handle_status_modal(
    ctx: &serenity::Context,
    interaction: &serenity::ModalInteraction,
    data: &Data,
) -> Result<(), Error> {
    let custom_id = &interaction.data.custom_id;

    match custom_id.as_str() {
        "time_edit_modal" => handle_time_edit_modal(ctx, interaction, data).await,
        "add_start_modal" => handle_add_start_modal(ctx, interaction, data).await,
        "add_end_modal" => handle_add_end_modal(ctx, interaction, data).await,
        _ => {
            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content("未実装のモーダルです")
                            .ephemeral(true),
                    ),
                )
                .await?;
            Ok(())
        }
    }
}

async fn handle_time_edit_modal(
    ctx: &serenity::Context,
    interaction: &serenity::ModalInteraction,
    data: &Data,
) -> Result<(), Error> {
    // Get time input from modal
    let time_input = interaction
        .data
        .components
        .first()
        .and_then(|row| row.components.first())
        .and_then(|component| {
            if let serenity::ActionRowComponent::InputText(input) = component {
                input.value.as_deref()
            } else {
                None
            }
        })
        .unwrap_or("");

    // Get record ID from modal
    let record_id_str = interaction
        .data
        .components
        .get(1)
        .and_then(|row| row.components.first())
        .and_then(|component| {
            if let serenity::ActionRowComponent::InputText(input) = component {
                input.value.as_deref()
            } else {
                None
            }
        })
        .unwrap_or("");

    // Parse record ID
    let record_id = match record_id_str.parse::<i64>() {
        Ok(id) => id,
        Err(_) => {
            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content(format_error_message("無効な記録IDです"))
                            .ephemeral(true),
                    ),
                )
                .await?;
            return Ok(());
        }
    };

    // Validate time format
    let new_time = match validate_time_format(time_input) {
        Ok(time) => time,
        Err(e) => {
            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content(format_error_message(&e.to_string()))
                            .ephemeral(true),
                    ),
                )
                .await?;
            return Ok(());
        }
    };

    // Combine with current date in JST
    let current_date = get_current_date_jst();
    let new_datetime = combine_date_time_jst(current_date, new_time);

    // Get current records for validation
    let user_id = interaction.user.id.to_string();
    let username = interaction.user.name.clone();
    let pool = &data.pool;

    let user = match queries::create_or_get_user(pool, &user_id, &username).await {
        Ok(user) => user,
        Err(e) => {
            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content(format_error_message(&format!(
                                "ユーザー情報の取得に失敗しました: {}",
                                e
                            )))
                            .ephemeral(true),
                    ),
                )
                .await?;
            return Ok(());
        }
    };

    let existing_records = match queries::get_today_records(pool, user.id, current_date).await {
        Ok(records) => records,
        Err(e) => {
            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content(format_error_message(&format!(
                                "記録の取得に失敗しました: {}",
                                e
                            )))
                            .ephemeral(true),
                    ),
                )
                .await?;
            return Ok(());
        }
    };

    // Get the record being modified to determine its type
    let record_being_modified = existing_records.iter().find(|r| r.id == record_id);

    if let Some(record) = record_being_modified {
        let record_type = RecordType::from(record.record_type.clone());

        // Validate the modification
        if let Err(e) = RecordValidator::validate_new_record(
            &existing_records,
            record_type,
            new_datetime,
            current_date,
            Some(record_id),
        ) {
            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content(format_error_message(&e.to_string()))
                            .ephemeral(true),
                    ),
                )
                .await?;
            return Ok(());
        }
    }

    // Update the record
    match queries::update_attendance_record_time(pool, record_id, new_datetime).await {
        Ok(()) => {
            // Recalculate sessions after modification
            let session_manager = SessionManager::new(pool.clone());
            let user_discord_id = interaction.user.id.to_string();
            let username = interaction.user.name.clone();

            if let Ok(user) = queries::create_or_get_user(pool, &user_discord_id, &username).await {
                if let Err(e) = session_manager
                    .trigger_recalculation(user.id, current_date)
                    .await
                {
                    tracing::error!("Failed to recalculate sessions: {}", e);
                }
            }

            let embed = create_success_embed(
                "時間修正完了",
                &format!("記録の時間を{}に修正しました", time_input),
            );
            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .embed(embed)
                            .ephemeral(true),
                    ),
                )
                .await?;
        }
        Err(e) => {
            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content(format_error_message(&format!(
                                "時間修正に失敗しました: {}",
                                e
                            )))
                            .ephemeral(true),
                    ),
                )
                .await?;
        }
    }

    Ok(())
}

async fn handle_add_start_modal(
    ctx: &serenity::Context,
    interaction: &serenity::ModalInteraction,
    data: &Data,
) -> Result<(), Error> {
    let time_input = interaction
        .data
        .components
        .first()
        .and_then(|row| row.components.first())
        .and_then(|component| {
            if let serenity::ActionRowComponent::InputText(input) = component {
                input.value.as_deref()
            } else {
                None
            }
        })
        .unwrap_or("");

    // Validate time format
    let new_time = match validate_time_format(time_input) {
        Ok(time) => time,
        Err(e) => {
            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content(format_error_message(&e.to_string()))
                            .ephemeral(true),
                    ),
                )
                .await?;
            return Ok(());
        }
    };

    // Get user information
    let user_id = interaction.user.id.to_string();
    let username = interaction.user.name.clone();
    let pool = &data.pool;

    // Get user from database
    let user = match queries::create_or_get_user(pool, &user_id, &username).await {
        Ok(user) => user,
        Err(e) => {
            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content(format_error_message(&format!(
                                "ユーザー情報の取得に失敗しました: {}",
                                e
                            )))
                            .ephemeral(true),
                    ),
                )
                .await?;
            return Ok(());
        }
    };

    // Combine with current date in JST
    let current_date = get_current_date_jst();
    let new_datetime = combine_date_time_jst(current_date, new_time);

    // Get existing records for validation
    let existing_records = match queries::get_today_records(pool, user.id, current_date).await {
        Ok(records) => records,
        Err(e) => {
            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content(format_error_message(&format!(
                                "記録の取得に失敗しました: {}",
                                e
                            )))
                            .ephemeral(true),
                    ),
                )
                .await?;
            return Ok(());
        }
    };

    // Validate the new start record
    if let Err(e) = RecordValidator::validate_new_record(
        &existing_records,
        RecordType::Start,
        new_datetime,
        current_date,
        None,
    ) {
        interaction
            .create_response(
                &ctx.http,
                serenity::CreateInteractionResponse::Message(
                    serenity::CreateInteractionResponseMessage::new()
                        .content(format_error_message(&e.to_string()))
                        .ephemeral(true),
                ),
            )
            .await?;
        return Ok(());
    }

    // Create attendance record
    match queries::create_attendance_record(pool, user.id, RecordType::Start, new_datetime).await {
        Ok(_) => {
            // Recalculate sessions after adding record
            let session_manager = SessionManager::new(pool.clone());
            if let Err(e) = session_manager
                .trigger_recalculation(user.id, current_date)
                .await
            {
                tracing::error!("Failed to recalculate sessions: {}", e);
            }

            let embed = create_success_embed(
                "記録追加完了",
                &format!("開始記録を{}に追加しました", time_input),
            );
            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .embed(embed)
                            .ephemeral(true),
                    ),
                )
                .await?;
        }
        Err(e) => {
            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content(format_error_message(&format!(
                                "開始記録の追加に失敗しました: {}",
                                e
                            )))
                            .ephemeral(true),
                    ),
                )
                .await?;
        }
    }

    Ok(())
}

async fn handle_add_end_modal(
    ctx: &serenity::Context,
    interaction: &serenity::ModalInteraction,
    data: &Data,
) -> Result<(), Error> {
    let time_input = interaction
        .data
        .components
        .first()
        .and_then(|row| row.components.first())
        .and_then(|component| {
            if let serenity::ActionRowComponent::InputText(input) = component {
                input.value.as_deref()
            } else {
                None
            }
        })
        .unwrap_or("");

    // Validate time format
    let new_time = match validate_time_format(time_input) {
        Ok(time) => time,
        Err(e) => {
            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content(format_error_message(&e.to_string()))
                            .ephemeral(true),
                    ),
                )
                .await?;
            return Ok(());
        }
    };

    // Get user information
    let user_id = interaction.user.id.to_string();
    let username = interaction.user.name.clone();
    let pool = &data.pool;

    // Get user from database
    let user = match queries::create_or_get_user(pool, &user_id, &username).await {
        Ok(user) => user,
        Err(e) => {
            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content(format_error_message(&format!(
                                "ユーザー情報の取得に失敗しました: {}",
                                e
                            )))
                            .ephemeral(true),
                    ),
                )
                .await?;
            return Ok(());
        }
    };

    // Combine with current date in JST
    let current_date = get_current_date_jst();
    let new_datetime = combine_date_time_jst(current_date, new_time);

    // Get existing records for validation
    let existing_records = match queries::get_today_records(pool, user.id, current_date).await {
        Ok(records) => records,
        Err(e) => {
            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content(format_error_message(&format!(
                                "記録の取得に失敗しました: {}",
                                e
                            )))
                            .ephemeral(true),
                    ),
                )
                .await?;
            return Ok(());
        }
    };

    // Validate the new end record
    if let Err(e) = RecordValidator::validate_new_record(
        &existing_records,
        RecordType::End,
        new_datetime,
        current_date,
        None,
    ) {
        interaction
            .create_response(
                &ctx.http,
                serenity::CreateInteractionResponse::Message(
                    serenity::CreateInteractionResponseMessage::new()
                        .content(format_error_message(&e.to_string()))
                        .ephemeral(true),
                ),
            )
            .await?;
        return Ok(());
    }

    // Create attendance record
    match queries::create_attendance_record(pool, user.id, RecordType::End, new_datetime).await {
        Ok(_) => {
            // Recalculate sessions after adding record
            let session_manager = SessionManager::new(pool.clone());
            if let Err(e) = session_manager
                .trigger_recalculation(user.id, current_date)
                .await
            {
                tracing::error!("Failed to recalculate sessions: {}", e);
            }

            let embed = create_success_embed(
                "記録追加完了",
                &format!("終了記録を{}に追加しました", time_input),
            );
            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .embed(embed)
                            .ephemeral(true),
                    ),
                )
                .await?;
        }
        Err(e) => {
            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content(format_error_message(&format!(
                                "終了記録の追加に失敗しました: {}",
                                e
                            )))
                            .ephemeral(true),
                    ),
                )
                .await?;
        }
    }

    Ok(())
}

async fn handle_confirm_delete_single(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    data: &Data,
) -> Result<(), Error> {
    // Parse custom_id to get record_id: "confirm_delete_single:user_id:record_id"
    let custom_id = &interaction.data.custom_id;
    let parts: Vec<&str> = custom_id.split(':').collect();

    let record_id = if parts.len() >= 3 {
        match parts[2].parse::<i64>() {
            Ok(id) => id,
            Err(_) => {
                interaction
                    .create_response(
                        &ctx.http,
                        serenity::CreateInteractionResponse::Message(
                            serenity::CreateInteractionResponseMessage::new()
                                .content(format_error_message("無効な記録IDです"))
                                .ephemeral(true),
                        ),
                    )
                    .await?;
                return Ok(());
            }
        }
    } else {
        interaction
            .create_response(
                &ctx.http,
                serenity::CreateInteractionResponse::Message(
                    serenity::CreateInteractionResponseMessage::new()
                        .content(format_error_message("記録IDが指定されていません"))
                        .ephemeral(true),
                ),
            )
            .await?;
        return Ok(());
    };

    // Get user information
    let user_id = interaction.user.id.to_string();
    let username = interaction.user.name.clone();
    let pool = &data.pool;

    // Get user from database
    let user = match queries::create_or_get_user(pool, &user_id, &username).await {
        Ok(user) => user,
        Err(e) => {
            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content(format_error_message(&format!(
                                "ユーザー情報の取得に失敗しました: {}",
                                e
                            )))
                            .ephemeral(true),
                    ),
                )
                .await?;
            return Ok(());
        }
    };

    let current_date = get_current_date_jst();

    // Get the specific record to verify it belongs to this user
    let records = match queries::get_today_records(pool, user.id, current_date).await {
        Ok(records) => records,
        Err(e) => {
            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content(format_error_message(&format!(
                                "勤務記録の取得に失敗しました: {}",
                                e
                            )))
                            .ephemeral(true),
                    ),
                )
                .await?;
            return Ok(());
        }
    };

    // Verify the record exists and belongs to this user
    let record_exists = records.iter().any(|record| record.id == record_id);
    if !record_exists {
        interaction
            .create_response(
                &ctx.http,
                serenity::CreateInteractionResponse::Message(
                    serenity::CreateInteractionResponseMessage::new()
                        .content(format_error_message("指定された記録が見つかりません"))
                        .ephemeral(true),
                ),
            )
            .await?;
        return Ok(());
    }

    // Delete the specific record using the simple queries
    match sqlx::query("DELETE FROM attendance_records WHERE id = ? AND user_id = ?")
        .bind(record_id)
        .bind(user.id)
        .execute(pool)
        .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                // Recalculate sessions after deletion
                let session_manager = SessionManager::new(pool.clone());
                if let Err(e) = session_manager
                    .trigger_recalculation(user.id, current_date)
                    .await
                {
                    tracing::error!("Failed to recalculate sessions: {}", e);
                }

                let embed = create_success_embed("削除完了", "選択した記録を削除しました");
                interaction
                    .create_response(
                        &ctx.http,
                        serenity::CreateInteractionResponse::UpdateMessage(
                            serenity::CreateInteractionResponseMessage::new()
                                .embed(embed)
                                .components(vec![]),
                        ),
                    )
                    .await?;
            } else {
                interaction
                    .create_response(
                        &ctx.http,
                        serenity::CreateInteractionResponse::Message(
                            serenity::CreateInteractionResponseMessage::new()
                                .content(format_error_message(
                                    "記録の削除に失敗しました（記録が見つかりません）",
                                ))
                                .ephemeral(true),
                        ),
                    )
                    .await?;
            }
        }
        Err(e) => {
            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content(format_error_message(&format!(
                                "記録の削除に失敗しました: {}",
                                e
                            )))
                            .ephemeral(true),
                    ),
                )
                .await?;
        }
    }

    Ok(())
}

async fn handle_confirm_delete_all(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    data: &Data,
) -> Result<(), Error> {
    // Get user information
    let user_id = interaction.user.id.to_string();
    let username = interaction.user.name.clone();
    let pool = &data.pool;

    // Get user from database
    let user = match queries::create_or_get_user(pool, &user_id, &username).await {
        Ok(user) => user,
        Err(e) => {
            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content(format_error_message(&format!(
                                "ユーザー情報の取得に失敗しました: {}",
                                e
                            )))
                            .ephemeral(true),
                    ),
                )
                .await?;
            return Ok(());
        }
    };

    let current_date = get_current_date_jst();

    // Delete all records for today
    match queries::delete_all_user_records_for_date(pool, user.id, current_date).await {
        Ok(()) => {
            // Recalculate sessions after deletion
            let session_manager = SessionManager::new(pool.clone());
            if let Err(e) = session_manager
                .trigger_recalculation(user.id, current_date)
                .await
            {
                tracing::error!("Failed to recalculate sessions: {}", e);
            }

            let embed = create_success_embed("削除完了", "当日のすべての記録を削除しました");
            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::UpdateMessage(
                        serenity::CreateInteractionResponseMessage::new()
                            .embed(embed)
                            .components(vec![]),
                    ),
                )
                .await?;
        }
        Err(e) => {
            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content(format_error_message(&format!(
                                "記録の削除に失敗しました: {}",
                                e
                            )))
                            .ephemeral(true),
                    ),
                )
                .await?;
        }
    }

    Ok(())
}

async fn handle_history_date_selected(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    data: &Data,
) -> Result<(), Error> {
    let selected_date_str = if let serenity::ComponentInteractionDataKind::StringSelect { values } =
        &interaction.data.kind
    {
        values.first().cloned().unwrap_or_default()
    } else {
        String::new()
    };

    // Parse the selected date
    let selected_date = match chrono::NaiveDate::parse_from_str(&selected_date_str, "%Y-%m-%d") {
        Ok(date) => date,
        Err(_) => {
            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content(format_error_message("無効な日付が選択されました"))
                            .ephemeral(true),
                    ),
                )
                .await?;
            return Ok(());
        }
    };

    // Get user information
    let user_id = interaction.user.id.to_string();
    let username = interaction.user.name.clone();
    let pool = &data.pool;

    let user = match queries::create_or_get_user(pool, &user_id, &username).await {
        Ok(user) => user,
        Err(e) => {
            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content(format_error_message(&format!(
                                "ユーザー情報の取得に失敗しました: {}",
                                e
                            )))
                            .ephemeral(true),
                    ),
                )
                .await?;
            return Ok(());
        }
    };

    // Get records for the selected date
    let records = match queries::get_records_by_date(pool, user.id, selected_date).await {
        Ok(records) => records,
        Err(e) => {
            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content(format_error_message(&format!(
                                "記録の取得に失敗しました: {}",
                                e
                            )))
                            .ephemeral(true),
                    ),
                )
                .await?;
            return Ok(());
        }
    };

    if records.is_empty() {
        interaction
            .create_response(
                &ctx.http,
                serenity::CreateInteractionResponse::UpdateMessage(
                    serenity::CreateInteractionResponseMessage::new()
                        .content(format!(
                            "📋 {} ({}) の記録はありません",
                            selected_date.format("%Y/%m/%d"),
                            get_weekday_jp(selected_date)
                        ))
                        .components(vec![]),
                ),
            )
            .await?;
        return Ok(());
    }

    // Format the historical records
    let content = format!(
        "📋 **{} ({}) の勤務記録**\n\n{}",
        selected_date.format("%Y/%m/%d"),
        get_weekday_jp(selected_date),
        crate::utils::format::format_attendance_status(&records)
    );

    interaction
        .create_response(
            &ctx.http,
            serenity::CreateInteractionResponse::UpdateMessage(
                serenity::CreateInteractionResponseMessage::new()
                    .content(&content)
                    .components(vec![]),
            ),
        )
        .await?;

    Ok(())
}

fn get_weekday_jp(date: NaiveDate) -> &'static str {
    match date.weekday() {
        chrono::Weekday::Mon => "月",
        chrono::Weekday::Tue => "火",
        chrono::Weekday::Wed => "水",
        chrono::Weekday::Thu => "木",
        chrono::Weekday::Fri => "金",
        chrono::Weekday::Sat => "土",
        chrono::Weekday::Sun => "日",
    }
}
