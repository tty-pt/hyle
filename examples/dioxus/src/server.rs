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
pub use server_state::{AppState, seed_roles, seed_tags, seed_users, register_providers};

#[cfg(not(target_arch = "wasm32"))]
mod server_state {
    use std::sync::{Arc, RwLock};
    use crate::blueprint::make_blueprint;
    use super::{User, Role, Tag};
    use hyle::csource::{register_source, source_put, FieldDef, FieldType};
    use hyle::{Row, Value};
    use indexmap::IndexMap;

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
    }

    impl AppState {
        pub fn new() -> Self {
            Self {
                users: Arc::new(RwLock::new(seed_users())),
                roles: Arc::new(RwLock::new(seed_roles())),
                tags: Arc::new(RwLock::new(seed_tags())),
                blueprint: Arc::new(make_blueprint()),
            }
        }
    }

    pub fn user_to_row(u: &User) -> Row {
        let mut row: Row = IndexMap::new();
        row.insert("id".into(),     Value::Int(u.id as i64));
        row.insert("name".into(),   Value::String(u.name.clone()));
        row.insert("email".into(),  Value::String(u.email.clone()));
        row.insert("role".into(),   Value::String(u.role.clone()));
        row.insert("tags".into(),   Value::Array(
            u.tags.iter().map(|s| Value::String(s.clone())).collect()
        ));
        row.insert("active".into(), Value::Bool(u.active));
        row
    }

    static USER_FIELDS: &[FieldDef] = &[
        FieldDef { name: "id",     kind: FieldType::Int            },
        FieldDef { name: "name",   kind: FieldType::String         },
        FieldDef { name: "email",  kind: FieldType::String         },
        FieldDef { name: "role",   kind: FieldType::Reference      },
        FieldDef { name: "tags",   kind: FieldType::MultiReference },
        FieldDef { name: "active", kind: FieldType::Bool           },
    ];
    static ROLE_FIELDS: &[FieldDef] = &[
        FieldDef { name: "id",   kind: FieldType::String },
        FieldDef { name: "name", kind: FieldType::String },
    ];
    static TAG_FIELDS: &[FieldDef] = &[
        FieldDef { name: "id",   kind: FieldType::String },
        FieldDef { name: "name", kind: FieldType::String },
    ];

    /// Register all sources with libhyle and populate with seed data.
    /// Must be called once at server startup before any queries.
    pub fn register_providers(state: &AppState) {
        register_source("user", USER_FIELDS);
        for u in state.users.read().unwrap().iter() {
            source_put("user", &user_to_row(u));
        }

        register_source("role", ROLE_FIELDS);
        for r in state.roles.read().unwrap().iter() {
            let mut row: Row = IndexMap::new();
            row.insert("id".into(),   Value::String(r.id.clone()));
            row.insert("name".into(), Value::String(r.name.clone()));
            source_put("role", &row);
        }

        register_source("tag", TAG_FIELDS);
        for t in state.tags.read().unwrap().iter() {
            let mut row: Row = IndexMap::new();
            row.insert("id".into(),   Value::String(t.id.clone()));
            row.insert("name".into(), Value::String(t.name.clone()));
            source_put("tag", &row);
        }
    }
}

// ── Server functions ──────────────────────────────────────────────────────────
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
                ("id".into(),     hyle::Value::Int(u.id as i64)),
                ("name".into(),   hyle::Value::String(u.name.clone())),
                ("email".into(),  hyle::Value::String(u.email.clone())),
                ("role".into(),   hyle::Value::String(u.role.clone())),
                ("tags".into(),   hyle::Value::Array(u.tags.iter().map(|s| hyle::Value::String(s.clone())).collect())),
                ("active".into(), hyle::Value::Bool(u.active)),
            ])
        })
        .collect();

    let role_rows: Vec<hyle::Row> = roles
        .iter()
        .map(|r| {
            IndexMap::from([
                ("id".into(),   hyle::Value::String(r.id.clone())),
                ("name".into(), hyle::Value::String(r.name.clone())),
            ])
        })
        .collect();

    let tag_rows: Vec<hyle::Row> = tags
        .iter()
        .map(|t| {
            IndexMap::from([
                ("id".into(),   hyle::Value::String(t.id.clone())),
                ("name".into(), hyle::Value::String(t.name.clone())),
            ])
        })
        .collect();

    let mut source = Source::new();
    source.insert("user".into(), ModelResult::many(user_rows));
    source.insert("role".into(), ModelResult::many(role_rows));
    source.insert("tag".into(), ModelResult::many(tag_rows));

    Ok(source)
}

/// Fetch filtered + paginated users (and full lookup tables) for the list view.
/// Filtering, sorting, and pagination are delegated to the C source registry.
#[server]
pub async fn get_user_page(query: hyle::Query) -> Result<Source, ServerFnError> {
    use axum::Extension;
    use hyle::ModelResult;

    let Extension(state): Extension<AppState> = FullstackContext::extract().await?;

    let roles = state.roles.read().unwrap().clone();
    let tags  = state.tags.read().unwrap().clone();

    let (user_rows, total) = hyle::csource::source_query("user", &query)
        .map_err(|e| ServerFnError::ServerError { message: e, code: 500, details: None })?;

    let role_rows: Vec<hyle::Row> = roles.iter().map(|r| {
        indexmap::IndexMap::from([
            ("id".into(),   hyle::Value::String(r.id.clone())),
            ("name".into(), hyle::Value::String(r.name.clone())),
        ])
    }).collect();

    let tag_rows: Vec<hyle::Row> = tags.iter().map(|t| {
        indexmap::IndexMap::from([
            ("id".into(),   hyle::Value::String(t.id.clone())),
            ("name".into(), hyle::Value::String(t.name.clone())),
        ])
    }).collect();

    let mut source = Source::new();
    source.insert("user".into(), ModelResult { result: hyle::ModelRows::Many(user_rows), total });
    source.insert("role".into(), ModelResult::many(role_rows));
    source.insert("tag".into(),  ModelResult::many(tag_rows));

    Ok(source)
}

/// Create a new user (JS path via server fn).
#[server]
pub async fn create_user(input: MutateInput) -> Result<Value, ServerFnError> {
    use axum::Extension;

    let Extension(state): Extension<AppState> = FullstackContext::extract().await?;

    let name = input.data.get("name").map(|s| s.trim().to_owned()).unwrap_or_default();
    if name.len() < 2 {
        return Err(ServerFnError::ServerError {
            message: "name: must be at least 2 characters".into(),
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
    hyle::csource::source_put("user", &server_state::user_to_row(&user));
    Ok(json!(user))
}

/// Update an existing user (JS path via server fn).
#[server]
pub async fn update_user(input: MutateInput) -> Result<Value, ServerFnError> {
    use axum::Extension;

    let Extension(state): Extension<AppState> = FullstackContext::extract().await?;

    let id = extract_id(&input.id);
    let row = hyle::row_from_form(&input.data);
    let _ = row; // validation handled server-side (C)

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
    hyle::csource::source_put("user", &server_state::user_to_row(user));
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
    hyle::csource::source_del("user", &id.to_string());
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn extract_id(v: &Option<hyle::Value>) -> u64 {
    v.as_ref()
        .and_then(|v| match v {
            hyle::Value::Int(n) => Some(*n as u64),
            hyle::Value::String(s) => s.parse().ok(),
            _ => None,
        })
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
    use hyle_dioxus::HyleRenderer;

    pub async fn handle_create_user(
        State(fullstack_state): State<FullstackState>,
        Extension(renderer): Extension<HyleRenderer>,
        Extension(app_state): Extension<AppState>,
        Form(pairs): Form<Vec<(String, String)>>,
    ) -> Response {
        let form = collect_form(pairs);
        let name = form.get("name").map(|s| s.trim().to_owned()).unwrap_or_default();
        if name.len() < 2 {
            let mut errors = IndexMap::new();
            errors.insert("name".to_owned(), "must be at least 2 characters".to_owned());
            return renderer.render_with_errors(fullstack_state, "/users/new", hyle_dioxus::FormErrors(errors)).await;
        }
        let user = {
            let mut users = app_state.users.write().unwrap();
            let next_id = users.iter().map(|u| u.id).max().unwrap_or(0) + 1;
            let active = form.get("active").map(|s| s == "true").unwrap_or(true);
            let tags = form.get("tags")
                .map(|s| s.split(',').filter(|t| !t.is_empty()).map(|t| t.trim().to_owned()).collect())
                .unwrap_or_default();
            let u = User {
                id: next_id,
                name:   form.get("name").cloned().unwrap_or_default(),
                email:  form.get("email").cloned().unwrap_or_default(),
                role:   form.get("role").cloned().unwrap_or_else(|| "viewer".into()),
                tags,
                active,
            };
            users.push(u.clone());
            u
        };
        hyle::csource::source_put("user", &server_state::user_to_row(&user));
        Redirect::to("/").into_response()
    }

    pub async fn handle_update_user(
        State(_fullstack_state): State<FullstackState>,
        Extension(_renderer): Extension<HyleRenderer>,
        Extension(app_state): Extension<AppState>,
        Path(id): Path<u64>,
        Form(pairs): Form<Vec<(String, String)>>,
    ) -> Response {
        let form = collect_form(pairs);
        let row = {
            let mut users = app_state.users.write().unwrap();
            if let Some(user) = users.iter_mut().find(|u| u.id == id) {
                if let Some(v) = form.get("name")   { user.name  = v.clone(); }
                if let Some(v) = form.get("email")  { user.email = v.clone(); }
                if let Some(v) = form.get("role")   { user.role  = v.clone(); }
                if let Some(v) = form.get("tags") {
                    user.tags = v.split(',').filter(|t| !t.is_empty()).map(|t| t.trim().to_owned()).collect();
                }
                if let Some(v) = form.get("active") { user.active = v == "true"; }
                Some(server_state::user_to_row(user))
            } else {
                None
            }
        };
        if let Some(row) = row {
            hyle::csource::source_put("user", &row);
        }
        Redirect::to("/").into_response()
    }

    pub async fn handle_delete_user(
        Extension(app_state): Extension<AppState>,
        Path(id): Path<u64>,
    ) -> impl IntoResponse {
        app_state.users.write().unwrap().retain(|u| u.id != id);
        hyle::csource::source_del("user", &id.to_string());
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
        super::register_providers(&app_state);
        StatusCode::OK
    }
}
