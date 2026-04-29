# hyle-dioxus-native

Pre-built Dioxus components for tables, filters, pagination, and forms. Bring the data; these components know how to show it. Pair with [`hyle-dioxus`](../hyle-dioxus/README.md) hooks and your CRUD UI is done with almost no layout code.

See [`examples/dioxus`](../../examples/dioxus/README.md) for a full working app.

## Quick start

The snippet below renders a filterable user list with pagination and an edit form. `HyleTablePanel` wraps everything in a `<form method="get">` so filters work even without JavaScript.

```rust
use hyle_dioxus_native::{HyleTablePanel, HyleTableFilters, HyleFormFields};
use hyle_dioxus::{use_list_with_filters, use_filters, use_form};

#[component]
fn UserList() -> Element {
    let filters = use_filters(Query::from("user"), Default::default());
    let list    = use_list_with_filters(filters);

    rsx! {
        HyleTablePanel { list, filters,
            HyleTableFilters {}   // Apply / Clear buttons
        }
    }
}

#[component]
fn UserForm() -> Element {
    let form = use_form(Query::from("user"), Default::default());
    rsx! {
        form { onsubmit: form.on_submit,
            HyleFormFields { filters: form.filters }
            button { r#type: "submit", "Save" }
        }
    }
}
```

## Installation

```toml
[dependencies]
hyle-dioxus-native = "0.1"
```

## Components

| Component | Props | Description |
|---|---|---|
| `HyleTablePanel` | `list`, `filters?`, `on_row_click?`, `selected_id?`, `row_href?`, `children` | Wraps table in a `<form method="get">`; provides filter context; intercepts submit for JS filter apply |
| `HyleTable` | `list`, `filters?`, `on_row_click?`, `selected_id?`, `row_href?` | `HyleTableBody` + `HyleTablePagination` |
| `HyleTableBody` | `list`, `filters?`, `on_row_click?`, `selected_id?`, `row_href?` | Sortable `<table>` with optional inline column filters |
| `HyleTableFilters` | — (reads context from `HyleTablePanel`) | Apply / Clear filter buttons |
| `HyleTablePagination` | `list: HyleListState` | Prev/Next + per-page select; works without JS via GET form |
| `HyleFormFields` | `filters: HyleFiltersState`, `only?` | Renders label + input for every field (or a subset via `only`) |

## Testing

```bash
cargo test -p hyle-dioxus-native
```
