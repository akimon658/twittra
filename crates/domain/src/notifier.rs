use crate::model::Message;
use async_trait::async_trait;

/// Trait for notifying external systems about message updates.
///
/// This abstraction allows the domain layer to remain decoupled from specific
/// notification mechanisms (e.g., WebSocket, Server-Sent Events, etc.).
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait MessageNotifier: Send + Sync {
    /// Notifies that messages have been updated.
    ///
    /// # Arguments
    ///
    /// * `messages` - The messages that were updated
    async fn notify_messages_updated(&self, messages: &[Message]);
}
