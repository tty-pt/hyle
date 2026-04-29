# @tty-pt/hyle-react-dom

Pre-built table, filter, and form components for React. Your data has shape — these components just show it. Pass in the state from `@tty-pt/hyle-react` and your CRUD UI is done.

Requires [`@tty-pt/hyle-react`](https://github.com/tty-pt/hyle/tree/main/packages/hyle-react). See [`examples/react`](https://github.com/tty-pt/hyle/tree/main/examples/react) for a full working app.

## Quick start

The snippet below renders a filterable, paginated table with a create form. Pass in the list and form state — `HyleTable` and `HyleFormFields` handle the rest.

```tsx
import { makeHyleHooks, makeRestAdapter } from '@tty-pt/hyle-react'
import { hyleDomComponents, HyleTable, HyleTableFilters } from '@tty-pt/hyle-react-dom'

const adapter = makeRestAdapter('/api')
const { useList, useFilters, useForm } = makeHyleHooks({
  blueprint,
  adapter,
  components: hyleDomComponents,  // wires filter inputs automatically
})

function UserList() {
  const filters = useFilters({ model: 'user' })
  const list    = useList(filters.query)

  return (
    <>
      <HyleTableFilters filters={filters} />
      <HyleTable list={list} filters={filters} />
    </>
  )
}

function UserForm() {
  const form = useForm({ model: 'user' })
  return (
    <form onSubmit={form.onSubmit}>
      <HyleFormFields model="user" Filter={form.Filter} />
      <button type="submit">Save</button>
    </form>
  )
}
```

## Installation

```bash
npm install @tty-pt/hyle-react-dom @tty-pt/hyle-react
```

Peer dependencies: `@tty-pt/hyle-react`, `react >= 19`

## Components

### Filter Inputs

| Component | Field type | Notes |
|---|---|---|
| `FilterString` | `string` | `<input type="text">` |
| `FilterNumber` | `number` | `<input type="number">` |
| `FilterBoolean` | `boolean` | Checkbox or three-state `<select>` via `appearance` prop |
| `FilterReference` | `reference` | `<select>` (default) or `<input>+<datalist>` autocomplete via `appearance` prop |
| `FilterFile` | `file` | `<input type="text">` |

### Value Renderers

| Component | Field type | Notes |
|---|---|---|
| `BooleanValue` | `boolean` | Read-only checkbox |
| `ReferenceValue` | `reference` | Resolved display label |

### Table

| Component | Props | Description |
|---|---|---|
| `HyleTable` | `list`, `blueprint?`, `filters?`, `onRowClick?`, `selectedId?`, `components?` | `HyleTableBody` + `HyleTablePagination` |
| `HyleTableBody` | same as above | Sortable `<table>` with optional inline column filters and custom value renderers |
| `HyleTableFilters` | `{ filters: HyleFiltersState }` | Apply / Clear buttons |
| `HyleTablePagination` | `{ list: HyleListState }` | Prev/Next + per-page `<select>` |

### Form

| Component | Props | Description |
|---|---|---|
| `HyleFormFields` | `{ blueprint?, model, Filter }` | Renders `label + Filter[key]` for every field in a model |

### Pre-built Component Map

| Export | Description |
|---|---|
| `hyleDomComponents` | Ready-to-use `HyleFieldComponents` map containing all filter inputs and value renderers. Pass as `components` to `makeHyleHooks`. |

## Testing

```bash
npm test --prefix packages/hyle-react-dom
```
