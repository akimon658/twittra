use domain::{
    event::{ClientEvent, ServerEvent, SubscribePayload, UnsubscribePayload},
    model::Message,
    notifier::MessageNotifier,
};
use serde_json::Value;
use socketioxide::{
    SocketIo,
    extract::{Data, SocketRef},
    layer::SocketIoLayer,
};
use std::error::Error;

/// Creates and configures the Socket.io layer with necessary namespaces.
pub fn create_socket_layer() -> (SocketIoLayer, SocketIo) {
    let (socket_layer, io) = SocketIo::new_layer();

    // Register default namespace handler with subscribe/unsubscribe handlers
    io.ns("/", |socket: SocketRef| async move {
        socket.on(
            ClientEvent::Subscribe(SubscribePayload {
                message_id: uuid::Uuid::nil(),
            })
            .name(),
            |socket: SocketRef, Data::<Value>(data)| async move {
                if let Err(e) = handle_subscribe(socket, data).await {
                    tracing::error!("Failed to handle subscribe: {:?}", e);
                }
            },
        );

        socket.on(
            ClientEvent::Unsubscribe(UnsubscribePayload {
                message_id: uuid::Uuid::nil(),
            })
            .name(),
            |socket: SocketRef, Data::<Value>(data)| async move {
                if let Err(e) = handle_unsubscribe(socket, data).await {
                    tracing::error!("Failed to handle unsubscribe: {:?}", e);
                }
            },
        );
    });

    (socket_layer, io)
}

async fn handle_subscribe(socket: SocketRef, data: Value) -> Result<(), Box<dyn Error>> {
    let payload: SubscribePayload = serde_json::from_value(data)?;
    let room = format!("message:{}", payload.message_id);
    socket.join(room.clone());
    tracing::debug!("Socket {} joined room {}", socket.id, room);
    Ok(())
}

async fn handle_unsubscribe(socket: SocketRef, data: Value) -> Result<(), Box<dyn Error>> {
    let payload: UnsubscribePayload = serde_json::from_value(data)?;
    let room = format!("message:{}", payload.message_id);
    socket.leave(room.clone());
    tracing::debug!("Socket {} left room {}", socket.id, room);
    Ok(())
}

/// Notifier implementation that broadcasts message updates via Socket.io to subscribed clients
pub struct SocketNotifier {
    io: SocketIo,
}

impl SocketNotifier {
    pub fn new(io: SocketIo) -> Self {
        Self { io }
    }
}

#[async_trait::async_trait]
impl MessageNotifier for SocketNotifier {
    async fn notify_message_updated(&self, message: &Message) {
        let room = format!("message:{}", message.id);
        tracing::info!(
            "Broadcasting message_updated for message {} to room {}",
            message.id,
            room
        );

        let event = ServerEvent::MessageUpdated(message.clone());

        if let Err(e) = self
            .io
            .to(room)
            .emit(
                event.name(),
                &match event {
                    ServerEvent::MessageUpdated(ref m) => m,
                },
            )
            .await
        {
            tracing::error!("Failed to broadcast message_updated: {:?}", e);
        }
    }
}
