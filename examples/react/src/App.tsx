import { useEffect, useMemo, type ReactNode } from "react";
import { Routes, Route, Link, useNavigate, useParams } from "react-router-dom";
import { HyleTable, HyleTableFilterBar, HyleTableFilters, HyleTablePanel, HyleFormFields } from "@tty-pt/hyle-react-dom";
import type { HyleFormState, Manifest, Result } from "@tty-pt/hyle-react";
import { query, blueprint } from "./demo-data";
import { useFilters, useList, useForm, useMutation } from "./hooks";
import { useSsrData } from "./ssr-context";

// ── App shell ─────────────────────────────────────────────────────────────────

export default function App() {
  return (
    <Routes>
      <Route path="/" element={<UserList />} />
      <Route path="/users/new" element={<UserNew />} />
      <Route path="/users/:id/edit" element={<UserEdit />} />
    </Routes>
  );
}

// ── UserList ──────────────────────────────────────────────────────────────────

function UserList() {
  const navigate = useNavigate();
  const ssrData = useSsrData();

  const initialCommitted = ssrData?.list?.filters as Record<string, unknown> | undefined;

  const initialResult = useMemo(
    () => ssrData?.list?.lookups ? { rows: [], total: 0, lookups: ssrData.list.lookups } as Result : undefined,
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [],
  );

  const initialListData = useMemo(() => {
    const list = ssrData?.list;
    if (!list) return undefined;
    const model = blueprint.models[query.model as keyof typeof blueprint.models];
    const fields = model ? Object.keys(model.fields) : [];
    const manifest: Manifest = {
      base: query.model,
      fields,
      filterFields: query.filters ?? [],
      ...(list.lookups && Object.keys(list.lookups).length > 0
        ? { lookups: Object.keys(list.lookups) }
        : {}),
    };
    const result: Result = {
      rows: list.rows,
      total: list.total,
      lookups: list.lookups,
    };
    return { rows: list.rows, total: list.total, result, manifest };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const filters = useFilters(query, { initialCommitted, initialResult });
  const list    = useList(filters.query, { initialData: initialListData });

  return (
    <main className="hyle-shell">
      <section className="hyle-toolbar">
        <div>
          <h1>hyle React example</h1>
          <p>Rust plans the query and resolves lookup data; React renders the table.</p>
        </div>
      </section>

      <div className="hyle-workspace">
        <section className="hyle-panel">
          <HyleTablePanel
            list={list}
            filters={filters}
            onRowClick={(row) => navigate(`/users/${row.id}/edit`)}
          >
            <header className="hyle-panel-header">
              <div className="hyle-panel-title-row">
                <h2>Users</h2>
                <details className="hyle-action-menu">
                  <summary className="hyle-action-menu-toggle">Actions</summary>
                  <ul className="hyle-action-menu-list">
                    <li><a href="/users/new" className="hyle-primary-button">Add user</a></li>
                  </ul>
                </details>
              </div>
              <HyleTableFilterBar
                filters={filters}
                only={["name", "email", "role", "tags", "active"]}
              >
                <HyleTableFilters />
              </HyleTableFilterBar>
            </header>
          </HyleTablePanel>
        </section>
      </div>

      <section className="hyle-debug-grid">
        {list.status === "ready" && (
          <>
            <JsonBlock title="Manifest from Rust" value={list.manifest} />
            <JsonBlock title="Outcome from Rust" value={list.result}   />
          </>
        )}
      </section>
    </main>
  );
}

// ── UserFormPanel ─────────────────────────────────────────────────────────────

function UserFormPanel({
  title,
  action,
  form,
  delete: deleteSlot,
}: {
  title: string;
  action: string;
  form: HyleFormState;
  delete?: ReactNode;
}) {
  return (
    <main className="hyle-shell">
      <section className="hyle-panel">
        <header className="hyle-panel-header">
          <div className="hyle-panel-title-row">
            <h2>{title}</h2>
            <Link to="/" className="hyle-close-button">×</Link>
          </div>
        </header>

        <form onSubmit={form.onSubmit} action={action} method="post">
          <HyleFormFields model="user" Filter={form.Filter} />

          {form.purifyErrors && form.purifyErrors.length > 0 && (
            <ul className="hyle-errors">
              {form.purifyErrors.map((e) => (
                <li key={e.field}>{e.field}: {e.message}</li>
              ))}
            </ul>
          )}

          {form.mutation?.error && (
            <p className="hyle-errors">{form.mutation.error.message}</p>
          )}

          <div className="hyle-edit-actions">
            <button
              type="submit"
              disabled={!form.isValid || !!form.mutation?.isPending}
              className="hyle-primary-button"
            >
              {form.mutation?.isPending ? "Saving…" : "Save"}
            </button>
            <Link to="/">Cancel</Link>
          </div>
        </form>

        {deleteSlot}

        <div className="hyle-edit-debug">
          <JsonBlock title="Form data" value={form.formData} />
        </div>
      </section>
    </main>
  );
}

// ── UserNew ───────────────────────────────────────────────────────────────────

function UserNew() {
  const navigate = useNavigate();
  const form = useForm(query);

  useEffect(() => {
    if (form.mutation?.isSuccess) navigate("/");
  }, [form.mutation?.isSuccess, navigate]);

  return (
    <UserFormPanel title="Add user" action="/users/new" form={form} />
  );
}

// ── UserEdit ──────────────────────────────────────────────────────────────────

function UserEdit() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();

  const editQuery = { ...query, where: { id: Number(id) }, method: "one" as const };
  const form = useForm(editQuery, {
    change: { email: (f) => ({ ...f, label: "Work email" }) },
  });
  const mut = useMutation("user");

  useEffect(() => {
    if (form.mutation?.isSuccess || mut?.delete.isSuccess) navigate("/");
  }, [form.mutation?.isSuccess, mut?.delete.isSuccess, navigate]);

  return (
    <UserFormPanel
      title="Edit user"
      action={`/users/${id}/edit`}
      form={form}
      delete={
        <form
          method="post"
          action={`/users/${id}/delete`}
          onSubmit={(e) => {
            e.preventDefault();
            mut?.delete.mutate({ id: Number(id), data: {} });
          }}
        >
          <div className="hyle-edit-actions">
            <button type="submit" className="hyle-danger-button" disabled={mut?.delete.isPending}>
              {mut?.delete.isPending ? "Deleting…" : "Delete"}
            </button>
          </div>
        </form>
      }
    />
  );
}

// ── Helpers ───────────────────────────────────────────────────────────────────

function JsonBlock({ title, value }: { title: string; value: unknown }) {
  return (
    <section className="hyle-debug-block">
      <h2>{title}</h2>
      <pre>{JSON.stringify(value, null, 2)}</pre>
    </section>
  );
}
