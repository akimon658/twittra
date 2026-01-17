use crate::{handler::AppState, session::AuthSession};
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
    use crate::test_helpers::TestAppBuilder;
    use axum::{body::Body, http::Request};
    use domain::service::MockTraqService;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_get_stamps_all() {
        let mut mock_traq_service = MockTraqService::new();

        let stamp = crate::test_factories::create_stamp();
        let stamps = vec![stamp.clone()];
        let stamps_clone = stamps.clone();

        mock_traq_service
            .expect_get_stamps()
            .returning(move || Ok(stamps_clone.clone()));

        let user = crate::test_factories::create_user();
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
        let cookie = login_res
            .headers()
            .get(http::header::SET_COOKIE)
            .unwrap()
            .clone();

        // Get Stamps
        let req = Request::builder()
            .uri("/api/v1/stamps")
            .header(http::header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }
}
