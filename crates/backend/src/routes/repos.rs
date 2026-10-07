//! Repository, source browsing, branches, tags and topics.

use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::handlers::*;
use crate::state::AppState;

pub(super) fn repo_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/repos", get(list_repos))
        .route("/api/v1/repos/{owner}/{repo}", get(get_repo))
        .route("/api/v1/user/repos", post(create_repo))
        .route(
            "/api/v1/repos/{owner}/{repo}/contents/{*path}",
            get(get_contents).put(update_file),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/contents",
            get(get_root_contents),
        )
        .route("/api/v1/repos/{owner}/{repo}/commits", get(list_commits))
        .route(
            "/api/v1/repos/{owner}/{repo}/topics",
            get(list_topics).put(update_topics),
        )
        .route("/api/v1/repos/{owner}/{repo}/star", post(star_repo))
        .route(
            "/api/v1/repos/{owner}/{repo}/user_status",
            get(get_user_repo_status),
        )
        .route("/api/v1/repos/{owner}/{repo}/watch", post(watch_repo))
        .route("/api/v1/repos/{owner}/{repo}/fork", post(fork_repo))
        .route("/api/v1/repos/search", get(search_repos))
        .route(
            "/api/v1/repos/{owner}/{repo}/settings",
            get(get_repo_settings).patch(update_repo_settings),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/hooks",
            get(list_hooks).post(create_hook),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/hooks/{id}/deliveries",
            get(list_hook_deliveries),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/secrets",
            get(list_secrets).post(create_secret),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/keys",
            get(list_deploy_keys).post(create_deploy_key),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/mirror-sync",
            post(mirror_sync),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/collaborators",
            get(list_collaborators),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/collaborators/{collaborator}",
            get(get_collaborator).put(add_collaborator),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/branches",
            get(list_branches).post(create_branch),
        )
        .route("/api/v1/repos/{owner}/{repo}/tags", get(list_tags))
        .route("/api/v1/repos/{owner}/{repo}/media", post(upload_media))
        .route(
            "/api/v1/repos/{owner}/{repo}/commits/{sha}/diff",
            get(get_commit_diff),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/raw/{*path}",
            get(get_raw_file),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/languages",
            get(get_repo_languages),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/branch_protections",
            get(list_branch_protections).post(create_branch_protection),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/branch_protections/{name}",
            delete(delete_branch_protection),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/statuses/{sha}",
            post(create_commit_status),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/commits/{ref}/statuses",
            get(list_commit_statuses),
        )
        .route("/api/v1/repos/migrate", post(migrate_repo))
        .route("/api/v1/repos/{owner}/{repo}/transfer", post(transfer_repo))
        .route("/api/v1/repos/{owner}/{repo}/search", get(search_repo_code))
        .route("/api/v1/repos/{owner}/{repo}/pulse", get(get_repo_pulse))
        .route(
            "/api/v1/repos/{owner}/{repo}/security/scan",
            post(run_security_scan).get(run_security_scan),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/wiki/pages",
            get(list_wiki_pages).post(create_wiki_page),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/wiki/pages/{page_name}",
            get(get_wiki_page).put(update_wiki_page),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/discussions",
            get(list_discussions).post(create_discussion),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/discussions/{id}",
            get(get_discussion)
                .patch(update_discussion)
                .delete(delete_discussion),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/discussions/{id}/comments",
            get(list_discussion_comments).post(create_discussion_comment),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/git/lfs/locks",
            get(list_lfs_locks).post(create_lfs_lock),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/git/lfs/locks/{id}/unlock",
            post(delete_lfs_lock),
        )
}
