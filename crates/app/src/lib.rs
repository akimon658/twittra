use anyhow::Result;
use axum::Router;
use tokio::net::TcpListener;
use utoipa::openapi::{Info, OpenApi, OpenApiBuilder, Server};
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use crate::handler::user;

mod handler;

pub fn create_app() -> (Router, OpenApi) {
    const API_ROOT: &str = "/api/v1";

    let openapi = OpenApiBuilder::new()
        .info(Info::new("Twittra", env!("CARGO_PKG_VERSION")))
        .servers(Some([Server::new(API_ROOT)]))
        .build();
    let (router, openapi) = OpenApiRouter::with_openapi(openapi)
        .routes(utoipa_axum::routes!(user::get_me))
        .split_for_parts();
    let router = axum::Router::new().nest(API_ROOT, router);

    (router, openapi)
}

pub async fn serve() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    let (router, openapi) = create_app();
    let router =
        router.merge(SwaggerUi::new("/docs/swagger-ui").url("/docs/openapi.json", openapi));

    axum::serve(listener, router).await?;

    Ok(())
}
