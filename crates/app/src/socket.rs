use domain::{
    event::{ServerEvent, SocketEvent, SubscribePayload, UnsubscribePayload},
    model::Message,
    notifier::MessageNotifier,
};
use serde_json::Value;
use socketioxide::{
    SocketIo,
    extract::{Data, SocketRef},
    layer::SocketIoLayer,
};
use std::{future::Future, sync::Arc};

/// Extension trait for SocketRef that provides type-safe event handler registration
trait SocketRefExt {
    fn register_handler<T, F, Fut>(&self, handler: F) -> &Self
    where
        T: SocketEvent,
        F: Fn(SocketRef, T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static;
}

impl SocketRefExt for SocketRef {
    fn register_handler<T, F, Fut>(&self, handler: F) -> &Self
    where
        T: SocketEvent,
        F: Fn(SocketRef, T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let event_name = T::event_name();
        let handler = Arc::new(handler);
        self.on(event_name, move |socket: SocketRef, Data::<Value>(data)| {
            let handler = Arc::clone(&handler);
            async move {
                match serde_json::from_value::<T>(data) {
                    Ok(payload) => handler(socket, payload).await,
                    Err(e) => {
                        tracing::error!("Failed to deserialize {}: {:?}", event_name, e);
                    }
                }
            }
        });

        self
    }
}

/// Creates and configures the Socket.io layer with necessary namespaces.
pub fn create_socket_layer() -> (SocketIoLayer, SocketIo) {
    let (socket_layer, io) = SocketIo::new_layer();

    // Register default namespace handler with subscribe/unsubscribe handlers
    io.ns("/", |socket: SocketRef| async move {
        socket
            .register_handler(handle_subscribe)
            .register_handler(handle_unsubscribe);
    });

    (socket_layer, io)
}

async fn handle_subscribe(socket: SocketRef, payload: SubscribePayload) {
    let room = format!("message:{}", payload.message_id);
    socket.join(room.clone());
    tracing::debug!("Socket {} joined room {}", socket.id, room);
}

async fn handle_unsubscribe(socket: SocketRef, payload: UnsubscribePayload) {
    let room = format!("message:{}", payload.message_id);
    socket.leave(room.clone());
    tracing::debug!("Socket {} left room {}", socket.id, room);
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
        let event_name: &'static str = (&event).into();

        if let Err(e) = self
            .io
            .to(room)
            .emit(
                event_name,
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
