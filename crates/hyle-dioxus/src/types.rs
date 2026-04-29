use std::rc::Rc;
use std::collections::HashMap;

use dioxus::prelude::Callback;
use dioxus::prelude::Element;
use dioxus_signals::{Memo, ReadSignal, Signal};
use indexmap::IndexMap;

use hyle::{Field, FieldChange, FieldType, HyleDataState, MutateInput, Outcome, Primitive, PurifyError, Query, Source, Value};
use serde_json::Value as JsonValue;

/// Internal model name used by `use_forma` for its synthetic query.
pub(crate) const FORMA_MODEL: &str = "forma";
// ── Dioxus-specific HyleFilterField ──────────────────────────────────────────

/// Dioxus specialisation of `hyle::HyleFilterField` — the `render` field
/// carries an optional per-field Dioxus render closure.
pub type HyleFilterField = hyle::HyleFilterField<Rc<dyn Fn(HyleFilterFieldProps) -> Element>>;

/// A single per-field Dioxus transform function.
pub type DioxusFieldChangeFn = Rc<dyn Fn(&Field) -> Option<hyle::FieldChange<Rc<dyn Fn(HyleFilterFieldProps) -> Element>>>>;

/// A map of per-field Dioxus transform functions (keyed by field name).
pub type DioxusFieldChangeMap = HashMap<String, DioxusFieldChangeFn>;

// ── Dioxus-specific filter/form options ───────────────────────────────────────

/// Options for [`use_filters`](crate::use_filters).
#[derive(Default)]
pub struct UseFiltersOptions {
    pub initial_committed: IndexMap<String, String>,
    /// Per-field transform map with Dioxus render override support.
    pub change: Option<DioxusFieldChangeMap>,
}

/// Options for [`use_form`](crate::use_form).
#[derive(Default)]
pub struct UseFormOptions {
    pub initial_committed: IndexMap<String, String>,
    /// Per-field transform map with Dioxus render override support.
    pub change: Option<DioxusFieldChangeMap>,
}

impl UseFormOptions {
    /// Add a plain field transform (no render override).
    ///
    /// The closure receives the current [`hyle::Field`] and returns
    /// `Some(FieldChange { .. })` to override label / metadata, or `None` to
    /// leave it unchanged.  Use [`FieldChange::label`] for the common case.
    pub fn with_change(
        mut self,
        key: &str,
        f: impl Fn(&hyle::Field) -> Option<hyle::FieldChange> + 'static,
    ) -> Self {
        self.change
            .get_or_insert_with(HashMap::new)
            .insert(
                key.to_owned(),
                Rc::new(move |field: &hyle::Field| {
                    f(field).map(|fc| hyle::FieldChange {
                        field: fc.field,
                        render: None,
                    })
                }),
            );
        self
    }
}

// ── Source adapter ────────────────────────────────────────────────────────────

/// The result of a source adapter call.
#[derive(Clone, PartialEq)]
pub enum HyleSourceState {
    Loading,
    Ready(Source),
    Error(String),
}

/// A reactive source handle — `ReadOnlySignal<HyleSourceState>`.
pub type UseSource = ReadSignal<HyleSourceState>;

// ── Adapter config ────────────────────────────────────────────────────────────

/// Unified adapter stored in Dioxus context.
///
/// Provide this at the app root via [`use_adapter_config!`](crate::use_adapter_config).
#[derive(Clone, Copy, PartialEq)]
pub struct HyleAdapter {
    pub source: UseSource,
    pub create: HyleMutation,
    pub update: HyleMutation,
    pub delete: HyleMutation,
}

// ── List state ────────────────────────────────────────────────────────────────

/// State returned by [`use_list`](crate::use_list).
#[derive(Clone, Copy, PartialEq)]
pub struct HyleListState {
    pub data: Memo<HyleDataState>,
    pub query: Memo<Query>,
    pub page: Signal<usize>,
    pub per_page: Signal<usize>,
    pub sort_field: Signal<Option<String>>,
    pub sort_ascending: Signal<bool>,
}

// ── Filter field props ────────────────────────────────────────────────────────

/// Props passed to a custom render function supplied to [`FilterField`](crate::FilterField).
#[derive(Clone)]
pub struct HyleFilterFieldProps {
    pub key: String,
    pub label: String,
    pub field: Field,
    pub options: Option<Vec<(String, String)>>,
    pub value: String,
    pub set: Callback<String>,
}

// ── Mutation ──────────────────────────────────────────────────────────────────

/// A mutation handle.
#[derive(Clone, Copy, PartialEq)]
pub struct HyleMutation {
    pub mutate: Callback<MutateInput>,
    pub reset: Callback<()>,
    pub is_pending: Signal<bool>,
    pub is_success: Signal<bool>,
    pub error: Signal<Option<String>>,
}

// ── Bound mutations ───────────────────────────────────────────────────────────

/// Input for a [`BoundMutation`] — like [`MutateInput`] but without `model`.
#[derive(Clone, Default)]
pub struct BoundMutateInput {
    pub id: Option<JsonValue>,
    pub data: IndexMap<String, String>,
}

/// A mutation handle with `model` pre-bound.
#[derive(Clone, Copy)]
pub struct BoundMutation {
    pub mutate: Callback<BoundMutateInput>,
    pub is_pending: Signal<bool>,
    pub is_success: Signal<bool>,
    pub error: Signal<Option<String>>,
}

/// Create/update/delete handles with `model` pre-bound.
#[derive(Clone, Copy)]
pub struct BoundMutations {
    pub create: BoundMutation,
    pub update: BoundMutation,
    pub delete: BoundMutation,
}

// ── Filters state ─────────────────────────────────────────────────────────────

/// State returned by [`use_filters`](crate::use_filters).
#[derive(Clone, Copy, PartialEq)]
pub struct HyleFiltersState {
    pub query: Memo<Query>,
    pub fields: Memo<Vec<HyleFilterField>>,
    pub form_data: Signal<IndexMap<String, String>>,
    pub set_field: Callback<(String, String)>,
    pub filter_apply: Callback<()>,
    pub filter_clear: Callback<()>,
    pub filter_reset_key: Signal<u32>,
    pub validate: Callback<()>,
    pub purify_errors: Signal<Option<Vec<PurifyError>>>,
}

// ── Value components ──────────────────────────────────────────────────────────

/// Props passed to a custom value cell renderer in `HyleComponents`.
#[derive(Clone)]
pub struct HyleValueProps {
    pub key: String,
    pub field: Field,
    pub value: Value,
    pub outcome: Outcome,
    pub model_name: String,
}

/// A map of component overrides for value cells and filter inputs.
#[derive(Clone, Default)]
pub struct HyleComponents {
    pub values: indexmap::IndexMap<String, fn(HyleValueProps) -> Element>,
    pub filters: indexmap::IndexMap<String, fn(HyleFilterFieldProps) -> Element>,
}

/// Derive the `HyleComponents` key string from a `FieldType`.
pub fn field_type_key(field_type: &FieldType) -> &'static str {
    match field_type {
        FieldType::Primitive { primitive } => match primitive {
            Primitive::String => "string",
            Primitive::Number => "number",
            Primitive::Boolean => "boolean",
            Primitive::File => "file",
        },
        FieldType::Reference { .. } => "reference",
        FieldType::Array { .. } => "array",
        FieldType::Shape { .. } => "shape",
    }
}

// ── Form hook state ───────────────────────────────────────────────────────────

/// State returned by [`use_form`](crate::use_form).
#[derive(Clone, Copy, PartialEq)]
pub struct HyleFormState {
    pub filters: HyleFiltersState,
    pub is_edit: bool,
    pub is_valid: bool,
    pub on_submit: Callback<()>,
    pub mutation: HyleMutation,
}
