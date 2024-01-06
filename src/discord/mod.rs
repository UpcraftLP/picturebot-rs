use anyhow::Context;
use rusty_interaction::handler::InteractionHandler;
use rusty_interaction::types::Snowflake;

use crate::discord::webhook::Webhook;

mod commands;
pub(crate) mod register;
mod webhook;

#[derive(Debug, Clone)]
pub struct BotInfo {
    pub app_id: Snowflake,
    pub owner_id: Snowflake,
    pub webhooks: Option<Vec<Webhook>>,
    pub webhook_logo_url: Option<String>,
}

pub(crate) async fn init() -> anyhow::Result<InteractionHandler> {
    log::info!("Initializing Discord Module");

    let app_id: Snowflake = std::env::var("DISCORD_APP_ID")
        .context("DISCORD_APP_ID not set")?.parse()
        .context("DISCORD_APP_ID is not a valid Snowflake")?;

    let public_key = std::env::var("DISCORD_PUBLIC_KEY")
        .context("DISCORD_PUBLIC_KEY not set")?;

    let token = std::env::var("DISCORD_TOKEN")
        .context("DISCORD_TOKEN not set")?;

    let owner_id: Snowflake = std::env::var("DISCORD_BOT_OWNER_ID")
        .context("DISCORD_BOT_OWNER_ID not set")?.parse()
        .context("DISCORD_BOT_OWNER_ID is not a valid Snowflake")?;

    let webhooks = std::env::var("DISCORD_WEBHOOK_URLS")
        .ok()
        .map(|s| s
            .split(',')
            .filter_map(|s| match Webhook::new(s.to_string()) {
                Ok(webhook) => Some(webhook),
                Err(e) => {
                    log::error!("Failed to create webhook: {e}");
                    None
                }
            })
            .collect::<Vec<Webhook>>()
        );

    let mut webhook_logo_url = None;
    if let Some(webhooks) = &webhooks {
        log::info!("Parsed {} webhooks", webhooks.len());

        webhook_logo_url = std::env::var("DISCORD_WEBHOOK_LOGO_URL").ok();
    }

    let mut handler = InteractionHandler::new(app_id, public_key, Some(&token));

    handler.add_data(BotInfo {
        app_id,
        owner_id,
        webhooks,
        webhook_logo_url,
    });

    commands::register_commands(&mut handler);

    Ok(handler)
}
