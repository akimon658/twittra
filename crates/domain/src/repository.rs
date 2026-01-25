use std::{fmt::Debug, sync::Arc};

use crate::error::RepositoryError;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::model::{Message, MessageListItem, Stamp, User};

#[derive(Clone, Debug)]
pub struct Repository {
    pub message: Arc<dyn MessageRepository>,
    pub stamp: Arc<dyn StampRepository>,
    pub user: Arc<dyn UserRepository>,
}

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait MessageRepository: Debug + Send + Sync {
    async fn find_latest_message_time(&self) -> Result<Option<OffsetDateTime>, RepositoryError>;
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Message>, RepositoryError>;

    /// Returns messages that may need refreshing from traQ.
    /// Returns tuples of (message_id, created_at, last_crawled_at) for messages created within the last 24 hours.
    async fn find_sync_candidates(
        &self,
    ) -> Result<Vec<(Uuid, OffsetDateTime, OffsetDateTime)>, RepositoryError>;
    /// Removes a reaction from a message.
    /// This is used for optimistic updates when deleting a stamp.
    async fn remove_reaction(
        &self,
        message_id: &Uuid,
        stamp_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<(), RepositoryError>;
    /// Saves a message to the repository.
    async fn save(&self, message: &Message) -> Result<(), RepositoryError>;
    /// Saves a batch of messages to the repository.
    /// It does nothing if `messages` is empty.
    async fn save_batch(&self, messages: &[Message]) -> Result<(), RepositoryError>;
    /// Marks messages as read by a user.
    async fn mark_messages_as_read(
        &self,
        user_id: &Uuid,
        message_ids: &[Uuid],
    ) -> Result<(), RepositoryError>;

    /// Finds top reacted messages (popularity-based).
    async fn find_top_reacted_messages(
        &self,
        user_id: &Uuid,
        limit: i64,
    ) -> Result<Vec<MessageListItem>, RepositoryError>;

    /// Finds messages from specific authors (user affinity).
    async fn find_messages_by_author_allowlist(
        &self,
        author_ids: &[Uuid],
        limit: i64,
        user_id: &Uuid,
    ) -> Result<Vec<MessageListItem>, RepositoryError>;

    /// Finds messages from specific channels (channel affinity).
    async fn find_messages_by_channel_allowlist(
        &self,
        channel_ids: &[Uuid],
        limit: i64,
        user_id: &Uuid,
    ) -> Result<Vec<MessageListItem>, RepositoryError>;
}

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait StampRepository: Debug + Send + Sync {
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Stamp>, RepositoryError>;
    async fn save(&self, stamp: &Stamp) -> Result<(), RepositoryError>;
    async fn save_batch(&self, stamps: &[Stamp]) -> Result<(), RepositoryError>;
    /// Finds channels that the user frequently stamps in.
    async fn find_frequently_stamped_channels_by(
        &self,
        user_id: &Uuid,
        limit: i64,
    ) -> Result<Vec<Uuid>, RepositoryError>;
}

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait UserRepository: Debug + Send + Sync {
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<User>, RepositoryError>;
    async fn find_random_valid_token(&self) -> Result<Option<String>, RepositoryError>;
    async fn find_token_by_user_id(
        &self,
        user_id: &Uuid,
    ) -> Result<Option<String>, RepositoryError>;
    async fn save(&self, user: &User) -> Result<(), RepositoryError>;
    async fn save_token(&self, user_id: &Uuid, access_token: &str) -> Result<(), RepositoryError>;
    /// Finds users who the target user frequently stamps to.
    async fn find_frequently_stamped_users_by(
        &self,
        user_id: &Uuid,
        limit: i64,
    ) -> Result<Vec<Uuid>, RepositoryError>;
    /// Finds users who have similar reaction patterns to the target user.
    async fn find_similar_users(
        &self,
        user_id: &Uuid,
        limit: i64,
    ) -> Result<Vec<Uuid>, RepositoryError>;
}
