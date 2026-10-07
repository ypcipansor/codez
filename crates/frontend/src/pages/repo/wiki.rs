//! Repository wiki pages.

use crate::api::{get, get_opt, get_or, local_resource, post_json, put_json, WRITE_ERROR};
use crate::components::RepoNav;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::*;
use shared::{CreateWikiPageOption, WikiPage};

#[component]
pub fn Wiki() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|params| params.get("owner").unwrap_or_default());
    let repo_name = move || params.with(|params| params.get("repo").unwrap_or_default());
    let page_name =
        move || params.with(|params| params.get("page_name").unwrap_or("Home".to_string()));

    let wiki_page = local_resource(
        move || (owner(), repo_name(), page_name()),
        move |(o, r, p)| async move {
            get_or::<Option<WikiPage>>(&format!("/api/v1/repos/{}/{}/wiki/pages/{}", o, r, p), None)
                .await
        },
    );

    let wiki_pages = local_resource(
        move || (owner(), repo_name()),
        |(o, r)| async move {
            get::<Vec<WikiPage>>(&format!("/api/v1/repos/{}/{}/wiki/pages", o, r)).await
        },
    );

    view! {
        <div class="wiki-page">
            <RepoNav/>
            <div class="wiki-container flex">
                <div class="wiki-sidebar">
                    <h4>"Pages"</h4>
                    <ul>
                        <Suspense fallback=move || view! { <li>"Loading..."</li> }>
                            {move || wiki_pages.get().map(|list| view! {
                                <For each=move || list.clone() key=|p| p.title.clone() children=move |p| {
                                    let href = format!("/repos/{}/{}/wiki/pages/{}", owner(), repo_name(), p.title);
                                    view! { <li><a href=href>{p.title}</a></li> }
                                }/>
                            })}
                        </Suspense>
                    </ul>
                </div>
                <div class="wiki-view">
                    <Suspense fallback=move || view! { <p>"Loading wiki..."</p> }>
                        {move || match wiki_page.get() {
                            Some(Some(page)) => {
                                let title = page.title.clone();
                                view! {
                                    <div class="wiki-header">
                                        <h3>{page.title}</h3>
                                        <a href=format!("/repos/{}/{}/wiki/pages/{}/edit", owner(), repo_name(), title) class="btn">"Edit"</a>
                                    </div>
                                    <div class="wiki-content">
                                        <pre>{page.content}</pre>
                                    </div>
                                }.into_any()
                            },
                            _ => {
                                let p = page_name();
                                view! {
                                    <div>
                                        <p>"Wiki page '" {p.clone()} "' not found."</p>
                                        <a href=format!("/repos/{}/{}/wiki/pages/{}/edit", owner(), repo_name(), p.clone())>
                                            "Create " {p.clone()} " Page"
                                        </a>
                                    </div>
                                }.into_any()
                            }
                        }}
                    </Suspense>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn WikiEdit() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|params| params.get("owner").unwrap_or_default());
    let repo_name = move || params.with(|params| params.get("repo").unwrap_or_default());
    let page_name =
        move || params.with(|params| params.get("page_name").unwrap_or("Home".to_string()));

    let (content, set_content) = signal("".to_string());
    let (message, set_message) = signal("".to_string());
    let (is_new, set_is_new) = signal(true);
    let (form_error, set_form_error) = signal(Option::<String>::None);

    // Load existing content if available
    let _ = local_resource(
        move || (owner(), repo_name(), page_name()),
        move |(o, r, p)| async move {
            if let Some(Some(page)) =
                get_opt::<Option<WikiPage>>(&format!("/api/v1/repos/{}/{}/wiki/pages/{}", o, r, p))
                    .await
            {
                set_content.set(page.content);
                set_is_new.set(false);
            }
        },
    );

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let payload = CreateWikiPageOption {
            title: page_name(),
            content: content.get(),
            message: Some(message.get()),
        };
        let o = owner();
        let r = repo_name();
        let p = page_name();
        let is_n = is_new.get();
        spawn_local(async move {
            // The page body is the user's only copy of their edit, so it stays put
            // unless the server confirms the save.
            let saved = if is_n {
                post_json(&format!("/api/v1/repos/{}/{}/wiki/pages", o, r), &payload).await
            } else {
                put_json(
                    &format!("/api/v1/repos/{}/{}/wiki/pages/{}", o, r, p),
                    &payload,
                )
                .await
            };
            if saved {
                set_is_new.set(false);
                set_form_error.set(None);
            } else {
                set_form_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    view! {
        <div class="wiki-edit">
        <RepoNav/>
            <h3>"Editing " {page_name}</h3>
            <form on:submit=on_submit>
                <textarea prop:value=content on:input=move |ev| set_content.set(event_target_value(&ev)) rows="10"></textarea>
                <input type="text" placeholder="Commit Message" prop:value=message on:input=move |ev| set_message.set(event_target_value(&ev)) />
                <button type="submit">"Save Page"</button>
                {move || form_error.get().map(|msg| view! {
                    <p class="form-error" role="alert">{msg}</p>
                })}
            </form>
        </div>
    }
}
