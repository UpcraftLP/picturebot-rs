use std::env;
use std::str::FromStr;

use anyhow::Context;
use s3::{Bucket, Region};
use s3::creds::Credentials;
use s3::error::S3Error;
use s3::serde_types::HeadObjectResult;

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
        let s3_region = env::var("S3_REGION").ok();
        let s3_url = env::var("S3_URL").ok();

        if s3_region.is_none() && s3_url.is_none() {
            anyhow::bail!("One of S3_REGION or S3_URL must be set when using S3 storage!");
        }

        let region = match s3_url {
            Some(endpoint) => Region::Custom {
                endpoint,
                region: s3_region.unwrap_or("custom".to_string()),
            },
            None => {
                let region_string = s3_region.unwrap();
                Region::from_str(region_string.as_str())
                    .context("Unknwon S3 region: {region_string}")?
            }
        };

        let access_key = env::var("S3_ACCESS_KEY_ID")
            .context("S3_ACCESS_KEY_ID is not set")?;
        let secret_key = env::var("S3_SECRET_ACCESS_KEY")
            .context("S3_SECRET_ACCESS_KEY is not set")?;

        let security_token = env::var("S3_SECURITY_TOKEN").ok();
        let session_token = env::var("S3_SESSION_TOKEN").ok();
        let profile = env::var("S3_PROFILE").ok();

        let credentials = Credentials::new(Some(access_key.as_str()), Some(secret_key.as_str()), security_token.as_deref(), session_token.as_deref(), profile.as_deref())
            .context("Failed to create S3 credentials")?;

        let bucket_name = env::var("S3_BUCKET_NAME")
            .context("S3_BUCKET_NAME is not set")?;

        let use_path_style = env::var("S3_USE_PATH_STYLE")
            .map(|s| s.parse::<bool>()).ok().transpose()
            .context("Failed to parse S3_USE_PATH_STYLE")?.unwrap_or(false);

        let storage_path = env::var("S3_STORAGE_PATH").unwrap_or("".to_string());

        let frontend_url = env::var("UPLOAD_FRONTEND_URL")
            .context("UPLOAD_FRONTEND_URL must be set when using S3 storage")?;

        S3Uploader::new(frontend_url.as_str(), credentials, region, &bucket_name, use_path_style, Some(storage_path.as_str()))
    }
}

impl UploaderImpl for S3Uploader {
    async fn upload(&self, path: &str, bytes: Vec<u8>, _content_type: &str) -> anyhow::Result<String> {
        let path = format!("{}/{}", self.storage_path, path);
        match check_file_exists(&self.bucket, path.as_str()).await? {
            None => {
                self.bucket.put_object(path.as_str(), bytes.as_slice()).await?;
                log::info!("Uploaded file to {bucket}:/{path}", bucket = &self.bucket.name);
                Ok(self.frontend_url(path.as_str()))
            }
            Some(_) => anyhow::bail!("File already exists"),
        }
    }

    fn frontend_url(&self, path: &str) -> String {
        format!("{}/{}", self.frontend_url, path)
    }
}

async fn check_file_exists(bucket: &Bucket, file: &str) -> Result<Option<HeadObjectResult>, S3Error> {
    match bucket.head_object(file).await {
        Ok((result, status)) => {
            if status == 200u16 {
                return Ok(Some(result));
            }
            Ok(None)
        }
        Err(e) => {
            if let S3Error::Http(code, _) = &e {
                if *code == 404u16 {
                    return Ok(None);
                }
            }
            Err(e)
        }
    }
}