//! Pull request list and detail components.

use super::CommitStatusList;
use crate::api::{get, get_opt, local_resource, patch_json, post_json};
use crate::components::RepoNav;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::*;
use shared::{
    CreateReviewOption, DiffFile, MergePullRequestOption, PullRequest, Review,
    UpdatePullRequestOption,
};

#[component]
pub fn PullRequestList() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|params| params.get("owner").unwrap_or_default());
    let repo_name = move || params.with(|params| params.get("repo").unwrap_or_default());

    let pulls = local_resource(
        move || (owner(), repo_name()),
        |(o, r)| async move {
            get::<Vec<PullRequest>>(&format!("/api/v1/repos/{}/{}/pulls", o, r)).await
        },
    );

    view! {
        <div class="pull-list">
            <RepoNav/>
            <div class="header flex-between">
                <h3>"Pull Requests for " {owner} "/" {repo_name}</h3>
                <a href="compare" class="btn-primary">"New Pull Request"</a>
            </div>
            <ul>
                <Suspense fallback=move || view! { <li>"Loading pull requests..."</li> }>
                    {move || pulls.get().map(|list| view! {
                        <For each=move || list.clone() key=|p| p.id children=move |p| {
                            let href = format!("/repos/{}/{}/pulls/{}", owner(), repo_name(), p.number);
                            view! { <li><a href=href>"#" {p.number} " " {p.title}</a> " (" {p.state} ")"</li> }
                        }/>
                    })}
                </Suspense>
            </ul>
        </div>
    }
}

#[component]
pub fn PullRequestDetail() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|params| params.get("owner").unwrap_or_default());
    let repo_name = move || params.with(|params| params.get("repo").unwrap_or_default());
    let index = move || {
        params.with(|params| {
            params
                .get("index")
                .unwrap_or_default()
                .parse::<u64>()
                .unwrap_or_default()
        })
    };

    let (merge_action, set_merge_action) = signal("merge".to_string());
    let (trigger_refresh, set_trigger_refresh) = signal(0);
    let (action_error, set_action_error) = signal(Option::<String>::None);

    // Fetch PR details to display status, title, body etc.
    let pull_request = local_resource(
        move || (owner(), repo_name(), index(), trigger_refresh.get()),
        |(o, r, i, _)| async move {
            get_opt::<PullRequest>(&format!("/api/v1/repos/{}/{}/pulls/{}", o, r, i)).await
        },
    );

    let pr_files = local_resource(
        move || (owner(), repo_name(), index()),
        |(o, r, i)| async move {
            get::<Vec<DiffFile>>(&format!("/api/v1/repos/{}/{}/pulls/{}/files", o, r, i)).await
        },
    );

    let on_merge = move |_| {
        let o = owner();
        let r = repo_name();
        let i = index();
        let action = merge_action.get();
        spawn_local(async move {
            let payload = MergePullRequestOption {
                merge_action: action,
                merge_title_field: None,
                merge_message_field: None,
            };
            if post_json(
                &format!("/api/v1/repos/{}/{}/pulls/{}/merge", o, r, i),
                &payload,
            )
            .await
            {
                set_action_error.set(None);
                set_trigger_refresh.update(|n| *n += 1);
            } else {
                set_action_error.set(Some(crate::api::WRITE_ERROR.to_string()));
            }
        });
    };

    let on_toggle_state = move |current_state: String| {
        let o = owner();
        let r = repo_name();
        let idx = index();
        let new_state = if current_state == "open" {
            "closed"
        } else {
            "open"
        };
        let payload = UpdatePullRequestOption {
            title: None,
            body: None,
            state: Some(new_state.to_string()),
        };
        spawn_local(async move {
            if patch_json(
                &format!("/api/v1/repos/{}/{}/pulls/{}", o, r, idx),
                &payload,
            )
            .await
            {
                set_action_error.set(None);
                set_trigger_refresh.update(|n| *n += 1);
            } else {
                set_action_error.set(Some(crate::api::WRITE_ERROR.to_string()));
            }
        });
    };

    let (is_editing, set_is_editing) = signal(false);
    let (edit_title, set_edit_title) = signal("".to_string());
    let (edit_body, set_edit_body) = signal("".to_string());

    let on_start_edit = move |t: String, b: String| {
        set_edit_title.set(t);
        set_edit_body.set(b);
        set_is_editing.set(true);
    };

    let on_cancel_edit = move |_| {
        set_is_editing.set(false);
    };

    let on_save_edit = move |_| {
        let o = owner();
        let r = repo_name();
        let idx = index();
        let payload = UpdatePullRequestOption {
            title: Some(edit_title.get()),
            body: Some(edit_body.get()),
            state: None,
        };
        spawn_local(async move {
            if patch_json(
                &format!("/api/v1/repos/{}/{}/pulls/{}", o, r, idx),
                &payload,
            )
            .await
            {
                set_is_editing.set(false);
                set_action_error.set(None);
                set_trigger_refresh.update(|n| *n += 1);
            } else {
                set_action_error.set(Some(crate::api::WRITE_ERROR.to_string()));
            }
        });
    };

    let reviews = local_resource(
        move || (owner(), repo_name(), index(), trigger_refresh.get()),
        |(o, r, i, _)| async move {
            get::<Vec<Review>>(&format!("/api/v1/repos/{}/{}/pulls/{}/reviews", o, r, i)).await
        },
    );

    let (review_body, set_review_body) = signal("".to_string());

    let on_submit_review = move |event: String| {
        let o = owner();
        let r = repo_name();
        let i = index();
        let payload = CreateReviewOption {
            body: review_body.get(),
            event,
        };
        spawn_local(async move {
            if post_json(
                &format!("/api/v1/repos/{}/{}/pulls/{}/reviews", o, r, i),
                &payload,
            )
            .await
            {
                set_review_body.set("".to_string());
                set_action_error.set(None);
                set_trigger_refresh.update(|n| *n += 1);
            } else {
                set_action_error.set(Some(crate::api::WRITE_ERROR.to_string()));
            }
        });
    };

    view! {
        <div class="pull-detail">
        <RepoNav/>
            <Suspense fallback=move || view! { <p>"Loading PR..."</p> }>
                {move || match pull_request.get() {
                    Some(Some(pr)) => {
                        let state_clone = pr.state.clone();
                        let state_for_toggle = state_clone.clone();
                        let title_clone = pr.title.clone();
                        let body_clone = pr.body.clone().unwrap_or_default();

                        view! {
                            <div class="pr-header">
                                {if is_editing.get() {
                                    view! {
                                        <input type="text" prop:value=edit_title on:input=move |ev| set_edit_title.set(event_target_value(&ev)) class="input-title" />
                                    }.into_any()
                                } else {
                                    view! { <h3>"Pull Request #" {index} ": " {pr.title.clone()}</h3> }.into_any()
                                }}
                                <span class="state">{pr.state.clone()}</span>
                                <span class="meta">" opened by " {pr.user.username}</span>
                                <button on:click=move |_| on_toggle_state(state_for_toggle.clone()) class="ml-2">
                                    {if state_clone == "open" { "Close PR" } else { "Reopen PR" }}
                                </button>
                                {if !is_editing.get() {
                                    view! { <button on:click=move |_| on_start_edit(title_clone.clone(), body_clone.clone()) class="ml-1">"Edit"</button> }.into_any()
                                } else {
                                     view! { <span></span> }.into_any()
                                }}
                            </div>
                            <div class="pr-body">
                                {if is_editing.get() {
                                    view! {
                                        <div>
                                            <textarea prop:value=edit_body on:input=move |ev| set_edit_body.set(event_target_value(&ev)) rows="5" class="w-100"></textarea>
                                            <button on:click=on_save_edit>"Save"</button>
                                            <button on:click=on_cancel_edit class="ml-1">"Cancel"</button>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! { <p>{pr.body.clone().unwrap_or_default()}</p> }.into_any()
                                }}
                            </div>
                            <CommitStatusList owner=owner() repo=repo_name() sha=pr.head_sha.clone() />
                        }
                    }.into_any(),
                    _ => view! { <p>"Pull Request not found"</p> }.into_any()
                }}
            </Suspense>

            <div class="pr-actions">
                <select on:change=move |ev| set_merge_action.set(event_target_value(&ev))>
                    <option value="merge">"Merge Commit"</option>
                    <option value="rebase">"Rebase and Merge"</option>
                    <option value="squash">"Squash and Merge"</option>
                </select>
                <button on:click=on_merge class="btn-merge">"Merge Pull Request"</button>
                {move || action_error.get().map(|msg| view! {
                    <p class="form-error" role="alert">{msg}</p>
                })}
            </div>
            <div class="pr-files">
                <h4>"Files Changed"</h4>
                <Suspense fallback=move || view! { <p>"Loading files..."</p> }>
                    {move || pr_files.get().map(|files| view! {
                        <For each=move || files.clone() key=|f| f.name.clone() children=move |f| {
                             view! {
                                <div class="file-diff">
                                    <div class="file-header">
                                        <strong>{f.name}</strong>
                                        <span class="diff-stats">" +"{f.additions} " -"{f.deletions}</span>
                                    </div>
                                    <p>"Binary or large file diff suppressed"</p>
                                </div>
                            }
                        }/>
                    })}
                </Suspense>
            </div>

            <div class="pr-reviews">
                <h4>"Reviews"</h4>
                <div class="review-list">
                    <Suspense fallback=move || view! { <p>"Loading reviews..."</p> }>
                        {move || reviews.get().map(|list| view! {
                            <For each=move || list.clone() key=|r| r.id children=move |r| {
                                view! {
                                    <div class="review-item">
                                        <div class="review-header">
                                            <strong>{r.user.username}</strong> " "
                                            <span style=format!("font-weight: bold; color: {}", match r.state.as_str() {
                                                "APPROVED" => "green",
                                                "CHANGES_REQUESTED" => "red",
                                                _ => "gray"
                                            })>{r.state.clone()}</span>
                                            " on " {r.created_at}
                                        </div>
                                        <div class="review-body">
                                            {r.body}
                                        </div>
                                    </div>
                                }
                            }/>
                        })}
                    </Suspense>
                </div>

                <div class="add-review">
                    <h5>"Submit Review"</h5>
                    <textarea prop:value=review_body on:input=move |ev| set_review_body.set(event_target_value(&ev)) placeholder="Leave a comment"></textarea>
                    <div class="flex gap-sm">
                        <button on:click=move |_| on_submit_review("COMMENT".to_string())>"Comment"</button>
                        <button on:click=move |_| on_submit_review("APPROVE".to_string()) class="text-success">"Approve"</button>
                        <button on:click=move |_| on_submit_review("REQUEST_CHANGES".to_string()) class="error-msg">"Request Changes"</button>
                    </div>
                    {move || action_error.get().map(|msg| view! {
                        <p class="form-error" role="alert">{msg}</p>
                    })}
                </div>
            </div>
        </div>
    }
}
