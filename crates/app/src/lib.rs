use std::{env, sync::Arc, time::Duration};

use anyhow::Result;
use axum::Router;
use axum_login::AuthManagerLayerBuilder;
use domain::crawler::MessageCrawler;
use infra::{repository::mariadb, traq_client::TraqClientImpl};
use oauth2::{AuthUrl, ClientId, ClientSecret, TokenUrl, basic::BasicClient};
use sqlx::MySqlPool;
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

use crate::{
    handler::{
        AppState,
        auth::{self},
        stamp, timeline, user,
    },
    session::Backend,
};

mod handler;
mod session;

const API_ROOT: &str = "/api/v1";

pub fn setup_openapi_routes() -> Result<(Router<AppState>, OpenApi)> {
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
    let openapi_router = OpenApiRouter::with_openapi(openapi)
        .routes(utoipa_axum::routes!(auth::login))
        .routes(utoipa_axum::routes!(auth::oauth_callback))
        .routes(utoipa_axum::routes!(stamp::get_stamp_by_id))
        .routes(utoipa_axum::routes!(stamp::get_stamp_image))
        .routes(utoipa_axum::routes!(timeline::get_timeline))
        .routes(utoipa_axum::routes!(user::get_me))
        .routes(utoipa_axum::routes!(user::get_user_by_id))
        .routes(utoipa_axum::routes!(user::get_user_icon))
        .split_for_parts();

    Ok(openapi_router)
}

pub async fn serve() -> Result<()> {
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
    let crawler = MessageCrawler::new(Arc::new(traq_client.clone()), repository.clone());

    task::spawn(async move {
        crawler.run().await;
    });

    let backend = Backend::new(client, traq_api_base_url, repository.user.clone());
    let app_state = AppState::new(repository, Arc::new(traq_client));
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();
    let (router, openapi) = setup_openapi_routes()?;
    let router = axum::Router::new()
        .nest(API_ROOT, router.layer(auth_layer))
        .merge(SwaggerUi::new("/docs/swagger-ui").url("/docs/openapi.json", openapi));

    axum::serve(listener, router.with_state(app_state)).await?;

    Ok(())
}
