use dioxus::prelude::*;

use hyle::{FieldType, Primitive};

use crate::context::use_hyle_components;
use crate::types::{field_type_key, HyleFilterField, HyleFilterFieldProps, HyleFiltersState};

/// A reactive filter input derived from `HyleFiltersState`.
///
/// Looks up the field definition in `state.fields`, reads the current value
/// from `state.form_data`, and calls `state.set_field` on change.
///
/// By default it renders:
/// - `Boolean` fields → `<select>` with "Any / Yes / No"
/// - `Reference` fields → `<select>` populated from pre-resolved `options`
/// - Everything else → `<input type="text">` (or the `input.kind` hint from
///   the blueprint, e.g. `"search"`, `"number"`)
///
/// Pass `render` to replace the default for a specific field:
///
/// ```rust,ignore
/// FilterField {
///     state: filters,
///     field_key: "role",
///     render: |props| rsx! { MySelectInput { props } },
/// }
/// ```
///
/// # Props
/// - `state` — the `HyleFiltersState` from `use_filters`
/// - `field_key` — the field name to render
/// - `render` — optional custom renderer receiving `HyleFilterFieldProps`
#[component]
pub fn FilterField(
    state: HyleFiltersState,
    field_key: String,
    render: Option<fn(HyleFilterFieldProps) -> Element>,
) -> Element {
    let filter_field: Option<HyleFilterField> = state
        .fields
        .read()
        .iter()
        .find(|f| f.key == field_key)
        .cloned();

    let Some(ff) = filter_field else {
        return rsx! {};
    };

    let key = ff.key.clone();
    let value = state
        .form_data
        .read()
        .get(&key)
        .cloned()
        .unwrap_or_default();

    let set: Callback<String> = {
        let key = key.clone();
        Callback::new(move |v: String| state.set_field.call((key.clone(), v)))
    };

    // Priority 1: per-field render from the `change` map (highest).
    if let Some(ref render_fn) = ff.render {
        return render_fn(HyleFilterFieldProps {
            key: ff.key,
            label: ff.label,
            field: ff.field,
            options: ff.options,
            value,
            set,
        });
    }

    // Priority 2: render prop supplied by the caller.
    if let Some(render_fn) = render {
        return render_fn(HyleFilterFieldProps {
            key: ff.key,
            label: ff.label,
            field: ff.field,
            options: ff.options,
            value,
            set,
        });
    }

    // Priority 3: global HyleComponents context.
    let props = HyleFilterFieldProps {
        key: ff.key.clone(),
        label: ff.label.clone(),
        field: ff.field.clone(),
        options: ff.options.clone(),
        value: value.clone(),
        set: set.clone(),
    };
    if let Some(components) = use_hyle_components() {
        let type_key = field_type_key(&ff.field.field_type);
        if let Some(render_fn) = components.filters.get(type_key).copied() {
            return render_fn(props);
        }
    }

    // Priority 4: built-in default.
    default_input(ff, value, set)
}

// ── Default input dispatch ────────────────────────────────────────────────────

fn default_input(ff: HyleFilterField, value: String, set: Callback<String>) -> Element {
    match &ff.field.field_type {
        FieldType::Primitive {
            primitive: Primitive::Boolean,
        } => boolean_select(ff.key, ff.label, value, set),

        FieldType::Reference { .. } => {
            reference_select(ff.key, ff.label, value, ff.options.unwrap_or_default(), set)
        }

        _ => text_input(ff, value, set),
    }
}

fn boolean_select(name: String, label: String, value: String, set: Callback<String>) -> Element {
    rsx! {
        select {
            name: "{name}",
            aria_label: "{label}",
            onchange: move |e| set.call(e.value()),
            option { value: "", selected: value.is_empty(), "Any" }
            option { value: "true", selected: value == "true", "Yes" }
            option { value: "false", selected: value == "false", "No" }
        }
    }
}

fn reference_select(
    name: String,
    label: String,
    value: String,
    options: Vec<(String, String)>,
    set: Callback<String>,
) -> Element {
    rsx! {
        select {
            name: "{name}",
            aria_label: "{label}",
            onchange: move |e| set.call(e.value()),
            option { value: "", selected: value.is_empty(), "All {label}s" }
            for (id, display) in options {
                option { key: "{id}", value: "{id}", selected: value == id, "{display}" }
            }
        }
    }
}

fn text_input(ff: HyleFilterField, value: String, set: Callback<String>) -> Element {
    let input_type = ff
        .field
        .options
        .input
        .as_ref()
        .map(|i| i.kind.clone())
        .unwrap_or_else(|| "text".to_owned());

    rsx! {
        input {
            r#type: "{input_type}",
            name: "{ff.key}",
            aria_label: "{ff.label}",
            placeholder: "{ff.label}",
            value: "{value}",
            oninput: move |e| set.call(e.value()),
        }
    }
}
