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
/// - `Boolean` fields → `<label>` wrapping a `<select>` with "Any / Yes / No"
/// - `Reference` fields → `<label>` wrapping a `<select>` populated from pre-resolved `options`
/// - `Array<Reference>` fields → `<fieldset>/<legend>` with one checkbox per option
/// - Everything else → `<label>` wrapping an `<input type="text">` (or the `input.kind` hint
///   from the blueprint, e.g. `"search"`, `"number"`)
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

/// Like [`FilterField`] but intended for form contexts: boolean fields render
/// as a self-labelling checkbox (label on the right) instead of a 3-state
/// select, matching the React `FilterBoolean` checkbox appearance.
#[component]
pub fn FormFilterField(
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

    if let Some(ref render_fn) = ff.render {
        return render_fn(HyleFilterFieldProps {
            key: ff.key, label: ff.label, field: ff.field, options: ff.options, value, set,
        });
    }
    if let Some(render_fn) = render {
        return render_fn(HyleFilterFieldProps {
            key: ff.key, label: ff.label, field: ff.field, options: ff.options, value, set,
        });
    }

    form_default_input(ff, value, set)
}

fn form_default_input(ff: HyleFilterField, value: String, set: Callback<String>) -> Element {
    match &ff.field.field_type {
        FieldType::Primitive {
            primitive: Primitive::Boolean,
        } => boolean_checkbox(ff.key, ff.label, value, set),
        _ => default_input(ff, value, set),
    }
}

// ── Default input dispatch ────────────────────────────────────────────────────

fn default_input(ff: HyleFilterField, value: String, set: Callback<String>) -> Element {
    match &ff.field.field_type {
        FieldType::Primitive {
            primitive: Primitive::Boolean,
        } => boolean_select(ff.key, ff.label, value, set),

        FieldType::Reference { .. } => match ff.options {
            Some(options) => reference_select(ff.key, ff.label, value, options, set),
            None => reference_select_loading(ff.key, ff.label),
        },

        FieldType::Array { .. } => {
            match ff.options {
                Some(options) => checkbox_reference_fieldset(ff.key, ff.label, value, options, set),
                None => rsx! {
                    fieldset {
                        legend { "{ff.label}" }
                        span { aria_busy: "true", "Loading…" }
                    }
                },
            }
        }

        _ => text_input(ff, value, set),
    }
}

fn boolean_select(name: String, label: String, value: String, set: Callback<String>) -> Element {
    rsx! {
        label {
            "{label}"
            select {
                name: "{name}",
                onchange: move |e| set.call(e.value()),
                option { value: "", selected: value.is_empty(), "Any" }
                option { value: "true", selected: value == "true", "Yes" }
                option { value: "false", selected: value == "false", "No" }
            }
        }
    }
}

pub(crate) fn boolean_checkbox(name: String, label: String, value: String, set: Callback<String>) -> Element {
    rsx! {
        fieldset {
            legend { "" }
            label {
                input {
                    r#type: "checkbox",
                    name: "{name}",
                    checked: value == "true",
                    onchange: move |e| set.call(if e.checked() { "true".to_owned() } else { String::new() }),
                }
                "{label}"
            }
        }
    }
}

fn reference_select_loading(name: String, label: String) -> Element {
    rsx! {
        label {
            "{label}"
            select { name: "{name}", disabled: true,
                option { "Loading…" }
            }
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
        label {
            "{label}"
            select {
                name: "{name}",
                onchange: move |e| set.call(e.value()),
                option { value: "", selected: value.is_empty(), "All {label}s" }
                for (id, display) in options {
                    option { key: "{id}", value: "{id}", selected: value == id, "{display}" }
                }
            }
        }
    }
}

fn checkbox_reference_fieldset(
    name: String,
    label: String,
    value: String,
    options: Vec<(String, String)>,
    set: Callback<String>,
) -> Element {
    let selected: Vec<String> = if value.is_empty() {
        vec![]
    } else {
        value.split(',').map(|s| s.trim().to_owned()).collect()
    };
    rsx! {
        fieldset {
            legend { "{label}" }
            for (id, display) in options {
                label {
                    key: "{id}",
                    input {
                        r#type: "checkbox",
                        name: "{name}",
                        value: "{id}",
                        checked: selected.contains(&id),
                        onchange: {
                            let id = id.clone();
                            let value = value.clone();
                            let set = set.clone();
                            move |e: Event<FormData>| {
                                let mut current: Vec<String> = if value.is_empty() {
                                    vec![]
                                } else {
                                    value.split(',').map(|s| s.trim().to_owned()).collect()
                                };
                                if e.checked() {
                                    if !current.contains(&id) {
                                        current.push(id.clone());
                                    }
                                } else {
                                    current.retain(|s| s != &id);
                                }
                                set.call(current.join(","));
                            }
                        },
                    }
                    " {display}"
                }
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
        label {
            "{ff.label}"
            input {
                r#type: "{input_type}",
                name: "{ff.key}",
                placeholder: "{ff.label}",
                value: "{value}",
                oninput: move |e| set.call(e.value()),
            }
        }
    }
}
