use domain::model::User;

use axum::{Json, response::IntoResponse};
use http::StatusCode;

use crate::handler::auth::AuthSession;

/// Get the current authenticated user's information.
#[utoipa::path(
    get,
    path = "/me",
    responses(
        (status = StatusCode::OK, body = User),
        (status = StatusCode::UNAUTHORIZED),
    ),
    tag = "user"
)]
pub async fn get_me(auth_session: AuthSession) -> impl IntoResponse {
    let user_handle = match auth_session.user {
        Some(user) => user.handle,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };

    Json(User {
        handle: user_handle,
    })
    .into_response()
}
