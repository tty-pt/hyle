use dioxus::prelude::*;

use hyle::{FieldType, Primitive};

use crate::context::{use_hyle_components, HyleConfig};
use crate::types::{field_type_key, HyleComponents, HyleFilterField, HyleFilterFieldProps, HyleFiltersState, HyleValueProps};

/// A reactive filter input derived from `HyleFiltersState`.
///
/// Looks up the field definition in `state.fields`, reads the current value
/// from `state.form_data`, and calls `state.set_field` on change.
///
/// Rendering priority:
/// 1. Per-field `render` function stored in the `HyleFilterField` (from `use_filters` change map)
/// 2. Matching entry in the `HyleComponents` context (registered via `use_hyle_components`)
/// 3. Built-in default based on field type:
///    - `Boolean` → `<select>` with Any / Yes / No
///    - `Reference` → `<select>` populated from pre-resolved options
///    - `Array<Reference>` → `<fieldset>` with one checkbox per option
///    - Everything else → `<input type="text">` (or the `input.kind` hint from the blueprint)
///
/// # Props
/// - `state` — the `HyleFiltersState` from `use_filters`
/// - `field_key` — the field name to render
#[component]
pub fn FilterField(
    state: HyleFiltersState,
    field_key: String,
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

    // Priority 2: global HyleComponents context.
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

    // Priority 3: built-in default.
    default_input(ff, value, set, false)
}

/// Like [`FilterField`] but intended for form contexts: boolean fields render
/// as a self-labelling checkbox (label on the right) instead of a 3-state
/// select, matching the React `FilterBoolean` checkbox appearance.
#[component]
pub fn FormFilterField(
    state: HyleFiltersState,
    field_key: String,
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

    form_default_input(ff, value, set)
}

fn form_default_input(ff: HyleFilterField, value: String, set: Callback<String>) -> Element {
    match &ff.field.field_type {
        FieldType::Primitive {
            primitive: Primitive::Boolean,
        } => boolean_checkbox(ff.key, ff.label, value, set),
        _ => default_input(ff, value, set, true),
    }
}

// ── Default input dispatch ────────────────────────────────────────────────────

fn default_input(ff: HyleFilterField, value: String, set: Callback<String>, is_form: bool) -> Element {
    match &ff.field.field_type {
        FieldType::Primitive {
            primitive: Primitive::Boolean,
        } => boolean_select(ff.key, ff.label, value, set),

        FieldType::Reference { .. } => match ff.options {
            Some(options) => reference_select(ff.key, ff.label, value, options, set),
            None => reference_select_loading(ff.key, ff.label),
        },

        FieldType::Array { .. } => {
            // Check if input hint specifies textarea
            let is_textarea = ff.field.options.input.as_ref()
                .map(|i| i.kind == "textarea")
                .unwrap_or(false);

            if is_textarea {
                let rows = ff.field.options.input.as_ref()
                    .and_then(|i| i.props.get("rows"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("4");
                return rsx! {
                    label {
                        "{ff.label}"
                        textarea {
                            name: "{ff.key}",
                            rows: "{rows}",
                            value: "{value}",
                            oninput: move |e| set.call(e.value()),
                            placeholder: "One type per line (e.g., folk\\nrock)",
                        }
                    }
                };
            }

            match ff.options {
                Some(options) => checkbox_reference_fieldset(ff.key, ff.label, value, options, ff.display_field_type.clone(), set),
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
    display_field_type: Option<FieldType>,
    set: Callback<String>,
) -> Element {
    let selected: Vec<String> = if value.is_empty() {
        vec![]
    } else {
        value.split(',').map(|s| s.trim().to_owned()).collect()
    };

    // Look up a value renderer for the display field type — same map as table cells (symmetric with React).
    let components: Option<HyleComponents> = use_hyle_components();
    let label_render_fn: Option<fn(HyleValueProps) -> Element> =
        display_field_type.as_ref().and_then(|ft| {
            components.as_ref().and_then(|c| {
                let key = field_type_key(ft);
                c.values.get(key).copied()
            })
        });

    // Blueprint from context — needed to build HyleValueProps for the label renderer.
    let blueprint = use_context::<HyleConfig>().blueprint;

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
                    if let Some(render_fn) = label_render_fn {
                        {
                            let ft = display_field_type.clone().unwrap_or(FieldType::Primitive { primitive: Primitive::String });
                            let field = hyle::Field { label: display.clone(), field_type: ft, options: Default::default() };
                            let display_val = hyle::Value::String(display.clone());
                            render_fn(HyleValueProps {
                                key: id.clone(),
                                field,
                                value: display_val,
                                outcome: hyle::Outcome::empty(),
                                model_name: String::new(),
                                blueprint: (*blueprint).clone(),
                                components: components.clone(),
                            })
                        }
                    } else {
                        " {display}"
                    }
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
