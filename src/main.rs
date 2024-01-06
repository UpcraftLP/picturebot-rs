use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use crate::discord::BotInfo;
use crate::util::UploadValidator;

mod discord;
mod upload;
mod util;
mod http;

pub mod build_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

static mut VERSION: &str = build_info::PKG_VERSION;
static mut ENVIRONMENT: &str = "development";

pub(crate) fn version() -> &'static str {
    unsafe { VERSION }
}
pub(crate) fn environment() -> &'static str {
    unsafe { ENVIRONMENT }
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    unsafe {
        if let Some(v) = option_env!("VERSION") {
            VERSION = v;
        }
        if let Some(e) = option_env!("RUST_ENVIRONMENT") {
            ENVIRONMENT = e;
        }
    }
    dotenvy::dotenv().ok();

    if environment() == "development" {
        env_logger::try_init_from_env(env_logger::Env::new().default_filter_or("info"))?;
    }
    else {
        tracing_subscriber::fmt().json().init();
    }
    log::info!("Starting PictureBot v{}", version());

    let uploader = upload::init().await?;
    let mut handler = discord::init().await?;
    handler.add_data(uploader);

    let validator = UploadValidator::from_env()?;
    handler.add_data(validator);

    let app_info = handler.data.get::<BotInfo>().expect("AppInfo not found");
    log::info!("Discord Application ID: {}", app_info.app_id);

    let bootstrap_file = PathBuf::from_str("./.bootstrap")?;
    if bootstrap_file.exists() {
        log::warn!("Found bootstrap file, registering commands...");

        discord::register::update_global_commands(&handler, app_info.app_id).await?;

        log::info!("Removing bootstrap file.");
        fs::remove_file(bootstrap_file)?;
    }
    else {
        log::debug!("No bootstrap file found, skipping.");
    }

    handler.run(3000).await?;

    Ok(())
}
