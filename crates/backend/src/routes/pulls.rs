//! Pull requests, reviews and merges.

use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers::*;
use crate::state::AppState;

pub(super) fn pull_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/repos/{owner}/{repo}/pulls",
            get(list_pulls).post(create_pull),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/pulls/{index}",
            get(get_pull).patch(update_pull),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/pulls/{index}/reviews",
            get(list_reviews).post(create_review),
        )
        .route("/api/v1/user/pulls", get(list_user_pulls))
        .route(
            "/api/v1/repos/{owner}/{repo}/pulls/{index}/merge",
            post(merge_pull),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/pulls/{index}/requested_reviewers",
            post(request_review),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/pulls/{index}/files",
            get(get_pr_files),
        )
}
