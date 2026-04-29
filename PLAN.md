# Dioxus 0.7 Upgrade + Plugin Architecture Plan

## Goal

Mirror the React plugin architecture on the Dioxus side:

| React | Dioxus |
|---|---|
| `@tty-pt/hyle-react` | `hyle-dioxus` — pure hooks + types, no HTTP |
| `@tty-pt/hyle-react-query` | `hyle-dioxus-query` — TanStack/dioxus-query adapter |
| `examples/react/src/hooks.ts` | `examples/dioxus/src/main.rs` app wiring |

---

## Phase 1 — Upgrade to Dioxus 0.7

### `crates/hyle-dioxus/Cargo.toml`
- `dioxus`, `dioxus-core`, `dioxus-hooks`, `dioxus-signals`: `"0.6"` → `"0.7"`

### `crates/hyle-dioxus-components/Cargo.toml`
- `dioxus`, `dioxus-signals`: `"0.6"` → `"0.7"`

### `examples/dioxus/Cargo.toml`
- `dioxus`: `"0.6"` → `"0.7"`
- `axum`: `"0.7"` → `"0.8"`
- `tower-http`: `"0.5"` → `"0.6"`

### `examples/dioxus/src/server.rs`
- Route paths: `"/api/user/{id}"` → `"/api/user/:id"` (×2, axum 0.8 changed syntax)

---

## Phase 2 — New `crates/hyle-dioxus-query/` crate

Mirrors `@tty-pt/hyle-react-query`. Plugin crate — the app depends on it, `hyle-dioxus` does not.

### Dependencies
- `hyle-dioxus` (local)
- `dioxus-query = "0.9"` (backed by Dioxus 0.7)
- `reqwest` (optional, for HTTP convenience helpers)

### Public API

```rust
// Source adapter — wraps dioxus-query's use_query
// Returns a UseSource (HyleSourceAdapter equivalent)
pub fn make_dioxus_source<F, Fut>(fetch_fn: F) -> UseSource
where
    F: Fn(Query) -> Fut + 'static + Clone,
    Fut: Future<Output = Source> + 'static;

// Mutation factory — wraps dioxus-query's Mutation
// Returns a fn() -> HyleMutation factory (passed as UseFiltersOptions::mutation)
pub fn make_dioxus_mutation<F, Fut>(
    mutate_fn: F,
    options: DioxusMutationOptions,
) -> impl Fn() -> HyleMutation
where
    F: Fn(MutateInput) -> Fut + 'static + Clone,
    Fut: Future<Output = ()> + 'static;

pub struct DioxusMutationOptions {
    /// Called after a successful mutation (e.g. to invalidate queries).
    pub on_success: Option<Box<dyn Fn()>>,
}
```

---

## Phase 3 — Clean up `hyle-dioxus`

- Delete `crates/hyle-dioxus/src/http.rs`
- Remove `http` feature + `reqwest` from `crates/hyle-dioxus/Cargo.toml`
- Remove `http` re-exports from `crates/hyle-dioxus/src/lib.rs`

---

## Phase 4 — Update `examples/dioxus`

- Add `hyle-dioxus-query = { path = "../../crates/hyle-dioxus-query" }` to `Cargo.toml`
- Remove `features = ["http"]` from `hyle-dioxus` dep
- In `src/main.rs`: replace `make_http_source`/`use_http_mutation` with
  `make_dioxus_source`/`make_dioxus_mutation` from `hyle_dioxus_query`
- The source/mutation wiring in `main.rs` becomes the Dioxus equivalent of
  `examples/react/src/hooks.ts`

---

## Verification

```sh
cargo build -p hyle-dioxus -p hyle-dioxus-components -p hyle-dioxus-query -p hyle-dioxus-example
cargo test -p hyle -p hyle-dioxus
```
