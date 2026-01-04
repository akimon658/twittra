use domain::{repository::Repository, service::TimelineService};

pub mod auth;
pub mod timeline;
pub mod user;

#[derive(Clone, Debug)]
pub struct AppState {
    pub repo: Repository,
    pub service: TimelineService,
}

impl AppState {
    pub fn new(repo: Repository) -> Self {
        Self {
            repo: repo.clone(),
            service: TimelineService::new(repo),
        }
    }
}
