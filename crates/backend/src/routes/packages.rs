//! Package registry endpoints.

use axum::{routing::get, Router};

use crate::handlers::*;
use crate::state::AppState;

pub(super) fn package_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/packages/{owner}",
            get(list_packages).post(upload_package),
        )
        .route(
            "/api/v1/packages/{owner}/{type}/{name}/{version}",
            get(get_package_detail).delete(delete_package),
        )
}
