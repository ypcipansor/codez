//! Repository settings, branches, webhooks, secrets and deploy keys.

use crate::api::{
    get, get_then, local_resource, patch_json, post, post_json, put_json, WRITE_ERROR,
};
use crate::components::RepoNav;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::*;
use shared::{
    CreateHookOption, CreateKeyOption, CreateProtectedBranchOption, CreateSecretOption, DeployKey,
    LfsLock, ProtectedBranch, RepoSettingsOption, RepoTopicOptions, Secret, TransferRepoOption,
    Webhook, WebhookDelivery,
};

#[component]
pub fn RepoSettings() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|params| params.get("owner").unwrap_or_default());
    let repo_name = move || params.with(|params| params.get("repo").unwrap_or_default());

    let (desc, set_desc) = signal("".to_string());
    let (website, set_website) = signal("".to_string());
    let (default_branch, set_default_branch) = signal("main".to_string());
    let (private, set_private) = signal(false);
    let (rebase, set_rebase) = signal(true);
    let (squash, set_squash) = signal(true);
    let (merge, set_merge) = signal(true);
    let (issues, set_issues) = signal(true);
    let (wiki, set_wiki) = signal(true);
    let (projects, set_projects) = signal(true);

    let (transfer_to, set_transfer_to) = signal("".to_string());
    let (topics, set_topics) = signal("".to_string());
    let (save_error, set_save_error) = signal(Option::<String>::None);

    // Load initial settings
    let _ = local_resource(
        move || (owner(), repo_name()),
        move |(o, r)| async move {
            get_then::<RepoSettingsOption, _>(
                &format!("/api/v1/repos/{}/{}/settings", o, r),
                |settings| {
                    set_desc.set(settings.description.unwrap_or_default());
                    set_website.set(settings.website.unwrap_or_default());
                    set_default_branch.set(settings.default_branch.unwrap_or("main".to_string()));
                    set_private.set(settings.private.unwrap_or(false));
                    set_rebase.set(settings.allow_rebase_merge.unwrap_or(true));
                    set_squash.set(settings.allow_squash_merge.unwrap_or(true));
                    set_merge.set(settings.allow_merge_commit.unwrap_or(true));
                    set_issues.set(settings.has_issues.unwrap_or(true));
                    set_wiki.set(settings.has_wiki.unwrap_or(true));
                    set_projects.set(settings.has_projects.unwrap_or(true));
                },
            )
            .await;
        },
    );

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let payload = RepoSettingsOption {
            description: Some(desc.get()),
            private: Some(private.get()),
            website: Some(website.get()),
            default_branch: Some(default_branch.get()),
            allow_rebase_merge: Some(rebase.get()),
            allow_squash_merge: Some(squash.get()),
            allow_merge_commit: Some(merge.get()),
            has_issues: Some(issues.get()),
            has_wiki: Some(wiki.get()),
            has_projects: Some(projects.get()),
        };
        let o = owner();
        let r = repo_name();
        spawn_local(async move {
            if patch_json(&format!("/api/v1/repos/{}/{}/settings", o, r), &payload).await {
                set_save_error.set(None);
            } else {
                set_save_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    let on_transfer = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let payload = TransferRepoOption {
            new_owner: transfer_to.get(),
        };
        let o = owner();
        let r = repo_name();
        spawn_local(async move {
            // Clear the target owner only after the transfer is accepted, so a
            // rejected transfer keeps the typed username.
            if post_json(&format!("/api/v1/repos/{}/{}/transfer", o, r), &payload).await {
                set_transfer_to.set("".to_string());
                set_save_error.set(None);
            } else {
                set_save_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    let on_update_topics = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let topic_list: Vec<String> = topics
            .get()
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        let payload = RepoTopicOptions { topics: topic_list };
        let o = owner();
        let r = repo_name();
        spawn_local(async move {
            if put_json(&format!("/api/v1/repos/{}/{}/topics", o, r), &payload).await {
                set_save_error.set(None);
            } else {
                set_save_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    let on_sync = move |_| {
        let o = owner();
        let r = repo_name();
        spawn_local(async move {
            if post(&format!("/api/v1/repos/{}/{}/mirror-sync", o, r)).await {
                set_save_error.set(None);
            } else {
                set_save_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    view! {
        <div class="repo-settings">
            <RepoNav/>
            <h3>"Repository Settings"</h3>
            <form on:submit=on_submit>
                <div>
                    <label>"Description"</label>
                    <input type="text" prop:value=desc on:input=move |ev| set_desc.set(event_target_value(&ev)) />
                </div>
                <div>
                    <label>"Website"</label>
                    <input type="text" prop:value=website on:input=move |ev| set_website.set(event_target_value(&ev)) />
                </div>
                <div>
                    <label>"Default Branch"</label>
                    <input type="text" prop:value=default_branch on:input=move |ev| set_default_branch.set(event_target_value(&ev)) />
                </div>

                <div class="mb-1">
                    <label><input type="checkbox" prop:checked=private on:change=move |ev| set_private.set(event_target_checked(&ev)) /> " Private Repository"</label>
                </div>

                <h4>"Merge Strategies"</h4>
                <div><label><input type="checkbox" prop:checked=merge on:change=move |ev| set_merge.set(event_target_checked(&ev)) /> " Allow Merge Commits"</label></div>
                <div><label><input type="checkbox" prop:checked=rebase on:change=move |ev| set_rebase.set(event_target_checked(&ev)) /> " Allow Rebase Merge"</label></div>
                <div><label><input type="checkbox" prop:checked=squash on:change=move |ev| set_squash.set(event_target_checked(&ev)) /> " Allow Squash Merge"</label></div>

                <h4>"Features"</h4>
                <div><label><input type="checkbox" prop:checked=issues on:change=move |ev| set_issues.set(event_target_checked(&ev)) /> " Enable Issues"</label></div>
                <div><label><input type="checkbox" prop:checked=wiki on:change=move |ev| set_wiki.set(event_target_checked(&ev)) /> " Enable Wiki"</label></div>
                <div><label><input type="checkbox" prop:checked=projects on:change=move |ev| set_projects.set(event_target_checked(&ev)) /> " Enable Projects"</label></div>

                <button type="submit" class="mt-3">"Update Settings"</button>
                {move || save_error.get().map(|msg| view! {
                    <p class="form-error" role="alert">{msg}</p>
                })}
            </form>

            <h4>"Topics"</h4>
            <form on:submit=on_update_topics>
                <input type="text" placeholder="rust, gitea, clone" prop:value=topics on:input=move |ev| set_topics.set(event_target_value(&ev)) />
                <button type="submit">"Update Topics"</button>
            </form>

            <h4>"Mirror Settings"</h4>
            <button on:click=on_sync>"Sync Now"</button>

            <h4>"Transfer Ownership"</h4>
            <form on:submit=on_transfer>
                <input type="text" placeholder="New Owner Username" prop:value=transfer_to on:input=move |ev| set_transfer_to.set(event_target_value(&ev)) />
                <button type="submit">"Transfer"</button>
            </form>

            <div class="settings-sections">
                <p><a href="collaborators">"Collaborators"</a></p>
                <p><a href="webhooks">"Webhooks"</a></p>
                <p><a href="secrets">"Secrets"</a></p>
                <p><a href="keys">"Deploy Keys"</a></p>
                <p><a href="branches">"Protected Branches"</a></p>
                <p><a href="lfs">"Git LFS Locks"</a></p>
            </div>
        </div>
    }
}

#[component]
pub fn CreateProtectedBranch(
    owner: Signal<String>,
    repo: Signal<String>,
    on_success: Action<(), ()>,
) -> impl IntoView {
    let (name, set_name) = signal(String::new());
    let (enable_push, set_enable_push) = signal(false);
    let (enable_force_push, set_enable_force_push) = signal(false);
    let (status_checks, set_status_checks) = signal(String::new());
    let (branch_error, set_branch_error) = signal(Option::<String>::None);

    let create_action = Action::new_local(move |_: &()| {
        let name_val = name.get();
        let enable_push_val = enable_push.get();
        let enable_force_push_val = enable_force_push.get();
        let status_checks_str = status_checks.get();

        let req_status_checks = if status_checks_str.trim().is_empty() {
            None
        } else {
            Some(
                status_checks_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect(),
            )
        };

        let opt = CreateProtectedBranchOption {
            name: name_val,
            enable_push: enable_push_val,
            enable_force_push: enable_force_push_val,
            required_status_checks: req_status_checks,
        };

        let o = owner.get();
        let r = repo.get();
        let on_success_clone = on_success;

        async move {
            if post_json(
                &format!("/api/v1/repos/{}/{}/branch_protections", o, r),
                &opt,
            )
            .await
            {
                on_success_clone.dispatch(());
                set_name.set(String::new());
                set_enable_push.set(false);
                set_enable_force_push.set(false);
                set_status_checks.set(String::new());
                set_branch_error.set(None);
            } else {
                set_branch_error.set(Some(WRITE_ERROR.to_string()));
            }
        }
    });

    view! {
        <div class="box create-protected-branch">
            <h4>"Add Branch Protection Rule"</h4>
            <form on:submit=move |ev| {
                ev.prevent_default();
                create_action.dispatch(());
            }>
                <div class="form-group">
                    <label>"Branch Name Pattern"</label>
                    <input type="text" placeholder="e.g. main, release-*" prop:value=name on:input=move |ev| set_name.set(event_target_value(&ev)) required />
                </div>
                <div class="form-group checkbox-group">
                    <label>
                        <input type="checkbox" prop:checked=enable_push on:change=move |ev| set_enable_push.set(event_target_checked(&ev)) />
                        " Enable Push"
                    </label>
                </div>
                <div class="form-group checkbox-group">
                    <label>
                        <input type="checkbox" prop:checked=enable_force_push on:change=move |ev| set_enable_force_push.set(event_target_checked(&ev)) />
                        " Enable Force Push"
                    </label>
                </div>
                <div class="form-group">
                    <label>"Required Status Checks (comma separated)"</label>
                    <input type="text" placeholder="e.g. ci/test, security-scan" prop:value=status_checks on:input=move |ev| set_status_checks.set(event_target_value(&ev)) />
                </div>
                <button type="submit" class="btn-primary">"Create Rule"</button>
                {move || branch_error.get().map(|msg| view! {
                    <p class="form-error" role="alert">{msg}</p>
                })}
            </form>
        </div>
    }
}

#[component]
pub fn ProtectedBranchList() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|params| params.get("owner").unwrap_or_default());
    let repo_name = move || params.with(|params| params.get("repo").unwrap_or_default());
    let owner_sig = Signal::derive(owner);
    let repo_sig = Signal::derive(repo_name);

    let (refetch_trigger, set_refetch_trigger) = signal(0);

    let branches = local_resource(
        move || (owner(), repo_name(), refetch_trigger.get()),
        |(o, r, _)| async move {
            get::<Vec<ProtectedBranch>>(&format!("/api/v1/repos/{}/{}/branch_protections", o, r))
                .await
        },
    );

    let on_success = Action::new_local(move |_: &()| {
        set_refetch_trigger.update(|v| *v += 1);
        async {}
    });

    view! {
        <div class="protected-branches-page">
        <RepoNav/>
            <CreateProtectedBranch owner=owner_sig repo=repo_sig on_success=on_success/>
            <div class="protected-branches">
                <h3>"Protected Branches"</h3>
                <ul>
                    <Suspense fallback=move || view! { <li>"Loading..."</li> }>
                        {move || branches.get().map(|list| view! {
                            <For each=move || list.clone() key=|b| b.name.clone() children=move |b| {
                                view! {
                                    <li>
                                        {b.name}
                                        " (Push: " {b.enable_push}
                                        ", Force: " {b.enable_force_push}
                                        {if !b.required_status_checks.is_empty() {
                                            format!(", Checks: {}", b.required_status_checks.join(", "))
                                        } else {
                                            "".to_string()
                                        }}
                                        ")"
                                    </li>
                                }
                            }/>
                        })}
                    </Suspense>
                </ul>
            </div>
        </div>
    }
}

#[component]
pub fn LfsLockList() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|params| params.get("owner").unwrap_or_default());
    let repo_name = move || params.with(|params| params.get("repo").unwrap_or_default());

    let locks = local_resource(
        move || (owner(), repo_name()),
        |(o, r)| async move {
            get::<Vec<LfsLock>>(&format!("/api/v1/repos/{}/{}/git/lfs/locks", o, r)).await
        },
    );

    view! {
        <div class="lfs-locks">
        <RepoNav/>
            <h3>"Git LFS Locks"</h3>
            <ul>
                <Suspense fallback=move || view! { <li>"Loading..."</li> }>
                    {move || locks.get().map(|list| view! {
                        <For each=move || list.clone() key=|l| l.id.clone() children=move |l| {
                            view! { <li>{l.path} " locked by " {l.owner.username}</li> }
                        }/>
                    })}
                </Suspense>
            </ul>
        </div>
    }
}

#[component]
pub fn WebhookList() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|params| params.get("owner").unwrap_or_default());
    let repo_name = move || params.with(|params| params.get("repo").unwrap_or_default());

    let (url, set_url) = signal("".to_string());
    let (form_error, set_form_error) = signal(Option::<String>::None);

    let hooks = local_resource(
        move || (owner(), repo_name()),
        |(o, r)| async move { get::<Vec<Webhook>>(&format!("/api/v1/repos/{}/{}/hooks", o, r)).await },
    );

    let on_create = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let payload = CreateHookOption {
            url: url.get(),
            events: vec!["push".to_string()],
            active: true,
        };
        let o = owner();
        let r = repo_name();
        spawn_local(async move {
            if post_json(&format!("/api/v1/repos/{}/{}/hooks", o, r), &payload).await {
                set_url.set("".to_string());
                set_form_error.set(None);
            } else {
                set_form_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    view! {
        <div class="webhook-list">
        <RepoNav/>
            <h3>"Webhooks"</h3>
            <ul>
                <Suspense fallback=move || view! { <li>"Loading..."</li> }>
                    {move || hooks.get().map(|list| view! {
                        <For each=move || list.clone() key=|h| h.id children=move |h| {
                            let hook_id = h.id;
                            let o = owner();
                            let r = repo_name();
                            let deliveries = local_resource(
                                move || (o.clone(), r.clone(), hook_id),
                                |(o, r, id)| async move {
                                    get::<Vec<WebhookDelivery>>(&format!("/api/v1/repos/{}/{}/hooks/{}/deliveries", o, r, id)).await
                                }
                            );

                            view! {
                                <li class="list-row pb-1">
                                    <div>
                                        {h.url} " (" {if h.active { "Active" } else { "Inactive" }} ")"
                                    </div>
                                    <div class="deliveries">
                                        <strong>"Recent Deliveries:"</strong>
                                        <Suspense fallback=move || view! { <span>"..."</span> }>
                                            {move || deliveries.get().map(|list| {
                                                if list.is_empty() {
                                                    view! { <div>"No deliveries yet"</div> }.into_any()
                                                } else {
                                                    view! {
                                                        <ul class="indent-list">
                                                            <For each=move || list.clone() key=|d| d.id children=move |d| {
                                                                view! { <li>{d.delivered_at} " - " {d.event} " - " {d.status} " (" {d.response_status} ")"</li> }
                                                            }/>
                                                        </ul>
                                                    }.into_any()
                                                }
                                            })}
                                        </Suspense>
                                    </div>
                                </li>
                            }
                        }/>
                    })}
                </Suspense>
            </ul>
            <h4>"Add Webhook"</h4>
            <form on:submit=on_create>
                <input type="text" placeholder="Payload URL" prop:value=url on:input=move |ev| set_url.set(event_target_value(&ev)) />
                <button type="submit">"Add Webhook"</button>
                {move || form_error.get().map(|msg| view! {
                    <p class="form-error" role="alert">{msg}</p>
                })}
            </form>
        </div>
    }
}

#[component]
pub fn SecretList() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|params| params.get("owner").unwrap_or_default());
    let repo_name = move || params.with(|params| params.get("repo").unwrap_or_default());

    let (name, set_name) = signal("".to_string());
    let (data, set_data) = signal("".to_string());
    let (form_error, set_form_error) = signal(Option::<String>::None);

    let secrets = local_resource(
        move || (owner(), repo_name()),
        |(o, r)| async move { get::<Vec<Secret>>(&format!("/api/v1/repos/{}/{}/secrets", o, r)).await },
    );

    let on_create = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let payload = CreateSecretOption {
            name: name.get(),
            data: data.get(),
        };
        let o = owner();
        let r = repo_name();
        spawn_local(async move {
            if post_json(&format!("/api/v1/repos/{}/{}/secrets", o, r), &payload).await {
                set_name.set("".to_string());
                set_data.set("".to_string());
                set_form_error.set(None);
            } else {
                set_form_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    view! {
        <div class="secret-list">
        <RepoNav/>
            <h3>"Secrets"</h3>
            <ul>
                <Suspense fallback=move || view! { <li>"Loading..."</li> }>
                    {move || secrets.get().map(|list| view! {
                        <For each=move || list.clone() key=|s| s.name.clone() children=move |s| {
                            view! { <li>{s.name}</li> }
                        }/>
                    })}
                </Suspense>
            </ul>
            <h4>"Add Secret"</h4>
            <form on:submit=on_create>
                <input type="text" placeholder="Name" prop:value=name on:input=move |ev| set_name.set(event_target_value(&ev)) />
                <input type="text" placeholder="Value" prop:value=data on:input=move |ev| set_data.set(event_target_value(&ev)) />
                <button type="submit">"Add Secret"</button>
                {move || form_error.get().map(|msg| view! {
                    <p class="form-error" role="alert">{msg}</p>
                })}
            </form>
        </div>
    }
}

#[component]
pub fn DeployKeyList() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|params| params.get("owner").unwrap_or_default());
    let repo_name = move || params.with(|params| params.get("repo").unwrap_or_default());

    let (title, set_title) = signal("".to_string());
    let (key, set_key) = signal("".to_string());
    let (form_error, set_form_error) = signal(Option::<String>::None);

    let keys = local_resource(
        move || (owner(), repo_name()),
        |(o, r)| async move { get::<Vec<DeployKey>>(&format!("/api/v1/repos/{}/{}/keys", o, r)).await },
    );

    let on_create = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let payload = CreateKeyOption {
            title: title.get(),
            key: key.get(),
        };
        let o = owner();
        let r = repo_name();
        spawn_local(async move {
            if post_json(&format!("/api/v1/repos/{}/{}/keys", o, r), &payload).await {
                set_title.set("".to_string());
                set_key.set("".to_string());
                set_form_error.set(None);
            } else {
                set_form_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    view! {
        <div class="deploy-key-list">
        <RepoNav/>
            <h3>"Deploy Keys"</h3>
            <ul>
                <Suspense fallback=move || view! { <li>"Loading..."</li> }>
                    {move || keys.get().map(|list| view! {
                        <For each=move || list.clone() key=|k| k.id children=move |k| {
                            view! { <li>{k.title} " - " {k.fingerprint}</li> }
                        }/>
                    })}
                </Suspense>
            </ul>
            <h4>"Add Deploy Key"</h4>
            <form on:submit=on_create>
                <input type="text" placeholder="Title" prop:value=title on:input=move |ev| set_title.set(event_target_value(&ev)) />
                <textarea placeholder="Key" prop:value=key on:input=move |ev| set_key.set(event_target_value(&ev))></textarea>
                <button type="submit">"Add Key"</button>
                {move || form_error.get().map(|msg| view! {
                    <p class="form-error" role="alert">{msg}</p>
                })}
            </form>
        </div>
    }
}
