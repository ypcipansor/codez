//! Releases and release assets.

use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers::*;
use crate::state::AppState;

pub(super) fn release_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/repos/{owner}/{repo}/releases",
            get(list_releases).post(create_release),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/releases/{id}",
            get(get_release)
                .patch(update_release)
                .delete(delete_release),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/releases/{id}/assets",
            post(upload_release_asset),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/releases/{id}/assets/{asset_id}",
            get(download_release_asset),
        )
}
