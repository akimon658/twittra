use axum::{Json, extract::State, response::IntoResponse};
use domain::model::MessageListItem;
use http::StatusCode;

use crate::{handler::AppState, session::AuthSession};

/// Get messages for the timeline.
#[utoipa::path(
    get,
    path = "/timeline",
    responses(
        (status = StatusCode::OK, body = [MessageListItem]),
        (status = StatusCode::UNAUTHORIZED),
        (status = StatusCode::INTERNAL_SERVER_ERROR),
    ),
    security(
        ("cookieAuth" = []),
    ),
    tag = "timeline",
)]
#[tracing::instrument]
pub async fn get_timeline(
    auth_session: AuthSession,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if auth_session.user.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let messages = match state.timeline_service.get_recommended_messages().await {
        Ok(messages) => messages,
        Err(e) => {
            tracing::error!("{:?}", e);

            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    Json(messages).into_response()
}
