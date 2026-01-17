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
    use crate::mocks::MockMessageRepository;
    use crate::test_helpers::TestAppBuilder;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_get_timeline_success() {
        let mut mock_message_repo = MockMessageRepository::new();
        mock_message_repo
            .expect_find_recent_messages()
            .times(1)
            .returning(|| Ok(vec![]));

        let user = crate::test_factories::create_user();

        let app = TestAppBuilder::new()
            .with_message_repo(mock_message_repo)
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
    }

    #[tokio::test]
    async fn test_get_timeline_unauthorized() {
        let mock_message_repo = MockMessageRepository::new();
        // No user => logic shouldn't even check repo if unauthorized
        // checking repo times(0) is default

        let app = TestAppBuilder::new()
            .with_message_repo(mock_message_repo)
            .build();

        let req = Request::builder()
            .uri("/api/v1/timeline")
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }
}
