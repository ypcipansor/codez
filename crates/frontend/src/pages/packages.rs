use crate::api::{get, get_or, local_resource, post_json, WRITE_ERROR};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::*;
use shared::{CreatePackageOption, Package};

#[component]
pub fn PackageList() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|params| params.get("owner").unwrap_or_default());

    let (show_upload, set_show_upload) = signal(false);
    let (refresh, set_refresh) = signal(0);

    let packages = local_resource(
        move || (owner(), refresh.get()),
        |(owner_name, _)| async move {
            get::<Vec<Package>>(&format!("/api/v1/packages/{}", owner_name)).await
        },
    );

    view! {
        <div class="package-list">
            <div class="header flex-between">
                <h3>"Packages for " {owner}</h3>
                <button on:click=move |_| set_show_upload.set(!show_upload.get())>
                    {move || if show_upload.get() { "Cancel" } else { "Upload Package" }}
                </button>
            </div>

            {move || if show_upload.get() {
                view! { <UploadPackageForm owner=owner() on_success=move || { set_show_upload.set(false); set_refresh.update(|n| *n += 1); } /> }.into_any()
            } else {
                view! { <span></span> }.into_any()
            }}

            <ul>
                <Suspense fallback=move || view! { <li>"Loading..."</li> }>
                    {move || packages.get().map(|list| view! {
                        <For each=move || list.clone() key=|p| p.id children=move |p| {
                            let href = format!("/packages/{}/{}/{}/{}", owner(), p.package_type, p.name, p.version);
                            view! {
                                <li class="boxed">
                                    <a href=href class="bold">{p.name}</a>
                                    <span class="ml-2 text-muted">"v" {p.version} " (" {p.package_type} ")"</span>
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
fn UploadPackageForm<F>(owner: String, on_success: F) -> impl IntoView
where
    F: Fn() + Clone + 'static,
{
    let (name, set_name) = signal("".to_string());
    let (version, set_version) = signal("".to_string());
    let (pkg_type, set_pkg_type) = signal("npm".to_string());
    let (form_error, set_form_error) = signal(Option::<String>::None);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let payload = CreatePackageOption {
            name: name.get(),
            version: version.get(),
            package_type: pkg_type.get(),
        };
        let o = owner.clone();
        let on_success_clone = on_success.clone();
        spawn_local(async move {
            if post_json(&format!("/api/v1/packages/{}", o), &payload).await {
                set_form_error.set(None);
                on_success_clone();
            } else {
                set_form_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    view! {
        <form on:submit=on_submit class="panel">
            <div>
                <input type="text" placeholder="Package Name" prop:value=name on:input=move |ev| set_name.set(event_target_value(&ev)) required />
            </div>
            <div>
                <input type="text" placeholder="Version (e.g. 1.0.0)" prop:value=version on:input=move |ev| set_version.set(event_target_value(&ev)) required />
            </div>
            <div>
                <select on:change=move |ev| set_pkg_type.set(event_target_value(&ev))>
                    <option value="npm">"npm"</option>
                    <option value="maven">"Maven"</option>
                    <option value="cargo">"Cargo"</option>
                    <option value="docker">"Docker"</option>
                    <option value="generic">"Generic"</option>
                </select>
            </div>
            <button type="submit">"Upload"</button>
            {move || form_error.get().map(|msg| view! {
                <p class="form-error" role="alert">{msg}</p>
            })}
        </form>
    }
}

#[component]
pub fn PackageDetail() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|params| params.get("owner").unwrap_or_default());
    let name = move || params.with(|params| params.get("name").unwrap_or_default());
    let version = move || params.with(|params| params.get("version").unwrap_or_default());
    let pkg_type = move || params.with(|params| params.get("type").unwrap_or_default());

    let package = local_resource(
        move || (owner(), pkg_type(), name(), version()),
        |(o, t, n, v)| async move {
            get_or::<Option<Package>>(&format!("/api/v1/packages/{}/{}/{}/{}", o, t, n, v), None)
                .await
        },
    );

    view! {
        <div class="package-detail">
            <Suspense fallback=move || view! { <h3>"Loading..."</h3> }>
                {move || match package.get() {
                    Some(Some(p)) => {
                        let name = p.name.clone();
                        let owner = p.owner.clone();
                        let version = p.version.clone();
                        let pkg_type = p.package_type.clone();
                        let install_cmd = match pkg_type.as_str() {
                                    "npm" => format!("npm install {}@{}", name, version),
                                    "cargo" => format!("cargo add {}@{}", name, version),
                                    _ => "See documentation".to_string()
                        };

                        view! {
                            <h3>"Package: " {name}</h3>
                            <div class="meta">
                                <p><strong>"Owner:"</strong> " " {owner}</p>
                                <p><strong>"Version:"</strong> " " {version}</p>
                                <p><strong>"Type:"</strong> " " {pkg_type}</p>
                            </div>
                            <div class="install-instructions">
                                <h4>"Installation"</h4>
                                <pre>{install_cmd}</pre>
                            </div>
                        }.into_any()
                    },
                    _ => view! { <h3>"Package Not Found"</h3> }.into_any()
                }}
            </Suspense>
        </div>
    }
}
