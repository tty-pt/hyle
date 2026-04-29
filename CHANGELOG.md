# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] — 2026-05-04

### Added

- `crates/hyle` — core blueprint/manifest/query engine, WASM-compilable, framework-agnostic
  - Pure-logic adapter layer (`hyle::adapter`) with types and helpers usable by any Rust UI framework: `HyleManifestState`, `HyleDataState`, `HyleDataField`, `HyleFilterField<R>`, `FieldChange<R>`, `FormErrors`, `UseFormaOptions`, `FORMA_MODEL`
  - Helpers: `compute_manifest`, `compute_data`, `build_effective_query`, `run_purify`, `build_filter_fields`, `apply_change`, `compute_forma_result`
  - 17 unit tests covering all pure helpers
- `crates/hyle-dioxus` — Dioxus 0.6 hooks: `use_manifest`, `use_data`, `use_filters`, `use_form`, `use_forma`
  - Dioxus-typed `HyleFilterField`, `DioxusFieldChangeFn`, `DioxusFieldChangeMap`, `UseFiltersOptions`, `UseFormOptions`
- `crates/hyle-dioxus-native` — thin re-export crate for native (non-WASM) Dioxus targets
- `packages/hyle-react` (`@tty-pt/hyle-react`) — React hooks and context provider: `HyleProvider`, `useHyle`, `useHyleResolved`, `useFilters`, `useForm`, `useForma`, `useList`, `useData`
- `packages/hyle-react-dom` (`@tty-pt/hyle-react-dom`) — DOM field components built on `@tty-pt/hyle-react`
- `crates/hyle/pkg-src` (`@tty-pt/hyle`) — TypeScript client wrapping the compiled WASM module
- `.github/workflows/ci.yml` — CI: `cargo test -p hyle`, `cargo test -p hyle-dioxus`, npm build + test for `@tty-pt/hyle-react` and `@tty-pt/hyle-react-dom`

### Changed

- Renamed `UseFiltersOptions` / `UseFormOptions` in `hyle` core to `AdapterFiltersOptions` / `AdapterFormOptions` to distinguish them from the Dioxus-specific options types of the same name in `hyle-dioxus`
- Consolidated `lookups` and `inlines` into unified `model` terminology throughout
- Dropped `hyle-react-query`, `hyle-dioxus-query`, `hyle-wasm`, `hyle-dioxus-axum` — functionality absorbed into the remaining crates
