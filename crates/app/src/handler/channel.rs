use crate::{handler::AppState, session::AuthSession};
use axum::{
    Json,
    extract::{Path, Query, State},
    response::IntoResponse,
};
use domain::model::MessageListItem;
use http::StatusCode;
use serde::Deserialize;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetChannelMessagesQuery {
    #[serde(default, with = "time::serde::rfc3339::option")]
    since: Option<OffsetDateTime>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    until: Option<OffsetDateTime>,
    order: Option<String>,
}

/// Get messages from a specific channel.
#[utoipa::path(
    get,
    path = "/channels/{channelId}/messages",
    params(
        ("channelId" = Uuid, Path, description = "Channel ID"),
        ("since" = Option<OffsetDateTime>, Query, description = "Fetch messages created after this timestamp (RFC3339)"),
        ("until" = Option<OffsetDateTime>, Query, description = "Fetch messages created before this timestamp (RFC3339)"),
        ("order" = Option<String>, Query, description = "Sort order (asc/desc)"),
    ),
    responses(
        (status = StatusCode::OK, body = [MessageListItem]),
        (status = StatusCode::UNAUTHORIZED),
        (status = StatusCode::INTERNAL_SERVER_ERROR),
    ),
    security(
        ("cookieAuth" = []),
    ),
    tag = "channel",
)]
#[tracing::instrument(skip_all)]
pub async fn get_channel_messages(
    auth_session: AuthSession,
    State(state): State<AppState>,
    Path(channel_id): Path<Uuid>,
    Query(query): Query<GetChannelMessagesQuery>,
) -> impl IntoResponse {
    let user = match auth_session.user {
        Some(user) => user,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };

    let messages = match state
        .traq_service
        .get_channel_messages(
            &user.id,
            &channel_id,
            Some(50),
            query.since,
            query.until,
            query.order,
        )
        .await
    {
        Ok(messages) => messages,
        Err(e) => {
            tracing::error!("{:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    Json(messages).into_response()
}
