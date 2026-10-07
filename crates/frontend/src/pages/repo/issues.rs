//! Issue, label and milestone components.

use crate::api::{delete, get, get_or, local_resource, patch_json, post_json, put};
use crate::components::RepoNav;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::*;
use shared::{
    Comment, CreateCommentOption, CreateIssueOption, CreateLabelOption, CreateMilestoneOption,
    Issue, Label, Milestone, MilestoneStats, UpdateIssueOption,
};

#[component]
pub fn IssueList() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|params| params.get("owner").unwrap_or_default());
    let repo_name = move || params.with(|params| params.get("repo").unwrap_or_default());

    let (state_filter, set_state_filter) = signal("open".to_string());
    let (search_query, set_search_query) = signal("".to_string());
    let (label_filter, set_label_filter) = signal("".to_string());
    let (assignee_filter, set_assignee_filter) = signal("".to_string());
    let query_map = use_query_map();
    let initial_milestone = query_map.with(|q| q.get("milestone_id").unwrap_or_default());
    let (milestone_filter, set_milestone_filter) = signal(initial_milestone);
    let (sort, set_sort) = signal("created".to_string());
    let (direction, set_direction) = signal("desc".to_string());
    let (page, set_page) = signal(1);
    let (show_new_issue, set_show_new_issue) = signal(false);
    let (new_issue_title, set_new_issue_title) = signal("".to_string());
    let (new_issue_body, set_new_issue_body) = signal("".to_string());
    let (new_issue_milestone, set_new_issue_milestone) = signal("".to_string());
    let (refresh, set_refresh) = signal(0);
    let (form_error, set_form_error) = signal(Option::<String>::None);

    let issues = local_resource(
        move || {
            (
                owner(),
                repo_name(),
                state_filter.get(),
                search_query.get(),
                label_filter.get(),
                assignee_filter.get(),
                milestone_filter.get(),
                sort.get(),
                direction.get(),
                page.get(),
                refresh.get(),
            )
        },
        |(o, r, s, q, l, a, m, srt, dir, p, _)| async move {
            let mut url = format!(
                "/api/v1/repos/{}/{}/issues?state={}&q={}&sort={}&direction={}&page={}&limit=10",
                o, r, s, q, srt, dir, p
            );
            if !l.is_empty() {
                url.push_str(&format!("&label_id={}", l));
            }
            if !a.is_empty() {
                url.push_str(&format!("&assignee_username={}", a));
            }
            if !m.is_empty() {
                url.push_str(&format!("&milestone_id={}", m));
            }
            get::<Vec<Issue>>(&url).await
        },
    );

    let labels = local_resource(
        move || (owner(), repo_name()),
        |(o, r)| async move { get::<Vec<Label>>(&format!("/api/v1/repos/{}/{}/labels", o, r)).await },
    );

    let users = local_resource(
        || (),
        |_| async move {
            // Mock users for filtering
            vec![
                shared::User::new(1, "admin".to_string(), None),
                shared::User::new(2, "user".to_string(), None),
            ]
        },
    );

    let milestones = local_resource(
        move || (owner(), repo_name()),
        |(o, r)| async move {
            get::<Vec<shared::Milestone>>(&format!("/api/v1/repos/{}/{}/milestones", o, r)).await
        },
    );

    let on_create_issue = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let ms_val = new_issue_milestone.get();
        let milestone = if ms_val.is_empty() {
            None
        } else {
            ms_val.parse::<u64>().ok()
        };
        let payload = CreateIssueOption {
            title: new_issue_title.get(),
            body: Some(new_issue_body.get()),
            milestone,
        };
        let o = owner();
        let r = repo_name();
        spawn_local(async move {
            // Only clear the form once the server accepted the issue; a
            // rejected write must leave the user's input intact.
            if post_json(&format!("/api/v1/repos/{}/{}/issues", o, r), &payload).await {
                set_new_issue_title.set("".to_string());
                set_new_issue_body.set("".to_string());
                set_show_new_issue.set(false);
                set_form_error.set(None);
                set_refresh.update(|n| *n += 1);
            } else {
                set_form_error.set(Some(crate::api::WRITE_ERROR.to_string()));
            }
        });
    };

    view! {
        <div class="issue-list">
            <RepoNav/>
            <div class="header flex-between">
                <h3>"Issues for " {owner} "/" {repo_name}</h3>
                <button on:click=move |_| set_show_new_issue.set(!show_new_issue.get())>
                    {move || if show_new_issue.get() { "Cancel" } else { "New Issue" }}
                </button>
            </div>

            {move || if show_new_issue.get() {
                view! {
                    <form on:submit=on_create_issue class="panel">
                        <input type="text" placeholder="Title" prop:value=new_issue_title on:input=move |ev| set_new_issue_title.set(event_target_value(&ev))  required />
                        <textarea placeholder="Body" prop:value=new_issue_body on:input=move |ev| set_new_issue_body.set(event_target_value(&ev))  rows="5"></textarea>
                        <select on:change=move |ev| set_new_issue_milestone.set(event_target_value(&ev))>
                            <option value="">"No Milestone"</option>
                            <Suspense fallback=move || view! { <option>"Loading..."</option> }>
                                {move || milestones.get().map(|list| {
                                    list.into_iter().map(|m| {
                                        view! { <option value=m.id.to_string()>{m.title}</option> }
                                    }).collect_view()
                                })}
                            </Suspense>
                        </select>
                        <button type="submit">"Create Issue"</button>
                        {move || form_error.get().map(|msg| view! {
                            <p class="form-error" role="alert">{msg}</p>
                        })}
                    </form>
                }.into_any()
            } else {
                view! { <span></span> }.into_any()
            }}

            <div class="issue-filters">
                <button on:click=move |_| set_state_filter.set("open".to_string()) class:active=move || state_filter.get() == "open">"Open"</button>
                <button on:click=move |_| set_state_filter.set("closed".to_string()) class:active=move || state_filter.get() == "closed">"Closed"</button>
                <button on:click=move |_| set_state_filter.set("all".to_string()) class:active=move || state_filter.get() == "all">"All"</button>

                <input type="text" placeholder="Search issues..." prop:value=search_query on:input=move |ev| set_search_query.set(event_target_value(&ev)) />

                <select on:change=move |ev| set_milestone_filter.set(event_target_value(&ev))>
                    <option value="">"Milestone"</option>
                    <Suspense fallback=move || view! { <option>"Loading..."</option> }>
                        {move || milestones.get().map(|list| {
                            list.into_iter().map(|m| {
                                view! { <option value=m.id.to_string() selected=move || milestone_filter.get() == m.id.to_string()>{m.title}</option> }
                            }).collect_view()
                        })}
                    </Suspense>
                </select>

                <Suspense fallback=move || view! { <span>"Loading labels..."</span> }>
                    {move || labels.get().map(|list| view! {
                        <select on:change=move |ev| set_label_filter.set(event_target_value(&ev))>
                            <option value="">"Label"</option>
                            <For each=move || list.clone() key=|l| l.id children=move |l| {
                                view! { <option value={l.id}>{l.name}</option> }
                            }/>
                        </select>
                    })}
                </Suspense>

                <Suspense fallback=move || view! { <span>"Loading users..."</span> }>
                    {move || users.get().map(|list| view! {
                        <select on:change=move |ev| set_assignee_filter.set(event_target_value(&ev))>
                            <option value="">"Assignee"</option>
                            <For each=move || list.clone() key=|u| u.id children=move |u| {
                                view! { <option value={u.username.clone()}>{u.username.clone()}</option> }
                            }/>
                        </select>
                    })}
                </Suspense>

                <select on:change=move |ev| set_sort.set(event_target_value(&ev))>
                    <option value="created">"Created"</option>
                    <option value="updated">"Updated"</option>
                    <option value="comments">"Comments"</option>
                </select>

                <button on:click=move |_| set_direction.update(|d| *d = if d == "asc" { "desc".to_string() } else { "asc".to_string() })>
                    {move || if direction.get() == "asc" { "Asc" } else { "Desc" }}
                </button>
            </div>
            <ul>
                <Suspense fallback=move || view! { <li>"Loading issues..."</li> }>
                    {move || issues.get().map(|list| view! {
                        <For each=move || list.clone() key=|i| i.id children=move |i| {
                            let href = format!("/repos/{}/{}/issues/{}", owner(), repo_name(), i.id);
                            view! { <li><a href=href>"#" {i.number} " " {i.title}</a> " (" {i.state} ")"</li> }
                        }/>
                    })}
                </Suspense>
            </ul>
            <div class="pagination mt-1">
                <button on:click=move |_| set_page.update(|p| if *p> 1 { *p -= 1 }) disabled=move || page.get() <= 1>"Previous"</button>
                <span class="mx-2">"Page " {page}</span>
                <button on:click=move |_| set_page.update(|p| *p += 1)>"Next"</button>
            </div>
        </div>
    }
}

#[component]
pub fn IssueDetail() -> impl IntoView {
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

    let (new_comment, set_new_comment) = signal("".to_string());
    let (trigger_refresh, set_trigger_refresh) = signal(0);
    let (editing_comment_id, set_editing_comment_id) = signal(None::<u64>);
    let (edit_comment_body, set_edit_comment_body) = signal("".to_string());
    let (action_error, set_action_error) = signal(Option::<String>::None);

    let issue = local_resource(
        move || (owner(), repo_name(), index(), trigger_refresh.get()),
        |(o, r, i, _)| async move {
            get_or::<Option<Issue>>(&format!("/api/v1/repos/{}/{}/issues/{}", o, r, i), None).await
        },
    );

    let comments = local_resource(
        move || (owner(), repo_name(), index(), trigger_refresh.get()),
        |(o, r, i, _)| async move {
            get::<Vec<Comment>>(&format!("/api/v1/repos/{}/{}/issues/{}/comments", o, r, i)).await
        },
    );

    let available_milestones = local_resource(
        move || (owner(), repo_name()),
        |(o, r)| async move {
            get::<Vec<Milestone>>(&format!("/api/v1/repos/{}/{}/milestones", o, r)).await
        },
    );

    let available_users = local_resource(
        || (),
        |_| async move {
            // Mock users for assignment - in real app, fetch from collaborators or org members
            vec![
                shared::User::new(1, "admin".to_string(), None),
                shared::User::new(2, "user".to_string(), None),
            ]
        },
    );

    let on_lock = move |_| {
        let o = owner();
        let r = repo_name();
        let i = index();
        spawn_local(async move {
            if put(&format!("/api/v1/repos/{}/{}/issues/{}/lock", o, r, i)).await {
                set_action_error.set(None);
                set_trigger_refresh.update(|n| *n += 1);
            } else {
                set_action_error.set(Some(crate::api::WRITE_ERROR.to_string()));
            }
        });
    };

    let on_unlock = move |_| {
        let o = owner();
        let r = repo_name();
        let i = index();
        spawn_local(async move {
            if delete(&format!("/api/v1/repos/{}/{}/issues/{}/lock", o, r, i)).await {
                set_action_error.set(None);
                set_trigger_refresh.update(|n| *n += 1);
            } else {
                set_action_error.set(Some(crate::api::WRITE_ERROR.to_string()));
            }
        });
    };

    let on_submit_comment = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let payload = CreateCommentOption {
            body: new_comment.get(),
        };
        let o = owner();
        let r = repo_name();
        let i = index();

        spawn_local(async move {
            // Keep the drafted comment if the server rejects it.
            if post_json(
                &format!("/api/v1/repos/{}/{}/issues/{}/comments", o, r, i),
                &payload,
            )
            .await
            {
                set_new_comment.set("".to_string());
                set_action_error.set(None);
                set_trigger_refresh.update(|n| *n += 1);
            } else {
                set_action_error.set(Some(crate::api::WRITE_ERROR.to_string()));
            }
        });
    };

    let on_delete_comment = move |comment_id: u64| {
        let o = owner();
        let r = repo_name();
        spawn_local(async move {
            if delete(&format!(
                "/api/v1/repos/{}/{}/issues/comments/{}",
                o, r, comment_id
            ))
            .await
            {
                set_action_error.set(None);
                set_trigger_refresh.update(|n| *n += 1);
            } else {
                set_action_error.set(Some(crate::api::WRITE_ERROR.to_string()));
            }
        });
    };

    let on_add_reaction = move |comment_id: u64, content: String| {
        let o = owner();
        let r = repo_name();
        let payload = shared::CreateReactionOption { content };
        spawn_local(async move {
            if post_json(
                &format!(
                    "/api/v1/repos/{}/{}/issues/comments/{}/reactions",
                    o, r, comment_id
                ),
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

    let on_start_edit_comment = move |comment_id: u64, current_body: String| {
        set_editing_comment_id.set(Some(comment_id));
        set_edit_comment_body.set(current_body);
    };

    let on_cancel_edit_comment = move |_| {
        set_editing_comment_id.set(None);
        set_edit_comment_body.set("".to_string());
    };

    let on_save_edit_comment = move |comment_id: u64| {
        let o = owner();
        let r = repo_name();
        let body = edit_comment_body.get();
        let payload = shared::UpdateCommentOption { body };

        spawn_local(async move {
            // Close the editor only when the edit was persisted, so a rejected
            // edit does not throw away the user's text.
            if patch_json(
                &format!("/api/v1/repos/{}/{}/issues/comments/{}", o, r, comment_id),
                &payload,
            )
            .await
            {
                set_editing_comment_id.set(None);
                set_edit_comment_body.set("".to_string());
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
        let payload = UpdateIssueOption {
            title: None,
            body: None,
            state: Some(new_state.to_string()),
            milestone_id: None,
        };
        spawn_local(async move {
            if patch_json(
                &format!("/api/v1/repos/{}/{}/issues/{}", o, r, idx),
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

    // Label management
    let (new_label_name, set_new_label_name) = signal("".to_string());
    let on_add_label = move |_| {
        let o = owner();
        let r = repo_name();
        let i = index();
        let name = new_label_name.get();
        if !name.is_empty() {
            spawn_local(async move {
                let payload = CreateLabelOption {
                    name,
                    color: "#cccccc".to_string(),
                    description: None,
                };
                if post_json(
                    &format!("/api/v1/repos/{}/{}/issues/{}/labels", o, r, i),
                    &payload,
                )
                .await
                {
                    set_new_label_name.set("".to_string());
                    set_action_error.set(None);
                    set_trigger_refresh.update(|n| *n += 1);
                } else {
                    set_action_error.set(Some(crate::api::WRITE_ERROR.to_string()));
                }
            });
        }
    };

    let on_remove_label = move |label_id: u64| {
        let o = owner();
        let r = repo_name();
        let i = index();
        spawn_local(async move {
            if delete(&format!(
                "/api/v1/repos/{}/{}/issues/{}/labels/{}",
                o, r, i, label_id
            ))
            .await
            {
                set_action_error.set(None);
                set_trigger_refresh.update(|n| *n += 1);
            } else {
                set_action_error.set(Some(crate::api::WRITE_ERROR.to_string()));
            }
        });
    };

    // Assignee management
    let (selected_assignee, set_selected_assignee) = signal("".to_string());
    let on_add_assignee = move |_| {
        let username = selected_assignee.get();
        if !username.is_empty() {
            let o = owner();
            let r = repo_name();
            let i = index();
            // In a real app, we'd need the full user object or ID, but the backend accepts a User struct
            // We'll construct a minimal one for the payload
            let payload = shared::User::new(0, username, None);
            spawn_local(async move {
                if post_json(
                    &format!("/api/v1/repos/{}/{}/issues/{}/assignees", o, r, i),
                    &payload,
                )
                .await
                {
                    set_selected_assignee.set("".to_string());
                    set_action_error.set(None);
                    set_trigger_refresh.update(|n| *n += 1);
                } else {
                    set_action_error.set(Some(crate::api::WRITE_ERROR.to_string()));
                }
            });
        }
    };

    let on_remove_assignee = move |username: String| {
        let o = owner();
        let r = repo_name();
        let i = index();
        spawn_local(async move {
            if delete(&format!(
                "/api/v1/repos/{}/{}/issues/{}/assignees/{}",
                o, r, i, username
            ))
            .await
            {
                set_action_error.set(None);
                set_trigger_refresh.update(|n| *n += 1);
            } else {
                set_action_error.set(Some(crate::api::WRITE_ERROR.to_string()));
            }
        });
    };

    let on_change_milestone = move |ev: leptos::ev::Event| {
        let val_str = event_target_value(&ev);
        let m_id = val_str.parse::<u64>().unwrap_or(0);
        let o = owner();
        let r = repo_name();
        let idx = index();

        let payload = UpdateIssueOption {
            title: None,
            body: None,
            state: None,
            milestone_id: Some(m_id),
        };
        spawn_local(async move {
            if patch_json(
                &format!("/api/v1/repos/{}/{}/issues/{}", o, r, idx),
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
        let payload = UpdateIssueOption {
            title: Some(edit_title.get()),
            body: Some(edit_body.get()),
            state: None,
            milestone_id: None,
        };
        spawn_local(async move {
            if patch_json(
                &format!("/api/v1/repos/{}/{}/issues/{}", o, r, idx),
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

    view! {
            <div class="issue-detail">
            <RepoNav/>
                <Suspense fallback=move || view! { <p>"Loading issue..."</p> }>
                    {move || match issue.get() {
                        Some(Some(i)) => {
                            let state_clone = i.state.clone();
                            let state_for_toggle = state_clone.clone();
                            let title_clone = i.title.clone();
                            let body_clone = i.body.clone().unwrap_or_default();
                            let milestone_clone = i.milestone.clone();
                            view! {
                                <div class="issue-header">
                                    {if is_editing.get() {
                                        view! {
                                            <input type="text" prop:value=edit_title on:input=move |ev| set_edit_title.set(event_target_value(&ev)) class="input-title" />
                                        }.into_any()
                                    } else {
                                        view! { <h2>{i.title.clone()} " #" {i.number}</h2> }.into_any()
                                    }}
                                    <span class="state">{i.state.clone()}</span>
                                    <span class="meta">" opened by " {i.user.username}</span>
                                    <button on:click=move |_| on_toggle_state(state_for_toggle.clone()) class="ml-2">
                                        {if state_clone == "open" { "Close Issue" } else { "Reopen Issue" }}
                                    </button>
                                    {if !is_editing.get() {
                                        view! { <button on:click=move |_| on_start_edit(title_clone.clone(), body_clone.clone()) class="ml-1">"Edit"</button> }.into_any()
                                    } else {
                                         view! { <span></span> }.into_any()
                                    }}
                                    {if i.is_locked {
                                        view! { <button on:click=on_unlock class="ml-1 text-danger">"Unlock Conversation"</button> }.into_any()
                                    } else {
                                        view! { <button on:click=on_lock class="ml-1 text-danger">"Lock Conversation"</button> }.into_any()
                                    }}
                                </div>
                                <div class="issue-container flex">
                                    <div class="issue-main">
                                        <div class="issue-body">
                                            {if is_editing.get() {
                                                view! {
                                                    <div>
                                                        <textarea prop:value=edit_body on:input=move |ev| set_edit_body.set(event_target_value(&ev)) rows="10" class="w-100"></textarea>
                                                        <button on:click=on_save_edit>"Save"</button>
                                                        <button on:click=on_cancel_edit class="ml-1">"Cancel"</button>
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! { <p>{i.body.clone().unwrap_or_default()}</p> }.into_any()
                                            }}
                                        </div>
                                    </div>
                                    <div class="issue-sidebar">
                                    <div class="sidebar-item">
                                        <strong>"Assignees"</strong>
                                        <div class="assignees-list">
                                            <For each=move || i.assignees.clone() key=|u| u.id children=move |u| {
                                                let username = u.username.clone();
                                                view! {
                                                    <div class="flex-between mb-0">
                                                        <span>{username.clone()}</span>
                                                        <button on:click=move |_| on_remove_assignee(username.clone()) class="text-small ml-1 pointer">"x"</button>
                                                    </div>
                                                }
                                            }/>
                                        </div>
                                        <div class="add-assignee">
                                            <Suspense fallback=move || view! { <span>"Loading users..."</span> }>
                                                {move || available_users.get().map(|users| view! {
                                                    <select on:change=move |ev| set_selected_assignee.set(event_target_value(&ev))>
                                                        <option value="">"Add Assignee"</option>
                                                        <For each=move || users.clone() key=|u| u.id children=move |u| {
                                                            let username = u.username.clone();
                                                            view! { <option value={username.clone()}>{username.clone()}</option> }
                                                        }/>
                                                    </select>
                                                    <button on:click=on_add_assignee>"+"</button>
                                                })}
                                            </Suspense>
                                        </div>
                                    </div>
                                    <div class="sidebar-item">
                                        <strong>"Labels"</strong>
                                        <div class="labels-list">
                                            <For each=move || i.labels.clone() key=|l| l.id children=move |l| {
                                                let label_id = l.id;
                                                view! {
                                                    <div style=format!("background-color: {}; color: #fff; padding: 2px 5px; border-radius: 3px; display: inline-block; margin-right: 5px; margin-bottom: 2px;", l.color)>
                                                        {l.name}
                                                        <span on:click=move |_| on_remove_label(label_id) class="ml-1 bold pointer">"x"</span>
                                                    </div>
                                                }
                                            }/>
                                        </div>
                                        <div class="add-label">
                                            <input type="text" prop:value=new_label_name on:input=move |ev| set_new_label_name.set(event_target_value(&ev)) placeholder="New Label" class="input-narrow"/>
                                            <button on:click=on_add_label>"+"</button>
                                        </div>
                                    </div>
                                    <div class="sidebar-item">
                                        <strong>"Milestone"</strong>
                                        <div>
                                            <Suspense fallback=move || view! { <span>"Loading..."</span> }>
                                                {
                                                    let milestone_clone_2 = milestone_clone.clone();
                                                    move || available_milestones.get().map(|list| {
                                                        let current_id = milestone_clone_2.as_ref().map(|m| m.id).unwrap_or(0);
                                                        view! {
                                                            <select on:change=on_change_milestone>
                                                                <option value="0" selected={current_id == 0}>"No Milestone"</option>
                                                                <For each=move || list.clone() key=|m| m.id children=move |m| {
                                                                    let selected = m.id == current_id;
                                                                    view! { <option value={m.id} selected={selected}>{m.title}</option> }
                                                                }/>
                                                            </select>
                                                        }
                                                    })
                                                }
                                            </Suspense>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        }.into_any() },
                        _ => view! { <p>"Issue not found"</p> }.into_any()
                    }}
                </Suspense>

                <div class="comments-section">
                    <h3>"Comments"</h3>
                    <Suspense fallback=move || view! { <p>"Loading comments..."</p> }>
                        {move || comments.get().map(|list| view! {
                            <For each=move || list.clone() key=|c| c.id children=move |c| {
                                let comment_id = c.id;
                                let comment_body = c.body.clone();
                                view! {
                                    <div class="comment">
                                        <div class="comment-header">
                                            <strong>{c.user.username}</strong> " commented on " {c.created_at}
                                            {
                                                // Mock admin check: allow edit/delete for everyone in mock
                                                view! {
                                                    <span class="float-right">
                                                        <button on:click=move |_| on_start_edit_comment(comment_id, comment_body.clone())>"Edit"</button>
                                                        <button on:click=move |_| on_delete_comment(comment_id)>"Delete"</button>
                                                    </span>
                                                }
                                            }
                                        </div>
                                        <div class="comment-body">
                                            {move || if editing_comment_id.get() == Some(comment_id) {
                                                view! {
                                                    <div>
                                                        <textarea prop:value=edit_comment_body on:input=move |ev| set_edit_comment_body.set(event_target_value(&ev))></textarea>
                                                        <button on:click=move |_| on_save_edit_comment(comment_id)>"Save"</button>
                                                        <button on:click=on_cancel_edit_comment>"Cancel"</button>
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! { <p>{c.body.clone()}</p> }.into_any()
                                            }}
                                        </div>
                                        <div class="comment-reactions">
                                            <div class="reactions-list">
                                                <For each=move || c.reactions.clone() key=|r| r.id children=move |r| {
                                                    view! { <span title={r.user.username} class="reaction-pill">{r.content}</span> }
                                                }/>
                                            </div>
                                            <div class="reaction-picker">
                                                <button on:click=move |_| on_add_reaction(comment_id, "👍".to_string()) title="+1">"👍"</button>
                                                <button on:click=move |_| on_add_reaction(comment_id, "👎".to_string()) title="-1">"👎"</button>
                                                <button on:click=move |_| on_add_reaction(comment_id, "😄".to_string()) title="laugh">"😄"</button>
                                                <button on:click=move |_| on_add_reaction(comment_id, "😕".to_string()) title="confused">"😕"</button>
                                                <button on:click=move |_| on_add_reaction(comment_id, "❤️".to_string()) title="heart">"❤️"</button>
                                                <button on:click=move |_| on_add_reaction(comment_id, "🎉".to_string()) title="hooray">"🎉"</button>
                                                <button on:click=move |_| on_add_reaction(comment_id, "👀".to_string()) title="eyes">"👀"</button>
                                                <button on:click=move |_| on_add_reaction(comment_id, "🚀".to_string()) title="rocket">"🚀"</button>
                                            </div>
                                        </div>
                                    </div>
                                }
                            }/>
                        })}
                    </Suspense>

                    {
                        let issue_for_form = issue;
                        move || {
                            match issue_for_form.get() {
                                Some(Some(i)) if i.is_locked => {
                                    view! { <p class="text-danger">"This conversation is locked"</p> }.into_any()
                                },
                                _ => {
                                    view! {
                                        <form on:submit=on_submit_comment class="comment-form">
                                            <textarea
                                                prop:value=new_comment
                                                on:input=move |ev| set_new_comment.set(event_target_value(&ev))
                                                placeholder="Leave a comment"
    ></textarea>
                                            <button type="submit">"Comment"</button>
                                            {move || action_error.get().map(|msg| view! {
                                                <p class="form-error" role="alert">{msg}</p>
                                            })}
                                        </form>
                                    }.into_any()
                                }
                            }
                        }
                    }
                </div>
            </div>
        }
}

#[component]
pub fn LabelList() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|params| params.get("owner").unwrap_or_default());
    let repo_name = move || params.with(|params| params.get("repo").unwrap_or_default());

    let (name, set_name) = signal("".to_string());
    let (form_error, set_form_error) = signal(Option::<String>::None);
    let (color, set_color) = signal("#000000".to_string());
    let (refresh, set_refresh) = signal(0u32);

    let labels = local_resource(
        move || (owner(), repo_name(), refresh.get()),
        |(o, r, _)| async move { get::<Vec<Label>>(&format!("/api/v1/repos/{}/{}/labels", o, r)).await },
    );

    let on_create = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let payload = CreateLabelOption {
            name: name.get(),
            color: color.get(),
            description: None,
        };
        let o = owner();
        let r = repo_name();
        spawn_local(async move {
            if post_json(&format!("/api/v1/repos/{}/{}/labels", o, r), &payload).await {
                set_name.set("".to_string());
                set_form_error.set(None);
                set_refresh.update(|n| *n += 1);
            } else {
                set_form_error.set(Some(crate::api::WRITE_ERROR.to_string()));
            }
        });
    };

    view! {
        <div class="label-list">
        <RepoNav/>
            <h3>"Labels"</h3>
            <ul>
                <Suspense fallback=move || view! { <li>"Loading..."</li> }>
                    {move || labels.get().map(|list| view! {
                        <For each=move || list.clone() key=|l| l.id children=move |l| {
                            view! {
                                <li style=format!("border-left: 5px solid {}", l.color)>
                                    {l.name}
                                </li>
                            }
                        }/>
                    })}
                </Suspense>
            </ul>
            <h4>"Create Label"</h4>
            <form on:submit=on_create>
                <input type="text" placeholder="Name" prop:value=name on:input=move |ev| set_name.set(event_target_value(&ev)) />
                <input type="color" prop:value=color on:input=move |ev| set_color.set(event_target_value(&ev)) />
                <button type="submit">"Create"</button>
                {move || form_error.get().map(|msg| view! {
                    <p class="form-error" role="alert">{msg}</p>
                })}
            </form>
        </div>
    }
}

#[component]
pub fn MilestoneList() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|params| params.get("owner").unwrap_or_default());
    let repo_name = move || params.with(|params| params.get("repo").unwrap_or_default());

    let (title, set_title) = signal("".to_string());
    let (form_error, set_form_error) = signal(Option::<String>::None);
    let (refresh, set_refresh) = signal(0u32);

    let milestones = local_resource(
        move || (owner(), repo_name(), refresh.get()),
        |(o, r, _)| async move {
            get::<Vec<Milestone>>(&format!("/api/v1/repos/{}/{}/milestones", o, r)).await
        },
    );

    let on_create = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let payload = CreateMilestoneOption {
            title: title.get(),
            description: None,
            due_on: None,
        };
        let o = owner();
        let r = repo_name();
        spawn_local(async move {
            if post_json(&format!("/api/v1/repos/{}/{}/milestones", o, r), &payload).await {
                set_title.set("".to_string());
                set_form_error.set(None);
                set_refresh.update(|n| *n += 1);
            } else {
                set_form_error.set(Some(crate::api::WRITE_ERROR.to_string()));
            }
        });
    };

    view! {
        <div class="milestone-list">
        <RepoNav/>
            <h3>"Milestones"</h3>
             <ul>
                <Suspense fallback=move || view! { <li>"Loading..."</li> }>
                    {move || milestones.get().map(|list| view! {
                        <For each=move || list.clone() key=|m| m.id children=move |m| {
                             let href = format!("/repos/{}/{}/milestones/{}", owner(), repo_name(), m.id);
                            view! {
                                <li>
                                    <a href=href>{m.title}</a> " (" {m.state} ")"
                                </li>
                            }
                        }/>
                    })}
                </Suspense>
            </ul>
             <h4>"Create Milestone"</h4>
            <form on:submit=on_create>
                <input type="text" placeholder="Title" prop:value=title on:input=move |ev| set_title.set(event_target_value(&ev)) />
                <button type="submit">"Create"</button>
                {move || form_error.get().map(|msg| view! {
                    <p class="form-error" role="alert">{msg}</p>
                })}
            </form>
        </div>
    }
}

#[component]
pub fn MilestoneDetail() -> impl IntoView {
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

    let milestone = local_resource(
        move || (owner(), repo_name(), index()),
        |(o, r, i)| async move {
            get_or::<Option<Milestone>>(
                &format!("/api/v1/repos/{}/{}/milestones/{}", o, r, i),
                None,
            )
            .await
        },
    );

    let stats = local_resource(
        move || (owner(), repo_name(), index()),
        |(o, r, i)| async move {
            get_or::<MilestoneStats>(
                &format!("/api/v1/repos/{}/{}/milestones/{}/stats", o, r, i),
                MilestoneStats {
                    open_issues: 0,
                    closed_issues: 0,
                },
            )
            .await
        },
    );

    view! {
        <div class="milestone-detail">
        <RepoNav/>
             <Suspense fallback=move || view! { <p>"Loading..."</p> }>
                {move || match milestone.get() {
                    Some(Some(m)) => view! {
                        <h3>"Milestone: " {m.title}</h3>
                        <p>{m.description.unwrap_or_default()}</p>
                        <p>"State: " {m.state}</p>
                    }.into_any(),
                    _ => view! { <p>"Milestone not found"</p> }.into_any()
                }}
            </Suspense>
             <Suspense fallback=move || view! { <p>"Loading stats..."</p> }>
                {move || stats.get().map(|s| {
                    let total = s.open_issues + s.closed_issues;
                    let percentage = if total> 0 { (s.closed_issues as f64 / total as f64 * 100.0) as u64 } else { 0 };

                    view! {
                    <div class="stats">
                        <div class="progress-bar">
                            <div style=format!("width: {}%; height: 100%; background: #2cbe4e;", percentage)></div>
                        </div>
                        <div>
                            <strong>{percentage} "% complete"</strong>
                            <span class="ml-2">{s.open_issues} " Open"</span>
                            <span class="ml-2">{s.closed_issues} " Closed"</span>
                        </div>
                    </div>
                }})}
            </Suspense>
        </div>
    }
}
