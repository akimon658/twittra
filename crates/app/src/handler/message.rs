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

    fn create_app(
        mock_message_repo: MockMessageRepository,
        mock_user_repo: MockUserRepository,
        mock_traq_client: MockTraqClient,
        user: Option<User>,
    ) -> Router {
        let mock_user_repo_arc = Arc::new(mock_user_repo);
        let repo = Repository {
            message: Arc::new(mock_message_repo),
            stamp: Arc::new(MockStampRepository::new()),
            user: mock_user_repo_arc.clone(),
        };
        let traq_client = Arc::new(mock_traq_client);
        let state = AppState::new(repo, traq_client);

        let backend = create_test_backend(mock_user_repo_arc);

        let session_layer = tower_sessions::SessionManagerLayer::new(tower_sessions::MemoryStore::default());
        let auth_layer = axum_login::AuthManagerLayerBuilder::new(backend, session_layer).build();

        Router::new()
            .route("/messages/{messageId}/stamps/{stampId}", 
                axum::routing::post(add_message_stamp).delete(remove_message_stamp))
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
    async fn test_add_message_stamp_success() {
        let mut mock_user_repo = MockUserRepository::new();
        let mut mock_traq_client = MockTraqClient::new();
        let mut mock_message_repo = MockMessageRepository::new();
        
        let user = crate::test_factories::create_user();
        let user_id = user.id;
        let message_id = Uuid::now_v7();
        let stamp_id = Uuid::now_v7();

        // Service gets token first
        mock_user_repo.expect_find_token_by_user_id()
            .with(mockall::predicate::eq(user_id))
            .returning(|_| Ok(Some("token".to_string())));

        mock_traq_client.expect_get_message()
            .with(mockall::predicate::always(), mockall::predicate::eq(message_id))
            .returning(move |_, _| Ok(domain::model::Message {
                id: message_id,
                channel_id: Uuid::now_v7(),
                user_id: Uuid::now_v7(),
                content: "content".to_string(),
                created_at: time::OffsetDateTime::now_utc(),
                updated_at: time::OffsetDateTime::now_utc(),
                reactions: vec![],
            }));

        // Expect message save
        mock_message_repo.expect_save()
            .returning(|_| Ok(()));

        // Then calls API
        mock_traq_client.expect_add_message_stamp()
            .with(
                mockall::predicate::eq("token"),
                mockall::predicate::eq(message_id),
                mockall::predicate::eq(stamp_id),
                mockall::predicate::eq(1)
            )
            .times(1)
            .returning(|_, _, _, _| Ok(()));

        let app = create_app(mock_message_repo, mock_user_repo, mock_traq_client, Some(user.clone()));

        // Login
        let login_req = Request::builder().uri("/login").method("POST").body(Body::empty()).unwrap();
        let login_res = app.clone().oneshot(login_req).await.unwrap();
        let cookie = login_res.headers().get(http::header::SET_COOKIE).unwrap().clone();

        // Add Stamp
        let req = Request::builder()
            .uri(&format!("/messages/{}/stamps/{}", message_id, stamp_id))
            .method("POST")
            .header(http::header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap();
        
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::NO_CONTENT);
    }
}
