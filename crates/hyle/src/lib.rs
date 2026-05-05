//! Framework-agnostic forma and query planning primitives.
//!
//! `hyle` deliberately has no React, DOM, async, or transport concepts.
//! It describes models and fields, derives backend query manifests, and resolves
//! raw backend sources into purified results plus lookup tables. UI frameworks should build
//! thin adapters on top of these serializable structures.

mod blueprint;
mod error;
mod field;
mod purify;
mod query;
mod raw;
mod forma;
mod view;
pub(crate) mod adapter;
#[cfg(feature = "wasm")]
#[doc(hidden)]
pub mod wasm;

/// The default component stylesheet for hyle UI components.
///
/// Dioxus consumers can link this into their app:
/// ```rust,ignore
/// document::Link { rel: "stylesheet", href: hyle::CSS }
/// ```
pub static CSS: manganis::Asset = manganis::asset!("assets/hyle.css");

pub use blueprint::{Blueprint, Model, ResolvedView};
pub use error::{Error, HyleResult};
pub use field::{
    make_field, Field, FieldOptions, FieldType, InputHint, Primitive, Reference, ShapeField, SortType,
};
pub use purify::{purify_row_sync, PurifyError, Purifier, SyncRule};
pub use query::{parse_query_params, Manifest, MutateInput, Query, Sort};
pub use raw::{is_single, row_from_form, row_from_value, ModelResult, Outcome, Row, Source, Value};
pub use forma::{Forma, FormaContext, FormaField, FormaFieldType, forma_to_query};
pub use view::{apply_view, Column, display_value, display_value_from_outcome, filter_rows};
pub use adapter::{
    apply_change, build_effective_query, build_filter_fields, compute_data, compute_forma_result,
    compute_manifest, run_purify, AdapterFiltersOptions, AdapterFormOptions, FieldChange,
    FieldChangeFn, FieldChangeMap, FormErrors, HyleDataField, HyleDataState, HyleFilterField,
    HyleManifestState, UseFormaOptions,
};
