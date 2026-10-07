use leptos::prelude::*;
use leptos_router::hooks::*;
use wasm_bindgen::JsCast;

/// Application-wide top navigation bar.
///
/// Renders the brand, primary links and a global search box that submits to
/// the `/search` route. Active route highlighting is handled by the router.
#[component]
pub fn Nav() -> impl IntoView {
    let location = use_location();
    let navigate = use_navigate();

    let is_active = move |path: &str| {
        let current = location.pathname.get();
        if path == "/" {
            current == "/"
        } else {
            current == path || current.starts_with(&format!("{path}/"))
        }
    };

    let on_search = {
        let navigate = navigate.clone();
        move |ev: leptos::ev::SubmitEvent| {
            ev.prevent_default();
            let input = ev
                .target()
                .and_then(|t| t.dyn_into::<web_sys::HtmlFormElement>().ok())
                .and_then(|form| form.elements().named_item("q"))
                .and_then(|el| el.dyn_into::<web_sys::HtmlInputElement>().ok())
                .map(|i| i.value())
                .unwrap_or_default();
            navigate(
                &format!("/search?q={}", crate::api::encode_query(&input)),
                Default::default(),
            );
        }
    };

    view! {
        <header class="app-header">
            <a class="brand" href="/">
                <span class="logo">"</>"</span>
                <span>"Codeza"</span>
            </a>
            <nav class="nav-links">
                <a href="/" class:active=move || is_active("/")>"Dashboard"</a>
                <a href="/explore" class:active=move || is_active("/explore")>"Explore"</a>
                <a href="/packages/admin" class:active=move || is_active("/packages")>"Packages"</a>
                <a href="/notifications" class:active=move || is_active("/notifications")>"Notifications"</a>
                <a href="/admin" class:active=move || is_active("/admin")>"Admin"</a>
                // The header search box is hidden at narrow widths, so keep a
                // plain link as the search entry point on small screens.
                <a href="/search" class="search-link" class:active=move || is_active("/search")>"Search"</a>
            </nav>
            <div class="spacer"></div>
            <form on:submit=on_search>
                <input
                    class="global-search"
                    type="search"
                    name="q"
                    placeholder="Search repositories or issues…"
                    aria-label="Search"
                />
            </form>
            <nav class="nav-links">
                <a href="/login">"Sign in"</a>
                <a href="/register">"Register"</a>
            </nav>
        </header>
    }
}
