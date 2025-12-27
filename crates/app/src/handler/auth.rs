use axum::{
    Router,
    extract::Query,
    response::{IntoResponse, Redirect},
    routing,
};
use http::StatusCode;

use crate::{handler::AppState, session::AuthSession};

const CSRF_STATE_KEY: &str = "oauth.csrf_state";

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/login", routing::get(login))
        .route("/auth/callback", routing::get(oauth_callback))
}

pub async fn login(auth_session: AuthSession) -> impl IntoResponse {
    let (authorize_url, csrf_state) = auth_session.backend.authorize_url();

    match auth_session
        .session
        .insert(CSRF_STATE_KEY, csrf_state.secret())
        .await
    {
        Ok(_) => {}
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    Redirect::to(authorize_url.as_str()).into_response()
}

#[derive(serde::Deserialize)]
pub struct AuthorizeQuery {
    code: String,
    state: String,
}

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
        Err(_) => {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if let Err(_) = auth_session.login(&user).await {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    Redirect::to("/").into_response()
}
