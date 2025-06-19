use crate::bot::{Data, Error};
use crate::database::queries;
use crate::database::models::RecordType;
use crate::utils::format::{format_success_message, format_error_message};
use crate::utils::validation::validate_time_format;
use crate::utils::record_selector::RecordSelector;
use crate::utils::time::{get_current_date_jst, combine_date_time_jst};
use crate::utils::session_manager::SessionManager;
use crate::utils::record_validator::RecordValidator;
use poise::serenity_prelude as serenity;

pub async fn handle_status_interaction(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    data: &Data,
) -> Result<(), Error> {
    let custom_id = &interaction.data.custom_id;
    
    match custom_id.as_str() {
        // Button interactions
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
        
        // Select menu interactions
        "edit_record_select" => handle_edit_record_selected(ctx, interaction, data).await,
        "delete_record_select" => handle_delete_record_selected(ctx, interaction, data).await,
        
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
                            .content(format_error_message(&format!("ユーザー情報の取得に失敗しました: {}", e)))
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
                            .content(format_error_message(&format!("勤務記録の取得に失敗しました: {}", e)))
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
    if let Some(select_menu) = record_selector.create_select_menu("edit_record_select", "修正する記録を選択してください") {
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
    // Create buttons for start/end selection
    let components = vec![serenity::CreateActionRow::Buttons(vec![
        serenity::CreateButton::new("add_start_record")
            .label("🟢 開始記録を追加")
            .style(serenity::ButtonStyle::Success),
        serenity::CreateButton::new("add_end_record")
            .label("🔴 終了記録を追加")
            .style(serenity::ButtonStyle::Danger),
        serenity::CreateButton::new("cancel_add")
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
                            .content(format_error_message(&format!("ユーザー情報の取得に失敗しました: {}", e)))
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
                            .content(format_error_message(&format!("勤務記録の取得に失敗しました: {}", e)))
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
    _data: &Data,
) -> Result<(), Error> {
    interaction
        .create_response(
            &ctx.http,
            serenity::CreateInteractionResponse::Message(
                serenity::CreateInteractionResponseMessage::new()
                    .content("📋 履歴機能は今後実装予定です")
                    .ephemeral(true),
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
    let modal = serenity::CreateModal::new("add_start_modal", "開始記録追加")
        .components(vec![serenity::CreateActionRow::InputText(
            serenity::CreateInputText::new(
                serenity::InputTextStyle::Short,
                "開始時間",
                "start_time",
            )
            .placeholder("HH:MM 形式で入力 (例: 09:00)")
            .required(true)
            .max_length(5),
        )]);

    interaction
        .create_response(
            &ctx.http,
            serenity::CreateInteractionResponse::Modal(modal),
        )
        .await?;

    Ok(())
}

async fn handle_add_end_record(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    _data: &Data,
) -> Result<(), Error> {
    let modal = serenity::CreateModal::new("add_end_modal", "終了記録追加")
        .components(vec![serenity::CreateActionRow::InputText(
            serenity::CreateInputText::new(
                serenity::InputTextStyle::Short,
                "終了時間",
                "end_time",
            )
            .placeholder("HH:MM 形式で入力 (例: 18:00)")
            .required(true)
            .max_length(5),
        )]);

    interaction
        .create_response(
            &ctx.http,
            serenity::CreateInteractionResponse::Modal(modal),
        )
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
    let selected_record_id = if let serenity::ComponentInteractionDataKind::StringSelect { values } = &interaction.data.kind {
        values.first().map(|s| s.clone()).unwrap_or_default()
    } else {
        String::new()
    };

    let modal = serenity::CreateModal::new("time_edit_modal", "時間修正")
        .components(vec![
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
        .create_response(
            &ctx.http,
            serenity::CreateInteractionResponse::Modal(modal),
        )
        .await?;

    Ok(())
}

async fn handle_delete_record_selected(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    _data: &Data,
) -> Result<(), Error> {
    let selected_value = if let serenity::ComponentInteractionDataKind::StringSelect { values } = &interaction.data.kind {
        values.first().map(|s| s.clone()).unwrap_or_default()
    } else {
        String::new()
    };

    let (content, button_id) = if selected_value == "delete_all" {
        ("すべての記録を削除しますか？", "confirm_delete_all")
    } else {
        ("選択した記録を削除しますか？", "confirm_delete_single")
    };

    let components = vec![serenity::CreateActionRow::Buttons(vec![
        serenity::CreateButton::new(button_id)
            .label("🗑️ 削除する")
            .style(serenity::ButtonStyle::Danger),
        serenity::CreateButton::new("cancel_delete")
            .label("❌ キャンセル")
            .style(serenity::ButtonStyle::Secondary),
    ])];

    interaction
        .create_response(
            &ctx.http,
            serenity::CreateInteractionResponse::UpdateMessage(
                serenity::CreateInteractionResponseMessage::new()
                    .content(&format!("⚠️ **確認**: {}", content))
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
        .get(0)
        .and_then(|row| row.components.get(0))
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
        .and_then(|row| row.components.get(0))
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
                            .content(format_error_message(&format!("ユーザー情報の取得に失敗しました: {}", e)))
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
                            .content(format_error_message(&format!("記録の取得に失敗しました: {}", e)))
                            .ephemeral(true),
                    ),
                )
                .await?;
            return Ok(());
        }
    };

    // Get the record being modified to determine its type
    let record_being_modified = existing_records.iter()
        .find(|r| r.id == record_id);
    
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
                if let Err(e) = session_manager.trigger_recalculation(user.id, current_date).await {
                    tracing::error!("Failed to recalculate sessions: {}", e);
                }
            }

            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content(format_success_message(&format!(
                                "記録の時間を{}に修正しました",
                                time_input
                            )))
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
        .get(0)
        .and_then(|row| row.components.get(0))
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
                            .content(format_error_message(&format!("ユーザー情報の取得に失敗しました: {}", e)))
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
                            .content(format_error_message(&format!("記録の取得に失敗しました: {}", e)))
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
            if let Err(e) = session_manager.trigger_recalculation(user.id, current_date).await {
                tracing::error!("Failed to recalculate sessions: {}", e);
            }

            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content(format_success_message(&format!(
                                "開始記録を{}に追加しました",
                                time_input
                            )))
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
        .get(0)
        .and_then(|row| row.components.get(0))
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
                            .content(format_error_message(&format!("ユーザー情報の取得に失敗しました: {}", e)))
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
                            .content(format_error_message(&format!("記録の取得に失敗しました: {}", e)))
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
            if let Err(e) = session_manager.trigger_recalculation(user.id, current_date).await {
                tracing::error!("Failed to recalculate sessions: {}", e);
            }

            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content(format_success_message(&format!(
                                "終了記録を{}に追加しました",
                                time_input
                            )))
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
    _data: &Data,
) -> Result<(), Error> {
    interaction
        .create_response(
            &ctx.http,
            serenity::CreateInteractionResponse::Message(
                serenity::CreateInteractionResponseMessage::new()
                    .content(format_success_message("個別削除機能は今後実装予定です"))
                    .ephemeral(true),
            ),
        )
        .await?;

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
                            .content(format_error_message(&format!("ユーザー情報の取得に失敗しました: {}", e)))
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
            if let Err(e) = session_manager.trigger_recalculation(user.id, current_date).await {
                tracing::error!("Failed to recalculate sessions: {}", e);
            }

            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::UpdateMessage(
                        serenity::CreateInteractionResponseMessage::new()
                            .content(format_success_message("当日のすべての記録を削除しました"))
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