use serde::Serialize;

#[derive(Serialize)]
pub struct User {
    pub handle: String,
}
