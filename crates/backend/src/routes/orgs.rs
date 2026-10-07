//! Organizations, teams, members and audit logs.

use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers::*;
use crate::state::AppState;

pub(super) fn org_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/orgs", post(create_org))
        .route("/api/v1/orgs/{org}", get(get_org))
        .route("/api/v1/orgs/{org}/repos", get(list_org_repos))
        .route(
            "/api/v1/orgs/{org}/teams",
            get(list_teams).post(create_team),
        )
        .route("/api/v1/orgs/{org}/members", get(list_org_members))
        .route(
            "/api/v1/orgs/{org}/members/{username}",
            post(add_org_member)
                .delete(remove_org_member)
                .put(update_org_member_role),
        )
        .route("/api/v1/orgs/{org}/audit-logs", get(list_org_audit_logs))
}
