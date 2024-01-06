use std::env;
use std::fmt::Display;
use std::str::FromStr;

use anyhow::{anyhow, Context};
use enum_dispatch::enum_dispatch;

use crate::upload::provider::http_bearer::HttpBearerUploader;
use crate::upload::Uploader::HttpBearer;

mod provider;

pub async fn init() -> anyhow::Result<Uploader> {
    let uploader = env::var("UPLOAD_PROVIDER")
        .context("UPLOAD_PROVIDER not set")?
        .parse::<Uploader>()
        .context("Failed to parse UPLOAD_PROVIDER")?;

    Ok(uploader)
}

#[enum_dispatch(UploaderImpl)]
#[derive(Clone)]
pub enum Uploader {
    HttpBearer(HttpBearerUploader),
}

#[enum_dispatch]
pub trait UploaderImpl {
    async fn upload(&self, path: &str, bytes: Vec<u8>, content_type: &str) -> anyhow::Result<String>;

    fn frontend_url(&self, path: &str) -> String;
}

impl Display for Uploader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            HttpBearer(_) => "http_bearer"
        })
    }
}

impl FromStr for Uploader {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "http_bearer" => Ok(HttpBearerUploader::from_env()?.into()),
            _ => Err(anyhow!("Unknown upload provider: {s}")),
        }
    }
}
