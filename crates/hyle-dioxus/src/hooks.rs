use dioxus_hooks::{use_callback, use_memo, use_signal, use_context};
use dioxus_signals::{Memo, ReadableExt, WritableExt};
use indexmap::IndexMap;
use serde_json::Value as JsonValue;

use hyle::{
    build_effective_query, build_filter_fields, compute_data, compute_forma_result,
    compute_manifest, run_purify,
    Forma, MutateInput, PurifyError, Query, Value,
    HyleDataState, HyleManifestState, UseFormaOptions,
};

use crate::context::use_hyle_config;
use crate::types::{
    HyleAdapter, HyleFilterField, HyleFiltersState, HyleListState,
    HyleSourceState, UseFiltersOptions,
};

// ── Hooks ─────────────────────────────────────────────────────────────────────

/// Reactively derive a `Manifest` from a query.
///
/// Because `hyle` is pure Rust there is no async loading phase — the result is
/// available synchronously on every render.
#[must_use]
pub fn use_manifest(query: Query) -> Memo<HyleManifestState> {
    let config = use_hyle_config();
    use_memo(move || compute_manifest(&config.blueprint, &query))
}

/// Reactively resolve data for a query.
#[must_use]
pub fn use_data(query: Query) -> Memo<HyleDataState> {
    let config = use_hyle_config();
    let adapter = use_context::<HyleAdapter>();
    use_memo(move || {
        let bp = config.blueprint.clone();
        let source = adapter.source;

        let manifest = match bp.manifest(query.clone()) {
            Ok(m) => m,
            Err(e) => return HyleDataState::Error { error: e.to_string(), manifest: None },
        };

        match source.read().clone() {
            HyleSourceState::Loading => HyleDataState::Loading { manifest: Some(manifest) },
            HyleSourceState::Error(e) => HyleDataState::Error { error: e, manifest: Some(manifest) },
            HyleSourceState::Ready(src) => compute_data(bp, manifest, src),
        }
    })
}

/// Shared data-memo body used by both `use_list` and `use_list_with_filters`.
fn use_list_data(effective_query: Memo<Query>) -> Memo<HyleDataState> {
    let config = use_hyle_config();
    let adapter = use_context::<HyleAdapter>();
    use_memo(move || {
        let bp = config.blueprint.clone();
        let source = adapter.source;
        let query = effective_query.read().clone();

        let manifest = match bp.manifest(query) {
            Ok(m) => m,
            Err(e) => return HyleDataState::Error { error: e.to_string(), manifest: None },
        };

        match source.read().clone() {
            HyleSourceState::Loading => HyleDataState::Loading { manifest: Some(manifest) },
            HyleSourceState::Error(e) => HyleDataState::Error { error: e, manifest: Some(manifest) },
            HyleSourceState::Ready(src) => compute_data(bp, manifest, src),
        }
    })
}

/// Reactive list view with pagination and sort signals.
#[must_use]
pub fn use_list(query: Query) -> HyleListState {
    let page = use_signal(|| query.page.unwrap_or(1));
    let per_page = use_signal(|| query.per_page.unwrap_or(5));
    let sort_field = use_signal(|| query.sort.as_ref().map(|s| s.field.clone()));
    let sort_ascending = use_signal(|| query.sort.as_ref().map(|s| s.ascending).unwrap_or(true));

    let effective_query = use_memo(move || {
        build_effective_query(
            &query,
            &IndexMap::new(),
            page(),
            per_page(),
            sort_field().as_deref(),
            sort_ascending(),
        )
    });

    let data = use_list_data(effective_query);

    HyleListState { data, query: effective_query, page, per_page, sort_field, sort_ascending }
}

/// Reactive list view driven by a live `HyleFiltersState`.
#[must_use]
pub fn use_list_with_filters(filters: HyleFiltersState) -> HyleListState {
    let base = filters.query.read().clone();
    let page = use_signal(|| base.page.unwrap_or(1));
    let per_page = use_signal(|| base.per_page.unwrap_or(5));
    let sort_field = use_signal(|| base.sort.as_ref().map(|s| s.field.clone()));
    let sort_ascending = use_signal(|| base.sort.as_ref().map(|s| s.ascending).unwrap_or(true));
    let filter_query = filters.query;

    let effective_query = use_memo(move || {
        let base = filter_query.read().clone();
        build_effective_query(
            &base,
            &IndexMap::new(),
            page(),
            per_page(),
            sort_field().as_deref(),
            sort_ascending(),
        )
    });

    let data = use_list_data(effective_query);

    HyleListState { data, query: effective_query, page, per_page, sort_field, sort_ascending }
}

/// Reactive filter/form state with validation.
///
/// - `set_field` updates `form_data` without committing.
/// - `filter_apply` merges `form_data` into the effective `where_` clause.
/// - `filter_clear` resets both `form_data` and committed state.
/// - `validate` runs `purify_row_sync` and updates `purify_errors`.
///
/// When `query.where_` contains an `"id"` key, `use_data` is called internally
/// to seed `form_data` from the existing record.
#[must_use]
pub fn use_filters(
    query: Query,
    options: UseFiltersOptions,
) -> HyleFiltersState {
    let config = use_hyle_config();

    let initial = options.initial_committed;
    let initial2 = initial.clone();
    let mut committed = use_signal(move || initial.clone());
    let mut form_data = use_signal(move || initial2.clone());
    let mut filter_reset_key = use_signal(|| 0u32);
    let mut purify_errors = use_signal(|| Option::<Vec<PurifyError>>::None);
    let change = options.change;

    // When an id is present, fetch the existing record to seed form_data.
    // When no id is present (filter mode), fetch without filters/pagination so
    // we still get a manifest + lookups to populate HyleFilterField metadata.
    let has_id = query.where_.contains_key("id");
    let seed_query = if has_id {
        query.clone()
    } else {
        Query { model: query.model.clone(), select: query.select.clone(), ..Default::default() }
    };
    let seed_data = use_data(seed_query);

    let bp_for_fields = config.blueprint.clone();
    let bp_for_validate = config.blueprint.clone();

    // Derive fields reactively from seed_data so they are available on SSR
    // (use_effect doesn't run during server-side rendering).
    let fields = use_memo(move || {
        let raw_fields = match &*seed_data.read() {
            HyleDataState::Ready { row: Some(r), manifest, outcome, .. } => {
                // Seed form_data from the row the first time it arrives.
                let seeded: IndexMap<String, String> = r
                    .iter()
                    .map(|(k, v)| {
                        let s = match v {
                            Value::String(s) => s.clone(),
                            Value::Null => String::new(),
                            other => other.to_string(),
                        };
                        (k.clone(), s)
                    })
                    .collect();
                form_data.set(seeded);
                // NOTE: `form_data.set()` is a side-effect inside a `use_memo` (normally a
                // pure computation). Ideally this would live in a `use_effect`, but Dioxus
                // SSR does not execute `use_effect` during server-side rendering, so moving
                // it there would break pre-rendered forms. The extra render cycle this causes
                // is acceptable given the SSR constraint.
                build_filter_fields(&bp_for_fields, manifest, outcome)
                    .into_iter()
                    .map(|f| HyleFilterField { key: f.key, label: f.label, field: f.field, options: f.options, render: None })
                    .collect()
            }
            HyleDataState::Ready { manifest, outcome, .. } => {
                build_filter_fields(&bp_for_fields, manifest, outcome)
                    .into_iter()
                    .map(|f| HyleFilterField { key: f.key, label: f.label, field: f.field, options: f.options, render: None })
                    .collect()
            }
            _ => vec![],
        };
        if let Some(ref c) = change {
            hyle::apply_change(raw_fields, c)
        } else {
            raw_fields
        }
    });

    let effective_query = use_memo(move || {
        let q = query.clone();
        let committed_snapshot = committed.cloned();
        build_effective_query(&q, &committed_snapshot, q.page.unwrap_or(1), q.per_page.unwrap_or(5), None, true)
    });

    let set_field = use_callback(move |(name, value): (String, String)| {
        form_data.with_mut(|m: &mut IndexMap<String, String>| { m.insert(name, value); });
    });

    let filter_apply = use_callback(move |()| {
        let snapshot = form_data.cloned();
        committed.with_mut(|c: &mut IndexMap<String, String>| c.extend(snapshot));
    });

    let filter_clear = use_callback(move |()| {
        form_data.set(IndexMap::new());
        committed.set(IndexMap::new());
        filter_reset_key.with_mut(|k| *k += 1);
    });

    let validate = use_callback(move |()| {
        let snapshot = form_data.cloned();
        let model_name = effective_query.read().model.clone();
        let active_keys: std::collections::HashSet<String> =
            fields.read().iter().map(|f| f.key.clone()).collect();
        let active_snapshot: IndexMap<String, String> = snapshot
            .iter()
            .filter(|(k, _)| active_keys.contains(*k))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let errors = run_purify(&bp_for_validate, &model_name, &active_snapshot);
        purify_errors.set(errors);
    });

    HyleFiltersState {
        query: effective_query,
        fields,
        form_data,
        set_field,
        filter_apply,
        filter_clear,
        filter_reset_key,
        validate,
        purify_errors,
    }
}

/// Auto-wired form hook. Derives edit/create mode from the query, reads the
/// appropriate mutation from `HyleAdapterConfig` context, and delegates filter
/// state to [`use_filters`].
///
/// # Panics
///
/// Panics if `use_adapter_config!` has not been called at the app root.
#[must_use]
pub fn use_form(
    query: Query,
    opts: crate::types::UseFormOptions,
) -> crate::types::HyleFormState {
    use dioxus::prelude::try_consume_context;

    let adapter = try_consume_context::<HyleAdapter>()
        .expect("HyleAdapter must be provided via use_adapter_config! at the app root");

    let is_edit = query.where_.contains_key("id");
    let model = query.model.clone();
    let mutation = if is_edit { adapter.update } else { adapter.create };

    let filters = use_filters(
        query,
        crate::types::UseFiltersOptions {
            initial_committed: opts.initial_committed,
            change: opts.change,
        },
    );

    let is_valid = filters.purify_errors.read().is_none();

    let on_submit = use_callback(move |()| {
        filters.validate.call(());
        if filters.purify_errors.read().is_some() {
            return;
        }
        let snapshot = filters.form_data.cloned();
        let id = snapshot.get("id").map(|v| {
            v.parse::<u64>().map(JsonValue::from).unwrap_or_else(|_| JsonValue::String(v.clone()))
        });
        mutation.mutate.call(MutateInput { model: model.clone(), id, data: snapshot });
    });

    crate::types::HyleFormState { filters, is_edit, is_valid, on_submit, mutation }
}

/// Returns create/update/delete mutation handles with `model` pre-bound.
///
/// # Panics
///
/// Panics if `use_adapter_config!` has not been called at the app root.
#[must_use]
pub fn use_mutation(model: &'static str) -> crate::types::BoundMutations {
    use dioxus::prelude::try_consume_context;
    use crate::types::{BoundMutation, BoundMutateInput, BoundMutations};

    let adapter = try_consume_context::<HyleAdapter>()
        .expect("HyleAdapter must be provided via use_adapter_config! at the app root");

    let bind = |hm: crate::types::HyleMutation| -> BoundMutation {
        let mutate = use_callback(move |input: BoundMutateInput| {
            hm.mutate.call(MutateInput { model: model.to_owned(), id: input.id, data: input.data });
        });
        BoundMutation { mutate, is_pending: hm.is_pending, is_success: hm.is_success, error: hm.error }
    };

    BoundMutations {
        create: bind(adapter.create),
        update: bind(adapter.update),
        delete: bind(adapter.delete),
    }
}

/// Fetch a forma definition from the `"forma"` model and derive a query for
/// the target table.
///
/// Returns a `Memo` that resolves to `(Option<Query>, Option<Forma>)`.
#[must_use]
pub fn use_forma(
    table_name: &'static str,
    id: Option<JsonValue>,
    opts: UseFormaOptions,
) -> Memo<(Option<Query>, Option<Forma>)> {
    use crate::types::FORMA_MODEL;

    let forma_query = Query {
        model: FORMA_MODEL.to_owned(),
        where_: indexmap::indexmap! { "id".to_owned() => JsonValue::String(table_name.to_owned()) },
        method: Some("one".to_owned()),
        select: vec![
            "fields".to_owned(),
            "detail".to_owned(),
            "form".to_owned(),
            "column".to_owned(),
            "filters".to_owned(),
        ],
        ..Default::default()
    };

    let data = use_data(forma_query);

    use_memo(move || {
        compute_forma_result(&data.cloned(), table_name, id.clone(), &opts.context)
    })
}

// ── Utilities ─────────────────────────────────────────────────────────────────

/// Convert an `IndexMap<String, String>` (e.g. a parsed HTML form body) into a
/// JSON [`Value`] object.
///
/// ```rust
/// use indexmap::IndexMap;
/// use hyle_dioxus::{form_body, Value};
///
/// let mut form = IndexMap::new();
/// form.insert("name".into(), "Alice".into());
/// let body = form_body(&form);
/// assert_eq!(body["name"], Value::String("Alice".into()));
/// ```
pub fn form_body(data: &IndexMap<String, String>) -> Value {
    let map: serde_json::Map<String, Value> = data
        .iter()
        .map(|(k, v)| (k.clone(), Value::String(v.clone())))
        .collect();
    Value::Object(map)
}
