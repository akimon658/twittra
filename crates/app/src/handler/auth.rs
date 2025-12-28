use axum::{
    extract::Query,
    response::{IntoResponse, Redirect},
};
use http::StatusCode;

use crate::session::AuthSession;

const CSRF_STATE_KEY: &str = "oauth.csrf_state";

/// Start the OAuth2 login process by redirecting to the authorization URL.
#[utoipa::path(
    get,
    path = "/auth/login",
    responses(
        (status = StatusCode::FOUND),
        (status = StatusCode::INTERNAL_SERVER_ERROR),
    ),
    tag = "auth",
)]
#[tracing::instrument]
pub async fn login(auth_session: AuthSession) -> impl IntoResponse {
    let (authorize_url, csrf_state) = auth_session.backend.authorize_url();

    match auth_session
        .session
        .insert(CSRF_STATE_KEY, csrf_state.secret())
        .await
    {
        Ok(_) => {}
        Err(e) => {
            tracing::error!("{:?}", e);

            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    Redirect::to(authorize_url.as_str()).into_response()
}

#[derive(serde::Deserialize)]
pub struct AuthorizeQuery {
    code: String,
    state: String,
}

/// Handle the OAuth2 callback.
#[utoipa::path(
    get,
    params(
        ("code" = String, Query, description = "The authorization code returned by the OAuth2 provider."),
        ("state" = String, Query, description = "The CSRF state returned by the OAuth2 provider."),
    ),
    path = "/auth/callback",
    responses(
        (status = StatusCode::FOUND),
        (status = StatusCode::BAD_REQUEST),
        (status = StatusCode::UNAUTHORIZED),
        (status = StatusCode::INTERNAL_SERVER_ERROR),
    ),
    tag = "auth",
)]
pub async fn oauth_callback(
    mut auth_session: AuthSession,
    Query(AuthorizeQuery {
        code,
        state: new_state,
    }): Query<AuthorizeQuery>,
) -> impl IntoResponse {
    let Ok(Some(old_state)) = auth_session.session.get::<String>(CSRF_STATE_KEY).await else {
        return StatusCode::BAD_REQUEST.into_response();
    };

    if old_state != new_state {
        return StatusCode::BAD_REQUEST.into_response();
    }

    let user = match auth_session.authenticate(code).await {
        Ok(Some(user)) => user,
        Ok(None) => return StatusCode::UNAUTHORIZED.into_response(),
        Err(e) => {
            tracing::error!("{:?}", e);

            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if let Err(e) = auth_session.login(&user).await {
        tracing::error!("{:?}", e);

        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    Redirect::to("/").into_response()
}
