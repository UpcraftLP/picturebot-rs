use rusty_interaction::Builder;
use rusty_interaction::handler::InteractionHandler;
use rusty_interaction::types::application::{ApplicationCommand, ApplicationCommandOption, ApplicationCommandOptionType, SlashCommandDefinitionBuilder};
use rusty_interaction::types::Snowflake;

const BASE_URL: &str = rusty_interaction::BASE_URL;

pub(crate) async fn update_global_commands(handler: &InteractionHandler, app_id: Snowflake) -> anyhow::Result<()> {
    let commands: Vec<ApplicationCommand> = vec![
        SlashCommandDefinitionBuilder::default()
            .name("reload")
            .description("Reload the commands")
            .default_permission(false)
            .build().unwrap(),
        SlashCommandDefinitionBuilder::default()
            .name("upload")
            .description("Upload an image or video")
            .add_option(ApplicationCommandOption::default()
                            .name("file")
                            .option_type(&ApplicationCommandOptionType::Attachment)
                            .required(&true)
                            .description("The image or video to upload"),
            )
            .add_option(ApplicationCommandOption::default()
                            .name("file-name")
                            .option_type(&ApplicationCommandOptionType::String)
                            .required(&false)
                            .description("The desired file name, otherwise uses the attachment name"),
            )
            .build().unwrap(),
    ];

    let url = format!("{BASE_URL}/applications/{app_id}/commands");
    let response = handler.client().clone().put(url).json(&commands).send().await?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to update global commands: {:?}", response.text().await?);
    }

    Ok(())
}