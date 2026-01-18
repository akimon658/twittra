use crate::{handler::AppState, session::AuthSession};
use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use domain::model::User;
use http::{StatusCode, header};
use uuid::Uuid;

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

    ([(header::CONTENT_TYPE, content_type)], icon).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::TestAppBuilder;
    use axum::{
        body::{self, Body},
        http::Request,
    };
    use domain::{service::MockTraqService, test_factories::UserBuilder};
    use mockall::predicate;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_get_me_success() {
        let mut mock_traq_service = MockTraqService::new();
        let user = UserBuilder::new().build();
        let user_id = user.id;
        let user_clone = user.clone();

        mock_traq_service
            .expect_get_user_by_id()
            .with(predicate::eq(user_id))
            .times(1)
            .returning(move |_| Ok(user_clone.clone()));

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

        // Get Me
        let req = Request::builder()
            .uri("/api/v1/me")
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        // Validate response body
        let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let response_user: User = serde_json::from_slice(&body).unwrap();
        assert_eq!(response_user.id, user.id);
        assert_eq!(response_user.handle, user.handle);
        assert_eq!(response_user.display_name, user.display_name);
    }
}
