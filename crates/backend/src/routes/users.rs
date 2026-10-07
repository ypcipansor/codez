//! Authentication, profile, keys, follows and settings.

use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::handlers::*;
use crate::state::AppState;

pub(super) fn user_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/users/{username}", get(get_user))
        .route("/api/v1/users/login", post(login_user))
        .route("/api/v1/users/register", post(register_user))
        .route("/api/v1/user/starred", get(list_starred_repos))
        .route(
            "/api/v1/user/settings",
            get(get_user_settings).patch(update_user_settings),
        )
        .route("/api/v1/user/keys", get(list_keys).post(create_key))
        .route("/api/v1/user/feeds", get(list_feeds))
        .route("/api/v1/user/2fa", get(get_2fa).post(update_2fa))
        .route(
            "/api/v1/user/gpg_keys",
            get(list_gpg_keys).post(create_gpg_key),
        )
        .route("/api/v1/user/gpg_keys/{id}", delete(delete_gpg_key))
        .route("/api/v1/user/oauth2", get(list_oauth2_providers))
        .route("/api/v1/users/{username}/followers", get(list_followers))
        .route("/api/v1/users/{username}/following", get(list_following))
        .route(
            "/api/v1/users/{username}/follow",
            post(follow_user).delete(unfollow_user),
        )
        .route("/api/v1/users/{username}/heatmap", get(get_user_heatmap))
        .route("/api/v1/user/emails", get(list_emails))
        .route(
            "/api/v1/user/applications/oauth2",
            get(list_oauth2_apps).post(create_oauth2_app),
        )
        .route(
            "/api/v1/user/applications/oauth2/{id}",
            delete(delete_oauth2_app),
        )
        .route("/api/v1/user/keys/{id}", delete(delete_ssh_key))
        .route("/api/v1/user/gpg_keys/{id}/verify", post(verify_gpg_key))
}
