use std::sync::Arc;

use dioxus::prelude::*;
use hyle::{MutateInput, Query, Value};
use hyle_dioxus_example::blueprint::make_blueprint;
use hyle_dioxus::{FieldChange, FormErrors, HyleAdapter, HyleDataState,
    make_fullstack_adapter, use_adapter_config, use_context_provider, use_filters, use_form,
    use_fullstack_source, use_list_with_filters, use_mutation, BoundMutateInput, HyleConfig,
    HyleFormState, InvalidationSignal, UseFiltersOptions, UseFormOptions,
};
use hyle_dioxus_native::{HyleFormFields, HyleTableFilterBar, HyleTableFilters, HyleTablePanel};


use hyle_dioxus_example::server::{create_user, delete_user, get_page_params, get_source, get_user_page, update_user};
use serde_json::json;
fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    #[cfg(feature = "server")]
    {
        use axum::{Extension, Router, routing::{get, post}};
        use dioxus::server::{serve, DioxusRouterExt, FullstackState, ServeConfig, render_handler};
        use hyle_dioxus_example::server::AppState;
        use hyle_dioxus_example::server::register_providers;
        use hyle_dioxus_example::server::post_handlers::{
            handle_create_user, handle_delete_user, handle_reset, handle_update_user,
        };
        use hyle_dioxus::HyleRenderer;

        serve(|| async {
            let state = AppState::new();
            register_providers(&state);
            let fullstack_state = FullstackState::new(ServeConfig::new(), app);
            Ok(Router::<FullstackState>::new()
                .route("/users/new",         get(render_handler).post(handle_create_user))
                .route("/users/{id}/edit",   get(render_handler).post(handle_update_user))
                .route("/users/{id}/delete", post(handle_delete_user))
                .route("/__reset",           get(handle_reset))
                .register_server_functions()
                .serve_static_assets()
                .fallback(get(render_handler))
                .with_state(fullstack_state)
                .layer(Extension(HyleRenderer))
                .layer(Extension(state)))
        });
    }

    #[cfg(not(feature = "server"))]
    dioxus::LaunchBuilder::new()
        .with_cfg(dioxus::web::Config::new().hydrate(true))
        .launch(app);
}

// ── Routes ────────────────────────────────────────────────────────────────────

#[derive(Routable, Clone, PartialEq)]
enum Route {
    #[route("/")]
    UserList {},
    #[route("/users/new")]
    UserNew {},
    #[route("/users/:id/edit")]
    UserEdit { id: u64 },
}

// ── Blueprint & query ─────────────────────────────────────────────────────────

fn base_query(page: usize, per_page: usize) -> Query {
    Query::new("user")
        .select(["name", "email", "role", "tags", "active"])
        .sort_by("name", true)
        .page(page, per_page)
}

// ── App shell ─────────────────────────────────────────────────────────────────

#[component]
fn app() -> Element {
    use_context_provider(|| HyleConfig { blueprint: Arc::new(make_blueprint()) });
    use_context_provider(|| Signal::new(0u32) as InvalidationSignal);
    use_adapter_config!(make_fullstack_adapter(
        get_source,
        |input: MutateInput| async move { create_user(input).await.map(|_| ()).map_err(|e| e.to_string()) },
        |input: MutateInput| async move { update_user(input).await.map(|_| ()).map_err(|e| e.to_string()) },
        |input: MutateInput| async move { delete_user(input).await.map(|_| ()).map_err(|e| e.to_string()) },
    ));

    rsx! {
        dioxus::document::Stylesheet { href: hyle_dioxus::CSS }
        style { {include_str!("style.css")} }
        Router::<Route> {}
    }
}

// ── UserList ──────────────────────────────────────────────────────────────────

#[component]
fn UserList() -> Element {
    let page_params = use_server_future(get_page_params);
    let (init_page, init_per_page, init_filters) = match &page_params {
        Ok(f) => f.read().as_ref().and_then(|r| r.as_ref().ok().cloned()).unwrap_or((1, 5, Default::default())),
        Err(_) => (1, 5, Default::default()),
    };

    let filters = use_filters(
        base_query(init_page, init_per_page),
        UseFiltersOptions { initial_committed: init_filters.clone(), ..Default::default() },
    );

    let init_query = {
        let mut q = base_query(init_page, init_per_page);
        for (k, v) in &init_filters {
            q.where_.insert(k.clone(), Value::String(v.clone()));
        }
        q
    };
    let source = use_fullstack_source(move || {
        let q = init_query.clone();
        async move { get_user_page(q).await.map_err(Into::into) }
    });
    let root_adapter = use_context::<HyleAdapter>();
    use_context_provider(|| HyleAdapter { source, ..root_adapter });

    let list = use_list_with_filters(filters);

    rsx! {
        main { class: "hyle-shell",
            section { class: "hyle-toolbar",
                div {
                    h1 { "hyle Dioxus example" }
                    p { "Rust plans the query and resolves lookup data; Dioxus renders the table." }
                }
            }

            div { class: "hyle-workspace",
                section { class: "hyle-panel",
                    SuspenseBoundary {
                        fallback: |_| rsx! { p { "Loading…" } },
                        HyleTablePanel {
                            list,
                            filters,
                            row_href: Callback::new(move |row: hyle_dioxus::Row| {
                                let id = row.get("id")
                                    .and_then(|v| match v {
                                        Value::Int(n) => Some(*n as u64),
                                        Value::String(s) => s.parse().ok(),
                                        _ => None,
                                    })
                                    .unwrap_or(0);
                                format!("/users/{id}/edit")
                            }),
                            header { class: "hyle-panel-header",
                                div { class: "hyle-panel-title-row",
                                    h2 { "Users" }
                                    details { class: "hyle-action-menu",
                                        summary { class: "hyle-action-menu-toggle", "Actions" }
                                        ul { class: "hyle-action-menu-list",
                                            li {
                                                a { href: "/users/new", class: "hyle-primary-button", "Add user" }
                                            }
                                        }
                                    }
                                }
                                HyleTableFilterBar {
                                    filters,
                                    only: vec![
                                        "name".into(), "email".into(),
                                        "role".into(), "tags".into(), "active".into(),
                                    ],
                                    HyleTableFilters {}
                                }
                            }
                        }
                    }
                }
            }

            section { class: "hyle-debug-grid",
                {
                    match &*list.data.read() {
                        HyleDataState::Ready { manifest, outcome, .. } => rsx! {
                            DebugBlock { title: "Manifest from Rust".to_string(), value: json!(manifest.clone()) }
                            DebugBlock { title: "Outcome from Rust".to_string(), value: json!(outcome.clone()) }
                        },
                        _ => rsx! {},
                    }
                }
            }
        }
    }
}

// ── UserFormPanel ─────────────────────────────────────────────────────────────

#[component]
fn UserFormPanel(
    title: &'static str,
    action: String,
    form: HyleFormState,
    errors: FormErrors,
    #[props(optional)] delete: Option<Element>,
) -> Element {
    let filters = form.filters;
    rsx! {
        main { class: "hyle-shell",
            section { class: "hyle-panel",
                header { class: "hyle-panel-header",
                    div { class: "hyle-panel-title-row",
                        h2 { "{title}" }
                        a { href: "/", class: "hyle-close-button", "×" }
                    }
                }

                form {
                    method: "post",
                    action: "{action}",
                    onsubmit: move |e| { e.prevent_default(); form.on_submit.call(()); },

                    if !errors.is_empty() {
                        ul { class: "hyle-errors",
                            for (field, msg) in &errors.0 {
                                li { key: "{field}", "{field}: {msg}" }
                            }
                        }
                    }

                    HyleFormFields { filters }

                    if let Some(ref msg) = *form.mutation.error.read() {
                        p { class: "hyle-errors", "{msg}" }
                    }

                    div { class: "hyle-edit-actions",
                        button {
                            r#type: "submit",
                            class: "hyle-primary-button",
                            disabled: *form.mutation.is_pending.read(),
                            { if *form.mutation.is_pending.read() { "Saving…" } else { "Save" } }
                        }
                        a { href: "/", "Cancel" }
                    }
                }

                { delete }

                div { class: "hyle-edit-debug",
                    DebugBlock { title: "Form data".to_string(), value: json!(*filters.form_data.read()) }
                }
            }
        }
    }
}

// ── UserNew ───────────────────────────────────────────────────────────────────

#[component]
fn UserNew() -> Element {
    let errors = try_use_context::<FormErrors>().unwrap_or_default();
    let form = use_form(base_query(1, 5), UseFormOptions::default());

    use_effect(move || {
        if *form.mutation.is_success.read() {
            #[cfg(target_arch = "wasm32")]
            web_sys::window().unwrap().location().assign("/").unwrap();
        }
    });

    rsx! {
        UserFormPanel { title: "Add user", action: "/users/new", form, errors }
    }
}

// ── UserEdit ──────────────────────────────────────────────────────────────────

#[component]
fn UserEdit(id: u64) -> Element {
    let errors = try_use_context::<FormErrors>().unwrap_or_default();

    let mut query = base_query(1, 1);
    query.where_.insert("id".to_owned(), Value::Int(id as i64));
    query.method = Some("one".to_owned());

    let form = use_form(query, UseFormOptions::default()
        .with_change("email", |f| FieldChange::label(f, "Work email")));
    let mut_ = use_mutation("user");

    use_effect(move || {
        if *form.mutation.is_success.read() || *mut_.delete.is_success.read() {
            #[cfg(target_arch = "wasm32")]
            web_sys::window().unwrap().location().assign("/").unwrap();
        }
    });

    let edit_uri = format!("/users/{id}/edit");
    let delete_uri = format!("/users/{id}/delete");

    rsx! {
        UserFormPanel {
            title: "Edit user",
            action: edit_uri,
            form,
            errors,
            delete: rsx! {
                form {
                    method: "post",
                    action: "{delete_uri}",
                    onsubmit: move |e| {
                        e.prevent_default();
                        mut_.delete.mutate.call(BoundMutateInput { id: Some(Value::Int(id as i64)), ..Default::default() });
                    },
                    div { class: "hyle-edit-actions",
                        button {
                            r#type: "submit",
                            class: "hyle-danger-button",
                            disabled: *mut_.delete.is_pending.read(),
                            { if *mut_.delete.is_pending.read() { "Deleting…" } else { "Delete" } }
                        }
                    }
                }
            },
        }
    }
}

// ── DebugBlock ────────────────────────────────────────────────────────────────

#[component]
fn DebugBlock(title: String, value: serde_json::Value) -> Element {
    rsx! {
        section { class: "hyle-debug-block",
            h2 { "{title}" }
            pre { "{serde_json::to_string_pretty(&value).unwrap()}" }
        }
    }
}
