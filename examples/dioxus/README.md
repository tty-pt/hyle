# Dioxus example

A fullstack CRUD app built with `hyle-dioxus`, `hyle-dioxus-native`, and Axum — server-rendered, with optional client-side hydration.

## What you'll see

- A server-rendered user list with filters and pagination
- Clicking a row opens an edit form; the "Add" button opens a create form; delete is on the edit form
- Form errors render server-side before JavaScript loads, then update live once the page hydrates — the same validation logic runs in both places.
- A debug panel at the bottom exposes the raw Manifest and Outcome from the Rust engine — useful for understanding what the query planner produced.

## You'll need

- Rust (stable)
- [Dioxus CLI](https://dioxuslabs.com/learn/0.6/getting_started/)

```bash
cargo install dioxus-cli
```

## Install

Dependencies are fetched automatically by Cargo — nothing extra to install.

## Run in development

```bash
~/.cargo/bin/dx serve
```

The app will be available at `http://localhost:8080`.

## Build

```bash
~/.cargo/bin/dx build --platform web --release
```

## Run (after build)

The server binary needs to know where the pre-built wasm assets are:

```bash
DIOXUS_PUBLIC_PATH=target/dx/hyle-dioxus-example/release/web/public \
  cargo run -p hyle-dioxus-example --release
```

## Test (E2E)

First build the wasm assets, then start the server, then run the tests:

```bash
# 1. Build wasm assets
~/.cargo/bin/dx build --platform web

# 2. Start the server (in one terminal)
DIOXUS_PUBLIC_PATH=target/dx/hyle-dioxus-example/debug/web/public \
  cargo run -p hyle-dioxus-example

# 3. Run the tests (in another terminal)
npx playwright test --config examples/dioxus/e2e/playwright.config.ts
```

`npm run test:all` from the repo root handles all of this automatically.

## How it's structured

A few files do most of the work:

| File | What it does |
|---|---|
| `src/main.rs` | App entry point — blueprint, Axum routes, Dioxus setup, and all page components |
| `src/server.rs` | Server state, seed data, and server functions for CRUD operations |
| `src/blueprint.rs` | The Blueprint definition — fields, types, validation rules, and cross-model references |
| `e2e/` | Playwright end-to-end tests |
