//! Typed HTTP helpers for talking to the Codeza API.
//!
//! Every page fetches through these helpers instead of hand-rolling
//! `Request::get(..).send().await.unwrap().json()` chains. Read helpers are
//! infallible from the caller's point of view: a transport or decode failure
//! yields `T::default()` (or `None` / a supplied fallback), which keeps Leptos
//! resources simple and keeps the UI rendering an empty state rather than
//! panicking. Write helpers return `bool` — `true` only when the request was
//! sent *and* the server replied `2xx` — so callers can clear a form, navigate
//! or refresh on success and surface an error on failure instead of discarding
//! a rejected write.
//!
//! Paths passed to these helpers are absolute API paths (`/api/v1/repos/owner/name`),
//! not API-relative ones; the helpers do not rewrite the URL. Build paths with
//! [`api_url`] (or format them explicitly) so the versioned base stays in one
//! place rather than being hand-written at every call site.

use gloo_net::http::Request;
use leptos::prelude::LocalResource;
use serde::{de::DeserializeOwned, Serialize};
use std::future::Future;

pub const BASE_URL: &str = "/api/v1";

/// Reactive async resource for client-side rendering.
///
/// `LocalResource` never has to cross a thread boundary, so unlike `Resource`
/// it accepts a non-`Send` future, which is what `gloo_net` request futures
/// are. `source` is tracked reactively; when it yields a new value the
/// `fetcher` runs again.
pub fn local_resource<S, T, Fut>(
    source: impl Fn() -> S + 'static,
    fetcher: impl Fn(S) -> Fut + 'static,
) -> LocalResource<T>
where
    S: PartialEq + 'static,
    T: 'static,
    Fut: Future<Output = T> + 'static,
{
    LocalResource::new(move || fetcher(source()))
}

/// Build an absolute API URL from a path relative to `/api/v1`.
pub fn api_url(path: &str) -> String {
    format!("{BASE_URL}{path}")
}

/// Percent-encode a string for use as a query-parameter value (RFC 3986
/// unreserved set). Keeps spaces, `&`, `#`, and other reserved characters from
/// breaking out of the query string.
pub fn encode_query(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

/// `GET url`, decoding `T`. Falls back to `T::default()` on error.
pub async fn get<T: DeserializeOwned + Default>(url: &str) -> T {
    match Request::get(url).send().await {
        Ok(resp) => resp.json::<T>().await.unwrap_or_default(),
        Err(_) => T::default(),
    }
}

/// `GET url`, decoding `T`. Returns `None` on error.
pub async fn get_opt<T: DeserializeOwned>(url: &str) -> Option<T> {
    match Request::get(url).send().await {
        Ok(resp) => resp.json::<T>().await.ok(),
        Err(_) => None,
    }
}

/// `GET url`, decoding `T`. Falls back to `fallback` on error.
pub async fn get_or<T: DeserializeOwned>(url: &str, fallback: T) -> T {
    match Request::get(url).send().await {
        Ok(resp) => resp.json::<T>().await.unwrap_or(fallback),
        Err(_) => fallback,
    }
}

/// `GET url` as raw text. Falls back to an empty string on error.
pub async fn get_text(url: &str) -> String {
    match Request::get(url).send().await {
        Ok(resp) => resp.text().await.unwrap_or_default(),
        Err(_) => String::new(),
    }
}

/// `GET url`, invoking `on_ok` with the decoded value when the request succeeds
/// and the body decodes. Used by screens that populate signals instead of
/// returning a value (e.g. form pre-fill).
pub async fn get_then<T, F>(url: &str, on_ok: F)
where
    T: DeserializeOwned,
    F: FnOnce(T),
{
    if let Ok(resp) = Request::get(url).send().await {
        if let Ok(value) = resp.json::<T>().await {
            on_ok(value);
        }
    }
}

/// Send a prepared write request and report whether the server accepted it.
///
/// Takes the *result* of building the request so a body that fails to serialize
/// is treated like any other failure. `true` only when the request was actually
/// sent *and* the response status is `2xx`; a transport error or a non-2xx
/// status both yield `false`, so callers never mistake a rejected write for a
/// successful one.
async fn send_ok(req: Result<Request, gloo_net::Error>) -> bool {
    match req {
        Ok(req) => matches!(req.send().await, Ok(resp) if resp.ok()),
        Err(_) => false,
    }
}

/// `POST url` with no body. Returns whether the server replied `2xx`.
pub async fn post(url: &str) -> bool {
    send_ok(Request::post(url).build()).await
}

/// `PATCH url` with no body. Returns whether the server replied `2xx`.
pub async fn patch(url: &str) -> bool {
    send_ok(Request::patch(url).build()).await
}

/// `PUT url` with no body. Returns whether the server replied `2xx`.
pub async fn put(url: &str) -> bool {
    send_ok(Request::put(url).build()).await
}

/// `DELETE url`. Returns whether the server replied `2xx`.
pub async fn delete(url: &str) -> bool {
    send_ok(Request::delete(url).build()).await
}

/// `POST url` with a JSON body. Returns whether the server replied `2xx`.
///
/// A body that cannot be serialized is a failure (`false`), not a silent no-op.
pub async fn post_json<B: Serialize + ?Sized>(url: &str, body: &B) -> bool {
    send_ok(Request::post(url).json(body)).await
}

/// `PATCH url` with a JSON body. Returns whether the server replied `2xx`.
pub async fn patch_json<B: Serialize + ?Sized>(url: &str, body: &B) -> bool {
    send_ok(Request::patch(url).json(body)).await
}

/// `PUT url` with a JSON body. Returns whether the server replied `2xx`.
pub async fn put_json<B: Serialize + ?Sized>(url: &str, body: &B) -> bool {
    send_ok(Request::put(url).json(body)).await
}

/// `POST url` with a JSON body, decoding the response into `T`. `None` when the
/// request fails *or* the server replies with a non-2xx status — a rejected
/// write must not be mistaken for success, since error responses carry a
/// placeholder body that decodes fine.
pub async fn post_json_resp<B, T>(url: &str, body: &B) -> Option<T>
where
    B: Serialize + ?Sized,
    T: DeserializeOwned,
{
    let req = Request::post(url).json(body).ok()?;
    let resp = req.send().await.ok()?;
    if !resp.ok() {
        return None;
    }
    resp.json::<T>().await.ok()
}

/// The message every failed write shows, so error text stays consistent across
/// pages instead of each call site inventing its own wording.
pub const WRITE_ERROR: &str = "Could not save your changes. Please try again.";
