# hyle

The core data layer — schema definition, query planning, resolution, and validation. No UI, no async, no transport: pure logic you can use directly from Rust or compile to WebAssembly for JavaScript. The schema, the query plan, the resolved rows — all of it derived from a single Blueprint you define. The rest is yours.

Used by [`hyle-dioxus`](https://github.com/tty-pt/hyle/tree/main/crates/hyle-dioxus) and by JavaScript via the `wasm` feature (see below).

## Quick start

### 1. Define a Blueprint

A Blueprint is the shape of your data — models, fields, types, and the references between them.

```rust
use hyle::{Blueprint, Field, Model};

let blueprint = Blueprint::new()
    .model(
        "user",
        Model::new()
            .field("name",   Field::string("Name"))
            .field("email",  Field::string("Email"))
            .field("role",   Field::reference("Role", "role"))
            .field("active", Field::boolean("Active")),
    )
    .model("role", Model::new().field("name", Field::string("Role name")));
```

### 2. Ask for data

```rust
use hyle::{Query, Source};

let query = Query::from("user")
    .select(["name", "email", "role", "active"])
    .sort_by("name", true)
    .page(1, 20);

let manifest = blueprint.manifest(query).unwrap();

// Your data, from wherever it lives
let source: Source = todo!("fetch from your database or API");

let view = blueprint.resolve_and_view(&manifest, &source).unwrap();
// view.rows    — Vec<Row> ready to render
// view.columns — Vec<Column> with labels and field metadata
// view.is_single — true if the query returns a single record
```

### 3. Validate a row before saving

```rust
use hyle::purify_row_sync;

if let Err(errors) = purify_row_sync(&blueprint, "user", &row) {
    for e in &errors {
        println!("{}: {}", e.field, e.message);
    }
}
```

## Installation

```toml
[dependencies]
hyle = "0.1"
```

## WASM feature

Enable the `wasm` feature to compile `hyle` as a WebAssembly module for use from JavaScript (consumed by `@tty-pt/hyle-react`):

```toml
[dependencies]
hyle = { version = "0.1", features = ["wasm"] }
```

Build:

```bash
cargo build -p hyle --features wasm --release --target wasm32-unknown-unknown
wasm-bindgen target/wasm32-unknown-unknown/release/hyle.wasm --target web --out-dir <out-dir>
```

Once compiled, the same logic that runs in your Rust server is available anywhere JavaScript runs — as simple JSON-in / JSON-out functions:

| Function | Description |
|---|---|
| `manifest(blueprint, query)` | Derive a Manifest |
| `resolve(blueprint, manifest, source)` | Resolve a Manifest against a Source |
| `resolve_query(blueprint, query, source)` | Plan + resolve in one step → `{ manifest, result, rows }` |
| `resolve_and_view(blueprint, manifest, source)` | Plan + resolve + view → `{ result, rows, isSingle, columns }` |
| `rows_array(outcome)` | Flatten an Outcome to `Row[]` |
| `is_single(manifest, outcome)` | Whether the result is a single row |
| `columns(blueprint, manifest)` | Derive `Column[]` |
| `filter_rows(rows, filters)` | Filter rows client-side |
| `filter_layout(blueprint, manifest)` | Derive 2-D filter layout |
| `display_value(blueprint, outcome, model, field, value)` | Resolve a display string |
| `make_field(kind, label, entity)` | Build a `Field` by kind string |
| `forma_to_query(forma, table, context, id)` | Build a Query from a Forma |
| `purify_row(blueprint, model, row)` | Validate a row → `null` or `PurifyError[]` |
| `apply_view(rows, manifest)` | Filter + sort + paginate rows |

## Public API

### Blueprint

| Item | Description |
|---|---|
| `Blueprint` | Registry of models. Central schema object. |
| `Blueprint::manifest(query)` | Derive a `Manifest` from a `Query` |
| `Blueprint::resolve(manifest, source)` | Resolve a `Manifest` against a `Source` → `Outcome` |
| `Blueprint::resolve_query(query, source)` | Plan + resolve in one step |
| `Blueprint::resolve_and_view(manifest, source)` | Plan + resolve + view → `ResolvedView` |
| `Model` | Named collection of fields |
| `Field`, `FieldOptions`, `FieldType` | Field metadata and type definitions |
| `Primitive` | `String`, `Number`, `Boolean`, `File` |
| `Reference` | Cross-model reference: `entity`, `display_field` |
| `make_field(kind, label, entity)` | Build a `Field` by kind string |

### Query & Manifest

| Item | Description |
|---|---|
| `Query` | Input: `from`, `select`, `where_`, `page`, `per_page`, `sort`, `method` |
| `Manifest` | Derived plan: joins, needs, filter fields, sort, pagination |
| `Sort` | `field`, `ascending` |
| `parse_query_params(str)` | Parse a URL query string into a `Query` |

### Source & Outcome

| Item | Description |
|---|---|
| `Source` | `IndexMap<String, ModelResult>` — raw data keyed by model name |
| `Outcome` | Resolution result |
| `ModelResult` | `One(Row)` or `Many { rows, total }` |
| `Row` | `IndexMap<String, Value>` |
| `ResolvedView` | `{ outcome, rows, is_single, columns }` |
| `rows_from_outcome(outcome)` | Extract `Vec<Row>` from an `Outcome` |
| `is_single(manifest, outcome)` | Whether the result is a single row |

### View helpers

| Function | Description |
|---|---|
| `apply_view(rows, manifest)` | Filter, sort, and paginate rows |
| `derive_columns(bp, manifest)` | Build `Vec<Column>` for the query |
| `derive_filter_layout(bp, manifest)` | Build filter field layout |
| `display_value(bp, outcome, model, field, value)` | Resolve a display string for a value |
| `filter_rows(rows, filters)` | Filter rows by a map of field→value |
| `Column` | `key`, `label`, `field` |

### Validation

| Item | Description |
|---|---|
| `purify_row_sync(bp, model, row)` | Validate a row; returns `Err(Vec<PurifyError>)` on failure |
| `PurifyError` | `field`, `rule`, `message` |
| `Purifier` | Orchestrates sync validation rules |

### Forms

| Item | Description |
|---|---|
| `Forma` | Dynamic form schema derived from a model |
| `FormaContext` | `Column`, `Form`, `Detail`, `Filters` |
| `FormaField`, `FormaFieldType` | Form field descriptors |
| `forma_to_query(forma, table, context, id)` | Build a `Query` from a `Forma` |
| `row_from_form(data)` | Convert form data to a `Row` |
| `row_from_value(value)` | Convert a JSON value to a `Row` |

## Testing

```bash
cargo test -p hyle
```
