use crate::api::{get, get_or, local_resource, post, post_json};
use crate::components::RepoNav;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::*;
use shared::{
    CreateProjectCardOption, CreateProjectColumnOption, CreateProjectOption, Issue, Project,
    ProjectCard, ProjectColumn,
};

#[component]
pub fn ProjectList() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|params| params.get("owner").unwrap_or_default());
    let repo_name = move || params.with(|params| params.get("repo").unwrap_or_default());

    let (show_create, set_show_create) = signal(false);
    let (new_title, set_new_title) = signal("".to_string());
    let (new_desc, set_new_desc) = signal("".to_string());
    let (form_error, set_form_error) = signal(Option::<String>::None);

    let projects = local_resource(
        move || (owner(), repo_name(), show_create.get()), // refresh on create toggle/submit
        |(o, r, _)| async move {
            get::<Vec<Project>>(&format!("/api/v1/repos/{}/{}/projects", o, r)).await
        },
    );

    let on_create = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let payload = CreateProjectOption {
            title: new_title.get(),
            description: if new_desc.get().is_empty() {
                None
            } else {
                Some(new_desc.get())
            },
        };
        let o = owner();
        let r = repo_name();
        spawn_local(async move {
            if post_json(&format!("/api/v1/repos/{}/{}/projects", o, r), &payload).await {
                set_new_title.set("".to_string());
                set_new_desc.set("".to_string());
                set_show_create.set(false);
                set_form_error.set(None);
            } else {
                set_form_error.set(Some(crate::api::WRITE_ERROR.to_string()));
            }
        });
    };

    view! {
        <div class="project-list">
        <RepoNav/>
            <div class="header flex-between">
                <h3>"Projects"</h3>
                <button on:click=move |_| set_show_create.set(!show_create.get())>
                    {move || if show_create.get() { "Cancel" } else { "New Project" }}
                </button>
            </div>

            {move || if show_create.get() {
                view! {
                    <form on:submit=on_create class="panel mb-2">
                        <input type="text" placeholder="Project Title" prop:value=new_title on:input=move |ev| set_new_title.set(event_target_value(&ev))  required />
                        <textarea placeholder="Description" prop:value=new_desc on:input=move |ev| set_new_desc.set(event_target_value(&ev))></textarea>
                        <button type="submit">"Create Project"</button>
                        {move || form_error.get().map(|msg| view! {
                            <p class="form-error" role="alert">{msg}</p>
                        })}
                    </form>
                }.into_any()
            } else {
                view! { <span></span> }.into_any()
            }}

            <ul>
                <Suspense fallback=move || view! { <li>"Loading projects..."</li> }>
                    {move || projects.get().map(|list| {
                         if list.is_empty() {
                            view! { <li>"No projects found."</li> }.into_any()
                        } else {
                            view! {
                                <For each=move || list.clone() key=|p| p.id children=move |p| {
                                    let href = format!("/repos/{}/{}/projects/{}", owner(), repo_name(), p.id);
                                    view! {
                                        <li class="boxed">
                                            <a href=href class="text-lg bold">{p.title}</a>
                                            <p class="my-1">{p.description.unwrap_or_default()}</p>
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
pub fn ProjectDetail() -> impl IntoView {
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

    let (refresh, set_refresh) = signal(0);
    let (board_error, set_board_error) = signal(Option::<String>::None);

    let project = local_resource(
        move || (owner(), repo_name(), id(), refresh.get()),
        |(o, r, i, _)| async move {
            get_or::<Option<Project>>(&format!("/api/v1/repos/{}/{}/projects/{}", o, r, i), None)
                .await
        },
    );

    let columns = local_resource(
        move || (owner(), repo_name(), id(), refresh.get()),
        |(o, r, i, _)| async move {
            get::<Vec<ProjectColumn>>(&format!("/api/v1/repos/{}/{}/projects/{}/columns", o, r, i))
                .await
        },
    );

    let (new_col_title, set_new_col_title) = signal("".to_string());

    let on_add_column = move |_| {
        let o = owner();
        let r = repo_name();
        let i = id();
        let payload = CreateProjectColumnOption {
            title: new_col_title.get(),
        };
        if !payload.title.is_empty() {
            spawn_local(async move {
                if post_json(
                    &format!("/api/v1/repos/{}/{}/projects/{}/columns", o, r, i),
                    &payload,
                )
                .await
                {
                    set_new_col_title.set("".to_string());
                    set_board_error.set(None);
                    set_refresh.update(|n| *n += 1);
                } else {
                    set_board_error.set(Some(crate::api::WRITE_ERROR.to_string()));
                }
            });
        }
    };

    let on_toggle_close = move |is_closed: bool| {
        let o = owner();
        let r = repo_name();
        let i = id();
        let action = if is_closed { "reopen" } else { "close" };
        spawn_local(async move {
            if post(&format!(
                "/api/v1/repos/{}/{}/projects/{}/{}",
                o, r, i, action
            ))
            .await
            {
                set_board_error.set(None);
                set_refresh.update(|n| *n += 1); // Trigger resource reload
            } else {
                set_board_error.set(Some(crate::api::WRITE_ERROR.to_string()));
            }
        });
    };

    view! {
        <div class="project-board">
        <RepoNav/>
            <Suspense fallback=move || view! { <h3>"Loading Project..."</h3> }>
                {move || project.get().map(|p| match p {
                    Some(proj) => {
                        let is_closed = proj.is_closed;
                        view! {
                            <div class="board-header">
                                <div>
                                    <h3>{proj.title} {if is_closed { " (Closed)" } else { "" }}</h3>
                                    <p>{proj.description.unwrap_or_default()}</p>
                                </div>
                                <button on:click=move |_| on_toggle_close(is_closed)>
                                    {if is_closed { "Reopen Project" } else { "Close Project" }}
                                </button>
                            </div>
                        }.into_any()
                    },
                    None => view! { <h3>"Project Not Found"</h3> }.into_any()
                })}
            </Suspense>

            <div class="board-columns">
                 <Suspense fallback=move || view! { <div>"Loading columns..."</div> }>
                    {move || columns.get().map(|cols| view! {
                        <For each=move || cols.clone() key=|c| c.id children=move |c| {
                            view! { <ProjectColumnView column=c repo_owner=owner() repo_name=repo_name() _project_id=id() _refresh_signal=set_refresh /> }
                        }/>
                    })}
                </Suspense>

                <div class="add-column">
                    <input type="text" placeholder="New Column" prop:value=new_col_title on:input=move |ev| set_new_col_title.set(event_target_value(&ev)) />
                    <button on:click=on_add_column>"Add Column"</button>
                    {move || board_error.get().map(|msg| view! {
                        <p class="form-error" role="alert">{msg}</p>
                    })}
                </div>
            </div>
        </div>
    }
}

#[component]
fn ProjectColumnView(
    column: ProjectColumn,
    repo_owner: String,
    repo_name: String,
    _project_id: u64,
    _refresh_signal: WriteSignal<i32>,
) -> impl IntoView {
    let (refresh_cards, set_refresh_cards) = signal(0);
    let (new_card_content, set_new_card_content) = signal("".to_string());
    let (issue_id_input, set_issue_id_input) = signal("".to_string());
    let (card_error, set_card_error) = signal(Option::<String>::None);

    let column_id = column.id;
    let o = repo_owner.clone();
    let r = repo_name.clone();

    let o_cards = o.clone();
    let r_cards = r.clone();
    let cards = local_resource(
        move || {
            (
                o_cards.clone(),
                r_cards.clone(),
                column_id,
                refresh_cards.get(),
            )
        }, // also depends on global refresh? No, local is enough unless moved
        move |(o, r, c, _)| async move {
            get::<Vec<ProjectCard>>(&format!(
                "/api/v1/repos/{}/{}/projects/columns/{}/cards",
                o, r, c
            ))
            .await
        },
    );

    let o_issues = o.clone();
    let r_issues = r.clone();
    let issues = local_resource(
        move || (o_issues.clone(), r_issues.clone()),
        move |(o, r)| async move {
            get::<Vec<Issue>>(&format!("/api/v1/repos/{}/{}/issues?state=open", o, r)).await
        },
    );

    let on_add_card = move |_| {
        let o = repo_owner.clone();
        let r = repo_name.clone();
        let c = column_id;

        let issue_id = issue_id_input.get().parse::<u64>().ok();
        let content = new_card_content.get();

        let payload = CreateProjectCardOption {
            content: if content.is_empty() {
                None
            } else {
                Some(content)
            },
            note: None,
            issue_id,
        };

        if payload.content.is_some() || payload.issue_id.is_some() {
            spawn_local(async move {
                if post_json(
                    &format!("/api/v1/repos/{}/{}/projects/columns/{}/cards", o, r, c),
                    &payload,
                )
                .await
                {
                    set_new_card_content.set("".to_string());
                    set_issue_id_input.set("".to_string());
                    set_card_error.set(None);
                    set_refresh_cards.update(|n| *n += 1);
                } else {
                    set_card_error.set(Some(crate::api::WRITE_ERROR.to_string()));
                }
            });
        }
    };

    view! {
        <div class="column">
            <h4 class="mt-0">{column.title}</h4>
            <div class="cards">
                <Suspense fallback=move || view! { <div>"Loading..."</div> }>
                    {move || cards.get().map(|list| view! {
                        <For each=move || list.clone() key=|card| card.id children=move |card| {
                            let issue_link = card.issue_id.map(|id| format!("Issue #{}", id));
                            view! {
                                <div class="card">
                                    {if let Some(link) = issue_link {
                                        view! { <div class="text-accent">{link}</div> }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
                                    }}
                                    <div>{card.content.unwrap_or_default()}</div>
                                </div>
                            }
                        }/>
                    })}
                </Suspense>
            </div>
             <div class="add-card mt-1">
                <textarea placeholder="Card content..." prop:value=new_card_content on:input=move |ev| set_new_card_content.set(event_target_value(&ev)) class="box-border"></textarea>

                <Suspense fallback=move || view! { <span style="font-size: small">"Loading issues..."</span> }>
                    {move || issues.get().map(|list| view! {
                        <select on:change=move |ev| set_issue_id_input.set(event_target_value(&ev))>
                            <option value="">"Select Issue (Optional)"</option>
                            <For each=move || list.clone() key=|i| i.id children=move |i| {
                                view! { <option value={i.id}>"#" {i.number} " " {i.title}</option> }
                            }/>
                        </select>
                    })}
                </Suspense>

                <button on:click=on_add_card class="w-100">"Add Card"</button>
                {move || card_error.get().map(|msg| view! {
                    <p class="form-error" role="alert">{msg}</p>
                })}
            </div>
        </div>
    }
}
