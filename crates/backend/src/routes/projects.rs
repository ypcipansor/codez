//! Projects, columns and cards.

use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers::*;
use crate::state::AppState;

pub(super) fn project_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/repos/{owner}/{repo}/projects",
            get(list_projects).post(create_project),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/projects/{id}",
            get(get_project),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/projects/{id}/close",
            post(close_project),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/projects/{id}/reopen",
            post(reopen_project),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/projects/{id}/columns",
            get(list_project_columns).post(create_project_column),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/projects/columns/{id}/cards",
            get(list_project_cards).post(create_project_card),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/projects/cards/{id}/move",
            post(move_project_card),
        )
}
