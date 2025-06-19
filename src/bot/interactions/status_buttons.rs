use crate::bot::{Data, Error};
use crate::utils::format::{format_success_message, format_error_message};
use crate::utils::validation::validate_time_format;
use poise::serenity_prelude as serenity;

pub async fn handle_status_interaction(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    data: &Data,
) -> Result<(), Error> {
    let custom_id = &interaction.data.custom_id;
    
    match custom_id.as_str() {
        "time_edit" => handle_time_edit(ctx, interaction, data).await,
        "end_register" => handle_end_register(ctx, interaction, data).await,
        "delete_record" => handle_delete_record(ctx, interaction, data).await,
        "history_view" => handle_history_view(ctx, interaction, data).await,
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

async fn handle_time_edit(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    _data: &Data,
) -> Result<(), Error> {
    let modal = serenity::CreateModal::new("time_edit_modal", "時間修正")
        .components(vec![serenity::CreateActionRow::InputText(
            serenity::CreateInputText::new(
                serenity::InputTextStyle::Short,
                "時間",
                "edit_time",
            )
            .placeholder("HH:MM 形式で入力 (例: 09:30)")
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

async fn handle_end_register(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    _data: &Data,
) -> Result<(), Error> {
    let modal = serenity::CreateModal::new("end_register_modal", "終了時間登録")
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

async fn handle_delete_record(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    _data: &Data,
) -> Result<(), Error> {
    let components = vec![serenity::CreateActionRow::Buttons(vec![
        serenity::CreateButton::new("delete_start")
            .label("開始時刻を削除")
            .style(serenity::ButtonStyle::Danger),
        serenity::CreateButton::new("delete_end")
            .label("終了時刻を削除")
            .style(serenity::ButtonStyle::Danger),
        serenity::CreateButton::new("delete_all")
            .label("全て削除")
            .style(serenity::ButtonStyle::Danger),
        serenity::CreateButton::new("cancel_delete")
            .label("キャンセル")
            .style(serenity::ButtonStyle::Secondary),
    ])];

    interaction
        .create_response(
            &ctx.http,
            serenity::CreateInteractionResponse::UpdateMessage(
                serenity::CreateInteractionResponseMessage::new()
                    .content("どの記録を削除しますか？")
                    .components(components),
            ),
        )
        .await?;

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

pub async fn handle_status_modal(
    ctx: &serenity::Context,
    interaction: &serenity::ModalInteraction,
    data: &Data,
) -> Result<(), Error> {
    let custom_id = &interaction.data.custom_id;
    
    match custom_id.as_str() {
        "time_edit_modal" => handle_time_edit_modal(ctx, interaction, data).await,
        "end_register_modal" => handle_end_register_modal(ctx, interaction, data).await,
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
    _data: &Data,
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

    match validate_time_format(time_input) {
        Ok(_time) => {
            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content(format_success_message("時間修正機能は今後実装予定です"))
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
                            .content(format_error_message(&e.to_string()))
                            .ephemeral(true),
                    ),
                )
                .await?;
        }
    }

    Ok(())
}

async fn handle_end_register_modal(
    ctx: &serenity::Context,
    interaction: &serenity::ModalInteraction,
    _data: &Data,
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

    match validate_time_format(time_input) {
        Ok(_time) => {
            interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content(format_success_message("終了登録機能は今後実装予定です"))
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
                            .content(format_error_message(&e.to_string()))
                            .ephemeral(true),
                    ),
                )
                .await?;
        }
    }

    Ok(())
}