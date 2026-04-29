# hyle-dioxus

Dioxus hooks and context for building CRUD UIs in Rust. You write the server functions; hyle wires them into a consistent data layer. Calls the core directly — no WASM bridge, no extra dependencies.

See also: [`hyle-dioxus-native`](../hyle-dioxus-native/README.md) for pre-built table and form components, and [`examples/dioxus`](../../examples/dioxus/README.md) for a full working app.

## Quick start

### 1. Install

```toml
[dependencies]
hyle-dioxus = "0.1"
```

### 2. Set up context at the app root

```rust
use std::sync::Arc;
use dioxus::prelude::*;
use hyle_dioxus::{
    HyleConfig, InvalidationSignal,
    make_fullstack_adapter, use_adapter_config,
};

fn app() -> Element {
    // Provide your blueprint
    use_context_provider(|| HyleConfig { blueprint: Arc::new(make_blueprint()) });

    // Provide an invalidation signal — increment it to refresh all active queries
    use_context_provider(|| Signal::new(0u32) as InvalidationSignal);

    // Wire up your server functions as the data adapter
    use_adapter_config!(make_fullstack_adapter(
        get_source,    // async server fn: () -> Source
        create_row,    // async server fn: (MutateInput) -> ()
        update_row,    // async server fn: (MutateInput) -> ()
        delete_row,    // async server fn: (MutateInput) -> ()
    ));

    rsx! { Router::<Route> {} }
}
```

### 3. Render a list

```rust
use hyle::Query;
use hyle_dioxus::use_list;

#[component]
fn UserList() -> Element {
    let list = use_list(Query::from("user").select(["name", "email"]));
    // list.data, list.page, list.per_page, list.sort_field, list.sort_ascending
    rsx! { /* render list.data */ }
}
```

Want pre-built table, filter, and form components? See [`hyle-dioxus-native`](../hyle-dioxus-native/README.md).

## Axum feature

The `axum` feature adds `HyleRenderer`, an Axum extension for progressive enhancement: on POST validation failure, it re-renders the page server-side with `FormErrors` injected, so forms give meaningful feedback even before JavaScript loads.

```toml
hyle-dioxus = { version = "0.1", features = ["axum"] }
```

```rust
use hyle_dioxus::HyleRenderer;

// Register at startup
.layer(Extension(HyleRenderer))

// In a POST handler
async fn handle_create(
    State(fs): State<FullstackState>,
    Extension(renderer): Extension<HyleRenderer>,
) -> Response {
    let errors = validate(&form);
    if !errors.is_empty() {
        return renderer.render_with_errors(fs, "/users/new", errors.into()).await;
    }
    // ...
}
```

## API

### Setup

| Item | Description |
|---|---|
| `HyleConfig` | `{ blueprint: Arc<Blueprint> }` — provide at app root via `use_context_provider` |
| `use_adapter_config!(adapter)` | Macro — provide a `HyleAdapter` as Dioxus context |
| `make_fullstack_adapter(source, create, update, delete)` | Build a `HyleAdapter` from four async server functions |
| `InvalidationSignal` | `Signal<u32>` — increment to invalidate all active queries |

### Hooks

| Hook | Returns | Description |
|---|---|---|
| `use_manifest(query)` | `Memo<HyleManifestState>` | Derive a manifest from a query |
| `use_data(query)` | `Memo<HyleDataState>` | Fetch and resolve data for a query |
| `use_list(query)` | `HyleListState` | List with pagination and sorting signals |
| `use_list_with_filters(filters)` | `HyleListState` | List driven by a `HyleFiltersState` |
| `use_filters(query, opts)` | `HyleFiltersState` | Filter form state |
| `use_form(query, opts)` | `HyleFormState` | Create/edit form state with validation |
| `use_forma(table, id, opts)` | `Memo<(Option<Query>, Option<Forma>)>` | Dynamic form schema |
| `use_mutation(from)` | `BoundMutations` | Create/update/delete mutations bound to a model |
| `use_dioxus_mutation(mutate_fn, opts)` | `HyleMutation` | Wrap an async mutation function |
| `use_fullstack_source(fetch_fn)` | `UseSource` | Drive source fetching from a server function |

### State Types

| Type | Key Fields |
|---|---|
| `HyleManifestState` | `Ready { manifest }` / `Error { error }` |
| `HyleDataState` | `Loading` / `Ready { manifest, outcome, rows, columns, fields }` / `Error` |
| `HyleListState` | `data`, `query`, `page`, `per_page`, `sort_field`, `sort_ascending` (Signals) |
| `HyleFiltersState` | `query`, `fields`, `form_data`, `set_field`, `filter_apply`, `filter_clear`, `validate`, `purify_errors` |
| `HyleFormState` | Wraps `HyleFiltersState` + `is_edit`, `is_valid`, `on_submit`, `mutation` |
| `HyleAdapter` | `source`, `create`, `update`, `delete` (all `HyleMutation`) |
| `HyleMutation` | `mutate`, `reset`, `is_pending`, `is_success`, `error` (Signals) |
| `BoundMutations` | `create`, `update`, `delete: BoundMutation` |
| `HyleSourceState` | `Loading` / `Ready(Source)` / `Error(String)` |

### Components

| Component | Props | Description |
|---|---|---|
| `FilterField` | `state`, `field_key`, `render?` | Render a single filter input field |

### Utilities

| Item | Description |
|---|---|
| `form_body(data)` | Convert `IndexMap<String, String>` to a JSON `Value` |
| `field_type_key(field_type)` | Map a `FieldType` to its string key |
| `UseFiltersOptions` | `initial_committed`, `change: Option<FieldChangeMap>` |
| `UseFormOptions` | same as above; `.with_change(key, fn)` builder |
| `UseFormaOptions` | `context: FormaContext` |

## Testing

```bash
cargo test -p hyle-dioxus
```
