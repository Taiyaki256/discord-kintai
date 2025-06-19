use crate::bot::{Data, Error};
use crate::bot::interactions::status_buttons;
use poise::serenity_prelude as serenity;

pub async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot } => {
            tracing::info!("Bot logged in as {}", data_about_bot.user.name);
        }
        serenity::FullEvent::InteractionCreate { interaction } => {
            match interaction {
                serenity::Interaction::Component(component_interaction) => {
                    if let Err(e) = status_buttons::handle_status_interaction(ctx, component_interaction, data).await {
                        tracing::error!("Error handling component interaction: {:?}", e);
                    }
                }
                serenity::Interaction::Modal(modal_interaction) => {
                    if let Err(e) = status_buttons::handle_status_modal(ctx, modal_interaction, data).await {
                        tracing::error!("Error handling modal interaction: {:?}", e);
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
    Ok(())
}