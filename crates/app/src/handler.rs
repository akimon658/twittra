use domain::{
    repository::Repository,
    service::{TimelineService, TraqService},
    traq_client::TraqClient,
};
use std::sync::Arc;

pub mod auth;
pub mod stamp;
pub mod timeline;
pub mod user;

#[derive(Clone, Debug)]
pub struct AppState {
    pub repo: Repository,
    pub traq_service: TraqService,
    pub timeline_service: TimelineService,
}

impl AppState {
    pub fn new(repo: Repository, traq_client: Arc<dyn TraqClient>) -> Self {
        Self {
            repo: repo.clone(),
            traq_service: TraqService::new(repo.clone(), traq_client),
            timeline_service: TimelineService::new(repo),
        }
    }
}
