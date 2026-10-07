use crate::api::{delete, get, get_or, local_resource, post, post_json, WRITE_ERROR};
use crate::components::RepoNav;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::*;
use shared::{CreateReleaseOption, Release};

#[component]
pub fn ReleaseList() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|params| params.get("owner").unwrap_or_default());
    let repo_name = move || params.with(|params| params.get("repo").unwrap_or_default());

    let releases = local_resource(
        move || (owner(), repo_name()),
        |(o, r)| async move {
            get::<Vec<Release>>(&format!("/api/v1/repos/{}/{}/releases", o, r)).await
        },
    );

    view! {
        <div class="release-list">
        <RepoNav/>
            <div class="flex-between">
                <h3>"Releases for " {owner} "/" {repo_name}</h3>
                <a href=format!("/repos/{}/{}/releases/new", owner(), repo_name()) class="btn">"Draft a new release"</a>
            </div>
            <ul>
                <Suspense fallback=move || view! { <li>"Loading releases..."</li> }>
                    {move || releases.get().map(|list| view! {
                        <For each=move || list.clone() key=|r| r.id children=move |r| {
                            let href = format!("/repos/{}/{}/releases/{}", owner(), repo_name(), r.id);
                            view! {
                                <li class="list-row">
                                    <div class="flex-between">
                                        <div>
                                            <h4><a href=href>{r.name}</a></h4>
                                            <span class="badge badge-success">{r.tag_name}</span>
                                            {if r.draft { view! { <span class="badge badge-neutral ml-1">"Draft"</span> }.into_any() } else { view! { <span></span> }.into_any() }}
                                            {if r.prerelease { view! { <span class="badge badge-warning ml-1">"Pre-release"</span> }.into_any() } else { view! { <span></span> }.into_any() }}
                                        </div>
                                        <div class="text-small text-muted">
                                            {r.created_at}
                                        </div>
                                    </div>
                                    <p>{r.body.unwrap_or_default()}</p>
                                    <div class="assets mt-1">
                                         <For each=move || r.assets.clone() key=|a| a.id children=move |a| {
                                            view! {
                                                <div class="text-small">
                                                    <a href=a.download_url>"📦 " {a.name}</a> " (" {a.size} " bytes)"
                                                </div>
                                            }
                                         }/>
                                    </div>
                                </li>
                            }
                        }/>
                    })}
                </Suspense>
            </ul>
        </div>
    }
}

#[component]
pub fn ReleaseDetail() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|params| params.get("owner").unwrap_or_default());
    let repo_name = move || params.with(|params| params.get("repo").unwrap_or_default());
    let id = move || params.with(|params| params.get("id").unwrap_or_default());

    // Trigger to refresh assets after upload
    let (trigger, set_trigger) = signal(0);
    let (action_error, set_action_error) = signal(Option::<String>::None);

    let release = local_resource(
        move || (owner(), repo_name(), id(), trigger.get()),
        |(o, r, i, _)| async move {
            get_or::<Option<Release>>(&format!("/api/v1/repos/{}/{}/releases/{}", o, r, i), None)
                .await
        },
    );

    let on_upload_asset = move |_| {
        let o = owner();
        let r = repo_name();
        let i = id();
        spawn_local(async move {
            if post(&format!("/api/v1/repos/{}/{}/releases/{}/assets", o, r, i)).await {
                set_action_error.set(None);
                set_trigger.update(|n| *n += 1);
            } else {
                set_action_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    let on_delete = move |_| {
        let o = owner();
        let r = repo_name();
        let i = id();
        spawn_local(async move {
            // Navigate away only once the release is actually gone; a rejected
            // delete must not strand the user on a dead page.
            if delete(&format!("/api/v1/repos/{}/{}/releases/{}", o, r, i)).await {
                set_action_error.set(None);
                let window = web_sys::window().unwrap();
                let _ = window
                    .location()
                    .set_href(&format!("/repos/{}/{}/releases", o, r));
            } else {
                set_action_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    view! {
        <div class="release-detail">
        <RepoNav/>
            <Suspense fallback=move || view! { <p>"Loading..."</p> }>
                {move || match release.get() {
                    Some(Some(r)) => view! {
                        <div>
                            <div class="flex-between">
                                <h2>{r.name}</h2>
                                <div>
                                    <button on:click=on_delete class="btn btn-danger-solid">"Delete Release"</button>
                                </div>
                            </div>
                            <div class="meta">
                                <span class="bold">{r.tag_name}</span>
                                " | " {r.created_at} " | "
                                {if r.draft { "Draft" } else { "Published" }}
                            </div>
                            <div class="body">
                                {r.body.unwrap_or_default()}
                            </div>
                            <div class="assets-section">
                                <h3>"Assets"</h3>
                                <ul>
                                    <For each=move || r.assets.clone() key=|a| a.id children=move |a| {
                                        view! {
                                            <li>
                                                <a href=a.download_url>"📦 " {a.name}</a> " (" {a.size} " bytes)"
                                            </li>
                                        }
                                    }/>
                                </ul>
                                <button on:click=on_upload_asset>"Upload New Asset (Mock)"</button>
                                {move || action_error.get().map(|msg| view! {
                                    <p class="form-error" role="alert">{msg}</p>
                                })}
                            </div>
                        </div>
                    }.into_any(),
                    _ => view! { <p>"Release not found"</p> }.into_any()
                }}
            </Suspense>
        </div>
    }
}

#[component]
pub fn ReleaseCreate() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|params| params.get("owner").unwrap_or_default());
    let repo_name = move || params.with(|params| params.get("repo").unwrap_or_default());

    let (tag, set_tag) = signal("".to_string());
    let (name, set_name) = signal("".to_string());
    let (body, set_body) = signal("".to_string());
    let (draft, set_draft) = signal(false);
    let (prerelease, set_prerelease) = signal(false);
    let (form_error, set_form_error) = signal(Option::<String>::None);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let payload = CreateReleaseOption {
            tag_name: tag.get(),
            name: name.get(),
            body: Some(body.get()),
            draft: draft.get(),
            prerelease: prerelease.get(),
        };
        let o = owner();
        let r = repo_name();

        spawn_local(async move {
            if post_json(&format!("/api/v1/repos/{}/{}/releases", o, r), &payload).await {
                set_form_error.set(None);
                let window = web_sys::window().unwrap();
                let _ = window
                    .location()
                    .set_href(&format!("/repos/{}/{}/releases", o, r));
            } else {
                set_form_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    view! {
        <div class="release-create">
        <RepoNav/>
            <h3>"Create a new release"</h3>
            <form on:submit=on_submit>
                <div class="mb-1">
                    <label>"Tag version"</label>
                    <input type="text" prop:value=tag on:input=move |ev| set_tag.set(event_target_value(&ev)) placeholder="v1.0.0" required class="w-100"/>
                </div>
                <div class="mb-1">
                    <label>"Release title"</label>
                    <input type="text" prop:value=name on:input=move |ev| set_name.set(event_target_value(&ev)) placeholder="Release title" required class="w-100"/>
                </div>
                <div class="mb-1">
                    <label>"Description"</label>
                    <textarea prop:value=body on:input=move |ev| set_body.set(event_target_value(&ev)) rows="10" class="w-100"></textarea>
                </div>
                <div class="mb-1">
                    <label>
                        <input type="checkbox" prop:checked=draft on:change=move |ev| set_draft.set(event_target_checked(&ev)) />
                        " This is a draft"
                    </label>
                </div>
                <div class="mb-1">
                    <label>
                        <input type="checkbox" prop:checked=prerelease on:change=move |ev| set_prerelease.set(event_target_checked(&ev)) />
                        " This is a pre-release"
                    </label>
                </div>
                <button type="submit" class="btn btn-success-solid">"Publish Release"</button>
                {move || form_error.get().map(|msg| view! {
                    <p class="form-error" role="alert">{msg}</p>
                })}
            </form>
        </div>
    }
}
