use crate::components::*;
use crate::pages::*;
use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;

mod api;
mod components;
mod pages;

fn main() {
    mount_to_body(|| view! { <App/> })
}

/// Root component: mounts the top navigation and the router outlet.
#[component]
fn App() -> impl IntoView {
    view! {
        <Router>
            <Nav/>
            <main class="page">
                <Routes fallback=|| view! { <div class="not-found">"404 - Page not found"</div> }>
                    <Route path=path!("/") view=UserDashboard/>
                    <Route path=path!("/explore") view=Explore/>
                    <Route path=path!("/admin") view=AdminDashboard/>
                    <Route path=path!("/admin/users") view=AdminUsers/>
                    <Route path=path!("/search") view=Search/>
                    <Route path=path!("/repos/:owner/:repo/search") view=RepoCodeSearch/>
                    <Route path=path!("/packages/:owner") view=PackageList/>
                    <Route path=path!("/packages/:owner/:type/:name/:version") view=PackageDetail/>
                    <Route path=path!("/notifications") view=NotificationList/>
                    <Route path=path!("/login") view=Login/>
                    <Route path=path!("/register") view=Register/>
                    <Route path=path!("/repo/create") view=CreateRepo/>
                    <Route path=path!("/repo/migrate") view=MigrateRepo/>
                    <Route path=path!("/org/create") view=CreateOrg/>
                    <Route path=path!("/users/:username") view=UserProfile/>
                    <Route path=path!("/users/:username/followers") view=UserFollowers/>
                    <Route path=path!("/users/:username/following") view=UserFollowing/>
                    <Route path=path!("/settings/profile") view=UserSettings/>
                    <Route path=path!("/orgs/:org") view=OrgProfile/>
                    <Route path=path!("/repos/:owner/:repo") view=RepoDetail/>
                    <Route path=path!("/repos/:owner/:repo/issues") view=IssueList/>
                    <Route path=path!("/repos/:owner/:repo/issues/:index") view=IssueDetail/>
                    <Route path=path!("/repos/:owner/:repo/pulls") view=PullRequestList/>
                    <Route path=path!("/repos/:owner/:repo/pulls/:index") view=PullRequestDetail/>
                    <Route path=path!("/repos/:owner/:repo/compare") view=CompareView/>
                    <Route path=path!("/repos/:owner/:repo/actions") view=ActionsList/>
                    <Route path=path!("/repos/:owner/:repo/security") view=security::SecurityDashboard/>
                    <Route path=path!("/repos/:owner/:repo/pulse") view=RepoPulse/>
                    <Route path=path!("/repos/:owner/:repo/actions/workflows/:id") view=WorkflowRunsList/>
                    <Route path=path!("/repos/:owner/:repo/branches") view=BranchList/>
                    <Route path=path!("/repos/:owner/:repo/tags") view=TagList/>
                    <Route path=path!("/repos/:owner/:repo/src/*path") view=RepoCode/>
                    <Route path=path!("/repos/:owner/:repo/commits") view=CommitList/>
                    <Route path=path!("/repos/:owner/:repo/commits/:sha") view=CommitDiff/>
                    <Route path=path!("/repos/:owner/:repo/releases") view=ReleaseList/>
                    <Route path=path!("/repos/:owner/:repo/releases/new") view=ReleaseCreate/>
                    <Route path=path!("/repos/:owner/:repo/releases/:id") view=ReleaseDetail/>
                    <Route path=path!("/repos/:owner/:repo/labels") view=LabelList/>
                    <Route path=path!("/repos/:owner/:repo/milestones") view=MilestoneList/>
                    <Route path=path!("/repos/:owner/:repo/milestones/:index") view=MilestoneDetail/>
                    <Route path=path!("/repos/:owner/:repo/projects") view=ProjectList/>
                    <Route path=path!("/repos/:owner/:repo/projects/:id") view=ProjectDetail/>
                    <Route path=path!("/repos/:owner/:repo/discussions") view=DiscussionList/>
                    <Route path=path!("/repos/:owner/:repo/discussions/:id") view=DiscussionDetail/>
                    <Route path=path!("/repos/:owner/:repo/wiki") view=Wiki/>
                    <Route path=path!("/repos/:owner/:repo/wiki/pages/:page_name") view=Wiki/>
                    <Route path=path!("/repos/:owner/:repo/wiki/pages/:page_name/edit") view=WikiEdit/>
                    <Route path=path!("/repos/:owner/:repo/edit/*path") view=FileEdit/>
                    <Route path=path!("/repos/:owner/:repo/settings") view=RepoSettings/>
                    <Route path=path!("/repos/:owner/:repo/collaborators") view=CollaboratorList/>
                    <Route path=path!("/repos/:owner/:repo/settings/webhooks") view=WebhookList/>
                    <Route path=path!("/repos/:owner/:repo/settings/secrets") view=SecretList/>
                    <Route path=path!("/repos/:owner/:repo/settings/keys") view=DeployKeyList/>
                    <Route path=path!("/repos/:owner/:repo/settings/branches") view=ProtectedBranchList/>
                    <Route path=path!("/repos/:owner/:repo/settings/lfs") view=LfsLockList/>
                </Routes>
            </main>
            <footer class="site-footer">
                "Codeza — a lightweight, Gitea-inspired code hosting platform built with Rust."
            </footer>
        </Router>
    }
}
