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

#[tracing::instrument(skip(socket, payload), fields(socket_id = %socket.id, message_id = %payload.message_id))]
async fn handle_subscribe(socket: SocketRef, payload: SubscribePayload) {
    socket.join(format!("message:{}", payload.message_id));
    tracing::info!("Client subscribed to message updates");
}

#[tracing::instrument(skip(socket, payload), fields(socket_id = %socket.id, message_id = %payload.message_id))]
async fn handle_unsubscribe(socket: SocketRef, payload: UnsubscribePayload) {
    socket.leave(format!("message:{}", payload.message_id));
    tracing::info!("Client unsubscribed from message updates");
}

/// Notifier implementation that broadcasts message updates via Socket.io to subscribed clients
#[derive(Debug)]
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
    #[tracing::instrument(skip(self, message), fields(message_id = %message.id))]
    async fn notify_message_updated(&self, message: &Message) {
        let room = format!("message:{}", message.id);
        tracing::info!("Broadcasting messageUpdated");

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
            tracing::error!("Failed to broadcast messageUpdated: {:?}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use domain::{event::SubscribePayload, model::Message};
    use futures_util::FutureExt;
    use rust_socketio::{
        Payload,
        asynchronous::{Client, ClientBuilder},
    };
    use std::sync::{Arc, Mutex};
    use tokio::net::TcpListener;

    /// Spawns a test server with Socket.IO layer and returns the server address and notifier
    async fn start_test_server() -> (String, Arc<SocketNotifier>) {
        let (socket_layer, io) = create_socket_layer();
        let notifier = Arc::new(SocketNotifier::new(io));

        let app = Router::new().layer(socket_layer);
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        // Give the server a moment to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        (format!("http://{}", addr), notifier)
    }

    #[tokio::test]
    async fn test_socket_message_update() {
        let (server_addr, notifier) = start_test_server().await;

        // Track received events
        let received_events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = Arc::clone(&received_events);

        // Connect Socket.IO client
        let client = ClientBuilder::new(server_addr)
            .namespace("/")
            .on(
                "messageUpdated",
                move |payload: Payload, _client: Client| {
                    let events = Arc::clone(&events_clone);
                    async move {
                        if let Payload::Text(values) = payload {
                            if let Some(value) = values.first() {
                                events.lock().unwrap().push(value.clone());
                            }
                        }
                    }
                    .boxed()
                },
            )
            .connect()
            .await
            .expect("Failed to connect to Socket.IO server");

        // Wait for connection to establish
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        let message = domain::test_factories::MessageBuilder::new().build();

        // Subscribe to message updates
        let subscribe_payload = SubscribePayload {
            message_id: message.id,
        };
        client
            .emit(
                "subscribe",
                serde_json::to_value(&subscribe_payload).unwrap(),
            )
            .await
            .expect("Failed to emit subscribe event");

        // Wait for subscription to be processed
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        // Trigger notification
        notifier.notify_message_updated(&message).await;

        // Wait for event to be received
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

        // Verify the event was received
        let events = received_events.lock().unwrap();
        assert_eq!(events.len(), 1, "Should receive exactly one event");

        // Verify the payload matches the sent message
        let received_message: Message =
            serde_json::from_value(events[0].clone()).expect("Failed to deserialize message");
        assert_eq!(received_message.id, message.id);
        assert_eq!(received_message.content, message.content);

        // Disconnect client
        client.disconnect().await.expect("Failed to disconnect");
    }
}
