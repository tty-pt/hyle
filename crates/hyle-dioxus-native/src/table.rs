use dioxus::prelude::*;
use hyle::HyleDataState;
use hyle_dioxus::{use_context_provider, use_hyle_components, field_type_key, HyleFiltersState, HyleListState, HyleValueProps, FilterField, HyleConfig};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use web_sys::AddEventListenerOptions;

// ── HyleTableBody ─────────────────────────────────────────────────────────────

/// Renders the `<table>` body from a `HyleListState`.
///
/// - Column headers are sortable (click to sort, click again to toggle direction).
/// - Rows are clickable when `on_row_click` is supplied; the selected row is
///   highlighted when its id matches `selected_id`.
#[component]
pub fn HyleTableBody(
    list: HyleListState,
    on_row_click: Option<Callback<hyle_dioxus::Row>>,
    selected_id: Option<hyle_dioxus::Value>,
    row_href: Option<Callback<hyle_dioxus::Row, String>>,
) -> Element {
    let data = list.data.read();
    match &*data {
        HyleDataState::Loading { .. } => rsx! {
            div { "Loading…" }
        },
        HyleDataState::Error { error, .. } => rsx! {
            div { class: "hyle-error", "{error}" }
        },
        HyleDataState::Ready { manifest, outcome, rows, columns, .. } => {
            let manifest = manifest.clone();
            let outcome = outcome.clone();
            let rows = rows.clone();
            let columns = columns.clone();
            let sort_field = list.sort_field.read().clone();
            let sort_ascending = *list.sort_ascending.read();
            let components = use_hyle_components();
            let blueprint = use_context::<HyleConfig>().blueprint;

            rsx! {
                div { class: "hyle-table-wrap",
                    table {
                        thead {
                            tr {
                                for col in columns.clone() {
                                    {
                                        let col_key = col.key.clone();
                                        let is_active = sort_field.as_deref() == Some(&col.key);
                                        let sort_indicator = if is_active {
                                            if sort_ascending { " ▲" } else { " ▼" }
                                        } else { "" };
                                        let mut sort_field_sig = list.sort_field;
                                        let mut sort_asc_sig = list.sort_ascending;
                                        rsx! {
                                            th { key: "{col.key}",
                                                button {
                                                    r#type: "button",
                                                    class: "hyle-sort-button",
                                                    onclick: move |_| {
                                                        if sort_field_sig.read().as_deref() == Some(&col_key) {
                                                            sort_asc_sig.toggle();
                                                        } else {
                                                            sort_field_sig.set(Some(col_key.clone()));
                                                            sort_asc_sig.set(true);
                                                        }
                                                    },
                                                    "{col.label}{sort_indicator}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        tbody {
                            if rows.is_empty() {
                                tr {
                                    td { colspan: "{columns.len()}", class: "hyle-empty-state",
                                        "No results match the current filters."
                                    }
                                }
                            } else {
                                for row in rows {
                                    {
                                        let row_id = row.get("id").cloned().unwrap_or(hyle_dioxus::Value::Null);
                                        let is_selected = selected_id.as_ref()
                                            .map(|sid| sid == &row_id)
                                            .unwrap_or(false);
                                        let has_click = on_row_click.is_some();
                                        let class = if is_selected {
                                            "hyle-row-selected"
                                        } else if has_click || row_href.is_some() {
                                            "hyle-row-clickable"
                                        } else {
                                            ""
                                        };
                                        let row2 = row.clone();
                                        let href = row_href.map(|cb| cb.call(row.clone()));
                                        rsx! {
                                            tr {
                                                key: "{row_id}",
                                                class: "{class}",
                                                onclick: move |_| {
                                                    if let Some(cb) = on_row_click {
                                                        cb.call(row2.clone());
                                                    }
                                                },
                                                for (i, col) in columns.clone().into_iter().enumerate() {
                                                    {
                                                        let val = row.get(&col.key)
                                                            .cloned()
                                                            .unwrap_or(hyle_dioxus::Value::Null);
                                                        let type_key = field_type_key(&col.field.field_type);
                                                        let custom_render = components
                                                            .as_ref()
                                                            .and_then(|c| c.values.get(type_key).copied());
                                                        let cell_content = if let Some(render_fn) = custom_render {
                                                            render_fn(HyleValueProps {
                                                                key: col.key.clone(),
                                                                field: col.field.clone(),
                                                                value: val.clone(),
                                                                outcome: outcome.clone(),
                                                                model_name: manifest.base.clone(),
                                                                blueprint: (*blueprint).clone(),
                                                                components: components.clone(),
                                                            })
                                        } else {
                                            let display = hyle::display_value(&blueprint, &outcome, &manifest.base, &col.key, &val);
                                            rsx! { "{display}" }
                                        };
                                                        if i == 0 {
                                                            if let Some(ref url) = href {
                                                                rsx! {
                                                                    td { key: "{col.key}", "data-label": "{col.label}",
                                                                        a { href: "{url}", {cell_content} }
                                                                    }
                                                                }
                                                            } else {
                                                                rsx! {
                                                                    td { key: "{col.key}", "data-label": "{col.label}", {cell_content} }
                                                                }
                                                            }
                                                        } else {
                                                            rsx! {
                                                                td { key: "{col.key}", "data-label": "{col.label}", {cell_content} }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── HyleTableFilterBar ────────────────────────────────────────────────────────

/// Renders a row of filter inputs above the table, one per field.
///
/// Reads `HyleFiltersState` from context (provided by `HyleTablePanel`) so no
/// explicit prop threading is needed when used inside `HyleTablePanel`.
///
/// Pass `only` to restrict which fields are shown (by key). When `None` all
/// fields from `filters.fields` are rendered.
#[component]
pub fn HyleTableFilterBar(
    filters: HyleFiltersState,
    only: Option<Vec<String>>,
    children: Option<Element>,
) -> Element {
    let fields = filters.fields.read();
    let visible: Vec<_> = fields.iter().filter(|f| {
        only.as_ref().map(|keys| keys.contains(&f.key)).unwrap_or(true)
    }).cloned().collect();
    drop(fields);

    rsx! {
        div { class: "hyle-filter-bar",
            for field_meta in visible {
                FilterField {
                    key: "{field_meta.key}",
                    state: filters,
                    field_key: field_meta.key.clone(),
                }
            }
            {children}
        }
    }
}

// ── HyleTableFilters ──────────────────────────────────────────────────────────

/// Renders Apply / Clear filter buttons.
///
/// Must be rendered inside a `HyleTablePanel` so the buttons are within the
/// enclosing `<form method="get">`.
///
/// When JS is enabled, Apply triggers the form's `onsubmit` (which calls
/// `filter_apply` and prevents navigation).  Clear reads `HyleFiltersState`
/// from context (set by `HyleTablePanel`) to call `filter_clear` directly,
/// also preventing default so no navigation occurs.
#[component]
pub fn HyleTableFilters() -> Element {
    let filters = dioxus_core::has_context::<HyleFiltersState>();
    rsx! {
        div { class: "hyle-filter-actions",
            button {
                r#type: "reset",
                onclick: move |e| {
                    if let Some(fs) = filters {
                        e.prevent_default();
                        fs.filter_clear.call(());
                    }
                },
                "Clear"
            }
            button { r#type: "submit", "Apply" }
        }
    }
}

// ── HyleTablePagination ───────────────────────────────────────────────────────

/// Renders page-navigation controls for a `HyleListState`.
///
/// Controls are native `<button type="submit">` elements inside the outer
/// `<form method="get">` wrapping the table, so pagination works without JS.
/// JS signal mutations are kept as well so client-side navigation still works
/// when JS is available (progressive enhancement).
#[component]
pub fn HyleTablePagination(list: HyleListState) -> Element {
    let data = list.data.read();
    let (total, row_count) = match &*data {
        HyleDataState::Ready { outcome, rows, .. } => (outcome.total, rows.len()),
        _ => return rsx! {},
    };
    drop(data);

    let page = *list.page.read();
    let per_page = *list.per_page.read();
    let mut page_sig = list.page;
    let mut per_page_sig = list.per_page;
    let mut page_sig2 = list.page;

    let prev_page = page.saturating_sub(1).max(1);
    let next_page = page + 1;

    rsx! {
        div { class: "hyle-table-footer",
            div { class: "hyle-pagination",
                button {
                    r#type: "submit",
                    name: "page",
                    value: "{prev_page}",
                    disabled: page <= 1,
                    onclick: move |e| {
                        e.prevent_default();
                        page_sig.with_mut(|p| *p = p.saturating_sub(1).max(1));
                    },
                    "← Prev"
                }
                span { "Page {page}" }
                button {
                    r#type: "submit",
                    name: "page",
                    value: "{next_page}",
                    disabled: row_count < per_page,
                    onclick: move |e| {
                        e.prevent_default();
                        page_sig2.with_mut(|p| *p += 1);
                    },
                    "Next →"
                }
                select {
                    name: "per_page",
                    value: "{per_page}",
                    onchange: move |e| {
                        if let Ok(n) = e.value().parse::<usize>() {
                            per_page_sig.set(n);
                            page_sig.set(1);
                        }
                    },
                    for n in [5usize, 10, 20, 50, 100] {
                        option { value: "{n}", selected: n == per_page, "{n} / page" }
                    }
                }
                // No-JS fallback: submit button for per-page change.
                // With JS the select's onchange auto-submits; without JS the
                // user clicks this button after selecting a value.
                button { r#type: "submit", "Apply" }
            }
            span { class: "hyle-row-count",
                "{row_count} of {total} rows"
            }
        }
    }
}

// ── HyleTable ─────────────────────────────────────────────────────────────────

/// Composes `HyleTableBody` + `HyleTablePagination`.
///
/// Does not own a `<form>` — use `HyleTablePanel` when you need the full
/// no-JS GET-form wrapper (filters + table + pagination inside one form).
#[component]
pub fn HyleTable(
    list: HyleListState,
    on_row_click: Option<Callback<hyle_dioxus::Row>>,
    selected_id: Option<hyle_dioxus::Value>,
    row_href: Option<Callback<hyle_dioxus::Row, String>>,
) -> Element {
    rsx! {
        HyleTableBody {
            list,
            on_row_click,
            selected_id,
            row_href,
        }
        HyleTablePagination { list }
    }
}

// ── HyleTablePanel ────────────────────────────────────────────────────────────

/// Wraps a `<form method="get">` around a header slot, `HyleTableBody`, and
/// `HyleTablePagination` so that `HyleTableFilters` buttons, filter inputs, and
/// pagination controls all belong to the same native form.
///
/// Place your header (including `HyleTableFilterBar`, `HyleTableFilters`, and
/// any other controls) as `children`; they will be rendered inside the form
/// before the table.
///
/// When JS is enabled the form `onsubmit` is intercepted: `filter_apply` is
/// called on the filters state and the page is reset to 1, so the table updates
/// reactively without a full-page navigation.  Without JS the native GET submit
/// proceeds unchanged (progressive enhancement).
///
/// `HyleFiltersState` is provided as context so that `HyleTableFilterBar`,
/// `HyleTableFilters`, and `HyleTablePagination` can read it without requiring
/// explicit prop threading.
///
/// # Example
/// ```rust,ignore
/// HyleTablePanel { list, filters,
///     header { class: "panelHeader",
///         h2 { "Users" }
///         HyleTableFilterBar { only: vec!["name".into(), "role".into()] }
///         HyleTableFilters {}
///     }
/// }
/// ```
#[component]
pub fn HyleTablePanel(
    list: HyleListState,
    filters: Option<HyleFiltersState>,
    on_row_click: Option<Callback<hyle_dioxus::Row>>,
    selected_id: Option<hyle_dioxus::Value>,
    row_href: Option<Callback<hyle_dioxus::Row, String>>,
    children: Element,
) -> Element {
    // Provide filters state as context so HyleTableFilterBar / HyleTableFilters
    // / HyleTablePagination can call filter_apply / reset page without explicit
    // prop drilling.
    if let Some(fs) = filters {
        use_context_provider(|| fs);
    }

    let mut page_sig = list.page;

    // On wasm, attach a capture-phase submit listener to the form so that
    // preventDefault() fires before the browser commits to navigation.
    // Dioxus's bubble-phase onsubmit handler is too late for reliable prevention.
    #[cfg(target_arch = "wasm32")]
    use_effect(|| {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let closure = Closure::<dyn Fn(web_sys::Event)>::new(|e: web_sys::Event| {
            e.prevent_default();
        });
        let opts = AddEventListenerOptions::new();
        opts.set_capture(true);
        document
            .query_selector("form[data-hyle-panel]")
            .ok()
            .flatten()
            .and_then(|el| el.dyn_into::<web_sys::EventTarget>().ok())
            .map(|et| et.add_event_listener_with_callback_and_add_event_listener_options(
                "submit",
                closure.as_ref().unchecked_ref(),
                &opts,
            ));
        closure.forget();
    });

    rsx! {
        form {
            method: "get",
            "data-hyle-panel": "true",
            onsubmit: move |e| {
                e.prevent_default();
                // If the submit was triggered by a pagination button (name="page"),
                // the onclick handler has already updated the page signal — don't
                // reset it to 1 and don't re-apply filters.
                let is_pagination = e.values().iter().any(|(k, _)| k == "page");
                if !is_pagination {
                    if let Some(fs) = filters {
                        fs.filter_apply.call(());
                    }
                    page_sig.set(1);
                }
            },
            {children}
            HyleTableBody {
                list,
                on_row_click,
                selected_id,
                row_href,
            }
            HyleTablePagination { list }
        }
    }
}
