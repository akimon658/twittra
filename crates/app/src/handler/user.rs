use domain::model::User;

use axum::{Json, extract::State, response::IntoResponse};
use http::StatusCode;

use crate::handler::{AppState, auth::AuthSession};

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
pub async fn get_me(auth_session: AuthSession, State(state): State<AppState>) -> impl IntoResponse {
    let user_id = match auth_session.user {
        Some(user) => user.id,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };
    let user = match state.repo.user.find_by_id(&user_id).await {
        Ok(user) => user,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    Json(user).into_response()
}
