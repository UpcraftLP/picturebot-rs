mod reload;
mod upload;

use rusty_interaction::handler::InteractionHandler;

pub(crate) fn register_commands(handler: &mut InteractionHandler) {
    handler.add_global_command("reload", reload::reload_commands);
    handler.add_global_command("upload", upload::upload_command);
}

