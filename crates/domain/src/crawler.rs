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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Message;
    use crate::repository::{MockMessageRepository, MockStampRepository, MockUserRepository};
    use crate::traq_client::MockTraqClient;
    use time::{Duration, OffsetDateTime};
    use uuid::Uuid;

    fn test_message() -> Message {
        Message {
            id: Uuid::now_v7(),
            user_id: Uuid::now_v7(),
            channel_id: Uuid::now_v7(),
            content: "test message".to_string(),
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            reactions: vec![],
        }
    }

    #[tokio::test]
    async fn crawl_success_with_existing_messages() {
        let mut mock_message_repo = MockMessageRepository::new();
        let mut mock_user_repo = MockUserRepository::new();
        let mut mock_client = MockTraqClient::new();

        let latest_message_time = OffsetDateTime::now_utc() - Duration::hours(1);
        let token = "test_token".to_string();
        let messages = vec![test_message()];
        let messages_clone = messages.clone();

        // 1. Get latest message time
        mock_message_repo
            .expect_find_latest_message_time()
            .times(1)
            .returning(move || Ok(Some(latest_message_time)));

        // 2. Get valid token
        mock_user_repo
            .expect_find_random_valid_token()
            .times(1)
            .returning(move || Ok(Some(token.clone())));

        // 3. Fetch messages from traQ
        mock_client
            .expect_fetch_messages_since()
            .with(
                mockall::predicate::eq("test_token"),
                mockall::predicate::eq(latest_message_time),
            )
            .times(1)
            .returning(move |_, _| Ok(messages_clone.clone()));

        // 4. Save messages to repository
        mock_message_repo
            .expect_save_batch()
            .times(1)
            .returning(|_| Ok(()));

        let repo = Repository {
            message: Arc::new(mock_message_repo),
            stamp: Arc::new(MockStampRepository::new()),
            user: Arc::new(mock_user_repo),
        };

        let crawler = MessageCrawler::new(Arc::new(mock_client), repo);
        let result = crawler.crawl().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn crawl_success_no_previous_messages_fallback() {
        let mut mock_message_repo = MockMessageRepository::new();
        let mut mock_user_repo = MockUserRepository::new();
        let mut mock_client = MockTraqClient::new();

        // 1. No latest message (returns None)
        mock_message_repo
            .expect_find_latest_message_time()
            .times(1)
            .returning(move || Ok(None));

        // 2. Get valid token
        mock_user_repo
            .expect_find_random_valid_token()
            .times(1)
            .returning(move || Ok(Some("test_token".to_string())));

        // 3. Fetch messages from traQ - should fallback to 1 day ago
        // We can't easily check exact time due to dynamic fallback, so just check call existence
        mock_client
            .expect_fetch_messages_since()
            .times(1)
            .returning(move |_, _| Ok(vec![]));

        // 4. Save batch (empty)
        mock_message_repo
            .expect_save_batch()
            .times(1)
            .returning(|_| Ok(()));

        let repo = Repository {
            message: Arc::new(mock_message_repo),
            stamp: Arc::new(MockStampRepository::new()),
            user: Arc::new(mock_user_repo),
        };

        let crawler = MessageCrawler::new(Arc::new(mock_client), repo);
        let result = crawler.crawl().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn crawl_skips_when_no_token() {
        let mut mock_message_repo = MockMessageRepository::new();
        let mut mock_user_repo = MockUserRepository::new();

        mock_message_repo
            .expect_find_latest_message_time()
            .returning(|| Ok(None));

        // No token
        mock_user_repo
            .expect_find_random_valid_token()
            .returning(|| Ok(None));

        let repo = Repository {
            message: Arc::new(mock_message_repo),
            stamp: Arc::new(MockStampRepository::new()),
            user: Arc::new(mock_user_repo),
        };

        let crawler = MessageCrawler::new(Arc::new(MockTraqClient::new()), repo);
        let result = crawler.crawl().await;

        // Should succeed (return Ok) but log warning and skip fetch
        assert!(result.is_ok());
    }
}
