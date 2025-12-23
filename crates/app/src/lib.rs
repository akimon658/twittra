use anyhow::Result;
use axum::{Router, routing};
use tokio::net::TcpListener;

use crate::handler::user;

mod handler;

pub async fn serve() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    let router = Router::new().route("/api/v1/me", routing::get(user::get_me));

    axum::serve(listener, router).await?;

    Ok(())
}
