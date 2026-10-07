//! Search, notifications and template helpers.

use axum::{
    routing::{get, patch},
    Router,
};

use crate::handlers::*;
use crate::state::AppState;

pub(super) fn misc_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/notifications", get(list_notifications))
        .route("/api/v1/licenses", get(list_licenses))
        .route("/api/v1/gitignore/templates", get(list_gitignores))
        .route(
            "/api/v1/notifications/threads/{id}",
            patch(mark_notification_read),
        )
}
