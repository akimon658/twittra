use crate::model::Message;
use serde::{Deserialize, Serialize};
use strum::{EnumDiscriminants, IntoStaticStr};
use utoipa::ToSchema;
use uuid::Uuid;

/// Trait for Socket.io events that provides type-safe event name access
pub trait SocketEvent: for<'de> Deserialize<'de> + Send + 'static {
    fn event_name() -> &'static str;
}

/// Client-to-server events for Socket.io
#[derive(Deserialize, ToSchema, EnumDiscriminants)]
#[serde(tag = "type", content = "payload")]
#[serde(rename_all = "camelCase")]
#[strum_discriminants(derive(IntoStaticStr))]
#[strum_discriminants(strum(serialize_all = "camelCase"))]
pub enum ClientEvent {
    Subscribe(SubscribePayload),
    Unsubscribe(UnsubscribePayload),
}

/// Payload for the subscribe event
#[derive(Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SubscribePayload {
    pub message_id: Uuid,
}

impl SocketEvent for SubscribePayload {
    fn event_name() -> &'static str {
        ClientEventDiscriminants::Subscribe.into()
    }
}

/// Payload for the unsubscribe event
#[derive(Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UnsubscribePayload {
    pub message_id: Uuid,
}

impl SocketEvent for UnsubscribePayload {
    fn event_name() -> &'static str {
        ClientEventDiscriminants::Unsubscribe.into()
    }
}

/// Server-to-client events for Socket.io
#[derive(Serialize, ToSchema, IntoStaticStr)]
#[serde(tag = "type", content = "payload")]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum ServerEvent {
    MessageUpdated(Message),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscribe_payload_event_name() {
        assert_eq!(SubscribePayload::event_name(), "subscribe");
    }

    #[test]
    fn test_unsubscribe_payload_event_name() {
        assert_eq!(UnsubscribePayload::event_name(), "unsubscribe");
    }

    #[test]
    fn test_server_event_message_updated_name() {
        let event = ServerEvent::MessageUpdated(Message {
            id: Uuid::nil(),
            user_id: Uuid::nil(),
            channel_id: Uuid::nil(),
            content: String::new(),
            created_at: time::OffsetDateTime::UNIX_EPOCH,
            updated_at: time::OffsetDateTime::UNIX_EPOCH,
            reactions: vec![],
        });
        let event_name: &'static str = (&event).into();
        assert_eq!(event_name, "messageUpdated");
    }
}
