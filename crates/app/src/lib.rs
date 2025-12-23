use anyhow::Result;
use tokio::net::TcpListener;
use utoipa::openapi::{Info, OpenApiBuilder, Server};
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use crate::handler::user;

mod handler;

pub async fn serve() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    let openapi = OpenApiBuilder::new()
        .info(Info::new("Twittra", env!("CARGO_PKG_VERSION")))
        .servers(Some([Server::new("/api/v1")]))
        .build();
    let (router, openapi_with_routes) = OpenApiRouter::with_openapi(openapi)
        .routes(utoipa_axum::routes!(user::get_me))
        .split_for_parts();

    axum::serve(
        listener,
        router.merge(
            SwaggerUi::new("/docs/swagger-ui").url("/docs/openapi.json", openapi_with_routes),
        ),
    )
    .await?;

    Ok(())
}
