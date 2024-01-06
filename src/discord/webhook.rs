use anyhow::Context;
use rusty_interaction::types::interaction::WebhookMessage;

#[derive(Debug, Clone)]
pub(crate) struct Webhook {
    pub(crate) url: String,
    client: reqwest::Client,
}

impl Webhook {
    pub(crate) fn new(url: String) -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent(crate::http::get_user_agent())
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Webhook {
            url,
            client,
        })
    }

    pub(crate) async fn send(&self, message: &WebhookMessage) -> anyhow::Result<()> {
        match self.client.post(&self.url).json(&message).send().await {
            Ok(response) => {
                if !response.status().is_success() {
                    anyhow::bail!("Failed to send webhook - {}: {:?}", response.status(), response.text().await?);
                }
            }
            Err(e) => {
                anyhow::bail!("Failed to send webhook: {e}");
            }
        }

        Ok(())
    }
}

