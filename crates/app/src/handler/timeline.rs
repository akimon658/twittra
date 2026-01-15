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
    use crate::handler::AppState;
    use crate::mocks::{MockMessageRepository, MockStampRepository, MockTraqClient, MockUserRepository};
    use crate::session::{AuthSession, UserSession};
    use crate::test_helpers::create_test_backend;
    use axum::{Router, body::Body, http::Request};
    use domain::{
        model::User,
        repository::Repository,
    };
    use std::sync::Arc;
    use tower::ServiceExt;
    use uuid::Uuid;

    fn create_app(
        mock_message_repo: MockMessageRepository,
        user: Option<User>,
    ) -> Router {
        let mock_user_repo = Arc::new(MockUserRepository::new());
        let repo = Repository {
            message: Arc::new(mock_message_repo),
            stamp: Arc::new(MockStampRepository::new()),
            user: mock_user_repo.clone(),
        };
        let traq_client = Arc::new(MockTraqClient::new());
        let state = AppState::new(repo, traq_client);

        let backend = create_test_backend(mock_user_repo);

        let session_layer = tower_sessions::SessionManagerLayer::new(tower_sessions::MemoryStore::default());
        let auth_layer = axum_login::AuthManagerLayerBuilder::new(backend, session_layer).build();

        Router::new()
            .route("/timeline", axum::routing::get(get_timeline))
            // Test-only login route to establish session
            .route("/login", axum::routing::post(|mut auth: AuthSession| async move {
                 if let Some(user_session) = user.map(|u| UserSession { id: u.id }) {
                     auth.login(&user_session).await.unwrap();
                     StatusCode::OK
                 } else {
                     StatusCode::UNAUTHORIZED
                 }
            }))
            .layer(auth_layer)
            .with_state(state)
    }

    #[tokio::test]
    async fn test_get_timeline_success() {
        let mut mock_message_repo = MockMessageRepository::new();
        mock_message_repo
            .expect_find_recent_messages()
            .times(1)
            .returning(|| Ok(vec![]));

        let user = crate::test_factories::create_user();

        let app = create_app(mock_message_repo, Some(user.clone()));

        // 1. Login to get session cookie
        let login_req = Request::builder()
            .uri("/login")
            .method("POST")
            .body(Body::empty())
            .unwrap();
        
        let login_res = app.clone().oneshot(login_req).await.unwrap();
        assert_eq!(login_res.status(), StatusCode::OK);
        
        let cookie = login_res.headers().get(http::header::SET_COOKIE).unwrap().clone();

        // 2. Access timeline with cookie
        let req = Request::builder()
            .uri("/timeline")
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

        let app = create_app(mock_message_repo, None);

        let req = Request::builder()
            .uri("/timeline")
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }
}
