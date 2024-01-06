use std::env;
use std::str::FromStr;

use anyhow::Context;
use reqwest::header::HeaderName;

use crate::http;
use crate::upload::UploaderImpl;

#[derive(Debug, Clone)]
pub struct HttpBearerUploader {
    upload_url: String,
    frontend_url: String,
    client: reqwest::Client,
}

impl HttpBearerUploader {
    pub(crate) fn new(upload_url: String, frontend_url: String, auth_header_name: Option<String>, auth_header_value: String) -> anyhow::Result<Self> {
        let user_agent = http::get_user_agent();
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(reqwest::header::ACCEPT, reqwest::header::HeaderValue::from_static("*/*"));
        let parsed_auth_header_name = auth_header_name.map(|s| HeaderName::from_str(s.as_str())
            .with_context(|| format!("Failed to parse auth header name: {s}"))
        ).transpose()?.unwrap_or(reqwest::header::AUTHORIZATION);
        headers.insert(parsed_auth_header_name, reqwest::header::HeaderValue::from_str(auth_header_value.as_str())
            .with_context(|| format!("Failed to parse auth header value: {auth_header_value}"))?);

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .user_agent(user_agent)
            .timeout(std::time::Duration::from_secs(20))
            .build()
            .context("Failed to build HTTP client")?.into();

        let mut upload_url_mut = upload_url.clone();
        if upload_url_mut.ends_with("/") {
            upload_url_mut.pop();
        }

        let mut frontend_url_mut = frontend_url.clone();
        if frontend_url_mut.ends_with("/") {
            frontend_url_mut.pop();
        }

        Ok(HttpBearerUploader {
            upload_url,
            frontend_url,
            client,
        })
    }

    pub(crate) fn from_env() -> anyhow::Result<Self> {
        let upload_url = env::var("UPLOAD_URL")
            .context("UPLOAD_URL is not set")?;

        let mut frontend_url = env::var("UPLOAD_FRONTEND_URL").unwrap_or(upload_url.clone());
        if frontend_url.ends_with("/") {
            frontend_url.pop();
        }
        let auth_header_name = env::var("UPLOAD_AUTH_HEADER_NAME").ok();
        let auth_header_value = env::var("UPLOAD_AUTH_HEADER_VALUE")
            .context("UPLOAD_AUTH_HEADER_VALUE is not set")?;

        Ok(HttpBearerUploader::new(upload_url, frontend_url, auth_header_name, auth_header_value)?)
    }
}

impl UploaderImpl for HttpBearerUploader {
    async fn upload(&self, path: &String, bytes: Vec<u8>, content_type: &String) -> anyhow::Result<String> {

        let target_url = format!("{}/{}", self.upload_url, path);

        let frontend_url = self.frontend_url(path);
        let response = self.client.get(&target_url).send().await
            .with_context(|| format!("Failed to make GET request to {frontend_url}"))?;
        if response.status() != reqwest::StatusCode::NOT_FOUND {
            anyhow::bail!("File already exists at {frontend_url}");
        }

        self.client.put(&target_url)
            .header(reqwest::header::CONTENT_TYPE, content_type)
            .body(bytes)
            .send()
            .await
            .with_context(|| format!("Failed to make PUT request to {frontend_url}"))?;

        Ok(self.frontend_url(path))
    }

    fn frontend_url(&self, path: &String) -> String {
        format!("{}/{}", self.frontend_url, path)
    }
}
