use domain::service::{TimelineService, TraqService};
use std::sync::Arc;

pub mod auth;
pub mod message;
pub mod stamp;
pub mod timeline;
pub mod user;

#[derive(Clone, Debug)]
pub struct AppState {
    pub traq_service: Arc<dyn TraqService>,
    pub timeline_service: Arc<dyn TimelineService>,
}

impl AppState {
    pub fn new(
        traq_service: Arc<dyn TraqService>,
        timeline_service: Arc<dyn TimelineService>,
    ) -> Self {
        Self {
            traq_service,
            timeline_service,
        }
    }
}
