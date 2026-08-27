# hyle

*You bring the prime matter — your fetch strategy, your components, your views. hyle gives it form.*

hyle makes CRUD easy.

Define your data models once as a Blueprint. Get filtered, paginated lists, create/edit/delete forms, and validation — in React or Dioxus — without writing boilerplate fetch or validation code. Works in Rust natively or compiled to WebAssembly for JavaScript.

## C Libraries & Bindings

| Library | Directory | Description |
|---|---|---|
| `libhyle-source` | `c/libhyle-source/` | Standalone C persistence engine: dataset metadata, CRUD, DSV, JSON state overlays, and pluggable storage drivers (`store_fs`, `store_mem`, custom stores). |
| `libhyle-bud` | `c/libhyle-bud/` | Bridge layer connecting Hyle schemas to Bud UI components (`hyle_bud_state_apply`, filters, pickers). |
| Core C API | `include/hyle/` | Canonical schema (`hyle/schema.h`), neutral picker types (`hyle/picker.h`), dataset query and FTS index. |

## Where do you want to go?

| I want to… | Start here |
|---|---|
| Build a React CRUD app | [`packages/hyle-react`](packages/hyle-react/README.md) |
| Add pre-built React table and form components | [`packages/hyle-react-dom`](packages/hyle-react-dom/README.md) |
| See a full React example | [`examples/react`](examples/react/README.md) |
| Build a Dioxus CRUD app | [`crates/hyle-dioxus`](crates/hyle-dioxus/README.md) |
| Add pre-built Dioxus table and form components | [`crates/hyle-dioxus-native`](crates/hyle-dioxus-native/README.md) |
| See a full Dioxus example | [`examples/dioxus`](examples/dioxus/README.md) |
| Use the core Rust library directly | [`crates/hyle`](crates/hyle/README.md) |

## Testing

```bash
# Full build + E2E suite
npm run test:all

# E2E only (requires prior build)
npm test

# Unit tests
cargo test
npm test --prefix packages/hyle-react
npm test --prefix packages/hyle-react-dom
```
