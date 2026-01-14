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
#[tracing::instrument]
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
