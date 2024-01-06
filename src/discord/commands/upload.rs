use rusty_interaction::{defer, slash_command};
use rusty_interaction::handler::InteractionHandler;
use rusty_interaction::types::interaction::{Context, InteractionResponse};
use rusty_interaction::types::Snowflake;

use crate::upload::{Uploader, UploaderImpl};
use crate::util::UploadValidator;

const DISALLOWED_CHARACTERS: [char; 31] = ['(', ')', '[', ']', '{', '}', '-', '+', '*', '=', '&', '@', '!', '?', '\'', '#', '$', '%', '^', '~', '^', '´', '`', ':', ',', ';', '<', '>', '|', '\"', '\\'];

#[defer]
#[slash_command]
pub(crate) async fn upload_command(handler: &mut InteractionHandler, ctx: Context) -> InteractionResponse {
    let validator = handler.data.get::<UploadValidator>().unwrap();
    let uploader = handler.data.get::<Uploader>().unwrap();


    let data = &ctx.interaction.data.clone().unwrap();
    let opts = data.options.clone().unwrap();
    let attachment_option = opts.iter().find(|&o| o.name == "file").expect("No attachment provided");
    let file_name_option = opts.iter().find(|&o| o.name == "file-name");

    let attachment_id: Snowflake = attachment_option.value.parse().expect("Invalid attachment id");
    let attachments = data.resolved.clone().unwrap().attachments.unwrap();
    let attachment = attachments.get(&attachment_id).expect("Attachment not found");

    let desired_file_name = file_name_option.map(|o| o.value.clone()).unwrap_or_else(|| attachment.filename.clone());

    let user_id_string = format!("{:0width$}", &ctx.interaction.member.clone().map(|m| m.user.id).unwrap_or(0), width = 20);
    let prefix = &user_id_string[16..];
    assert_eq!(prefix.len(), 4, "Prefix must be 4 characters long");

    let filename = format!("{prefix}_{desired_file_name}").to_ascii_lowercase();

    if !filename.is_ascii() {
        return ctx.respond().is_ephemeral(true).content("File name must be valid ASCII").finish();
    }

    if filename.chars().any(|c| DISALLOWED_CHARACTERS.contains(&c)) {
        return ctx.respond().is_ephemeral(true).content(format!("File name must not contain any of the following characters: {}", DISALLOWED_CHARACTERS.iter().collect::<String>())).finish();
    }

    let frontend_url = uploader.frontend_url(&filename);
    if let Err(message) = validator.check(&frontend_url, &attachment.filename.to_ascii_lowercase()) {
        return ctx.respond().is_ephemeral(true).content(message).finish();
    }

    let bytes: Vec<u8>;
    match handler.client().clone().get(attachment.url.clone()).send().await {
        Ok(response) => {
            if !response.status().is_success() {
                return ctx.respond().is_ephemeral(true).content("Failed to download attachment").finish();
            }
            match response.bytes().await {
                Ok(b) => {
                    bytes = b.to_vec();
                },
                Err(e) => {
                    return ctx.respond().is_ephemeral(true).content(format!("Failed to download attachment: {e}")).finish();
                }
            };
        }
        Err(e) => {
            return ctx.respond().is_ephemeral(true).content(format!("Failed to download attachment: {e}")).finish();
        }
    }

    let content_type = attachment.content_type.clone().unwrap_or("application/octet-stream".to_string());
    match uploader.upload(&filename, bytes, &content_type).await {
        Ok(result) => {
            log::info!("Successfully uploaded file at {result}");
            ctx.respond().content(format!("successfully uploaded as <{result}>")).finish()
        }
        Err(e) => {
            log::error!("Failed to upload file: {e}");
            ctx.respond().is_ephemeral(true).content(format!("Failed to upload file: {e}")).finish()
        }
    }
}