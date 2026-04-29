# React example

A full CRUD single-page app built with `@tty-pt/hyle-react`, `@tty-pt/hyle-react-dom`, TanStack Query, and React Router.

## What you'll see

- A paginated, sortable user list with inline column filters
- Clicking a row opens an edit form; the "Add" button opens a create form; delete is on the edit form
- Validation runs in Rust/WASM before the request is sent, with errors shown inline next to the relevant fields
- A debug panel at the bottom exposes the raw Manifest and Outcome from the Rust engine — useful for understanding what the query planner produced.

## You'll need

- Node.js >= 18
- Rust + `wasm-bindgen-cli` (to build `@tty-pt/hyle`)

```bash
cargo install wasm-bindgen-cli
rustup target add wasm32-unknown-unknown
```

## Install

```bash
npm install --prefix examples/react
```

## Run in development

```bash
# Build WASM + start the Vite dev server
npm run dev --prefix examples/react

# Start the REST API server (in a separate terminal)
npm run server --prefix examples/react
```

## Build

```bash
npm run build --prefix examples/react
```

## Test (E2E)

```bash
npm run test:e2e:react
```

The API server and preview server need to be running. This is handled automatically by `npm run test:all` from the root.

## How it's structured

A few files do most of the work:

| File | What it does |
|---|---|
| `src/blueprint.ts` | Defines the User and Role models — the single source of truth for fields, types, and validation rules |
| `src/hooks.ts` | Wires `makeHyleHooks` with the REST adapter and exports the ready-to-use hooks |
| `src/App.tsx` | React Router routes and top-level providers |
| `src/pages/` | List, detail, create, and edit page components |
| `server.js` | A small Express REST API with seeded in-memory data |
| `e2e/` | Playwright end-to-end tests |
