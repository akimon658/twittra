use domain::model::User;

use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use http::StatusCode;
use uuid::Uuid;

use crate::{handler::AppState, session::AuthSession};

/// Get the current authenticated user's information.
#[utoipa::path(
    get,
    path = "/me",
    responses(
        (status = StatusCode::OK, body = User),
        (status = StatusCode::UNAUTHORIZED),
        (status = StatusCode::INTERNAL_SERVER_ERROR),
    ),
    security(
        ("cookieAuth" = []),
    ),
    tag = "user",
)]
#[tracing::instrument]
pub async fn get_me(auth_session: AuthSession, State(state): State<AppState>) -> impl IntoResponse {
    let user_id = match auth_session.user {
        Some(user) => user.id,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };
    let user = match state.user_service.get_user_by_id(&user_id).await {
        Ok(user) => user,
        Err(e) => {
            tracing::error!("{:?}", e);

            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    Json(user).into_response()
}

/// Get a user's information by user ID.
#[utoipa::path(
    get,
    params(
        ("userId" = Uuid, Path, description = "The ID of the user to retrieve"),
    ),
    path = "/users/{userId}",
    responses(
        (status = StatusCode::OK, body = User),
        (status = StatusCode::INTERNAL_SERVER_ERROR),
    ),
    security(
        ("cookieAuth" = []),
    ),
    tag = "user",
)]
#[tracing::instrument]
pub async fn get_user_by_id(
    State(state): State<AppState>,
    user_id: Path<Uuid>,
) -> impl IntoResponse {
    let user = match state.user_service.get_user_by_id(&user_id).await {
        Ok(user) => user,
        Err(e) => {
            tracing::error!("{:?}", e);

            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    Json(user).into_response()
}
