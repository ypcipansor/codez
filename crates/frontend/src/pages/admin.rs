use crate::api::{delete, get, get_opt, local_resource, WRITE_ERROR};
use leptos::prelude::*;
use leptos::task::spawn_local;
use shared::{AdminStats, SystemNotice, User};

#[component]
pub fn AdminDashboard() -> impl IntoView {
    let stats = local_resource(
        || (),
        |_| async move { get_opt::<AdminStats>("/api/v1/admin/stats").await },
    );

    view! {
        <div class="admin-dashboard">
            <h2>"Admin Dashboard"</h2>
            <p><a href="/admin/users">"Manage Users"</a></p>
            <Suspense fallback=move || view! { <p>"Loading..."</p> }>
                {move || match stats.get() {
                    Some(Some(s)) => view! {
                        <div>
                            <p>"Users: " {s.users}</p>
                            <p>"Repos: " {s.repos}</p>
                            <p>"Orgs: " {s.orgs}</p>
                            <p>"Issues: " {s.issues}</p>
                        </div>
                    }.into_any(),
                    _ => view! { <p>"No stats"</p> }.into_any()
                }}
            </Suspense>
            <AdminNotices/>
        </div>
    }
}

#[component]
pub fn AdminUsers() -> impl IntoView {
    let (refresh, set_refresh) = signal(0);
    let (action_error, set_action_error) = signal(Option::<String>::None);

    let users = local_resource(
        move || refresh.get(),
        |_| async move { get::<Vec<User>>("/api/v1/admin/users").await },
    );

    let on_delete = move |username: String| {
        spawn_local(async move {
            if delete(&format!("/api/v1/admin/users/{}", username)).await {
                set_action_error.set(None);
                set_refresh.update(|n| *n += 1);
            } else {
                set_action_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    view! {
        <div class="admin-users">
            <h3>"User Management"</h3>
            {move || action_error.get().map(|msg| view! {
                <p class="form-error" role="alert">{msg}</p>
            })}
            <table>
                <thead><tr><th>"ID"</th><th>"Username"</th><th>"Email"</th><th>"Actions"</th></tr></thead>
                <tbody>
                    <Suspense fallback=move || view! { <tr><td colspan="4">"Loading..."</td></tr> }>
                        {move || users.get().map(|list| view! {
                            <For each=move || list.clone() key=|u| u.id children=move |u| {
                                let uname = u.username.clone();
                                view! {
                                    <tr>
                                        <td>{u.id}</td>
                                        <td>{u.username}</td>
                                        <td>{u.email}</td>
                                        <td>
                                            <button on:click=move |_| {
                                                let u = uname.clone();
                                                on_delete(u);
                                            }>"Delete"</button>
                                        </td>
                                    </tr>
                                }
                            }/>
                        })}
                    </Suspense>
                </tbody>
            </table>
        </div>
    }
}

#[component]
pub fn AdminNotices() -> impl IntoView {
    let notices = local_resource(
        || (),
        |_| async move { get::<Vec<SystemNotice>>("/api/v1/admin/notices").await },
    );

    view! {
        <div class="admin-notices">
            <h3>"System Notices"</h3>
            <ul>
                <Suspense fallback=move || view! { <li>"Loading..."</li> }>
                    {move || notices.get().map(|list| view! {
                        <For each=move || list.clone() key=|n| n.id children=move |n| {
                            view! { <li>[{n.type_.clone()}] " " {n.description.clone()}</li> }
                        }/>
                    })}
                </Suspense>
            </ul>
        </div>
    }
}
