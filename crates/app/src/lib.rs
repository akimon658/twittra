use crate::{
    handler::{
        AppState,
        auth::{self},
        message, stamp, timeline, user,
    },
    session::Backend,
};
use axum::Router;
use axum_login::AuthManagerLayerBuilder;
use domain::{
    crawler::MessageCrawler,
    model::Message,
    notifier::MessageNotifier,
    service::{TimelineServiceImpl, TraqServiceImpl},
};
use infra::{repository::mariadb, traq_client::TraqClientImpl};
use oauth2::{AuthUrl, ClientId, ClientSecret, TokenUrl, basic::BasicClient};
use serde_json::json;
use socketioxide::{SocketIo, extract::SocketRef, layer::SocketIoLayer};
use sqlx::MySqlPool;
use std::{env, error::Error, sync::Arc, time::Duration};
use tokio::{net::TcpListener, task};
use tower_sessions::{SessionManagerLayer, cookie::SameSite, session_store::ExpiredDeletion};
use tower_sessions_sqlx_store::MySqlStore;
use tracing_subscriber::fmt;
use utoipa::openapi::{
    Components, Info, OpenApi, OpenApiBuilder, Server,
    security::{ApiKey, ApiKeyValue, SecurityScheme},
};
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

mod handler;
mod session;
#[cfg(test)]
pub mod test_helpers;

const API_ROOT: &str = "/api/v1";

/// Creates and configures the Socket.io layer with necessary namespaces.
pub fn create_socket_layer() -> (SocketIoLayer, SocketIo) {
    let (socket_layer, io) = SocketIo::new_layer();

    // Register default namespace handler to prevent panic when emitting
    io.ns("/", |_: SocketRef| async move {});

    (socket_layer, io)
}

/// Notifier implementation that broadcasts message updates via Socket.io
struct SocketNotifier {
    io: SocketIo,
}

impl SocketNotifier {
    fn new(io: SocketIo) -> Self {
        Self { io }
    }
}

#[async_trait::async_trait]
impl MessageNotifier for SocketNotifier {
    async fn notify_messages_updated(&self, messages: &[Message]) {
        tracing::info!(
            "Broadcasting messages_updated for {} messages",
            messages.len()
        );

        let data = json!({ "messages": messages });
        if let Err(e) = self.io.emit("messages_updated", &data).await {
            tracing::error!("Failed to broadcast messages_updated: {:?}", e);
        }
    }
}

pub fn setup_openapi_routes() -> (Router<AppState>, OpenApi) {
    let mut components = Components::new();

    components.add_security_scheme(
        "cookieAuth",
        SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("id".to_string()))),
    );

    let openapi = OpenApiBuilder::new()
        .info(Info::new("Twittra", env!("CARGO_PKG_VERSION")))
        .servers(Some([Server::new(API_ROOT)]))
        .components(Some(components))
        .build();

    OpenApiRouter::with_openapi(openapi)
        .routes(utoipa_axum::routes!(auth::login))
        .routes(utoipa_axum::routes!(auth::oauth_callback))
        .routes(utoipa_axum::routes!(
            message::add_message_stamp,
            message::remove_message_stamp
        ))
        .routes(utoipa_axum::routes!(stamp::get_stamp_by_id))
        .routes(utoipa_axum::routes!(stamp::get_stamps))
        .routes(utoipa_axum::routes!(stamp::get_stamp_image))
        .routes(utoipa_axum::routes!(timeline::get_timeline))
        .routes(utoipa_axum::routes!(user::get_me))
        .routes(utoipa_axum::routes!(user::get_user_by_id))
        .routes(utoipa_axum::routes!(user::get_user_icon))
        .split_for_parts()
}

pub async fn serve() -> Result<(), Box<dyn Error>> {
    if cfg!(debug_assertions) {
        // Load .env file if exists
        dotenvy::from_filename(".env.local").ok();
        dotenvy::dotenv().ok();
    }

    fmt::init();

    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    let database_url = env::var("DATABASE_URL")?;
    let pool = MySqlPool::connect(&database_url).await?;
    let session_store = MySqlStore::new(pool.clone());

    session_store.migrate().await?;

    task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(Duration::from_mins(10)),
    );

    let session_layer = SessionManagerLayer::new(session_store).with_same_site(SameSite::Lax);
    let client_id = env::var("TRAQ_CLIENT_ID").map(ClientId::new)?;
    let client_secret = env::var("TRAQ_CLIENT_SECRET").map(ClientSecret::new)?;
    let traq_api_base_url = env::var("TRAQ_API_BASE_URL")?;
    let client = BasicClient::new(client_id)
        .set_client_secret(client_secret)
        .set_auth_uri(AuthUrl::new(format!(
            "{}/oauth2/authorize",
            traq_api_base_url
        ))?)
        .set_token_uri(TokenUrl::new(format!(
            "{}/oauth2/token",
            traq_api_base_url
        ))?);
    let repository = mariadb::new_repository(pool).await?;
    let traq_client = TraqClientImpl::new(traq_api_base_url.clone());

    let (socket_layer, io) = create_socket_layer();
    let notifier = Arc::new(SocketNotifier::new(io));
    let crawler = MessageCrawler::new(Arc::new(traq_client.clone()), repository.clone(), notifier);

    task::spawn(async move {
        crawler.run().await;
    });

    let backend = Backend::new(client, traq_api_base_url, repository.user.clone());
    let traq_service = TraqServiceImpl::new(repository.clone(), Arc::new(traq_client));
    let timeline_service = TimelineServiceImpl::new(repository);
    let app_state = AppState::new(Arc::new(traq_service), Arc::new(timeline_service));
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();
    let (router, openapi) = setup_openapi_routes();
    let router = axum::Router::new()
        .nest(API_ROOT, router.layer(auth_layer))
        .merge(SwaggerUi::new("/docs/swagger-ui").url("/docs/openapi.json", openapi))
        .layer(socket_layer);

    axum::serve(listener, router.with_state(app_state)).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_socket_layer_configuration_prevents_panic() {
        let (_, io) = create_socket_layer();

        // Verify that emitting to default namespace works
        let result = io.emit("test", &"test").await;
        assert!(result.is_ok());
    }
}
