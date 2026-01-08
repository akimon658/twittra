use domain::{
    repository::Repository,
    service::{TimelineService, UserService},
    traq_client::TraqClient,
};
use std::sync::Arc;

pub mod auth;
pub mod timeline;
pub mod user;

#[derive(Clone, Debug)]
pub struct AppState {
    pub repo: Repository,
    pub user_service: UserService,
    pub timeline_service: TimelineService,
}

impl AppState {
    pub fn new(repo: Repository, traq_client: Arc<dyn TraqClient>) -> Self {
        Self {
            repo: repo.clone(),
            user_service: UserService::new(repo.clone(), traq_client),
            timeline_service: TimelineService::new(repo),
        }
    }
}
