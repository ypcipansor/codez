//! Actions workflows and runs.

use axum::{
    routing::{get, patch, post},
    Router,
};

use crate::handlers::*;
use crate::state::AppState;

pub(super) fn action_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/repos/{owner}/{repo}/actions/workflows",
            get(list_workflows),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/actions/workflows/{id}/runs",
            get(list_workflow_runs).post(trigger_workflow),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/actions/runs/{run_id}",
            patch(update_workflow_run).delete(delete_workflow_run),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/actions/runs/{run_id}/logs",
            get(get_workflow_run_logs),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/actions/runs/{run_id}/rerun",
            post(rerun_workflow_run),
        )
}
