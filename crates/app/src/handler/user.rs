use domain::model::User;

use axum::Json;

pub async fn get_me() -> Json<User> {
    Json(User {
        handle: "alice".to_string(),
    })
}
