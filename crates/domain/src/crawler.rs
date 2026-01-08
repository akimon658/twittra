use std::sync::Arc;

use anyhow::Result;
use time::{Duration, OffsetDateTime};

use crate::{repository::Repository, traq_client::TraqClient};

/// Fetches new messages from traQ every 30 seconds and saves them to the repository.
pub struct MessageCrawler {
    client: Arc<dyn TraqClient>,
    repo: Repository,
}

impl MessageCrawler {
    pub fn new(client: Arc<dyn TraqClient>, repo: Repository) -> Self {
        Self { client, repo }
    }

    pub async fn run(&self) {
        loop {
            if let Err(e) = self.crawl().await {
                tracing::error!("Crawl failed: {:?}", e);
            }

            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        }
    }

    pub async fn crawl(&self) -> Result<()> {
        let last_fetched_at = self
            .repo
            .message
            .find_latest_message_time()
            .await?
            .unwrap_or_else(|| OffsetDateTime::now_utc() - Duration::days(1));
        let token = match self.repo.user.find_random_valid_token().await? {
            Some(t) => t,
            None => {
                tracing::warn!("No valid token found. Skipping crawl.");

                return Ok(());
            }
        };
        let messages = self
            .client
            .fetch_messages_since(&token, last_fetched_at)
            .await?;

        self.repo.message.save_batch(&messages).await?;

        Ok(())
    }
}
