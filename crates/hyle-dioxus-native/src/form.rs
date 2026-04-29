use dioxus::prelude::*;
use hyle_dioxus::{HyleFiltersState, FilterField};

// ── HyleFormFields ────────────────────────────────────────────────────────────

/// Renders a labeled input row for every field exposed by `filters`.
///
/// This mirrors `HyleFormFields` in `@tty-pt/hyle-react-dom`.  Fields with no
/// metadata (not in `filters.fields`) are skipped.
#[component]
pub fn HyleFormFields(
    filters: HyleFiltersState,
    /// Optionally restrict which fields to render (by key). When `None` all
    /// fields from `filters.fields` are shown.
    only: Option<Vec<String>>,
) -> Element {
    let fields = filters.fields.read();

    rsx! {
        div { class: "editFields",
            for field_meta in fields.iter().filter(|f| {
                only.as_ref().map(|keys| keys.contains(&f.key)).unwrap_or(true)
            }) {
                div { class: "fieldRow", key: "{field_meta.key}",
                    label { class: "fieldLabel",
                        "{field_meta.label}"
                    }
                    FilterField {
                        state: filters,
                        field_key: field_meta.key.clone(),
                    }
                }
            }
        }
    }
}
