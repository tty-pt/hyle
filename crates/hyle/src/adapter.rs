//! Framework-agnostic adapter types and pure helper functions.
//!
//! This module contains two categories of public items:
//!
//! ## End-user types
//!
//! State types and option structs used directly when working with hooks:
//! [`HyleManifestState`], [`HyleDataState`], [`HyleDataField`],
//! [`HyleFilterField`], [`FieldChange`], [`FieldChangeFn`], [`FieldChangeMap`],
//! [`FormErrors`], [`AdapterFiltersOptions`], [`AdapterFormOptions`],
//! [`UseFormaOptions`].
//!
//! ## For framework adapter authors
//!
//! Pure computation helpers that implement the core hook algorithms. These are
//! public so that adapter crates (e.g. `hyle-dioxus`, a future `hyle-leptos`)
//! can wrap them with their own reactive primitives.
//!
//! **End-users building UIs should not call these directly** — use the hooks
//! provided by `hyle-dioxus` or `@tty-pt/hyle-react` instead:
//! [`compute_manifest`], [`compute_data`], [`build_effective_query`],
//! [`run_purify`], [`build_filter_fields`], [`apply_change`],
//! [`compute_forma_result`].

use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use indexmap::IndexMap;
use serde_json::Value as JsonValue;

use crate::{
    Blueprint, Column, Field, FieldType, Forma, FormaContext, Manifest, Outcome,
    PurifyError, Query, Row, Source, Value,
    display_value, forma_to_query, purify_row_sync,
};
use crate::view::derive_columns;
use crate::raw::rows_from_outcome;

// ── Field change ──────────────────────────────────────────────────────────────

/// The result of a per-field `change` transform.
///
/// `R` is the per-field render override type supplied by framework adapters.
/// Defaults to `()` for adapters that do not use per-field render overrides.
pub struct FieldChange<R = ()> {
    pub field: Field,
    /// Optional per-field render closure supplied by the adapter.
    pub render: Option<R>,
}

impl<R> FieldChange<R> {
    /// Convenience constructor: override only the label, keep all other field
    /// metadata and no custom render.
    pub fn label(f: &Field, label: impl Into<String>) -> Option<FieldChange<R>> {
        Some(FieldChange { field: Field { label: label.into(), ..f.clone() }, render: None })
    }
}

/// A single per-field transform function (render type erased to `()`).
pub type FieldChangeFn = Rc<dyn Fn(&Field) -> Option<FieldChange>>;

/// A map of per-field transform functions keyed by field name.
///
/// Return `Some(FieldChange { .. })` to override a field's metadata.
/// Return `None` to exclude the field from the filter/form and from validation.
pub type FieldChangeMap = HashMap<String, FieldChangeFn>;

// ── Manifest state ────────────────────────────────────────────────────────────

/// State returned by a manifest hook (e.g. `use_manifest`).
#[derive(Clone, PartialEq)]
pub enum HyleManifestState {
    Ready { manifest: Manifest },
    Error { error: String },
}

// ── Data state ────────────────────────────────────────────────────────────────

/// A single resolved field value with a pre-built display renderer.
pub struct HyleDataField {
    pub key: String,
    pub label: String,
    pub field: Field,
    /// The raw JSON value for this field in the current row.
    pub raw: Value,
    /// Calls `hyle::display_value` with the captured blueprint/outcome/row context.
    pub render: Rc<dyn Fn() -> String>,
}

impl Clone for HyleDataField {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            label: self.label.clone(),
            field: self.field.clone(),
            raw: self.raw.clone(),
            render: self.render.clone(),
        }
    }
}

impl PartialEq for HyleDataField {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
            && self.label == other.label
            && self.field == other.field
            && self.raw == other.raw
    }
}

/// State returned by a data hook (e.g. `use_data`).
#[derive(Clone, PartialEq)]
pub enum HyleDataState {
    /// Waiting for the source adapter to return data.
    Loading { manifest: Option<Manifest> },
    /// The manifest derivation or data resolution failed.
    Error {
        error: String,
        manifest: Option<Manifest>,
    },
    /// Data resolved successfully.
    Ready {
        manifest: Manifest,
        outcome: Outcome,
        rows: Vec<Row>,
        /// Resolved columns (field key, label, field metadata).
        columns: Vec<Column>,
        /// Set when the manifest method is `"one"` or the result is a single record.
        row: Option<Row>,
        /// Per-field accessors; only populated when `row` is `Some`.
        fields: Vec<HyleDataField>,
    },
}

// ── Filter field ──────────────────────────────────────────────────────────────

/// Metadata for a single filter input, pre-computed by a filters hook.
///
/// For `Reference` fields, `options` is pre-resolved from the outcome lookups
/// so the filter component does not need access to the outcome.
///
/// The `R` type parameter is a per-field render override supplied by framework
/// adapters (e.g. `Rc<dyn Fn(Props) -> Element>` in `hyle-dioxus`).  It
/// defaults to `()` so callers that do not need per-field render overrides can
/// use `HyleFilterField` without a type argument.
pub struct HyleFilterField<R = ()> {
    pub key: String,
    pub label: String,
    pub field: Field,
    /// Pre-resolved `(id, display_label)` pairs for `Reference` fields.
    pub options: Option<Vec<(String, String)>>,
    /// Optional per-field render override installed by the `change` map.
    pub render: Option<R>,
}

impl<R: Clone> Clone for HyleFilterField<R> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            label: self.label.clone(),
            field: self.field.clone(),
            options: self.options.clone(),
            render: self.render.clone(),
        }
    }
}

impl<R> PartialEq for HyleFilterField<R> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
            && self.label == other.label
            && self.field == other.field
            && self.options == other.options
        // render intentionally excluded (opaque closure)
    }
}

// ── Filter/form options ───────────────────────────────────────────────────────

/// Options for a filters hook (e.g. `use_filters`) — framework-agnostic base.
///
/// Framework adapters define their own options type (e.g.
/// `hyle_dioxus::UseFiltersOptions`) that wraps this with a render-typed
/// `change` map.  Use this type when building a custom adapter without a
/// Dioxus or React render override.
#[derive(Default)]
pub struct AdapterFiltersOptions {
    /// Pre-seed the committed filter values (e.g. from URL query params on SSR).
    pub initial_committed: IndexMap<String, String>,
    /// Per-field transform map.
    pub change: Option<FieldChangeMap>,
}

/// Options for a forma hook (e.g. `use_forma`).
pub struct UseFormaOptions {
    /// Which field subset to use when deriving the query. Defaults to `Column`.
    pub context: FormaContext,
}

impl Default for UseFormaOptions {
    fn default() -> Self {
        Self { context: FormaContext::Column }
    }
}

/// Options for a form hook (e.g. `use_form`) — framework-agnostic base.
///
/// Framework adapters define their own options type (e.g.
/// `hyle_dioxus::UseFormOptions`) that wraps this with a render-typed
/// `change` map.
#[derive(Default)]
pub struct AdapterFormOptions {
    /// Pre-seed the committed form values (e.g. from URL params on SSR).
    pub initial_committed: IndexMap<String, String>,
    /// Per-field transform map — same semantics as `AdapterFiltersOptions.change`.
    pub change: Option<FieldChangeMap>,
}

impl AdapterFormOptions {
    /// Add a single field transform, building the change map lazily.
    pub fn with_change(
        mut self,
        key: &str,
        f: impl Fn(&Field) -> Option<FieldChange> + 'static,
    ) -> Self {
        self.change
            .get_or_insert_with(HashMap::new)
            .insert(key.to_owned(), Rc::new(f));
        self
    }
}

// ── Form errors ───────────────────────────────────────────────────────────────

/// Pre-rendered validation errors injected by a POST handler into the server
/// context so that form pages can display them without JS.
#[derive(Clone, PartialEq, Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct FormErrors(pub IndexMap<String, String>);

impl FormErrors {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

// ── Pure helpers (for framework adapter authors) ──────────────────────────────
//
// End-users building UIs should use the hooks from `hyle-dioxus` or
// `@tty-pt/hyle-react` rather than calling these functions directly.

/// Derive a `HyleManifestState` from a blueprint and query.
///
/// *For framework adapter authors.* Wraps `Blueprint::manifest` into the
/// standard loading/ready/error state enum used by manifest hooks.
#[doc(hidden)]
pub fn compute_manifest(blueprint: &Blueprint, query: &Query) -> HyleManifestState {
    match blueprint.manifest(query.clone()) {
        Ok(m) => HyleManifestState::Ready { manifest: m },
        Err(e) => HyleManifestState::Error { error: e.to_string() },
    }
}

/// Resolve a `HyleDataState` given a manifest and source.
///
/// *For framework adapter authors.* Core algorithm for data hooks
/// (`use_data`, `useData`): runs `Blueprint::resolve`, applies the view, and
/// builds the `fields` list from the columns.
#[doc(hidden)]
pub fn compute_data(
    blueprint: Arc<Blueprint>,
    manifest: Manifest,
    source: Source,
) -> HyleDataState {
    match blueprint.resolve_and_view(&manifest, &source) {
        Err(e) => HyleDataState::Error {
            error: e.to_string(),
            manifest: Some(manifest),
        },
        Ok(view) => {
            let row = if view.is_single { view.rows.first().cloned() } else { None };
            let fields = build_fields(&blueprint, &manifest, &view.outcome, &view.columns, row.as_ref());
            HyleDataState::Ready {
                manifest,
                outcome: view.outcome,
                rows: view.rows,
                columns: view.columns,
                row,
                fields,
            }
        }
    }
}

/// Build per-field display accessors for a single resolved row.
fn build_fields(
    blueprint: &Blueprint,
    manifest: &Manifest,
    outcome: &Outcome,
    columns: &[Column],
    row: Option<&Row>,
) -> Vec<HyleDataField> {
    let arc_bp = Arc::new(blueprint.clone());
    let arc_oc = Arc::new(outcome.clone());
    let base = manifest.base.clone();

    columns
        .iter()
        .map(|col| {
            let raw = row
                .and_then(|r| r.get(&col.key))
                .cloned()
                .unwrap_or(Value::Null);

            let bp2 = arc_bp.clone();
            let oc2 = arc_oc.clone();
            let base2 = base.clone();
            let key2 = col.key.clone();
            let raw2 = raw.clone();

            HyleDataField {
                key: col.key.clone(),
                label: col.label.clone(),
                field: col.field.clone(),
                raw,
                render: Rc::new(move || display_value(&bp2, &oc2, &base2, &key2, &raw2)),
            }
        })
        .collect()
}

/// Apply committed filter values and optional sort/pagination to produce the
/// effective query passed to the source adapter and resolver.
///
/// *For framework adapter authors.* Used by filter/list hooks to merge
/// user-committed filter state into the base query before fetching.
#[doc(hidden)]
pub fn build_effective_query(
    base: &Query,
    committed: &IndexMap<String, String>,
    page: usize,
    per_page: usize,
    sort_field: Option<&str>,
    sort_ascending: bool,
) -> Query {
    let mut where_ = base.where_.clone();
    for (k, v) in committed {
        if !v.is_empty() {
            where_.insert(k.clone(), JsonValue::String(v.clone()));
        }
    }

    let sort = sort_field
        .map(|f| crate::Sort { field: f.to_owned(), ascending: sort_ascending })
        .or_else(|| base.sort.clone());

    Query {
        where_,
        page: Some(page),
        per_page: Some(per_page),
        sort,
        ..base.clone()
    }
}

/// Run `purify_row_sync` and return errors as a `FormErrors` map, or `None` if valid.
///
/// *For framework adapter authors.* Thin adapter over [`purify_row_sync`] that
/// converts the error list into the `FormErrors` map used by form hooks.
#[doc(hidden)]
pub fn run_purify(
    blueprint: &Blueprint,
    model_name: &str,
    form_data: &IndexMap<String, String>,
) -> Option<Vec<PurifyError>> {
    let row: Row = form_data
        .iter()
        .map(|(k, v)| (k.clone(), JsonValue::String(v.clone())))
        .collect();
    purify_row_sync(blueprint, model_name, &row).err()
}

/// Build filter field metadata from a manifest and outcome.
///
/// *For framework adapter authors.* For `Reference` fields the lookup table
/// from the outcome is pre-resolved
/// into `(id, display_label)` pairs so filter components need no access to
/// the outcome at render time.
///
/// The returned fields carry `render: None` (type `()`).  Framework adapters
/// that need per-field render overrides should call `apply_change` with a
/// render-typed `FieldChangeMap<R>` afterwards.
#[doc(hidden)]
pub fn build_filter_fields(
    blueprint: &Blueprint,
    manifest: &Manifest,
    outcome: &Outcome,
) -> Vec<HyleFilterField> {
    let columns = match derive_columns(blueprint, manifest) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    columns
        .into_iter()
        .map(|col| {
            let options = match &col.field.field_type {
                FieldType::Reference { reference } => {
                    let pairs = outcome
                        .lookups
                        .get(&reference.entity)
                        .map(|lookup| {
                            lookup
                                .iter()
                                .map(|(id, row)| {
                                    let label = row
                                        .get(&reference.display_field)
                                        .and_then(|v| v.as_str())
                                        .unwrap_or(id.as_str())
                                        .to_owned();
                                    (id.clone(), label)
                                })
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                    Some(pairs)
                }
                FieldType::Array { item } => {
                    if let FieldType::Reference { reference } = item.as_ref() {
                        let pairs = outcome
                            .lookups
                            .get(&reference.entity)
                            .map(|lookup| {
                                lookup
                                    .iter()
                                    .map(|(id, row)| {
                                        let label = row
                                            .get(&reference.display_field)
                                            .and_then(|v| v.as_str())
                                            .unwrap_or(id.as_str())
                                            .to_owned();
                                        (id.clone(), label)
                                    })
                                    .collect::<Vec<_>>()
                            })
                            .unwrap_or_default();
                        Some(pairs)
                    } else {
                        // Scan rows for distinct primitive values of this field.
                        let mut seen = std::collections::HashSet::new();
                        let mut pairs = vec![];
                        for row in rows_from_outcome(outcome) {
                            if let Some(Value::Array(arr)) = row.get(col.key.as_str()) {
                                for item in arr {
                                    let s = match item {
                                        Value::String(s) => s.clone(),
                                        other => other.to_string(),
                                    };
                                    if seen.insert(s.clone()) {
                                        pairs.push((s.clone(), s));
                                    }
                                }
                            }
                        }
                        if pairs.is_empty() { None } else { Some(pairs) }
                    }
                }
                _ => None,
            };

            HyleFilterField { key: col.key, label: col.label, field: col.field, options, render: None }
        })
        .collect()
}

/// Apply a `FieldChangeMap` to a list of filter fields.
///
/// *For framework adapter authors.* Processes the per-field `change` map
/// from options structs (e.g. `UseFiltersOptions::change`) against the
/// list produced by `build_filter_fields`.
///
/// - Key absent from `change` → keep as-is (render stays `None`).
/// - `change[key](&field)` returns `None` → exclude the field entirely.
/// - Returns `Some(FieldChange { field, render })` → replace field metadata
///   and attach the optional render closure.
///
/// `R` is the render override type carried by the adapter's `FieldChange` /
/// `HyleFilterField`.  Use `FieldChangeMap` (which erases `R` to `()`) when
/// you do not need per-field render overrides.
#[doc(hidden)]
pub fn apply_change<R>(
    fields: Vec<HyleFilterField<R>>,
    change: &HashMap<String, Rc<dyn Fn(&Field) -> Option<FieldChange<R>>>>,
) -> Vec<HyleFilterField<R>> {
    fields
        .into_iter()
        .filter_map(|mut ff| {
            if let Some(change_fn) = change.get(&ff.key) {
                match change_fn(&ff.field) {
                    None => return None,
                    Some(FieldChange { field, render }) => {
                        ff.label = field.label.clone();
                        ff.field = field;
                        ff.render = render;
                    }
                }
            }
            Some(ff)
        })
        .collect()
}

/// Derive `(Option<Query>, Option<Forma>)` from a resolved "forma" data row.
///
/// *For framework adapter authors.* Core algorithm for `use_forma` /
/// `useForma`: parses the forma definition stored in the resolved data row
/// and converts it to a query for the actual data fetch.
#[doc(hidden)]
pub fn compute_forma_result(
    data: &HyleDataState,
    table_name: &str,
    id: Option<JsonValue>,
    context: &FormaContext,
) -> (Option<Query>, Option<Forma>) {
    let row = match data {
        HyleDataState::Ready { row: Some(r), .. } => r,
        _ => return (None, None),
    };

    let forma: Forma = match serde_json::from_value(JsonValue::Object(
        row.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
    )) {
        Ok(f) => f,
        Err(_) => return (None, None),
    };

    if forma.fields.is_empty() {
        return (None, Some(forma));
    }

    let derived = forma_to_query(&forma, table_name, context, id.as_ref());
    (Some(derived), Some(forma))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Field, Model};
    use crate::raw::ModelRows;
    use indexmap::IndexMap;
    use serde_json::json;

    fn user_blueprint() -> Blueprint {
        Blueprint::new()
            .model(
                "user",
                Model::new()
                    .field("name", Field::string("Name").with_metadata("required", json!(true)))
                    .field("email", Field::string("Email"))
                    .field("active", Field::boolean("Active")),
            )
    }

    fn user_query() -> Query {
        Query::new("user").select(["name", "email", "active"])
    }

    fn user_source(rows: Vec<Row>) -> Source {
        let mut src = Source::new();
        src.insert("user".to_owned(), crate::ModelResult::many(rows));
        src
    }

    fn alice() -> Row {
        indexmap::indexmap! {
            "id".to_owned()     => json!(1),
            "name".to_owned()   => json!("Alice"),
            "email".to_owned()  => json!("alice@example.test"),
            "active".to_owned() => json!(true),
        }
    }

    // ── compute_manifest ──────────────────────────────────────────────────────

    #[test]
    fn manifest_ok() {
        let bp = user_blueprint();
        let state = compute_manifest(&bp, &user_query());
        assert!(matches!(state, HyleManifestState::Ready { .. }));
        if let HyleManifestState::Ready { manifest } = state {
            assert_eq!(manifest.base, "user");
            assert!(manifest.fields.contains(&"name".to_owned()));
        }
    }

    #[test]
    fn manifest_unknown_model() {
        let bp = user_blueprint();
        let state = compute_manifest(&bp, &Query::new("ghost"));
        assert!(matches!(state, HyleManifestState::Error { .. }));
    }

    // ── compute_data ──────────────────────────────────────────────────────────

    #[test]
    fn data_ready_with_rows() {
        let bp = Arc::new(user_blueprint());
        let manifest = bp.manifest(user_query()).unwrap();
        let src = user_source(vec![alice()]);
        let state = compute_data(bp, manifest, src);
        assert!(matches!(state, HyleDataState::Ready { .. }));
        if let HyleDataState::Ready { rows, row, .. } = state {
            assert_eq!(rows.len(), 1);
            assert!(row.is_none());
        }
    }

    #[test]
    fn data_ready_single_record() {
        let bp = Arc::new(user_blueprint());
        let q = Query {
            model: "user".to_owned(),
            select: vec!["name".to_owned(), "email".to_owned(), "active".to_owned()],
            method: Some("one".to_owned()),
            ..Default::default()
        };
        let manifest = bp.manifest(q).unwrap();
        let state = compute_data(bp, manifest, user_source(vec![alice()]));
        if let HyleDataState::Ready { row, .. } = state {
            assert!(row.is_some());
        } else {
            panic!("expected Ready");
        }
    }

    #[test]
    fn data_fields_built_for_single_record() {
        let bp = Arc::new(user_blueprint());
        let q = Query {
            model: "user".to_owned(),
            select: vec!["name".to_owned()],
            method: Some("one".to_owned()),
            ..Default::default()
        };
        let manifest = bp.manifest(q).unwrap();
        let state = compute_data(bp, manifest, user_source(vec![alice()]));
        if let HyleDataState::Ready { fields, .. } = state {
            assert!(!fields.is_empty());
            let name_field = fields.iter().find(|f| f.key == "name").unwrap();
            assert_eq!(name_field.render.as_ref()(), "Alice");
        } else {
            panic!("expected Ready");
        }
    }

    // ── build_effective_query ─────────────────────────────────────────────────

    #[test]
    fn effective_query_merges_committed() {
        let mut committed = IndexMap::new();
        committed.insert("name".to_owned(), "Bob".to_owned());
        let q = build_effective_query(&user_query(), &committed, 1, 20, None, true);
        assert_eq!(q.where_.get("name"), Some(&json!("Bob")));
    }

    #[test]
    fn effective_query_skips_empty_values() {
        let mut committed = IndexMap::new();
        committed.insert("name".to_owned(), String::new());
        let q = build_effective_query(&user_query(), &committed, 1, 20, None, true);
        assert!(!q.where_.contains_key("name"));
    }

    #[test]
    fn effective_query_applies_sort() {
        let q = build_effective_query(&user_query(), &IndexMap::new(), 2, 50, Some("name"), false);
        assert_eq!(q.page, Some(2));
        assert_eq!(q.per_page, Some(50));
        let sort = q.sort.unwrap();
        assert_eq!(sort.field, "name");
        assert!(!sort.ascending);
    }

    // ── run_purify ────────────────────────────────────────────────────────────

    #[test]
    fn purify_passes_valid_row() {
        let bp = user_blueprint();
        let mut form = IndexMap::new();
        form.insert("name".to_owned(), "Alice".to_owned());
        assert!(run_purify(&bp, "user", &form).is_none());
    }

    #[test]
    fn purify_catches_required_violation() {
        let bp = user_blueprint();
        let errors = run_purify(&bp, "user", &IndexMap::new());
        assert!(errors.is_some());
        assert!(errors.unwrap().iter().any(|e| e.field == "name" && e.rule == "required"));
    }

    // ── apply_change ──────────────────────────────────────────────────────────

    fn make_filter_field(key: &str, label: &str) -> HyleFilterField {
        HyleFilterField {
            key: key.to_owned(),
            label: label.to_owned(),
            field: Field {
                label: label.to_owned(),
                field_type: crate::FieldType::Primitive { primitive: crate::Primitive::String },
                options: Default::default(),
            },
            options: None,
            render: None,
        }
    }

    #[test]
    fn apply_change_identity_when_no_map() {
        let fields = vec![make_filter_field("name", "Name"), make_filter_field("email", "Email")];
        let result = apply_change(fields, &HashMap::new());
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn apply_change_modifies_label() {
        let fields = vec![make_filter_field("name", "Name")];
        let mut change: FieldChangeMap = HashMap::new();
        change.insert(
            "name".to_owned(),
            Rc::new(|f: &Field| Some(FieldChange { field: Field { label: "Full Name".to_owned(), ..f.clone() }, render: None })),
        );
        let result = apply_change(fields, &change);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].label, "Full Name");
    }

    #[test]
    fn apply_change_none_removes_field() {
        let fields = vec![make_filter_field("name", "Name"), make_filter_field("email", "Email")];
        let mut change: FieldChangeMap = HashMap::new();
        change.insert("name".to_owned(), Rc::new(|_: &Field| None));
        let result = apply_change(fields, &change);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].key, "email");
    }

    #[test]
    fn apply_change_unknown_key_passes_through() {
        let fields = vec![make_filter_field("name", "Name")];
        let mut change: FieldChangeMap = HashMap::new();
        change.insert("ghost".to_owned(), Rc::new(|_: &Field| None));
        let result = apply_change(fields, &change);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].key, "name");
    }

    // ── compute_forma_result ──────────────────────────────────────────────────

    fn make_manifest(base: &str) -> Manifest {
        Manifest {
            base: base.to_owned(),
            id: None,
            fields: vec![],
            filter: Default::default(),
            lookups: vec![],
            inlines: vec![],
            page: None,
            per_page: None,
            sort: None,
            method: None,
            filter_fields: vec![],
        }
    }

    fn make_outcome() -> Outcome {
        Outcome { rows: ModelRows::Many(vec![]), total: 0, lookups: Default::default() }
    }

    #[test]
    fn forma_returns_none_while_loading() {
        let state = HyleDataState::Loading { manifest: None };
        let (q, f) = compute_forma_result(&state, "user", None, &FormaContext::Column);
        assert!(q.is_none());
        assert!(f.is_none());
    }

    #[test]
    fn forma_returns_none_query_when_no_fields() {
        let forma = crate::Forma { fields: vec![], ..Default::default() };
        let row: Row = serde_json::from_value(serde_json::to_value(&forma).unwrap()).unwrap();
        let state = HyleDataState::Ready {
            manifest: make_manifest("forma"),
            outcome: make_outcome(),
            rows: vec![row.clone()],
            columns: vec![],
            row: Some(row),
            fields: vec![],
        };
        let (q, f) = compute_forma_result(&state, "user", None, &FormaContext::Column);
        assert!(q.is_none());
        assert!(f.is_some());
    }

    #[test]
    fn forma_derives_query_with_fields() {
        use crate::{Forma, FormaField, FormaFieldType};
        let forma = Forma {
            fields: vec![FormaField {
                name: "name".to_owned(),
                label: "Name".to_owned(),
                field_type: FormaFieldType::Named("string".to_owned()),
                ..Default::default()
            }],
            column: Some(vec!["name".to_owned()]),
            ..Default::default()
        };
        let row: Row = serde_json::from_value(serde_json::to_value(&forma).unwrap()).unwrap();
        let state = HyleDataState::Ready {
            manifest: make_manifest("forma"),
            outcome: make_outcome(),
            rows: vec![row.clone()],
            columns: vec![],
            row: Some(row),
            fields: vec![],
        };
        let (q, f) = compute_forma_result(&state, "user", None, &FormaContext::Column);
        assert!(f.is_some());
        let q = q.expect("should have derived query");
        assert_eq!(q.model, "user");
        assert!(q.select.contains(&"name".to_owned()));
    }
}
