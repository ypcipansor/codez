//! Instance administration.

use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers::*;
use crate::state::AppState;

pub(super) fn admin_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/admin/stats", get(get_admin_stats))
        .route("/api/v1/admin/notices", get(list_notices))
        .route(
            "/api/v1/admin/users",
            get(admin_list_users).post(admin_create_user),
        )
        .route(
            "/api/v1/admin/users/{username}",
            post(admin_edit_user).delete(admin_delete_user),
        )
}
