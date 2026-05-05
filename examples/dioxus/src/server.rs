/// Server-side state and `#[server]` function definitions for the hyle Dioxus fullstack example.
///
/// `#[server]` functions are compiled on both targets:
/// - **Server**: the body runs in-process (direct memory access, no HTTP).
/// - **Client**: a stub is generated that POSTs to `/__dioxus/server_fn/...`.
///
/// `AppState` is stored as an Axum `Extension` and extracted inside each
/// server fn via `FullstackContext::extract`.

use dioxus::prelude::*;
use dioxus_fullstack_core::ServerFnError;
use hyle::Source;
use hyle::MutateInput;
#[cfg(not(target_arch = "wasm32"))]
use hyle::FormErrors;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use serde_json::{json, Value};
#[cfg(target_arch = "wasm32")]
use serde_json::Value;

// ── Data types ────────────────────────────────────────────────────────────────

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub role: String,
    pub tags: Vec<String>,
    pub active: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Role {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Tag {
    pub id: String,
    pub name: String,
}

// ── App state (server only) ───────────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
pub use server_state::{AppState, seed_roles, seed_tags, seed_users};

#[cfg(not(target_arch = "wasm32"))]
mod server_state {
    use std::sync::{Arc, RwLock};
    use hyle::Purifier;
    use crate::blueprint::make_blueprint;
    use super::{User, Role, Tag};

    pub fn seed_users() -> Vec<User> {
        vec![
            User { id: 1, name: "Alice".into(),   email: "alice@example.test".into(),   role: "admin".into(),  tags: vec!["rust".into(), "web".into()], active: true  },
            User { id: 2, name: "Bruno".into(),   email: "bruno@example.test".into(),   role: "editor".into(), tags: vec!["web".into()],                active: true  },
            User { id: 3, name: "Carla".into(),   email: "carla@example.test".into(),   role: "viewer".into(), tags: vec![],                            active: false },
            User { id: 4, name: "Dmitri".into(),  email: "dmitri@example.test".into(),  role: "editor".into(), tags: vec!["rust".into()],               active: true  },
            User { id: 5, name: "Evelyn".into(),  email: "evelyn@example.test".into(),  role: "viewer".into(), tags: vec!["web".into(), "rust".into()],  active: true  },
            User { id: 6, name: "Fatima".into(),  email: "fatima@example.test".into(),  role: "admin".into(),  tags: vec![],                            active: false },
            User { id: 7, name: "Gustavo".into(), email: "gustavo@example.test".into(), role: "viewer".into(), tags: vec!["rust".into()],               active: true  },
        ]
    }

    pub fn seed_roles() -> Vec<Role> {
        vec![
            Role { id: "admin".into(),  name: "Admin".into()  },
            Role { id: "editor".into(), name: "Editor".into() },
            Role { id: "viewer".into(), name: "Viewer".into() },
        ]
    }

    pub fn seed_tags() -> Vec<Tag> {
        vec![
            Tag { id: "rust".into(), name: "Rust".into() },
            Tag { id: "web".into(),  name: "Web".into()  },
        ]
    }

    #[derive(Clone)]
    pub struct AppState {
        pub users: Arc<RwLock<Vec<User>>>,
        pub roles: Arc<RwLock<Vec<Role>>>,
        pub tags: Arc<RwLock<Vec<Tag>>>,
        pub blueprint: Arc<hyle::Blueprint>,
        pub purifier: Arc<Purifier>,
    }

    impl AppState {
        pub fn new() -> Self {
            Self {
                users: Arc::new(RwLock::new(seed_users())),
                roles: Arc::new(RwLock::new(seed_roles())),
                tags: Arc::new(RwLock::new(seed_tags())),
                blueprint: Arc::new(make_blueprint()),
                purifier: Arc::new(Purifier::new()),
            }
        }
    }
}

// ── Server functions ──────────────────────────────────────────────────────────

/// Read `page`, `per_page`, and any other query params from the current
/// request's URI query string.
#[server]
pub async fn get_page_params() -> Result<(usize, usize, IndexMap<String, String>), ServerFnError> {
    use axum::extract::OriginalUri;

    let OriginalUri(uri) = FullstackContext::extract().await?;
    let query_str = uri.query().unwrap_or("");
    Ok(hyle::parse_query_params(query_str, 5))
}

/// Fetch the full source (all models) — used by `use_fullstack_source`.
#[server]
pub async fn get_source() -> Result<Source, ServerFnError> {
    use axum::Extension;
    use hyle::ModelResult;

    let Extension(state): Extension<AppState> = FullstackContext::extract().await?;

    let users = state.users.read().unwrap().clone();
    let roles = state.roles.read().unwrap().clone();
    let tags = state.tags.read().unwrap().clone();

    let user_rows: Vec<hyle::Row> = users
        .iter()
        .map(|u| {
            IndexMap::from([
                ("id".into(),     json!(u.id)),
                ("name".into(),   json!(u.name)),
                ("email".into(),  json!(u.email)),
                ("role".into(),   json!(u.role)),
                ("tags".into(),   json!(u.tags)),
                ("active".into(), json!(u.active)),
            ])
        })
        .collect();

    let role_rows: Vec<hyle::Row> = roles
        .iter()
        .map(|r| {
            IndexMap::from([
                ("id".into(),   json!(r.id)),
                ("name".into(), json!(r.name)),
            ])
        })
        .collect();

    let tag_rows: Vec<hyle::Row> = tags
        .iter()
        .map(|t| {
            IndexMap::from([
                ("id".into(),   json!(t.id)),
                ("name".into(), json!(t.name)),
            ])
        })
        .collect();

    let mut source = Source::new();
    source.insert("user".into(), ModelResult::many(user_rows));
    source.insert("role".into(), ModelResult::many(role_rows));
    source.insert("tag".into(), ModelResult::many(tag_rows));

    Ok(source)
}

/// Create a new user (JS path via server fn).
#[server]
pub async fn create_user(input: MutateInput) -> Result<Value, ServerFnError> {
    use axum::Extension;

    let Extension(state): Extension<AppState> = FullstackContext::extract().await?;

    let row = hyle::row_from_form(&input.data);
    if let Err(errors) = state.purifier.purify_row(&state.blueprint, "user", &row) {
        return Err(ServerFnError::ServerError {
            message: serde_json::to_string(&errors).unwrap_or_default(),
            code: 422,
            details: None,
        });
    }

    let mut users = state.users.write().unwrap();
    let next_id = users.iter().map(|u| u.id).max().unwrap_or(0) + 1;
    let active = match input.data.get("active").map(|s| s.as_str()) {
        Some("true") => true,
        Some("false") => false,
        _ => true,
    };
    let tags = input.data.get("tags")
        .map(|s| s.split(',').filter(|t| !t.is_empty()).map(|t| t.trim().to_owned()).collect())
        .unwrap_or_default();
    let user = User {
        id: next_id,
        name:   input.data.get("name").cloned().unwrap_or_default(),
        email:  input.data.get("email").cloned().unwrap_or_default(),
        role:   input.data.get("role").cloned().unwrap_or_else(|| "viewer".into()),
        tags,
        active,
    };
    users.push(user.clone());
    Ok(json!(user))
}

/// Update an existing user (JS path via server fn).
#[server]
pub async fn update_user(input: MutateInput) -> Result<Value, ServerFnError> {
    use axum::Extension;

    let Extension(state): Extension<AppState> = FullstackContext::extract().await?;

    let id = extract_id(&input.id);
    let row = hyle::row_from_form(&input.data);
    if let Err(errors) = state.purifier.purify_row(&state.blueprint, "user", &row) {
        return Err(ServerFnError::ServerError {
            message: serde_json::to_string(&errors).unwrap_or_default(),
            code: 422,
            details: None,
        });
    }

    let mut users = state.users.write().unwrap();
    let Some(user) = users.iter_mut().find(|u| u.id == id) else {
        return Err(ServerFnError::ServerError { message: "User not found".into(), code: 404, details: None });
    };
    if let Some(name)  = input.data.get("name")  { user.name  = name.clone();  }
    if let Some(email) = input.data.get("email") { user.email = email.clone(); }
    if let Some(role)  = input.data.get("role")  { user.role  = role.clone();  }
    if let Some(tags)  = input.data.get("tags")  {
        user.tags = tags.split(',').filter(|t| !t.is_empty()).map(|t| t.trim().to_owned()).collect();
    }
    if let Some(active) = input.data.get("active") {
        user.active = active == "true";
    }
    Ok(json!(user.clone()))
}

/// Delete a user by id (JS path via server fn).
#[server]
pub async fn delete_user(input: MutateInput) -> Result<(), ServerFnError> {
    use axum::Extension;

    let Extension(state): Extension<AppState> = FullstackContext::extract().await?;

    let id = extract_id(&input.id);
    let mut users = state.users.write().unwrap();
    let before = users.len();
    users.retain(|u| u.id != id);
    if users.len() == before {
        return Err(ServerFnError::ServerError { message: "User not found".into(), code: 404, details: None });
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn extract_id(v: &Option<Value>) -> u64 {
    v.as_ref()
        .and_then(|v| v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
        .unwrap_or(0)
}

// ── Native POST handlers ──────────────────────────────────────────────────────
//
// These axum handlers receive urlencoded form bodies from the no-JS
// `<form method="post">` elements.  On validation failure they re-render the
// appropriate page with `FormErrors` injected into the SSR context so the
// errors appear pre-rendered in the HTML response.  On success they redirect
// to "/".

#[cfg(feature = "server")]
pub mod post_handlers {
    use super::*;
    use axum::{
        Extension,
        extract::{Form, Path, State},
        http::StatusCode,
        response::{IntoResponse, Redirect, Response},
    };
    use dioxus::server::FullstackState;

    /// Fold a URL-encoded form with potentially repeated keys into an
    /// `IndexMap<String, String>`, joining repeated values with `","`.
    fn collect_form(pairs: Vec<(String, String)>) -> IndexMap<String, String> {
        let mut map: IndexMap<String, String> = IndexMap::new();
        for (k, v) in pairs {
            map.entry(k)
                .and_modify(|e| { e.push(','); e.push_str(&v); })
                .or_insert(v);
        }
        map
    }
    use hyle::row_from_form;
    use hyle_dioxus::HyleRenderer;

    pub async fn handle_create_user(
        State(fullstack_state): State<FullstackState>,
        Extension(renderer): Extension<HyleRenderer>,
        Extension(app_state): Extension<AppState>,
        Form(pairs): Form<Vec<(String, String)>>,
    ) -> Response {
        let form = collect_form(pairs);
        let row = row_from_form(&form);
        match app_state.purifier.purify_row(&app_state.blueprint, "user", &row) {
            Err(errs) => {
                let map = errs.into_iter()
                    .map(|e| (e.field.clone(), e.message.clone()))
                    .collect();
                renderer.render_with_errors(fullstack_state, "/users/new", FormErrors(map)).await
            }
            Ok(_) => {
                let mut users = app_state.users.write().unwrap();
                let next_id = users.iter().map(|u| u.id).max().unwrap_or(0) + 1;
                let active = form.get("active").map(|s| s == "true").unwrap_or(true);
        let tags = form.get("tags")
                    .map(|s| s.split(',').filter(|t| !t.is_empty()).map(|t| t.trim().to_owned()).collect())
                    .unwrap_or_default();
                users.push(User {
                    id: next_id,
                    name:   form.get("name").cloned().unwrap_or_default(),
                    email:  form.get("email").cloned().unwrap_or_default(),
                    role:   form.get("role").cloned().unwrap_or_else(|| "viewer".into()),
                    tags,
                    active,
                });
                Redirect::to("/").into_response()
            }
        }
    }

    pub async fn handle_update_user(
        State(fullstack_state): State<FullstackState>,
        Extension(renderer): Extension<HyleRenderer>,
        Extension(app_state): Extension<AppState>,
        Path(id): Path<u64>,
        Form(pairs): Form<Vec<(String, String)>>,
    ) -> Response {
        let form = collect_form(pairs);
        let row = row_from_form(&form);
        match app_state.purifier.purify_row(&app_state.blueprint, "user", &row) {
            Err(errs) => {
                let map = errs.into_iter()
                    .map(|e| (e.field.clone(), e.message.clone()))
                    .collect();
                let uri = format!("/users/{id}/edit");
                renderer.render_with_errors(fullstack_state, &uri, FormErrors(map)).await
            }
            Ok(_) => {
                let mut users = app_state.users.write().unwrap();
                if let Some(user) = users.iter_mut().find(|u| u.id == id) {
                    if let Some(v) = form.get("name")   { user.name  = v.clone(); }
                    if let Some(v) = form.get("email")  { user.email = v.clone(); }
                    if let Some(v) = form.get("role")   { user.role  = v.clone(); }
                    if let Some(v) = form.get("tags") {
                        user.tags = v.split(',').filter(|t| !t.is_empty()).map(|t| t.trim().to_owned()).collect();
                    }
                    if let Some(v) = form.get("active") { user.active = v == "true"; }
                }
                Redirect::to("/").into_response()
            }
        }
    }

    pub async fn handle_delete_user(
        Extension(app_state): Extension<AppState>,
        Path(id): Path<u64>,
    ) -> impl IntoResponse {
        app_state.users.write().unwrap().retain(|u| u.id != id);
        Redirect::to("/")
    }

    /// Reset all in-memory data back to the seed state.
    /// Used by the e2e test suite's `beforeEach` to ensure a clean slate.
    pub async fn handle_reset(
        Extension(app_state): Extension<AppState>,
    ) -> impl IntoResponse {
        *app_state.users.write().unwrap() = super::seed_users();
        *app_state.roles.write().unwrap() = super::seed_roles();
        *app_state.tags.write().unwrap() = super::seed_tags();
        StatusCode::OK
    }
}
