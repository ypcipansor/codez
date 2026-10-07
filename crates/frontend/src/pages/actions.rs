use crate::api::{delete, get, local_resource, patch_json, post, post_json, WRITE_ERROR};
use crate::components::RepoNav;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::*;
use shared::{ActionWorkflow, CreateWorkflowRunOption, UpdateWorkflowRunOption, WorkflowRun};

#[component]
pub fn ActionsList() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|params| params.get("owner").unwrap_or_default());
    let repo_name = move || params.with(|params| params.get("repo").unwrap_or_default());

    let workflows = local_resource(
        move || (owner(), repo_name()),
        |(o, r)| async move {
            get::<Vec<ActionWorkflow>>(&format!("/api/v1/repos/{}/{}/actions/workflows", o, r))
                .await
        },
    );

    view! {
        <div class="actions-list">
        <RepoNav/>
            <h3>"Actions Workflows"</h3>
            <ul>
                <Suspense fallback=move || view! { <li>"Loading workflows..."</li> }>
                    {move || workflows.get().map(|list| view! {
                        <For each=move || list.clone() key=|w| w.id children=move |w| {
                            let href = format!("/repos/{}/{}/actions/workflows/{}", owner(), repo_name(), w.id);
                            view! {
                                <li>
                                    <a href=href><strong>{w.name}</strong></a>
                                    " - " {w.status}
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
pub fn WorkflowRunsList() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|params| params.get("owner").unwrap_or_default());
    let repo_name = move || params.with(|params| params.get("repo").unwrap_or_default());
    let workflow_id = move || {
        params.with(|params| {
            params
                .get("id")
                .unwrap_or_default()
                .parse::<u64>()
                .unwrap_or_default()
        })
    };

    let (refresh, set_refresh) = signal(0);
    let (action_error, set_action_error) = signal(Option::<String>::None);

    let runs = local_resource(
        move || (owner(), repo_name(), workflow_id(), refresh.get()),
        |(o, r, id, _)| async move {
            get::<Vec<WorkflowRun>>(&format!(
                "/api/v1/repos/{}/{}/actions/workflows/{}/runs",
                o, r, id
            ))
            .await
        },
    );

    let on_run_workflow = move |_| {
        let o = owner();
        let r = repo_name();
        let id = workflow_id();
        let payload = CreateWorkflowRunOption {
            workflow_id: id,
            ref_name: "main".to_string(), // hardcoded for MVP
        };
        spawn_local(async move {
            if post_json(
                &format!("/api/v1/repos/{}/{}/actions/workflows/{}/runs", o, r, id),
                &payload,
            )
            .await
            {
                set_action_error.set(None);
                set_refresh.update(|n| *n += 1);
            } else {
                set_action_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    let on_rerun_workflow = move |run_id: u64| {
        let o = owner();
        let r = repo_name();
        spawn_local(async move {
            if post(&format!(
                "/api/v1/repos/{}/{}/actions/runs/{}/rerun",
                o, r, run_id
            ))
            .await
            {
                set_action_error.set(None);
                set_refresh.update(|n| *n += 1);
            } else {
                set_action_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    let on_cancel_run = move |run_id: u64| {
        let o = owner();
        let r = repo_name();
        let payload = UpdateWorkflowRunOption {
            status: "cancelled".to_string(),
        };
        spawn_local(async move {
            if patch_json(
                &format!("/api/v1/repos/{}/{}/actions/runs/{}", o, r, run_id),
                &payload,
            )
            .await
            {
                set_action_error.set(None);
                set_refresh.update(|n| *n += 1);
            } else {
                set_action_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    let on_delete_run = move |run_id: u64| {
        let o = owner();
        let r = repo_name();
        spawn_local(async move {
            if delete(&format!(
                "/api/v1/repos/{}/{}/actions/runs/{}",
                o, r, run_id
            ))
            .await
            {
                set_action_error.set(None);
                set_refresh.update(|n| *n += 1);
            } else {
                set_action_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    view! {
        <div class="workflow-runs">
        <RepoNav/>
            <div class="header">
                <h3>"Workflow Runs"</h3>
                <button class="run-workflow-btn" on:click=on_run_workflow>"Run Workflow"</button>
                {move || action_error.get().map(|msg| view! {
                    <p class="form-error" role="alert">{msg}</p>
                })}
            </div>
            <ul class="runs-list">
                <Suspense fallback=move || view! { <li>"Loading runs..."</li> }>
                    {move || runs.get().map(|list| view! {
                        <For each=move || list.clone() key=|r| r.id children=move |r| {
                            let status = r.status.clone();
                            let is_active = status == "queued" || status == "in_progress";
                            let run_id = r.id;
                            let step_logs = r.step_logs.clone();
                            let on_cancel = { move |_| on_cancel_run(run_id) };
                            let on_delete = { move |_| on_delete_run(run_id) };
                            let on_rerun = { move |_| on_rerun_workflow(run_id) };
                            view! {
                                <li class="run-item">
                                    <div class="flex-between">
                                        <span>"Run #" {r.id} " - " <span class="run-status">{r.status}</span> " (" {r.created_at} ")"</span>
                                        <div class="flex gap-sm">
                                            <button class="rerun-btn" on:click=on_rerun>"Re-run"</button>
                                            {if is_active {
                                                view! { <button class="cancel-run-btn" on:click=on_cancel>"Cancel"</button> }.into_any()
                                            } else {
                                                view! { <button class="delete-run-btn" on:click=on_delete>"Delete"</button> }.into_any()
                                            }}
                                        </div>
                                    </div>
                                    {if !step_logs.is_empty() {
                                        view! {
                                            <div class="log-viewer">
                                                <For each=move || step_logs.clone() key=|s| s.name.clone() children=move |s| {
                                                    let logs = s.logs.clone();
                                                    view! {
                                                        <div class="log-step">
                                                            <div class="log-step-title">"▶ " {s.name} " (" {s.status} ")"</div>
                                                            <For each=move || logs.clone() key=|l| l.clone() children=move |l| {
                                                                view! { <div class="log-line">{l}</div> }
                                                            }/>
                                                        </div>
                                                    }
                                                }/>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
                                    }}
                                </li>
                            }
                        }/>
                    })}
                </Suspense>
            </ul>
        </div>
    }
}
