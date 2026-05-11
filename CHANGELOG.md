# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.6] — 2026-05-12

### Added

- `FilterProps.context?: "filter" | "form"` prop — determines how filter components render
- `FilterContext` + `useFilterContext()` exported from `@tty-pt/hyle-react` — allows components to detect form vs filter-bar context
- `FilterFile` in `@tty-pt/hyle-react-dom`: renders `<input type="file">` when `context="form"`, text input when `context="filter"` (default)
- `FilterFile` in `@tty-pt/hyle-react-dom`: supports `accept`, `multiple`, `required` from field options
- `FilterBoolean` in `@tty-pt/hyle-react-dom`: defaults to checkbox appearance when `context="form"` (via context or explicit prop)
- `HyleFormFields` in `@tty-pt/hyle-react-dom`: wrapped with `FilterContext.Provider value="form"` so child inputs auto-detect form context

### Tests

- Added test cases for `FilterBoolean` with `context="form"` (renders checkbox)
- Added test cases for `FilterFile` with `context="form"` (renders file input with `accept`/`multiple`/`required`)

## [0.1.5] — 2026-05-07

### Fixed

- Dioxus: clicking Next/Prev pagination buttons no longer resets the page to 1 and clears applied filters. The `HyleTablePanel` form `onsubmit` handler now detects when a pagination button triggered the submit (via `name="page"` in the form values) and skips the filter-apply + page-reset logic in that case. The no-JS GET-form path is unaffected.

### Added

- E2E test: "filter then paginate — filter persists across pages" added to both React and Dioxus suites (50/50 Dioxus, 34/34 React)

## [0.1.4] — 2026-05-06

### Added

- `HyleClient` is now re-exported from `@tty-pt/hyle-react`, making it available to consumers without reaching into `@tty-pt/hyle` directly

### Fixed

- React SSR: `ssr-server.js` now correctly serves client assets from `dist/` (was incorrectly looking in `dist/client/`)
- React SSR: `ssrClient.displayValue` now returns the raw string value instead of `""`, so SSR-rendered table cells are no longer blank
- `HyleTableBody` (`hyle-react-dom`): guards against `list.manifest` being `undefined`; falls back to the raw cell value when `displayValue` returns `""`
- All 32 React e2e tests now pass (12 JS hydration + 4 no-JS SSR × 2 Playwright projects)
- `opts.capture()` replaced with `opts.set_capture()` in `hyle-dioxus-native` (deprecated web-sys API)

### Changed

- `FilterField` and `FormFilterField` (`hyle-dioxus`): removed the unused `render: Option<fn(...)>` prop — per-field render customisation is still available via the `HyleFilterField.render` field and the `HyleComponents` context
- `fn extract_id` and the `FormErrors`/`json` imports in the Dioxus example `server.rs` are now gated with `#[cfg(not(target_arch = "wasm32"))]` to suppress false-positive warnings on the WASM build target

### Removed

- Dead code and unused imports across `hyle`, `hyle-dioxus`, and `hyle-dioxus-native`: `Reference` (`blueprint.rs`), `FieldChange` (`types.rs`), `FormaFieldType` (`core.rs`), spurious `mut` on `opts` (`table.rs`)

## [0.1.3] — 2026-05-06

### Added

- `crates/hyle-dioxus`: new `FormFilterField` component — boolean fields render as a self-labelling checkbox instead of a 3-state select, matching the React `FilterBoolean` appearance

### Changed

- CSS refactored to a `hyle-*` class naming token system across all crates and packages (`crates/hyle/assets/hyle.css`, `examples/dioxus/src/style.css`, `examples/react/src/style.css`)
- `hyle-dioxus` `FilterField`: `Reference` fields now show a disabled "Loading…" select while options are resolving; `Array<Reference>` fields render as checkbox `<fieldset>` elements; all inputs are wrapped in `<label>`
- `hyle-react-dom` `ArrayValue`: delegates rendering of array items to a registered typed `ValueComp` (looked up via `componentKeyForField`) when `components` is provided
- `HyleFilterField` gains `display_field_type: Option<FieldType>` in the core adapter and Dioxus types
- Packaging: cleaned up stale committed `dist/` build artifacts; updated peer dependency ranges

### Fixed

- All e2e tests pass again after CSS token rename

## [0.1.2] — 2026-05-05

### Added

- `HyleFilterField` gains `display_field_type: Option<FieldType>` to expose the referenced entity's display field type to framework adapters
- `build_filter_fields` now populates `display_field_type` for both `Reference` and `Array<Reference>` columns by looking up the referenced model's display field in the blueprint

### Changed

- `hyle-react-dom` `ArrayValue`: uses `componentKeyForField` + a registered `components` map to render array items through typed value components
- `hyle-react-dom` `filters.tsx` and `values.tsx`: expanded inline/checkbox rendering for `Array<Reference>` fields

### Fixed

- `reference_select` in `hyle-dioxus` now renders a proper disabled "Loading…" state instead of panicking when `options` is `None`

## [0.1.1] — 2026-05-05

### Added

- `crates/hyle/assets/hyle.css` — shared CSS asset for Dioxus consumers
- `hyle-dioxus` `FilterField`: wraps all inputs in `<label>`; `Array<Reference>` fields render as checkbox `<fieldset>`; added `boolean_checkbox` internal helper
- `hyle-dioxus`: `FormFilterField` component (boolean fields render as checkbox)
- `hyle-react`: additional exports in `src/index.tsx`
- `examples/dioxus/e2e`: Playwright config and initial `users.spec.ts` test

### Changed

- `hyle-react-dom`: `filters.tsx`, `table.tsx`, `values.tsx`, and `form.tsx` significantly expanded to support checkbox fieldsets and typed value rendering for array/reference fields
- Removed committed `dist/` build artifacts from `packages/hyle-react`, `packages/hyle-react-dom`, and `crates/hyle/pkg-src` — build outputs are no longer tracked in git
- Dropped `packages/hyle-react-dom/src/table.css` in favour of the shared CSS token system

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
