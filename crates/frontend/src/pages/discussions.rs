use crate::api::{api_url, get, get_or, local_resource, post_json, WRITE_ERROR};
use crate::components::RepoNav;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::*;
use shared::{
    CreateDiscussionCommentOption, CreateDiscussionOption, Discussion, DiscussionComment,
};

#[component]
pub fn DiscussionList() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|params| params.get("owner").unwrap_or_default());
    let repo_name = move || params.with(|params| params.get("repo").unwrap_or_default());

    let (show_create, set_show_create) = signal(false);
    let (refresh, set_refresh) = signal(0);
    let (new_title, set_new_title) = signal("".to_string());
    let (new_body, set_new_body) = signal("".to_string());
    let (new_category, set_new_category) = signal("General".to_string());
    let (form_error, set_form_error) = signal(Option::<String>::None);

    let discussions = local_resource(
        move || (owner(), repo_name(), refresh.get()),
        |(o, r, _)| async move {
            let url = api_url(&format!("/repos/{}/{}/discussions", o, r));
            get::<Vec<Discussion>>(&url).await
        },
    );

    let on_create = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let payload = CreateDiscussionOption {
            title: new_title.get(),
            body: new_body.get(),
            category: new_category.get(),
        };
        let o = owner();
        let r = repo_name();
        spawn_local(async move {
            let url = api_url(&format!("/repos/{}/{}/discussions", o, r));
            if post_json(&url, &payload).await {
                set_new_title.set("".to_string());
                set_new_body.set("".to_string());
                set_show_create.set(false);
                set_form_error.set(None);
                set_refresh.update(|n| *n += 1);
            } else {
                set_form_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    view! {
        <div class="discussion-list">
        <RepoNav/>
            <div class="header flex-between">
                <h3>"Discussions"</h3>
                <button on:click=move |_| set_show_create.set(!show_create.get())>
                    {move || if show_create.get() { "Cancel" } else { "New Discussion" }}
                </button>
            </div>

            {move || if show_create.get() {
                view! {
                    <form on:submit=on_create class="panel mb-2">
                        <input type="text" placeholder="Title" prop:value=new_title on:input=move |ev| set_new_title.set(event_target_value(&ev))  required />
                        <textarea placeholder="Body" prop:value=new_body on:input=move |ev| set_new_body.set(event_target_value(&ev))  rows="5"></textarea>
                        <select on:change=move |ev| set_new_category.set(event_target_value(&ev))>
                            <option value="General">"General"</option>
                            <option value="Ideas">"Ideas"</option>
                            <option value="Q&A">"Q&A"</option>
                            <option value="Show and Tell">"Show and Tell"</option>
                        </select>
                        <button type="submit">"Start Discussion"</button>
                        {move || form_error.get().map(|msg| view! {
                            <p class="form-error" role="alert">{msg}</p>
                        })}
                    </form>
                }.into_any()
            } else {
                view! { <span></span> }.into_any()
            }}

            <ul>
                <Suspense fallback=move || view! { <li>"Loading discussions..."</li> }>
                    {move || discussions.get().map(|list| {
                        if list.is_empty() {
                            view! { <li>"No discussions found."</li> }.into_any()
                        } else {
                            view! {
                                <For each=move || list.clone() key=|d| d.id children=move |d| {
                                    let href = format!("/repos/{}/{}/discussions/{}", owner(), repo_name(), d.id);
                                    view! {
                                        <li class="boxed">
                                            <div class="text-small text-muted">{d.category}</div>
                                            <a href=href class="text-lg bold">{d.title}</a>
                                            <p class="my-1">{d.body}</p>
                                            <div class="text-small">"by " {d.user.username}</div>
                                        </li>
                                    }
                                }/>
                            }.into_any()
                        }
                    })}
                </Suspense>
            </ul>
        </div>
    }
}

#[component]
pub fn DiscussionDetail() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|params| params.get("owner").unwrap_or_default());
    let repo_name = move || params.with(|params| params.get("repo").unwrap_or_default());
    let id = move || {
        params.with(|params| {
            params
                .get("id")
                .unwrap_or_default()
                .parse::<u64>()
                .unwrap_or_default()
        })
    };

    let (refresh_comments, set_refresh_comments) = signal(0);
    let (new_comment_body, set_new_comment_body) = signal("".to_string());
    let (comment_error, set_comment_error) = signal(Option::<String>::None);

    let discussion = local_resource(
        move || (owner(), repo_name(), id()),
        |(o, r, i)| async move {
            let url = api_url(&format!("/repos/{}/{}/discussions/{}", o, r, i));
            get_or::<Option<Discussion>>(&url, None).await
        },
    );

    let comments = local_resource(
        move || (owner(), repo_name(), id(), refresh_comments.get()),
        |(o, r, i, _)| async move {
            let url = api_url(&format!("/repos/{}/{}/discussions/{}/comments", o, r, i));
            get::<Vec<DiscussionComment>>(&url).await
        },
    );

    let on_submit_comment = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let payload = CreateDiscussionCommentOption {
            body: new_comment_body.get(),
        };
        let o = owner();
        let r = repo_name();
        let i = id();

        spawn_local(async move {
            if post_json(
                &api_url(&format!("/repos/{}/{}/discussions/{}/comments", o, r, i)),
                &payload,
            )
            .await
            {
                set_new_comment_body.set("".to_string());
                set_comment_error.set(None);
                set_refresh_comments.update(|n| *n += 1);
            } else {
                set_comment_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    view! {
            <div class="discussion-detail">
            <RepoNav/>
                <Suspense fallback=move || view! { <h3>"Loading Discussion..."</h3> }>
                    {move || discussion.get().map(|d| match d {
                        Some(disc) => {
                            view! {
                                <div class="discussion-header">
                                    <span class="badge badge-neutral">{disc.category}</span>
                                    <h1>{disc.title}</h1>
                                    <div class="meta">
                                        "Started by " <strong>{disc.user.username}</strong> " on " {disc.created_at}
                                    </div>
                                </div>
                                <div class="discussion-body">
                                    <p>{disc.body}</p>
                                </div>

                                <div class="discussion-comments">
                                    <h3>"Comments"</h3>
                                    <ul class="list-reset">
                                        <Suspense fallback=move || view! { <li>"Loading comments..."</li> }>
                                            {move || comments.get().map(|list| {
                                                view! {
                                                    <For each=move || list.clone() key=|c| c.id children=move |c| {
                                                        view! {
                                                            <li class="divider-top mb-2">
                                                                <div class="text-small bold">{c.user.username} " commented:"</div>
                                                                <p class="my-1">{c.body}</p>
                                                                <div class="text-small text-muted">{c.created_at}</div>
                                                            </li>
                                                        }
                                                    }/>
                                                }
                                            })}
                                        </Suspense>
                                    </ul>

                                    <form on:submit=on_submit_comment class="divider-top mt-4">
                                        <h4>"Add a Comment"</h4>
                                        <textarea
                                            prop:value=new_comment_body
                                            on:input=move |ev| set_new_comment_body.set(event_target_value(&ev))
                                            class="textarea-tall"
                                            placeholder="Write your comment here..."
                                            required
    ></textarea>
                                        <button type="submit">"Post Comment"</button>
                                        {move || comment_error.get().map(|msg| view! {
                                            <p class="form-error" role="alert">{msg}</p>
                                        })}
                                    </form>
                                </div>
                            }.into_any()
                        },
                        None => view! { <h3>"Discussion Not Found"</h3> }.into_any()
                    })}
                </Suspense>
            </div>
        }
}
