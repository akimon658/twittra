use crate::model::Message;
use async_trait::async_trait;

/// Trait for notifying external systems about message updates.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait MessageNotifier: Send + Sync {
    /// Notifies that a message has been updated.
    async fn notify_message_updated(&self, message: &Message);
}
