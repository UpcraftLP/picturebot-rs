use std::env;

use anyhow::Context;
use s3::{Bucket, Region};
use s3::creds::Credentials;

use crate::upload::UploaderImpl;

#[derive(Debug, Clone)]
pub struct S3Uploader {
    bucket: Bucket,
    storage_path: String,
    frontend_url: String,
}

impl S3Uploader {

    pub(crate) fn new(frontend_url: &str, credentials: Credentials, region: Region, bucket_name: &str, use_path_style: bool, storage_path: Option<&str>) -> anyhow::Result<Self> {
        let mut bucket = Bucket::new(bucket_name, region, credentials)?;

        if use_path_style {
            bucket.set_path_style();
        }

        let mut storage_path_mut = storage_path.unwrap_or("").to_string();

        if storage_path_mut.ends_with('/') {
            storage_path_mut.pop();
        }

        let mut frontend_url_mut = frontend_url.to_string();
        if frontend_url_mut.ends_with('/') {
            frontend_url_mut.pop();
        }

        Ok(S3Uploader {
            bucket,
            storage_path: storage_path_mut,
            frontend_url: frontend_url_mut,
        })
    }

    pub(crate) fn from_env() -> anyhow::Result<Self> {
        // AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY, AWS_SECURITY_TOKEN, AWS_SESSION_TOKEN
        let credentials = Credentials::from_env()?;
        // AWS_ENDPOINT, AWS_REGION
        let region = Region::from_default_env()?;

        let bucket_name = env::var("AWS_S3_BUCKET_NAME")
            .context("AWS_S3_BUCKET_NAME is not set")?;

        let use_path_style = env::var("AWS_S3_USE_PATH_STYLE")
            .map(|s| s.parse::<bool>()).ok().transpose()
            .context("Failed to parse AWS_S3_USE_PATH_STYLE")?.unwrap_or(false);

        let storage_path = env::var("AWS_S3_STORAGE_PATH").unwrap_or("".to_string());

        let frontend_url = env::var("UPLOAD_FRONTEND_URL")
            .context("UPLOAD_FRONTEND_URL must be set when using S3 storage")?;

        S3Uploader::new(frontend_url.as_str(), credentials, region, &bucket_name, use_path_style, Some(storage_path.as_str()))
    }
}

impl UploaderImpl for S3Uploader {
    async fn upload(&self, path: &str, bytes: Vec<u8>, content_type: &str) -> anyhow::Result<String> {
        todo!("S3Uploader::upload")
    }

    fn frontend_url(&self, path: &str) -> String {
        format!("{}/{}", self.frontend_url, path)
    }
}