use dioxus::prelude::*;
use hyle_dioxus::{HyleFiltersState, FormFilterField};

// ── HyleFormFields ────────────────────────────────────────────────────────────

/// Renders a labeled input row for every field exposed by `filters`.
///
/// This mirrors `HyleFormFields` in `@tty-pt/hyle-react-dom`.  Fields with no
/// metadata (not in `filters.fields`) are skipped.
///
/// Boolean fields render as a self-labelling checkbox (label on the right).
#[component]
pub fn HyleFormFields(
    filters: HyleFiltersState,
    /// Optionally restrict which fields to render (by key). When `None` all
    /// fields from `filters.fields` are shown.
    only: Option<Vec<String>>,
) -> Element {
    let fields = filters.fields.read();

    rsx! {
        div { class: "hyle-edit-fields",
            for field_meta in fields.iter().filter(|f| {
                only.as_ref().map(|keys| keys.contains(&f.key)).unwrap_or(true)
            }) {
                {
                    let key = field_meta.key.clone();
                    rsx! {
                        div { class: "hyle-field-row", key: "{key}",
                            FormFilterField {
                                state: filters,
                                field_key: key,
                            }
                        }
                    }
                }
            }
        }
    }
}
