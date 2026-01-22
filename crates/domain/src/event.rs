use crate::model::Message;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Client-to-server events for Socket.io
#[derive(Deserialize, ToSchema)]
#[serde(tag = "type", content = "payload")]
#[serde(rename_all = "camelCase")]
pub enum ClientEvent {
    Subscribe(SubscribePayload),
    Unsubscribe(UnsubscribePayload),
}

impl ClientEvent {
    pub fn name(&self) -> &'static str {
        match self {
            ClientEvent::Subscribe(_) => "subscribe",
            ClientEvent::Unsubscribe(_) => "unsubscribe",
        }
    }
}

/// Payload for the subscribe event
#[derive(Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SubscribePayload {
    pub message_id: Uuid,
}

/// Payload for the unsubscribe event
#[derive(Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UnsubscribePayload {
    pub message_id: Uuid,
}

/// Server-to-client events for Socket.io
#[derive(Serialize, ToSchema)]
#[serde(tag = "type", content = "payload")]
#[serde(rename_all = "camelCase")]
pub enum ServerEvent {
    MessageUpdated(Message),
}

impl ServerEvent {
    pub fn name(&self) -> &'static str {
        match self {
            ServerEvent::MessageUpdated(_) => "messageUpdated",
        }
    }
}
