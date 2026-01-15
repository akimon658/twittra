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
#[tracing::instrument(skip_all)]
pub async fn get_me(auth_session: AuthSession, State(state): State<AppState>) -> impl IntoResponse {
    let user_id = match auth_session.user {
        Some(user) => user.id,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };
    let user = match state.traq_service.get_user_by_id(&user_id).await {
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
        (status = StatusCode::UNAUTHORIZED),
        (status = StatusCode::INTERNAL_SERVER_ERROR),
    ),
    security(
        ("cookieAuth" = []),
    ),
    tag = "user",
)]
#[tracing::instrument(skip(auth_session, state))]
pub async fn get_user_by_id(
    auth_session: AuthSession,
    State(state): State<AppState>,
    user_id: Path<Uuid>,
) -> impl IntoResponse {
    if auth_session.user.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let user = match state.traq_service.get_user_by_id(&user_id).await {
        Ok(user) => user,
        Err(e) => {
            tracing::error!("{:?}", e);

            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    Json(user).into_response()
}

/// Get a user's icon by user ID.
#[utoipa::path(
    get,
    params(
        ("userId" = Uuid, Path, description = "The ID of the user to retrieve"),
    ),
    path = "/users/{userId}/icon",
    responses(
        (
            status = StatusCode::OK,
            body = Vec<u8>,
            content(
                ("image/gif"),
                ("image/jpeg"),
                ("image/png"),
            )
        ),
        (status = StatusCode::UNAUTHORIZED),
        (status = StatusCode::INTERNAL_SERVER_ERROR),
    ),
    security(
        ("cookieAuth" = []),
    ),
    tag = "user",
)]
#[tracing::instrument]
pub async fn get_user_icon(
    auth_session: AuthSession,
    State(state): State<AppState>,
    user_id: Path<Uuid>,
) -> impl IntoResponse {
    if auth_session.user.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let (icon, content_type) = match state.traq_service.get_user_icon(&user_id).await {
        Ok(icon) => icon,
        Err(e) => {
            tracing::error!("{:?}", e);

            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    
    ([(http::header::CONTENT_TYPE, content_type)], icon).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handler::AppState;
    use crate::mocks::{MockMessageRepository, MockStampRepository, MockTraqClient, MockUserRepository};
    use crate::session::{AuthSession, Backend, UserSession};
    use axum::{Router, body::Body, http::Request};
    use domain::repository::Repository;
    use oauth2::{
        basic::BasicClient, AuthUrl, ClientId, ClientSecret, TokenUrl,
    };
    use std::sync::Arc;
    use tower::ServiceExt;

    fn create_app(
        mock_user_repo: MockUserRepository,
        mock_traq_client: MockTraqClient,
        user: Option<User>,
    ) -> Router {
        let mock_user_repo_arc = Arc::new(mock_user_repo);
        let repo = Repository {
            message: Arc::new(MockMessageRepository::new()),
            stamp: Arc::new(MockStampRepository::new()),
            user: mock_user_repo_arc.clone(),
        };
        let traq_client = Arc::new(mock_traq_client);
        let state = AppState::new(repo, traq_client);

        let client_id = ClientId::new("dummy_id".to_string());
        let client_secret = Some(ClientSecret::new("dummy_secret".to_string()));
        let auth_url = AuthUrl::new("http://dummy".to_string()).unwrap();
        let token_url = Some(TokenUrl::new("http://dummy".to_string()).unwrap());

        let oauth_client = BasicClient::new(client_id)
            .set_client_secret(client_secret.unwrap())
            .set_auth_uri(auth_url)
            .set_token_uri(token_url.unwrap());
        let backend = Backend::new(oauth_client, "http://dummy".to_string(), mock_user_repo_arc);

        let session_layer = tower_sessions::SessionManagerLayer::new(tower_sessions::MemoryStore::default());
        let auth_layer = axum_login::AuthManagerLayerBuilder::new(backend, session_layer).build();

        Router::new()
            .route("/me", axum::routing::get(get_me))
            .route("/users/{userId}", axum::routing::get(get_user_by_id))
            .route("/users/{userId}/icon", axum::routing::get(get_user_icon))
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
    async fn test_get_me_success() {
        let mut mock_user_repo = MockUserRepository::new();
        let user_id = Uuid::now_v7();
        let user = User {
            id: user_id,
            handle: "me".to_string(),
            display_name: "Me".to_string(),
        };

        // get_me calls get_user_by_id in service
        // Service first checks cache (user repo)
        let user_clone = user.clone();
        mock_user_repo
            .expect_find_by_id()
            .with(mockall::predicate::eq(user_id))
            .times(1)
            .returning(move |_| Ok(Some(user_clone.clone())));

        let app = create_app(mock_user_repo, MockTraqClient::new(), Some(user.clone()));

        // Login
        let login_req = Request::builder().uri("/login").method("POST").body(Body::empty()).unwrap();
        let login_res = app.clone().oneshot(login_req).await.unwrap();
        let cookie = login_res.headers().get(http::header::SET_COOKIE).unwrap().clone();

        // Get Me
        let req = Request::builder()
            .uri("/me")
            .header(http::header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }
}

