//! Framework-agnostic schema and query planning primitives.
//!
//! `hyle` deliberately has no React, DOM, async, or transport concepts.
//! It describes models and fields, derives backend query manifests, and resolves
//! raw backend sources into purified results plus lookup tables. UI frameworks should build
//! thin adapters on top of these serializable structures.

mod blueprint;
mod error;
mod field;
mod purify;
pub mod query;
pub mod raw;
pub mod de;
pub mod ser;
mod forma;
pub mod view;
pub mod source;

pub use blueprint::{Blueprint, Model};
pub use source::{
    set_provider, get_provider, load_source, find_item,
    SourceProvider,
    load_typed_item, load_typed_rows,
    FieldStorageType, SourceDef, SourceFieldDef,
    deser_row,
};
#[cfg(all(feature = "csource", not(target_arch = "wasm32")))]
pub use source::csource;
#[cfg(feature = "json")]
pub use source::{
    extract_field,
    source_to_json, load_source_json, load_item_json,
};
pub use error::{Error, HyleResult};
pub use field::{
    make_field, Field, FieldOptions, FieldType, InputHint, Primitive, Reference, ShapeField, SortType,
};
pub use purify::PurifyError;
pub use query::{parse_query_params, Manifest, MutateInput, Query, Sort};
pub use raw::{is_single, parse_index_items, row_from_form, row_from_value, ModelResult, Outcome, Row, RowItem, Source, Value, rows_from_outcome, ModelRows};
pub use forma::{Forma, FormaContext, FormaField, FormaFieldType, forma_to_query};
pub use view::{Column, display_value, display_value_from_outcome};

#[cfg(feature = "wasm")]
#[doc(hidden)]
pub mod wasm;
