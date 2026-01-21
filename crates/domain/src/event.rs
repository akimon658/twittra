use crate::model::MessageListItem;
use serde::Serialize;
use utoipa::ToSchema;

/// Discriminated union of all Socket.io server-to-client events
#[derive(Serialize, ToSchema)]
#[serde(tag = "type", content = "payload")]
#[serde(rename_all = "camelCase")]
pub enum ServerEvent {
    MessagesUpdated(MessagesUpdatedPayload),
}

impl ServerEvent {
    pub fn name(&self) -> &'static str {
        match self {
            ServerEvent::MessagesUpdated(_) => "messagesUpdated",
        }
    }
}

/// Payload for the messagesUpdated event
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MessagesUpdatedPayload {
    pub messages: Vec<MessageListItem>,
}
