use domain::model::User;

use axum::Json;

/// Get the current authenticated user's information.
#[utoipa::path(
    get,
    path = "/me",
    responses(
        (status = StatusCode::OK, body = User)
    ),
    tag = "user"
)]
pub async fn get_me() -> Json<User> {
    Json(User {
        handle: "alice".to_string(),
    })
}
