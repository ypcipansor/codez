//! Repository overview, source browser, commits, branches and tags.

use crate::api::{get, get_or, get_text, local_resource, post_json, put, put_json, WRITE_ERROR};
use crate::components::{RepoNav, RepoRefresh};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::*;
use leptos_router::NavigateOptions;
use shared::{
    Branch, CodeSearchResult, Collaborator, Commit, CommitStatus, DiffFile, FileEntry,
    LanguageStat, MigrateRepoOption, RepoPulseStats, RepoTopicOptions, Repository, Tag,
    UpdateFileOption,
};

#[component]
pub fn RepoDetail() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|params| params.get("owner").unwrap_or_default());
    let repo_name = move || params.with(|params| params.get("repo").unwrap_or_default());
    let trigger_refresh = RwSignal::new(0u32);
    // Share the invalidation trigger with RepoNav so a star/watch action (which
    // changes the server-side counts) refreshes both the header and this
    // overview instead of leaving the overview stats stale.
    provide_context(RepoRefresh(trigger_refresh));

    let repo = local_resource(
        move || (owner(), repo_name(), trigger_refresh.get()),
        |(o, r, _)| async move {
            get_or::<Option<Repository>>(&format!("/api/v1/repos/{}/{}", o, r), None).await
        },
    );

    let languages = local_resource(
        move || (owner(), repo_name()),
        |(o, r)| async move {
            get::<Vec<LanguageStat>>(&format!("/api/v1/repos/{}/{}/languages", o, r)).await
        },
    );

    let topics = local_resource(
        move || (owner(), repo_name(), trigger_refresh.get()),
        |(o, r, _)| async move {
            get::<Vec<shared::Topic>>(&format!("/api/v1/repos/{}/{}/topics", o, r)).await
        },
    );

    let (is_editing_topics, set_is_editing_topics) = signal(false);
    let (topics_input, set_topics_input) = signal("".to_string());
    let (topics_error, set_topics_error) = signal(Option::<String>::None);

    let start_editing_topics = move |_| {
        let current_topics = topics.get().unwrap_or_default();
        let topic_names: Vec<String> = current_topics.into_iter().map(|t| t.name).collect();
        set_topics_input.set(topic_names.join(", "));
        set_is_editing_topics.set(true);
    };

    let save_topics = move |_| {
        let o = owner();
        let r = repo_name();
        let input = topics_input.get();
        let topic_names: Vec<String> = input
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let payload = RepoTopicOptions {
            topics: topic_names,
        };

        spawn_local(async move {
            // Keep the editor open when the save is rejected so the typed topics
            // are not discarded.
            if put_json(&format!("/api/v1/repos/{}/{}/topics", o, r), &payload).await {
                set_is_editing_topics.set(false);
                set_topics_error.set(None);
                trigger_refresh.update(|n| *n += 1);
            } else {
                set_topics_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    view! {
            <div class="repo-detail">
                <RepoNav/>

                <div class="repo-languages mb-2">
                    <Suspense fallback=move || view! { <span></span> }>
                        {move || languages.get().map(|list| {
                            let list2 = list.clone();
                            view! {
                                <div class="language-bar">
                                    <For each=move || list.clone() key=|l| l.language.clone() children=move |l| {
                                        view! {
                                            <div
                                                style=format!("width: {}%; background-color: {};", l.percentage, l.color)
                                                title=format!("{} {}%", l.language, l.percentage)
    ></div>
                                        }
                                    }/>
                                </div>
                                <div class="language-legend">
                                    <For each=move || list2.clone() key=|l| l.language.clone() children=move |l| {
                                        view! {
                                            <span class="language-legend-item">
                                                <span style=format!("color: {}", l.color)>"● "</span>
                                                {l.language} " " {l.percentage} "%"
                                            </span>
                                        }
                                    }/>
                                </div>
                            }
                        })}
                    </Suspense>
                </div>

                <div class="flex-center mb-2">
                    {move || if is_editing_topics.get() {
                        view! {
                            <div class="flex gap-sm">
                                <input type="text" placeholder="rust, webassembly, leptos" prop:value=topics_input on:input=move |ev| set_topics_input.set(event_target_value(&ev)) />
                                <button class="btn-primary" on:click=save_topics>"Save"</button>
                                <button on:click=move |_| set_is_editing_topics.set(false)>"Cancel"</button>
                            </div>
                            {move || topics_error.get().map(|msg| view! {
                                <p class="form-error" role="alert">{msg}</p>
                            })}
                        }.into_any()
                    } else {
                        view! {
                            <Suspense fallback=move || view! { <span></span> }>
                                <div class="flex gap-sm flex-wrap">
                                    {move || topics.get().map(|list| view! {
                                        <For each=move || list.clone() key=|t| t.id children=move |t| {
                                            let link = format!("/search?q=topic:{}", t.name);
                                            view! { <a class="label label-accent" href=link>{t.name}</a> }
                                        }/>
                                    })}
                                    <button class="btn-sm" on:click=start_editing_topics>"Edit topics"</button>
                                </div>
                            </Suspense>
                        }.into_any()
                    }}
                </div>

                <Suspense fallback=move || view! { <p class="text-muted">"Loading…"</p> }>
                    {move || match repo.get() {
                        Some(Some(r)) => view! {
                            <div class="repo-stats mb-1">
                                <span title="Stars">"⭐ " {r.stars_count}</span>
                                <span title="Forks">"🍴 " {r.forks_count}</span>
                                <span title="Watchers">"👁️ " {r.watchers_count}</span>
                            </div>
                            <p class="text-muted">
                                "Clone URL: "
                                <code>"https://codeza.com/" {r.owner} "/" {r.name} ".git"</code>
                            </p>
                        }.into_any(),
                        _ => view! { <div class="empty-state">"Repository not found."</div> }.into_any()
                    }}
                </Suspense>
            </div>
        }
}

#[component]
pub fn RepoPulse() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|params| params.get("owner").unwrap_or_default());
    let repo_name = move || params.with(|params| params.get("repo").unwrap_or_default());

    let (period, set_period) = signal("weekly".to_string());

    let stats = local_resource(
        move || (owner(), repo_name(), period.get()),
        |(o, r, p)| async move {
            get_or::<RepoPulseStats>(
                &format!("/api/v1/repos/{}/{}/pulse?period={}", o, r, p),
                RepoPulseStats {
                    period: "weekly".to_string(),
                    active_issues: 0,
                    closed_issues: 0,
                    opened_prs: 0,
                    merged_prs: 0,
                    new_commits: 0,
                    active_authors: vec![],
                },
            )
            .await
        },
    );

    view! {
        <div class="repo-pulse">
        <RepoNav/>
            <h3>"Pulse (" {period} ")"</h3>
            <div class="pulse-controls mb-2">
                <button on:click=move |_| set_period.set("daily".to_string()) style=move || if period.get() == "daily" { "font-weight: bold;" } else { "" }>"Daily"</button>
                <button on:click=move |_| set_period.set("weekly".to_string()) style=move || if period.get() == "weekly" { "font-weight: bold; margin-left: 5px;" } else { "margin-left: 5px;" }>"Weekly"</button>
                <button on:click=move |_| set_period.set("monthly".to_string()) style=move || if period.get() == "monthly" { "font-weight: bold; margin-left: 5px;" } else { "margin-left: 5px;" }>"Monthly"</button>
            </div>

            <Suspense fallback=move || view! { <p>"Loading stats..."</p> }>
                {move || stats.get().map(|s| view! {
                    <div class="pulse-grid">
                        <div class="stat-card">
                            <div class="stat-value">{s.active_issues}</div>
                            <div>"Active Issues"</div>
                        </div>
                        <div class="stat-card">
                            <div class="stat-value">{s.closed_issues}</div>
                            <div>"Closed Issues"</div>
                        </div>
                        <div class="stat-card">
                            <div class="stat-value">{s.opened_prs}</div>
                            <div>"Opened PRs"</div>
                        </div>
                        <div class="stat-card">
                            <div class="stat-value">{s.merged_prs}</div>
                            <div>"Merged PRs"</div>
                        </div>
                        <div class="stat-card">
                            <div class="stat-value">{s.new_commits}</div>
                            <div>"New Commits"</div>
                        </div>
                    </div>

                    <div class="active-authors">
                        <h4>"Active Contributors"</h4>
                        {if s.active_authors.is_empty() {
                            view! { <p>"No active contributors in this period."</p> }.into_any()
                        } else {
                            view! {
                                <ul>
                                    <For each=move || s.active_authors.clone() key=|u| u.id children=move |u| {
                                        view! { <li>{u.username}</li> }
                                    }/>
                                </ul>
                            }.into_any()
                        }}
                    </div>
                })}
            </Suspense>
        </div>
    }
}

#[component]
pub fn RepoCode() -> impl IntoView {
    let params = use_params_map();
    let query = use_query_map();
    let navigate = use_navigate();
    let owner = move || params.with(|params| params.get("owner").unwrap_or_default());
    let repo_name = move || params.with(|params| params.get("repo").unwrap_or_default());
    let path = move || params.with(|params| params.get("path").unwrap_or_default());
    let branch_ref = move || query.with(|q| q.get("ref").unwrap_or_default());

    let branches = local_resource(
        move || (owner(), repo_name()),
        |(o, r)| async move { get::<Vec<Branch>>(&format!("/api/v1/repos/{}/{}/branches", o, r)).await },
    );

    let contents = local_resource(
        move || (owner(), repo_name(), path(), branch_ref()),
        |(o, r, p, b)| async move {
            let mut url = if p.is_empty() {
                format!("/api/v1/repos/{}/{}/contents", o, r)
            } else {
                format!("/api/v1/repos/{}/{}/contents/{}", o, r, p)
            };
            if !b.is_empty() {
                url.push_str(&format!("?ref={}", b));
            }
            get::<Vec<FileEntry>>(&url).await
        },
    );

    let repo_meta = local_resource(
        move || (owner(), repo_name()),
        |(o, r)| async move {
            get_or::<Option<Repository>>(&format!("/api/v1/repos/{}/{}", o, r), None).await
        },
    );

    let nav_ref = StoredValue::new(navigate);
    let owner_ref = StoredValue::new(owner);
    let repo_ref = StoredValue::new(repo_name);
    let path_ref = StoredValue::new(path);

    view! {
        <div class="repo-code">
        <RepoNav/>
            <div class="mb-1">
                <Suspense fallback=move || view! { <select disabled><option>"Loading branches..."</option></select> }>
                    {move || {
                        let repo_data = repo_meta.get().flatten();
                        let default_branch = repo_data.and_then(|r| r.default_branch).unwrap_or("main".to_string());
                        let current = if branch_ref().is_empty() { default_branch } else { branch_ref() };
                        branches.get().map(move |list| {
                            view! {
                                <select on:change=move |ev| {
                                    let val = event_target_value(&ev);
                                let p = path_ref.with_value(|p| p());
                                let o = owner_ref.with_value(|o| o());
                                let r = repo_ref.with_value(|r| r());
                                let url = if p.is_empty() {
                                    format!("/repos/{}/{}/src", o, r)
                                } else {
                                    format!("/repos/{}/{}/src/{}", o, r, p)
                                };
                                nav_ref.with_value(|n| n(&format!("{}?ref={}", url, val), Default::default()));
                            }>
                                <For each=move || list.clone() key=|b| b.name.clone() children=move |b| {
                                    let selected = b.name == current;
                                    view! { <option value={b.name.clone()} selected={selected}>{b.name.clone()}</option> }
                                }/>
                            </select>
                            }
                        })
                    }}
                </Suspense>
            </div>
            <h3>"Files in " {move || if path().is_empty() { "root".to_string() } else { path() }}</h3>
            <div class="code-search-link">
                <a href="search">"Search Code"</a>
            </div>
            <ul>
                <Suspense fallback=move || view! { <li>"Loading files..."</li> }>
                    {move || contents.get().map(|files| {
                        if files.is_empty() {
                             view! { <li>"No files found or empty directory."</li> }.into_any()
                        } else {
                            view! {
                                <For each=move || files.clone() key=|f| f.path.clone() children=move |f| {
                                    let is_dir = f.kind == "dir";
                                    let link = format!("/repos/{}/{}/src/{}", owner(), repo_name(), f.path);

                                    view! {
                                        <li>
                                            {if is_dir { "📁 " } else { "📄 " }}
                                            <a href=move || {
                                                if branch_ref().is_empty() {
                                                    link.clone()
                                                } else {
                                                    format!("{}?ref={}", link, branch_ref())
                                                }
                                            }>{f.name}</a>
                                            " (" {f.size} " bytes)"
                                            {if !is_dir {
                                                let edit_link = format!("/repos/{}/{}/edit/{}", owner(), repo_name(), f.path);
                                                let final_edit_link = if branch_ref().is_empty() { edit_link } else { format!("{}?ref={}", edit_link, branch_ref()) };
                                                view! { <a href=final_edit_link class="ml-2">"Edit"</a> }.into_any()
                                            } else {
                                                view! { <span></span> }.into_any()
                                            }}
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
pub fn FileEdit() -> impl IntoView {
    let params = use_params_map();
    let query = use_query_map();
    let owner = move || params.with(|params| params.get("owner").unwrap_or_default());
    let repo_name = move || params.with(|params| params.get("repo").unwrap_or_default());
    let path = move || params.with(|params| params.get("path").unwrap_or_default());
    let branch_ref = move || query.with(|q| q.get("ref").unwrap_or_default());

    let (content, set_content) = signal("".to_string());
    let (message, set_message) = signal("Update file".to_string());
    let (save_error, set_save_error) = signal(Option::<String>::None);

    let _ = local_resource(
        move || (owner(), repo_name(), path(), branch_ref()),
        move |(o, r, p, b)| async move {
            let mut url = format!("/api/v1/repos/{}/{}/raw/{}", o, r, p);
            if !b.is_empty() {
                url.push_str(&format!("?ref={}", b));
            }
            set_content.set(get_text(&url).await);
        },
    );

    let on_save = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let b = branch_ref();
        let branch_val = if b.is_empty() { None } else { Some(b) };

        let payload = UpdateFileOption {
            content: content.get(),
            message: message.get(),
            sha: "mock_sha".to_string(),
            branch: branch_val,
        };
        let o = owner();
        let r = repo_name();
        let p = path();
        spawn_local(async move {
            if put_json(
                &format!("/api/v1/repos/{}/{}/contents/{}", o, r, p),
                &payload,
            )
            .await
            {
                set_save_error.set(None);
            } else {
                set_save_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    view! {
        <div class="file-edit">
        <RepoNav/>
            <h3>"Editing " {path}</h3>
            <form on:submit=on_save>
                <textarea prop:value=content on:input=move |ev| set_content.set(event_target_value(&ev)) rows="20" class="w-100"></textarea>
                <input type="text" prop:value=message on:input=move |ev| set_message.set(event_target_value(&ev)) placeholder="Commit message" class="my-2" />
                <button type="submit">"Commit Changes"</button>
                {move || save_error.get().map(|msg| view! {
                    <p class="form-error" role="alert">{msg}</p>
                })}
            </form>
        </div>
    }
}

// ... rest of the file (IssueList, IssueDetail, CreateRepo, MigrateRepo, etc.) ...
// I will include the rest of the file content from my previous read/writes to ensure no data loss.

#[component]
pub fn CommitList() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|params| params.get("owner").unwrap_or_default());
    let repo_name = move || params.with(|params| params.get("repo").unwrap_or_default());

    let commits = local_resource(
        move || (owner(), repo_name()),
        |(o, r)| async move { get::<Vec<Commit>>(&format!("/api/v1/repos/{}/{}/commits", o, r)).await },
    );

    view! {
        <div class="commit-list">
        <RepoNav/>
            <h3>"Commit History for " {owner} "/" {repo_name}</h3>
            <ul>
                <Suspense fallback=move || view! { <li>"Loading commits..."</li> }>
                    {move || commits.get().map(|list| view! {
                        <For each=move || list.clone() key=|c| c.sha.clone() children=move |c| {
                            let href = format!("/repos/{}/{}/commits/{}", owner(), repo_name(), c.sha);
                            view! {
                                <li>
                                    <a href=href class="commit-sha">{c.sha.chars().take(7).collect::<String>()}</a>
                                    " - "
                                    <span class="commit-message">{c.message}</span>
                                    " by "
                                    <span class="commit-author">{c.author.username}</span>
                                    " on "
                                    <span class="commit-date">{c.date}</span>
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
pub fn CommitDiff() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|params| params.get("owner").unwrap_or_default());
    let repo_name = move || params.with(|params| params.get("repo").unwrap_or_default());
    let sha = move || params.with(|params| params.get("sha").unwrap_or_default());

    let diffs = local_resource(
        move || (owner(), repo_name(), sha()),
        |(o, r, s)| async move {
            get::<Vec<DiffFile>>(&format!("/api/v1/repos/{}/{}/commits/{}/diff", o, r, s)).await
        },
    );

    view! {
        <div class="commit-diff">
        <RepoNav/>
            <h3>"Commit Diff: " {sha}</h3>
            <div class="diff-container">
                <Suspense fallback=move || view! { <p>"Loading diff..."</p> }>
                    {move || diffs.get().map(|files| view! {
                        <For each=move || files.clone() key=|f| f.name.clone() children=move |f| {
                            view! {
                                <div class="file-diff">
                                    <div class="file-header">
                                        <strong>{f.name}</strong>
                                        <span class="diff-stats">
                                            " +"{f.additions} " -"{f.deletions}
                                        </span>
                                    </div>
                                    <pre class="diff-content">
                                        <For each=move || f.lines.clone() key=|l| format!("{}{:?}", l.content, l.line_no_old) children=move |line| {
                                            let class_name = match line.type_.as_str() {
                                                "add" => "diff-line-add",
                                                "delete" => "diff-line-delete",
                                                _ => "diff-line-context",
                                            };
                                            view! { <div class=class_name>{line.content}</div> }
                                        }/>
                                    </pre>
                                </div>
                            }
                        }/>
                    })}
                </Suspense>
            </div>

        </div>
    }
}

#[component]
pub fn BranchList() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|params| params.get("owner").unwrap_or_default());
    let repo_name = move || params.with(|params| params.get("repo").unwrap_or_default());

    let branches = local_resource(
        move || (owner(), repo_name()),
        |(o, r)| async move { get::<Vec<Branch>>(&format!("/api/v1/repos/{}/{}/branches", o, r)).await },
    );

    view! {
        <div class="branch-list">
        <RepoNav/>
            <h3>"Branches for " {owner} "/" {repo_name}</h3>
            <ul>
                <Suspense fallback=move || view! { <li>"Loading branches..."</li> }>
                    {move || branches.get().map(|list| view! {
                        <For each=move || list.clone() key=|b| b.name.clone() children=move |b| {
                            view! {
                                <li>
                                    <strong>{b.name}</strong>
                                    {if b.protected { " (Protected)" } else { "" }}
                                    " - " {b.commit.sha.chars().take(7).collect::<String>()}
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
pub fn TagList() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|params| params.get("owner").unwrap_or_default());
    let repo_name = move || params.with(|params| params.get("repo").unwrap_or_default());

    let tags = local_resource(
        move || (owner(), repo_name()),
        |(o, r)| async move { get::<Vec<Tag>>(&format!("/api/v1/repos/{}/{}/tags", o, r)).await },
    );

    view! {
        <div class="tag-list">
        <RepoNav/>
            <h3>"Tags for " {owner} "/" {repo_name}</h3>
            <ul>
                <Suspense fallback=move || view! { <li>"Loading tags..."</li> }>
                    {move || tags.get().map(|list| view! {
                        <For each=move || list.clone() key=|t| t.name.clone() children=move |t| {
                            view! {
                                <li>
                                    <strong>{t.name}</strong>
                                    " - " {t.commit.sha.chars().take(7).collect::<String>()}
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
pub fn CollaboratorList() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|params| params.get("owner").unwrap_or_default());
    let repo_name = move || params.with(|params| params.get("repo").unwrap_or_default());
    let (new_collab, set_new_collab) = signal("".to_string());
    let (form_error, set_form_error) = signal(Option::<String>::None);

    let collabs = local_resource(
        move || (owner(), repo_name()),
        |(o, r)| async move {
            get::<Vec<Collaborator>>(&format!("/api/v1/repos/{}/{}/collaborators", o, r)).await
        },
    );

    let on_add = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let o = owner();
        let r = repo_name();
        let c = new_collab.get();
        spawn_local(async move {
            if put(&format!("/api/v1/repos/{}/{}/collaborators/{}", o, r, c)).await {
                set_new_collab.set("".to_string());
                set_form_error.set(None);
            } else {
                set_form_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    view! {
        <div class="collaborators">
        <RepoNav/>
            <h3>"Collaborators"</h3>
            <ul>
                <Suspense fallback=move || view! { <li>"Loading..."</li> }>
                    {move || collabs.get().map(|list| view! {
                        <For each=move || list.clone() key=|c| c.user.id children=move |c| {
                            view! { <li>{c.user.username} " (" {c.permissions} ")"</li> }
                        }/>
                    })}
                </Suspense>
            </ul>
            <form on:submit=on_add>
                <input type="text" placeholder="Username" prop:value=new_collab on:input=move |ev| set_new_collab.set(event_target_value(&ev)) />
                <button type="submit">"Add Collaborator"</button>
                {move || form_error.get().map(|msg| view! {
                    <p class="form-error" role="alert">{msg}</p>
                })}
            </form>
        </div>
    }
}

#[component]
pub fn RepoCodeSearch() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|params| params.get("owner").unwrap_or_default());
    let repo_name = move || params.with(|params| params.get("repo").unwrap_or_default());
    let (query, set_query) = signal("".to_string());

    // Trigger resource when query changes (and is not empty)
    let search_results = local_resource(
        move || (owner(), repo_name(), query.get()),
        |(o, r, q)| async move {
            if q.is_empty() {
                return vec![];
            }
            get::<Vec<CodeSearchResult>>(&format!("/api/v1/repos/{}/{}/search?q={}", o, r, q)).await
        },
    );

    let on_search = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        // search_results resource will auto-update because it depends on `query()`
    };

    view! {
        <div class="repo-search">
        <RepoNav/>
            <h3>"Search Code"</h3>
            <form on:submit=on_search>
                <input type="text" prop:value=query on:input=move |ev| set_query.set(event_target_value(&ev)) placeholder="Search..."/>
                <button type="submit">"Search"</button>
            </form>
            <ul>
                <Suspense fallback=move || view! { <li>"Searching..."</li> }>
                    {move || search_results.get().map(|list| view! {
                        <For each=move || list.clone() key=|r| r.path.clone() children=move |r| {
                            let path_clone = r.path.clone();
                            view! {
                                <li>
                                    <strong>{r.path}</strong>
                                    // Link to line number or file
                                    <a href=format!("src/{}", path_clone)>"View"</a>
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
pub fn CommitStatusList(owner: String, repo: String, sha: String) -> impl IntoView {
    let statuses = local_resource(
        move || (owner.clone(), repo.clone(), sha.clone()),
        |(o, r, s)| async move {
            get::<Vec<CommitStatus>>(&format!("/api/v1/repos/{}/{}/commits/{}/statuses", o, r, s))
                .await
        },
    );

    view! {
        <div class="commit-statuses">
            <h4>"Checks"</h4>
            <Suspense fallback=move || view! { <span>"Loading checks..."</span> }>
                {move || statuses.get().map(|list| {
                    if list.is_empty() {
                        view! { <div>"No checks run."</div> }.into_any()
                    } else {
                        view! {
                            <ul class="list-reset">
                                <For each=move || list.clone() key=|s| s.id children=move |s| {
                                    let color = match s.state.as_str() {
                                        "success" => "green",
                                        "failure" | "error" => "red",
                                        _ => "orange",
                                    };
                                    let icon = match s.state.as_str() {
                                        "success" => "✔",
                                        "failure" | "error" => "✘",
                                        _ => "●",
                                    };
                                    view! {
                                        <li class="flex-center">
                                            <span style=format!("color: {}; margin-right: 10px; font-weight: bold;", color)>{icon}</span>
                                            <strong class="mr-2">{s.context}</strong>
                                            <span>{s.description.unwrap_or_default()}</span>
                                        </li>
                                    }
                                }/>
                            </ul>
                        }.into_any()
                    }
                })}
            </Suspense>
        </div>
    }
}

#[component]
pub fn MigrateRepo() -> impl IntoView {
    let (clone_addr, set_clone_addr) = signal("".to_string());
    let (repo_name, set_repo_name) = signal("".to_string());
    let (service, set_service) = signal("git".to_string());
    let (form_error, set_form_error) = signal(Option::<String>::None);
    let navigate = use_navigate();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let payload = MigrateRepoOption {
            clone_addr: clone_addr.get(),
            repo_name: repo_name.get(),
            service: service.get(),
            mirror: false,
        };
        let r_name = repo_name.get();
        let navigate = navigate.clone();
        spawn_local(async move {
            // Only navigate on success; a rejected migration keeps the form so the
            // clone URL and name can be corrected.
            if post_json("/api/v1/repos/migrate", &payload).await {
                set_form_error.set(None);
                navigate(
                    &format!("/repos/admin/{}", r_name),
                    NavigateOptions::default(),
                );
            } else {
                set_form_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    view! {
        <div class="migrate-repo">
            <h3>"Migrate Repository"</h3>
            <form on:submit=on_submit>
                <div class="form-group">
                    <label>"Migration Service"</label>
                    <select prop:value=service on:change=move |ev| set_service.set(event_target_value(&ev))>
                        <option value="git">"Git"</option>
                        <option value="github">"GitHub"</option>
                        <option value="gitlab">"GitLab"</option>
                        <option value="gitea">"Gitea"</option>
                    </select>
                </div>
                <div class="form-group">
                    <label>"Clone URL"</label>
                    <input type="text" placeholder="Clone URL" prop:value=clone_addr on:input=move |ev| set_clone_addr.set(event_target_value(&ev)) />
                </div>
                <div class="form-group">
                    <label>"Repository Name"</label>
                    <input type="text" placeholder="Repository Name" prop:value=repo_name on:input=move |ev| set_repo_name.set(event_target_value(&ev)) />
                </div>
                <button type="submit">"Migrate"</button>
                {move || form_error.get().map(|msg| view! {
                    <p class="form-error" role="alert">{msg}</p>
                })}
            </form>
        </div>
    }
}
