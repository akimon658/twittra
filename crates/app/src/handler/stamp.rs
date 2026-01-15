use axum::{
    Json,
    extract::{Path, Query, State},
    response::IntoResponse,
};
use domain::model::Stamp;
use http::StatusCode;
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

use crate::{handler::AppState, session::AuthSession};

#[derive(Debug, Deserialize, IntoParams)]
pub struct StampSearchQuery {
    pub name: Option<String>,
}

#[utoipa::path(
    get,
    params(
        ("stampId" = Uuid, Path, description = "The ID of the stamp to retrieve"),
    ),
    path = "/stamps/{stampId}",
    responses(
        (status = StatusCode::OK, body = Stamp),
        (status = StatusCode::UNAUTHORIZED),
        (status = StatusCode::INTERNAL_SERVER_ERROR),
    ),
    security(
        ("cookieAuth" = []),
    ),
    tag = "stamp",
)]
#[tracing::instrument(skip(auth_session, state))]
pub async fn get_stamp_by_id(
    auth_session: AuthSession,
    State(state): State<AppState>,
    stamp_id: Path<Uuid>,
) -> impl IntoResponse {
    if auth_session.user.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let stamp = match state.traq_service.get_stamp_by_id(&stamp_id).await {
        Ok(stamp) => stamp,
        Err(e) => {
            tracing::error!("{:?}", e);

            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    Json(stamp).into_response()
}

#[utoipa::path(
    get,
    params(
        ("stampId" = Uuid, Path, description = "The ID of the stamp to retrieve"),
    ),
    path = "/stamps/{stampId}/image",
    responses(
        (
            status = StatusCode::OK,
            body = Vec<u8>,
            content(
                ("image/gif"),
                ("image/jpeg"),
                ("image/png"),
                ("image/svg+xml"),
            )
        ),
        (status = StatusCode::UNAUTHORIZED),
        (status = StatusCode::INTERNAL_SERVER_ERROR),
    ),
    security(
        ("cookieAuth" = []),
    ),
    tag = "stamp",
)]
#[tracing::instrument(skip(auth_session, state))]
pub async fn get_stamp_image(
    auth_session: AuthSession,
    State(state): State<AppState>,
    stamp_id: Path<Uuid>,
) -> impl IntoResponse {
    if auth_session.user.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let (image, content_type) = match state.traq_service.get_stamp_image(&stamp_id).await {
        Ok(image) => image,
        Err(e) => {
            tracing::error!("{:?}", e);

            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    ([(http::header::CONTENT_TYPE, content_type)], image).into_response()
}

#[utoipa::path(
    get,
    path = "/stamps",
    params(
        ("name" = Option<String>, Query, description = "Filter stamps by name"),
    ),
    responses(
        (status = StatusCode::OK, body = Vec<Stamp>),
        (status = StatusCode::UNAUTHORIZED),
        (status = StatusCode::INTERNAL_SERVER_ERROR),
    ),
    security(
        ("cookieAuth" = []),
    ),
    tag = "stamp",
)]
#[tracing::instrument(skip(auth_session, state))]
pub async fn get_stamps(
    auth_session: AuthSession,
    State(state): State<AppState>,
    Query(query): Query<StampSearchQuery>,
) -> impl IntoResponse {
    if auth_session.user.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let stamps = if let Some(name) = query.name {
        match state.traq_service.search_stamps(&name).await {
            Ok(stamps) => stamps,
            Err(e) => {
                tracing::error!("{:?}", e);

                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
    } else {
        match state.traq_service.get_stamps().await {
            Ok(stamps) => stamps,
            Err(e) => {
                tracing::error!("{:?}", e);

                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
    };

    Json(stamps).into_response()
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
        mock_stamp_repo: MockStampRepository,
        mock_traq_client: MockTraqClient,
        mock_user_repo: MockUserRepository,
        user: Option<User>,
    ) -> Router {
        let mock_user_repo_arc = Arc::new(mock_user_repo);
        let repo = Repository {
            message: Arc::new(MockMessageRepository::new()),
            stamp: Arc::new(mock_stamp_repo),
            user: mock_user_repo_arc.clone(),
        };
        let traq_client = Arc::new(mock_traq_client);
        let state = AppState::new(repo, traq_client);

        let backend = create_test_backend(mock_user_repo_arc);

        let session_layer = tower_sessions::SessionManagerLayer::new(tower_sessions::MemoryStore::default());
        let auth_layer = axum_login::AuthManagerLayerBuilder::new(backend, session_layer).build();

        Router::new()
            .route("/stamps", axum::routing::get(get_stamps))
            .route("/stamps/{stampId}", axum::routing::get(get_stamp_by_id))
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
    async fn test_get_stamps_all() {
        let mut mock_stamp_repo = MockStampRepository::new();
        let mut mock_traq_client = MockTraqClient::new();
        let mut mock_user_repo = MockUserRepository::new();
        
        let stamp = crate::test_factories::create_stamp();
        let stamps = vec![stamp.clone()];
        
        // Service::get_stamps needs a token
        mock_user_repo.expect_find_random_valid_token().returning(|| Ok(Some("token".into())));
        
        let stamps_clone = stamps.clone();
        mock_traq_client.expect_get_stamps().returning(move |_| Ok(stamps_clone.clone()));
        mock_stamp_repo.expect_save_batch().returning(|_| Ok(()));

        let user = crate::test_factories::create_user();
        let app = create_app(mock_stamp_repo, mock_traq_client, mock_user_repo, Some(user.clone()));

        // Login
        let login_req = Request::builder().uri("/login").method("POST").body(Body::empty()).unwrap();
        let login_res = app.clone().oneshot(login_req).await.unwrap();
        let cookie = login_res.headers().get(http::header::SET_COOKIE).unwrap().clone();

        // Get Stamps
        let req = Request::builder()
            .uri("/stamps")
            .header(http::header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }
}

