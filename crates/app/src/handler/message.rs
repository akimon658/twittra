use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use http::StatusCode;
use uuid::Uuid;

use crate::{handler::AppState, session::AuthSession};

#[utoipa::path(
    post,
    params(
        ("messageId" = Uuid, Path, description = "The ID of the message to react to"),
        ("stampId" = Uuid, Path, description = "The ID of the stamp to add"),
    ),
    path = "/messages/{messageId}/stamps/{stampId}",
    responses(
        (status = StatusCode::NO_CONTENT),
        (status = StatusCode::UNAUTHORIZED),
        (status = StatusCode::INTERNAL_SERVER_ERROR),
    ),
    security(
        ("cookieAuth" = []),
    ),
    tag = "message",
)]
#[tracing::instrument(skip(auth_session, state))]
pub async fn add_message_stamp(
    auth_session: AuthSession,
    State(state): State<AppState>,
    Path((message_id, stamp_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let user = match auth_session.user {
        Some(user) => user,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };

    if let Err(e) = state
        .traq_service
        .add_message_stamp(&user.id, &message_id, &stamp_id, 1)
        .await
    {
        tracing::error!("{:?}", e);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    StatusCode::NO_CONTENT.into_response()
}

#[utoipa::path(
    delete,
    params(
        ("messageId" = Uuid, Path, description = "The ID of the message to remove reaction from"),
        ("stampId" = Uuid, Path, description = "The ID of the stamp to remove"),
    ),
    path = "/messages/{messageId}/stamps/{stampId}",
    responses(
        (status = StatusCode::NO_CONTENT),
        (status = StatusCode::UNAUTHORIZED),
        (status = StatusCode::INTERNAL_SERVER_ERROR),
    ),
    security(
        ("cookieAuth" = []),
    ),
    tag = "message",
)]
#[tracing::instrument(skip(auth_session, state))]
pub async fn remove_message_stamp(
    auth_session: AuthSession,
    State(state): State<AppState>,
    Path((message_id, stamp_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let user = match auth_session.user {
        Some(user) => user,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };

    if let Err(e) = state
        .traq_service
        .remove_message_stamp(&user.id, &message_id, &stamp_id)
        .await
    {
        tracing::error!("{:?}", e);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    StatusCode::NO_CONTENT.into_response()
}
