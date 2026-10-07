use crate::api::{delete, get, get_or, local_resource, patch_json, post_json, WRITE_ERROR};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::*;
use shared::{
    Contribution, CreateGpgKeyOption, CreateKeyOption, GpgKey, LoginOption, PublicKey,
    RegisterOption, User, UserSettingsOption,
};

#[component]
pub fn Login() -> impl IntoView {
    let (username, set_username) = signal("".to_string());
    let (password, set_password) = signal("".to_string());
    let (form_error, set_form_error) = signal(Option::<String>::None);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let payload = LoginOption {
            username: username.get(),
            password: password.get(),
        };
        spawn_local(async move {
            // Never clear the credential fields on failure — the user would have
            // to retype both.
            if post_json("/api/v1/users/login", &payload).await {
                set_form_error.set(None);
                leptos::logging::log!("Logged in");
            } else {
                set_form_error.set(Some("Invalid username or password.".to_string()));
            }
        });
    };

    view! {
        <div class="login">
            <h2>"Login"</h2>
            <form on:submit=on_submit>
                <input type="text" placeholder="Username" prop:value=username on:input=move |ev| set_username.set(event_target_value(&ev)) />
                <input type="password" placeholder="Password" prop:value=password on:input=move |ev| set_password.set(event_target_value(&ev)) />
                <button type="submit">"Login"</button>
                {move || form_error.get().map(|msg| view! {
                    <p class="form-error" role="alert">{msg}</p>
                })}
            </form>
        </div>
    }
}

#[component]
pub fn Register() -> impl IntoView {
    let (username, set_username) = signal("".to_string());
    let (email, set_email) = signal("".to_string());
    let (password, set_password) = signal("".to_string());
    let (form_error, set_form_error) = signal(Option::<String>::None);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let payload = RegisterOption {
            username: username.get(),
            email: email.get(),
            password: password.get(),
        };
        spawn_local(async move {
            if post_json("/api/v1/users/register", &payload).await {
                set_form_error.set(None);
                leptos::logging::log!("Registered");
            } else {
                set_form_error.set(Some("Could not create your account.".to_string()));
            }
        });
    };

    view! {
        <div class="register">
            <h2>"Register"</h2>
            <form on:submit=on_submit>
                <input type="text" placeholder="Username" prop:value=username on:input=move |ev| set_username.set(event_target_value(&ev)) />
                <input type="email" placeholder="Email" prop:value=email on:input=move |ev| set_email.set(event_target_value(&ev)) />
                <input type="password" placeholder="Password" prop:value=password on:input=move |ev| set_password.set(event_target_value(&ev)) />
                <button type="submit">"Register"</button>
                {move || form_error.get().map(|msg| view! {
                    <p class="form-error" role="alert">{msg}</p>
                })}
            </form>
        </div>
    }
}

#[component]
pub fn UserProfile() -> impl IntoView {
    let params = use_params_map();
    let username = move || params.with(|params| params.get("username").unwrap_or_default());

    let user = local_resource(username, |u| async move {
        get_or::<Option<User>>(&format!("/api/v1/users/{}", u), None).await
    });

    view! {
        <div class="user-profile">
            <h2>"User Profile: " {username}</h2>
            <div class="user-links">
                <a href="followers">"Followers"</a> " | " <a href="following">"Following"</a>
            </div>
            <Suspense fallback=move || view! { <p>"Loading..."</p> }>
                {move || match user.get() {
                    Some(Some(u)) => view! {
                        <div>
                            <p>"Email: " {u.email.unwrap_or("Hidden".to_string())}</p>
                            <UserHeatmap/>
                        </div>
                    }.into_any(),
                    _ => view! { <p>"User not found"</p> }.into_any()
                }}
            </Suspense>
        </div>
    }
}

#[component]
pub fn UserHeatmap() -> impl IntoView {
    let params = use_params_map();
    let username = move || params.with(|params| params.get("username").unwrap_or_default());
    let data = local_resource(username, |u| async move {
        get::<Vec<Contribution>>(&format!("/api/v1/users/{}/heatmap", u)).await
    });

    view! {
        <div class="user-heatmap">
            <h3>"Contributions"</h3>
            <div class="calendar-stub">
                <Suspense fallback=move || view! { <p>"Loading..."</p> }>
                    {move || data.get().map(|list| view! {
                        <For each=move || list.clone() key=|c| c.date.clone() children=move |c| {
                             let color = if c.count == 0 { "#ebedf0" } else if c.count < 5 { "#9be9a8" } else { "#30a14e" };
                             view! { <div title=format!("{} commits on {}", c.count, c.date) style=format!("width: 10px; height: 10px; background-color: {};", color)></div> }
                        }/>
                    })}
                </Suspense>
            </div>
        </div>
    }
}

#[component]
pub fn UserFollowers() -> impl IntoView {
    let params = use_params_map();
    let username = move || params.with(|params| params.get("username").unwrap_or_default());
    let users = local_resource(username, |u| async move {
        get::<Vec<User>>(&format!("/api/v1/users/{}/followers", u)).await
    });

    view! {
        <div class="followers">
            <h3>"Followers"</h3>
            <ul>
                <Suspense fallback=move || view! { <li>"Loading..."</li> }>
                    {move || users.get().map(|list| view! {
                        <For each=move || list.clone() key=|u| u.id children=move |u| {
                            view! { <li>{u.username}</li> }
                        }/>
                    })}
                </Suspense>
            </ul>
        </div>
    }
}

#[component]
pub fn UserFollowing() -> impl IntoView {
    let params = use_params_map();
    let username = move || params.with(|params| params.get("username").unwrap_or_default());
    let users = local_resource(username, |u| async move {
        get::<Vec<User>>(&format!("/api/v1/users/{}/following", u)).await
    });

    view! {
        <div class="following">
            <h3>"Following"</h3>
            <ul>
                <Suspense fallback=move || view! { <li>"Loading..."</li> }>
                    {move || users.get().map(|list| view! {
                        <For each=move || list.clone() key=|u| u.id children=move |u| {
                            view! { <li>{u.username}</li> }
                        }/>
                    })}
                </Suspense>
            </ul>
        </div>
    }
}

#[component]
pub fn UserSettings() -> impl IntoView {
    let settings = local_resource(
        || (),
        |_| async move {
            get_or::<UserSettingsOption>(
                "/api/v1/user/settings",
                UserSettingsOption {
                    full_name: None,
                    website: None,
                    description: None,
                    location: None,
                },
            )
            .await
        },
    );

    let (refresh, set_refresh) = signal(0);

    let keys = local_resource(
        move || refresh.get(),
        |_| async move { get::<Vec<PublicKey>>("/api/v1/user/keys").await },
    );

    let gpg_keys = local_resource(
        move || refresh.get(),
        |_| async move { get::<Vec<GpgKey>>("/api/v1/user/gpg_keys").await },
    );

    let (full_name, set_full_name) = signal("".to_string());

    let (ssh_title, set_ssh_title) = signal("".to_string());
    let (ssh_key, set_ssh_key) = signal("".to_string());

    let (gpg_key_content, set_gpg_key_content) = signal("".to_string());
    let (key_error, set_key_error) = signal(Option::<String>::None);

    let on_add_ssh_key = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let payload = CreateKeyOption {
            title: ssh_title.get(),
            key: ssh_key.get(),
        };
        spawn_local(async move {
            if post_json("/api/v1/user/keys", &payload).await {
                set_ssh_title.set("".to_string());
                set_ssh_key.set("".to_string());
                set_key_error.set(None);
                set_refresh.update(|n| *n += 1);
            } else {
                set_key_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    let on_delete_ssh_key = move |id: u64| {
        spawn_local(async move {
            if delete(&format!("/api/v1/user/keys/{}", id)).await {
                set_key_error.set(None);
                set_refresh.update(|n| *n += 1);
            } else {
                set_key_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    let on_add_gpg_key = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let payload = CreateGpgKeyOption {
            armored_public_key: gpg_key_content.get(),
        };
        spawn_local(async move {
            if post_json("/api/v1/user/gpg_keys", &payload).await {
                set_gpg_key_content.set("".to_string());
                set_key_error.set(None);
                set_refresh.update(|n| *n += 1);
            } else {
                set_key_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    let on_delete_gpg_key = move |id: u64| {
        spawn_local(async move {
            if delete(&format!("/api/v1/user/gpg_keys/{}", id)).await {
                set_key_error.set(None);
                set_refresh.update(|n| *n += 1);
            } else {
                set_key_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    let on_update_profile = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let payload = UserSettingsOption {
            full_name: Some(full_name.get()),
            website: None,
            description: None,
            location: None,
        };
        spawn_local(async move {
            if patch_json("/api/v1/user/settings", &payload).await {
                set_key_error.set(None);
            } else {
                set_key_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    view! {
        <div class="user-settings">
            <h2>"Settings"</h2>
            <div class="profile-settings">
                <h3>"Profile"</h3>
                <Suspense fallback=move || view! { <p>"Loading profile..."</p> }>
                    {move || settings.get().map(|s| view! {
                        <p>"Current Name: " {s.full_name.unwrap_or_default()}</p>
                    })}
                </Suspense>
                <form on:submit=on_update_profile>
                    <input type="text" placeholder="Full Name" prop:value=full_name on:input=move |ev| set_full_name.set(event_target_value(&ev)) />
                    <button type="submit">"Update Profile"</button>
                    {move || key_error.get().map(|msg| view! {
                        <p class="form-error" role="alert">{msg}</p>
                    })}
                </form>
            </div>

            <div class="ssh-keys">
                <h3>"SSH Keys"</h3>
                <ul>
                    <Suspense fallback=move || view! { <li>"Loading..."</li> }>
                        {move || keys.get().map(|list| view! {
                            <For each=move || list.clone() key=|k| k.id children=move |k| {
                                let key_id = k.id;
                                view! {
                                    <li>
                                        {k.title} " - " {k.fingerprint}
                                        <button on:click=move |_| on_delete_ssh_key(key_id) class="ml-2 text-danger">"Delete"</button>
                                    </li>
                                }
                            }/>
                        })}
                    </Suspense>
                </ul>
                <form on:submit=on_add_ssh_key class="panel-plain">
                    <h4>"Add SSH Key"</h4>
                    <input type="text" placeholder="Title" prop:value=ssh_title on:input=move |ev| set_ssh_title.set(event_target_value(&ev))  required />
                    <textarea placeholder="Key starting with ssh-rsa..." prop:value=ssh_key on:input=move |ev| set_ssh_key.set(event_target_value(&ev)) rows="4"  required></textarea>
                    <button type="submit">"Add SSH Key"</button>
                    {move || key_error.get().map(|msg| view! {
                        <p class="form-error" role="alert">{msg}</p>
                    })}
                </form>
            </div>

            <div class="gpg-keys">
                <h3>"GPG Keys"</h3>
                <ul>
                    <Suspense fallback=move || view! { <li>"Loading..."</li> }>
                        {move || gpg_keys.get().map(|list| view! {
                            <For each=move || list.clone() key=|k| k.id children=move |k| {
                                let key_id = k.id;
                                view! {
                                    <li>
                                        {k.key_id} " - " {k.primary_key_id}
                                        <button on:click=move |_| on_delete_gpg_key(key_id) class="ml-2 text-danger">"Delete"</button>
                                    </li>
                                }
                            }/>
                        })}
                    </Suspense>
                </ul>
                <form on:submit=on_add_gpg_key class="panel-plain">
                    <h4>"Add GPG Key"</h4>
                    <textarea placeholder="Armored GPG Public Key..." prop:value=gpg_key_content on:input=move |ev| set_gpg_key_content.set(event_target_value(&ev)) rows="6"  required></textarea>
                    <button type="submit">"Add GPG Key"</button>
                    {move || key_error.get().map(|msg| view! {
                        <p class="form-error" role="alert">{msg}</p>
                    })}
                </form>
            </div>
        </div>
    }
}
