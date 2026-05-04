//! Dioxus integration for hyle.
//!
//! Provides hooks and context utilities for building data-driven UIs with hyle
//! in Dioxus applications, calling the `hyle` crate directly without any WASM bridge.
//!
//! # Quick start
//!
//! ```rust,ignore
//! use std::sync::Arc;
//! use dioxus::prelude::*;
//! use hyle_dioxus::{use_context_provider, HyleConfig, use_data, HyleAdapter, use_adapter_config};
//!
//! fn app() -> Element {
//!     use_context_provider(|| HyleConfig { blueprint: Arc::new(my_blueprint()) });
//!     use_adapter_config!(make_rest_adapter("http://localhost:3001/api"));
//!     rsx! { Router::<Route> {} }
//! }
//! ```

mod context;
mod filter;
mod hooks;
mod query;
mod types;
#[cfg(feature = "axum")]
pub mod axum;
#[cfg(feature = "axum")]
pub use axum::HyleRenderer;

pub use context::{use_context_provider, use_hyle_components, HyleConfig};
pub use filter::{FilterField, FormFilterField};
pub use hooks::{form_body, use_data, use_filters, use_form, use_forma, use_list, use_list_with_filters, use_manifest, use_mutation};
pub use query::{
    make_fullstack_adapter, use_dioxus_mutation, use_fullstack_source,
    DioxusMutationOptions, InvalidationSignal,
};
pub use types::{
    field_type_key, BoundMutateInput, BoundMutation, BoundMutations, DioxusFieldChangeFn,
    DioxusFieldChangeMap, HyleAdapter,
    HyleComponents, HyleFilterField, HyleFilterFieldProps,
    HyleFiltersState, HyleFormState, HyleListState, HyleMutation,
    HyleSourceState, HyleValueProps, UseFiltersOptions, UseFormOptions,
    UseSource,
};

// Re-export hyle types that callers need when building blueprints and queries.
pub use hyle::{
    Blueprint, Column, Field, FieldType, Forma, FormaContext, FormaField, FormaFieldType, Manifest,
    Model, Primitive, PurifyError, Query, Row, Sort, Source, Value,
};

// ── Adapter config macro ──────────────────────────────────────────────────────

/// Provide a `HyleAdapter` as Dioxus context.
///
/// Call this at the **top level** of your root component, passing a fully
/// constructed `HyleAdapter` (e.g. from `make_fullstack_adapter`).
/// All hooks then read source and mutations from context automatically.
///
/// ```rust,ignore
/// use hyle_dioxus::use_adapter_config;
/// use hyle_dioxus::make_fullstack_adapter;
///
/// fn App() -> Element {
///     use_adapter_config!(make_fullstack_adapter(...));
///     // ...
/// }
/// ```
#[macro_export]
macro_rules! use_adapter_config {
    ($adapter:expr) => {
        {
            use dioxus::prelude::use_context_provider;
            use hyle_dioxus::HyleAdapter;
            let _hyle_adapter: HyleAdapter = $adapter;
            use_context_provider(|| _hyle_adapter);
        }
    };
}
