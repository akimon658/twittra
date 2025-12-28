use domain::repository::Repository;

pub mod auth;
pub mod user;

#[derive(Clone, Debug)]
pub struct AppState {
    pub repo: Repository,
}
