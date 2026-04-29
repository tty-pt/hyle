# @tty-pt/hyle-react

Add a full CRUD data layer to your React app in minutes — hooks for lists, forms, filters, and mutations, backed by a Rust/WASM engine for query planning and validation. You supply the blueprint and the adapter. The hooks take care of the rest.

See also: [`@tty-pt/hyle-react-dom`](https://github.com/tty-pt/hyle/tree/main/packages/hyle-react-dom) for pre-built table and form components, and [`examples/react`](https://github.com/tty-pt/hyle/tree/main/examples/react) for a full working app.

## Quick start

### 1. Install

```bash
npm install @tty-pt/hyle-react @tty-pt/hyle @tanstack/react-query
```

Peer dependencies: `@tty-pt/hyle`, `@tanstack/react-query >= 5`, `react >= 18`.

> `@tty-pt/hyle` is built from [`crates/hyle`](https://github.com/tty-pt/hyle/tree/main/crates/hyle) using the `wasm` feature.

### 2. Load the WASM engine and wrap your app

```tsx
import { createHyleClient, HyleProvider } from '@tty-pt/hyle-react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

const queryClient = new QueryClient()
const hyleClient = createHyleClient(() => import('./wasm-pkg/hyle_wasm'))

function Root() {
  return (
    <QueryClientProvider client={queryClient}>
      <HyleProvider client={hyleClient} blueprint={blueprint}>
        <App />
      </HyleProvider>
    </QueryClientProvider>
  )
}
```

### 3. Describe your data and where it lives

```ts
import { field, type TypedBlueprint } from '@tty-pt/hyle-react'
import { makeHyleHooks, makeRestAdapter } from '@tty-pt/hyle-react'

const blueprint = {
  models: {
    user: {
      fields: {
        name:   field.string("Name",   { metadata: { required: true } }),
        email:  field.string("Email"),
        active: field.boolean("Active"),
      },
    },
  },
} satisfies TypedBlueprint<any>

const adapter = makeRestAdapter('/api')

export const { useList, useFilters, useForm, useMutation } = makeHyleHooks({
  blueprint,
  adapter,
})
```

### 4. Render a list

```tsx
function UserList() {
  const list = useList({ model: 'user' })
  // list.rows, list.columns, list.page, list.setPage, list.filters, ...
}
```

Want pre-built table, filter, and form components? See [`@tty-pt/hyle-react-dom`](https://github.com/tty-pt/hyle/tree/main/packages/hyle-react-dom).

## Installation

```bash
npm install @tty-pt/hyle-react @tty-pt/hyle @tanstack/react-query
```

## API

### Provider & Client

| Export | Description |
|---|---|
| `createHyleClient(loader)` | Create a `HyleClient` from a WASM module loader |
| `HyleProvider` | Props: `{ client, blueprint?, children }`. Loads WASM; exposes `HyleContext`. |
| `useHyle()` | Access the raw WASM module methods directly |

### Hook Factory

```ts
const hooks = makeHyleHooks({ blueprint, adapter, components? })
```

Returns hooks that already know your schema and how to talk to your server:

| Hook | Returns | Description |
|---|---|---|
| `useManifest(query)` | `HyleManifestState` | Derive a manifest |
| `useData(query)` | `HyleDataState` | Fetch and resolve data |
| `useList(query, opts?)` | `HyleListState` | List with pagination, sorting, and filters |
| `useFilters(query, opts?)` | `HyleFiltersState` | Filter form state |
| `useForma(table, id?, opts?)` | `[Query \| null, Forma \| null]` | Dynamic form schema |
| `useForm(query, opts?)` | `HyleFormState` | Create/edit form with validation |
| `useMutation(model)` | `{ create, update, delete }` | CRUD mutations |

### TanStack Query Adapters

| Export | Description |
|---|---|
| `makeRestAdapter(baseUrl, opts?)` | Build a full `HyleAdapter` (source + CRUD) hitting REST endpoints |
| `makeReactQuerySource(queryFn)` | Wrap an async fetch function as a `HyleSourceAdapter` |
| `makeReactQueryMutation(mutationFn, opts?)` | Wrap an async function as a `HyleMutation` factory |
| `makeRestMutations(baseUrl, model, opts?)` | Returns `{ useSave, useAdd, useDelete }` for a single model |
| `hyleQueryKey(query)` | Derive a stable TanStack `QueryKey` from a `Query` |

### Optimistic Update Helpers

| Export | Description |
|---|---|
| `saveOptimistic(model)` | Replace the matching row in source (for PUT) |
| `addOptimistic(model)` | Append a new row and increment total (for POST) |
| `deleteOptimistic(model)` | Remove the matching row and decrement total (for DELETE) |

### Key Types

| Type | Description |
|---|---|
| `HyleAdapter` | `{ source, create, update, delete }` |
| `HyleMutation` | `{ mutate, isPending, isSuccess, error, reset }` |
| `HyleListState` | `{ rows, columns, page, setPage, perPage, setPerPage, sort, setSort, filters, isLoading, error }` |
| `HyleFiltersState` | `{ Filter, fields, formData, setField, apply, clear }` |
| `HyleFormState` | Extends `HyleFiltersState` with `isEdit`, `isValid`, `onSubmit`, `mutation` |
| `HyleDataState` | `Loading` / `Ready { manifest, outcome, rows, columns }` / `Error` |

## Testing

```bash
npm test --prefix packages/hyle-react
```
