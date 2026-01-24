use crate::{handler::AppState, session::AuthSession};
use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

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

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ReadMessagesRequest {
    pub message_ids: Vec<Uuid>,
}

#[utoipa::path(
    post,
    path = "/messages/read",
    request_body = ReadMessagesRequest,
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
#[tracing::instrument(skip(auth_session, state, payload))]
pub async fn mark_messages_as_read(
    auth_session: AuthSession,
    State(state): State<AppState>,
    Json(payload): Json<ReadMessagesRequest>,
) -> impl IntoResponse {
    let user = match auth_session.user {
        Some(user) => user,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };

    if let Err(e) = state
        .timeline_service
        .mark_messages_as_read(&user.id, &payload.message_ids)
        .await
    {
        tracing::error!("{:?}", e);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    StatusCode::NO_CONTENT.into_response()
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::TestAppBuilder;
    use axum::{body::Body, http::Request};
    use domain::{
        service::{MockTimelineService, MockTraqService},
        test_factories::UserBuilder,
    };
    use fake::{Fake, uuid::UUIDv4};
    use http::header;
    use mockall::predicate;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_mark_messages_as_read_success() {
        let mut mock_timeline_service = MockTimelineService::new();
        let user = UserBuilder::new().build();
        let user_id = user.id;
        let message_ids = vec![UUIDv4.fake(), UUIDv4.fake()];
        let message_ids_clone = message_ids.clone();

        mock_timeline_service
            .expect_mark_messages_as_read()
            .with(predicate::eq(user_id), predicate::eq(message_ids_clone))
            .times(1)
            .returning(|_, _| Ok(()));

        let app = TestAppBuilder::new()
            .with_timeline_service(mock_timeline_service)
            .with_user(user.clone())
            .build();

        let login_req = Request::builder()
            .uri("/login")
            .method("POST")
            .body(Body::empty())
            .unwrap();
        let login_res = app.clone().oneshot(login_req).await.unwrap();
        let cookie = login_res.headers().get(header::SET_COOKIE).unwrap().clone();

        let req = Request::builder()
            .uri("/api/v1/messages/read")
            .method("POST")
            .header(header::COOKIE, cookie)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                serde_json::to_string(&ReadMessagesRequest { message_ids }).unwrap(),
            ))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }
    #[tokio::test]
    async fn test_add_message_stamp_success() {
        let mut mock_traq_service = MockTraqService::new();

        let user = UserBuilder::new().build();
        let user_id = user.id;
        let message_id: Uuid = UUIDv4.fake();
        let stamp_id: Uuid = UUIDv4.fake();

        mock_traq_service
            .expect_add_message_stamp()
            .with(
                predicate::eq(user_id),
                predicate::eq(message_id),
                predicate::eq(stamp_id),
                predicate::eq(1),
            )
            .times(1)
            .returning(|_, _, _, _| Ok(()));

        let app = TestAppBuilder::new()
            .with_traq_service(mock_traq_service)
            .with_user(user.clone())
            .build();

        // Login
        let login_req = Request::builder()
            .uri("/login")
            .method("POST")
            .body(Body::empty())
            .unwrap();
        let login_res = app.clone().oneshot(login_req).await.unwrap();
        let cookie = login_res.headers().get(header::SET_COOKIE).unwrap().clone();

        // Add Stamp
        let req = Request::builder()
            .uri(format!(
                "/api/v1/messages/{}/stamps/{}",
                message_id, stamp_id
            ))
            .method("POST")
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::NO_CONTENT);
    }
}
