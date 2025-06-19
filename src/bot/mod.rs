pub mod commands;
pub mod handlers;
pub mod interactions;

use crate::config::Config;
use crate::database;
use sqlx::SqlitePool;
use anyhow::Result;
use poise::serenity_prelude as serenity;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Clone)]
pub struct Data {
    pub pool: SqlitePool,
    pub config: Config,
}

pub async fn create_bot(config: Config) -> Result<serenity::Client> {
    let pool = database::create_connection(&config.database_url).await?;
    
    let data = Data {
        pool,
        config: config.clone(),
    };

    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::attendance::start(),
                commands::attendance::end(),
                commands::status::status(),
                commands::reports::daily(),
                commands::reports::weekly(),
                commands::reports::monthly(),
            ],
            event_handler: |ctx, event, framework, data| {
                Box::pin(handlers::event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(data)
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(&config.discord_token, intents)
        .framework(framework)
        .await?;

    Ok(client)
}