use crate::api::{get_opt, local_resource, post, WRITE_ERROR};
use crate::components::RepoNav;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::*;
use shared::SecurityScanReport;

#[component]
pub fn SecurityDashboard() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|params| params.get("owner").unwrap_or_default());
    let repo_name = move || params.with(|params| params.get("repo").unwrap_or_default());

    let (scanning, set_scanning) = signal(false);
    let (refresh, set_refresh) = signal(0);
    let (scan_error, set_scan_error) = signal(Option::<String>::None);

    let report = local_resource(
        move || (owner(), repo_name(), refresh.get()),
        |(o, r, _)| async move {
            get_opt::<SecurityScanReport>(&format!("/api/v1/repos/{}/{}/security/scan", o, r)).await
        },
    );

    let on_run_scan = move |_| {
        let o = owner();
        let r = repo_name();
        set_scanning.set(true);
        spawn_local(async move {
            if post(&format!("/api/v1/repos/{}/{}/security/scan", o, r)).await {
                set_scan_error.set(None);
                set_refresh.update(|n| *n += 1);
            } else {
                set_scan_error.set(Some(WRITE_ERROR.to_string()));
            }
            set_scanning.set(false);
        });
    };

    view! {
            <div class="security-dashboard">
            <RepoNav/>
                <div class="security-header">
                    <h2>"🛡️ Security Audit & Vulnerability Scanner"</h2>
                    <button
                        class="btn-primary"
                        on:click=on_run_scan
                        disabled=move || scanning.get()
    >
                        {move || if scanning.get() { "Scanning..." } else { "Run Security Audit" }}
                    </button>
                    {move || scan_error.get().map(|msg| view! {
                        <p class="form-error" role="alert">{msg}</p>
                    })}
                </div>

                <Suspense fallback=move || view! { <div>"Loading security report..."</div> }>
                    {move || report.get().map(|rep| match rep {
                        Some(rep) => {
                            let score_color = if rep.score>= 80 { "#2da44e" } else if rep.score>= 50 { "#d97706" } else { "#cf222e" };
                            view! {
                                <div>
                                    <div class="security-score">
                                        <div class="score-ring" style=format!("color: {score_color}; border-color: {score_color}")>
                                            {rep.score}
                                        </div>
                                        <div>
                                            <h3 class="mt-0">"Security Health Score: " {rep.score} "/100"</h3>
                                            <p class="text-small text-muted mt-1">"Scanned at: " {rep.scanned_at}</p>
                                        </div>
                                    </div>

                                    <h3>"Vulnerabilities Found (" {rep.vulnerabilities.len()} ")"</h3>
                                    {if rep.vulnerabilities.is_empty() {
                                        view! {
                                            <div class="security-clear">
                                                "✅ No vulnerabilities or secret leaks detected in repository!"
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <div class="vulnerability-list">
                                                <For each=move || rep.vulnerabilities.clone() key=|v| v.id.clone() children=move |v| {
                                                    let badge_bg = match v.severity.as_str() {
                                                        "CRITICAL" => "#ffebe9",
                                                        "HIGH" => "#fff8c5",
                                                        _ => "#ddf4ff",
                                                    };
                                                    let badge_color = match v.severity.as_str() {
                                                        "CRITICAL" => "#cf222e",
                                                        "HIGH" => "#9a6700",
                                                        _ => "#0969da",
                                                    };
                                                    view! {
                                                        <div class="vulnerability" style=format!("background: {badge_bg}")>
                                                            <div class="flex-between">
                                                                <strong style=format!("color: {badge_color}")>
                                                                    "[" {v.severity} "] " {v.title}
                                                                </strong>
                                                                <span class="text-small text-muted">{v.id}</span>
                                                            </div>
                                                            <p class="vulnerability-desc">{v.description}</p>
                                                            <div class="vulnerability-detail">
                                                                <strong>"File: "</strong> {v.file_path} " (Line " {v.line_no} ")"<br/>
                                                                <strong>"Recommendation: "</strong> {v.recommendation}
                                                            </div>
                                                        </div>
                                                    }
                                                }/>
                                            </div>
                                        }.into_any()
                                    }}
                                </div>
                            }.into_any()
                        },
                        None => view! { <div>"Failed to load scan report."</div> }.into_any(),
                    })}
                </Suspense>
            </div>
        }
}
