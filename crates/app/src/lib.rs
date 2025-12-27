use std::env;

use anyhow::Result;
use axum::Router;
use axum_login::AuthManagerLayerBuilder;
use infra::repository::mysql;
use oauth2::{AuthUrl, ClientId, ClientSecret, TokenUrl, basic::BasicClient};
use tokio::net::TcpListener;
use tower_sessions::{MemoryStore, SessionManagerLayer, cookie::SameSite};
use utoipa::openapi::{Info, OpenApi, OpenApiBuilder, Server};
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use crate::handler::{
    AppState,
    auth::{self, Backend},
    user,
};

mod handler;

const API_ROOT: &str = "/api/v1";

pub fn setup_openapi_routes() -> Result<(Router<AppState>, OpenApi)> {
    let openapi = OpenApiBuilder::new()
        .info(Info::new("Twittra", env!("CARGO_PKG_VERSION")))
        .servers(Some([Server::new(API_ROOT)]))
        .build();
    let openapi_router = OpenApiRouter::with_openapi(openapi)
        .routes(utoipa_axum::routes!(user::get_me))
        .split_for_parts();

    Ok(openapi_router)
}

pub async fn serve() -> Result<()> {
    if cfg!(debug_assertions) {
        // Load .env file if exists
        dotenvy::from_filename(".env.local").ok();
        dotenvy::dotenv().ok();
    }

    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    let session_store = MemoryStore::default();
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
    let database_url = env::var("DATABASE_URL")?;
    let repository = mysql::new_repository(&database_url).await?;
    let backend = Backend::new(client, repository.user.clone());
    let app_state = AppState { repo: repository };
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();
    let (router, openapi) = setup_openapi_routes()?;
    let router = axum::Router::new()
        .nest(API_ROOT, router.merge(auth::router()).layer(auth_layer))
        .merge(SwaggerUi::new("/docs/swagger-ui").url("/docs/openapi.json", openapi));

    axum::serve(listener, router.with_state(app_state)).await?;

    Ok(())
}
