use crate::{handler::AppState, session::AuthSession};
use axum::{Json, extract::State, response::IntoResponse};
use domain::model::MessageListItem;
use http::StatusCode;

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
#[tracing::instrument(skip_all)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::TestAppBuilder;
    use axum::{body::Body, http::Request};
    use domain::service::MockTimelineService;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_get_timeline_success() {
        let mut mock_timeline_service = MockTimelineService::new();

        let message = crate::test_factories::create_message_list_item();
        let messages = vec![message.clone()];
        let messages_clone = messages.clone();

        mock_timeline_service
            .expect_get_recommended_messages()
            .times(1)
            .returning(move || Ok(messages_clone.clone()));

        let user = crate::test_factories::create_user();

        let app = TestAppBuilder::new()
            .with_timeline_service(mock_timeline_service)
            .with_user(user.clone())
            .build();

        // 1. Login to get session cookie
        let login_req = Request::builder()
            .uri("/login")
            .method("POST")
            .body(Body::empty())
            .unwrap();

        let login_res = app.clone().oneshot(login_req).await.unwrap();
        assert_eq!(login_res.status(), StatusCode::OK);

        let cookie = login_res
            .headers()
            .get(http::header::SET_COOKIE)
            .unwrap()
            .clone();

        // 2. Access timeline with cookie
        let req = Request::builder()
            .uri("/api/v1/timeline")
            .header(http::header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        // Validate response body
        let body = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        let response_messages: Vec<MessageListItem> = serde_json::from_slice(&body).unwrap();
        assert_eq!(response_messages.len(), 1);
        assert_eq!(response_messages[0].id, message.id);
        assert_eq!(response_messages[0].content, message.content);
        assert_eq!(response_messages[0].user_id, message.user_id);
    }

    #[tokio::test]
    async fn test_get_timeline_unauthorized() {
        let app = TestAppBuilder::new().build();
        let req = Request::builder()
            .uri("/api/v1/timeline")
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }
}
