import { useEffect } from "react";
import { Routes, Route, Link, useNavigate, useParams } from "react-router-dom";
import { HyleTable, HyleTableFilters, HyleFormFields } from "@tty-pt/hyle-react-dom";
import { query } from "./demo-data";
import { useFilters, useList, useForm, useMutation } from "./hooks";

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
  const filters = useFilters(query);
  const list    = useList(filters.query);

  return (
    <main className="shell">
      <section className="toolbar">
        <div>
          <h1>hyle React example</h1>
          <p>Rust plans the query and resolves lookup data; React renders the table.</p>
        </div>
      </section>

      <div className="workspace">
        <section className="panel">
          <header className="panelHeader">
            <div>
              <h2>Users</h2>
            </div>
            <div className="panelHeaderRight">
              <HyleTableFilters filters={filters} />
              <Link to="/users/new" className="primaryButton">+ Add</Link>
            </div>
          </header>

          <HyleTable
            list={list}
            filters={filters}
            onRowClick={(row) => navigate(`/users/${row.id}/edit`)}
          />
        </section>
      </div>

      <section className="debugGrid">
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

// ── UserNew ───────────────────────────────────────────────────────────────────

function UserNew() {
  const navigate = useNavigate();
  const form = useForm(query);

  useEffect(() => {
    if (form.mutation?.isSuccess) navigate("/");
  }, [form.mutation?.isSuccess, navigate]);

  return (
    <main className="shell">
      <div className="workspace">
        <section className="editPanel">
          <header className="panelHeader">
            <h2>Add user</h2>
            <Link to="/" className="closeButton">×</Link>
          </header>

          <form onSubmit={form.onSubmit}>
            <HyleFormFields model="user" Filter={form.Filter} />

            {form.purifyErrors && form.purifyErrors.length > 0 && (
              <ul className="errors">
                {form.purifyErrors.map((e) => (
                  <li key={e.field}>{e.field}: {e.message}</li>
                ))}
              </ul>
            )}

            {form.mutation?.error && (
              <p className="errors">{form.mutation.error.message}</p>
            )}

            <div className="editActions">
              <button type="submit" disabled={!form.isValid || form.mutation?.isPending} className="primaryButton">
                {form.mutation?.isPending ? "Saving…" : "Save"}
              </button>
              <Link to="/">Cancel</Link>
            </div>
          </form>

          <div className="editDebug">
            <JsonBlock title="Form data" value={form.formData} />
          </div>
        </section>
      </div>
    </main>
  );
}

// ── UserEdit ──────────────────────────────────────────────────────────────────

function UserEdit() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();

  const editQuery = { ...query, where: { id: Number(id) }, method: "one" as const };
  const form = useForm(editQuery, {
    // Demonstrate change: relabel "email" for the edit form.
    change: { email: (f) => ({ ...f, label: "Work email" }) },
  });
  const mut = useMutation("user");

  useEffect(() => {
    if (form.mutation?.isSuccess || mut?.delete.isSuccess) navigate("/");
  }, [form.mutation?.isSuccess, mut?.delete.isSuccess, navigate]);

  return (
    <main className="shell">
      <div className="workspace">
        <section className="editPanel">
          <header className="panelHeader">
            <h2>Edit user</h2>
            <Link to="/" className="closeButton">×</Link>
          </header>

          <form onSubmit={form.onSubmit}>
            <HyleFormFields model="user" Filter={form.Filter} />

            {form.purifyErrors && form.purifyErrors.length > 0 && (
              <ul className="errors">
                {form.purifyErrors.map((e) => (
                  <li key={e.field}>{e.field}: {e.message}</li>
                ))}
              </ul>
            )}

            {form.mutation?.error && (
              <p className="errors">{form.mutation.error.message}</p>
            )}

            <div className="editActions">
              <button type="submit" disabled={!form.isValid || form.mutation?.isPending} className="primaryButton">
                {form.mutation?.isPending ? "Saving…" : "Save"}
              </button>
              <Link to="/">Cancel</Link>
            </div>
          </form>

          <form onSubmit={(e) => { e.preventDefault(); mut?.delete.mutate({ id: Number(id), data: {} }); }}>
            <div className="editActions">
              <button type="submit" className="dangerButton" disabled={mut?.delete.isPending}>
                {mut?.delete.isPending ? "Deleting…" : "Delete"}
              </button>
            </div>
          </form>

          <div className="editDebug">
            <JsonBlock title="Form data" value={form.formData} />
          </div>
        </section>
      </div>
    </main>
  );
}

// ── Helpers ───────────────────────────────────────────────────────────────────

function JsonBlock({ title, value }: { title: string; value: unknown }) {
  return (
    <section className="debugBlock">
      <h2>{title}</h2>
      <pre>{JSON.stringify(value, null, 2)}</pre>
    </section>
  );
}
