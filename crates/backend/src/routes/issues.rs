//! Issues, comments, labels and milestones.

use axum::{
    routing::{delete, get, patch, post, put},
    Router,
};

use crate::handlers::*;
use crate::state::AppState;

pub(super) fn issue_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/repos/{owner}/{repo}/issues",
            get(list_issues).post(create_issue),
        )
        .route("/api/v1/user/issues", get(list_user_issues))
        .route(
            "/api/v1/repos/{owner}/{repo}/issues/{index}",
            get(get_issue).patch(update_issue),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/issues/{index}/lock",
            put(lock_issue).delete(unlock_issue),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/issues/{index}/comments",
            get(list_comments).post(create_comment),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/issues/comments/{id}",
            patch(update_comment).delete(delete_comment),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/labels",
            get(list_labels).post(create_label),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/labels/{id}",
            patch(update_label).delete(delete_label),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/milestones",
            get(list_milestones).post(create_milestone),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/milestones/{id}",
            get(get_milestone)
                .patch(update_milestone)
                .delete(delete_milestone),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/issues/comments/{id}/reactions",
            post(add_reaction),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/issues/{index}/assignees",
            post(add_issue_assignee),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/issues/{index}/assignees/{username}",
            delete(remove_issue_assignee),
        )
        .route("/api/v1/search/issues", get(search_issues_global))
        .route(
            "/api/v1/repos/{owner}/{repo}/milestones/{id}/stats",
            get(get_milestone_stats),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/issues/{index}/labels",
            post(add_issue_label),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/issues/{index}/labels/{id}",
            delete(remove_issue_label),
        )
}
