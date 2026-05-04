import { useId, createContext, useContext } from "react";
import type { FilterProps } from "@tty-pt/hyle-react";
import type { Row } from "@tty-pt/hyle-react";

// ── FilterAppearanceContext ────────────────────────────────────────────────────

export type FilterAppearance = {
  boolean?: "checkbox" | "select";
};

export const FilterAppearanceContext = createContext<FilterAppearance>({ boolean: "select" });

// ── FilterString ──────────────────────────────────────────────────────────────

export function FilterString({ label, value, onChange }: FilterProps<string>) {
  return (
    <input
      type="text"
      aria-label={label}
      placeholder={label}
      value={value}
      onChange={(e) => onChange(e.target.value)}
    />
  );
}

// ── FilterNumber ──────────────────────────────────────────────────────────────

export function FilterNumber({ label, value, onChange }: FilterProps<string>) {
  return (
    <input
      type="number"
      step="any"
      aria-label={label}
      placeholder={label}
      value={value}
      onChange={(e) => onChange(e.target.value)}
    />
  );
}

// ── FilterBoolean ─────────────────────────────────────────────────────────────

export type FilterBooleanProps = FilterProps<boolean | undefined> & {
  /**
   * - `"select"` (default) — three-state `<select>`: Any / Yes / No (good for filter bars)
   * - `"checkbox"` — unchecked = any/unset (`undefined`), checked = yes (`true`), self-labels
   * If not provided, reads from `FilterAppearanceContext` (default: `"select"`).
   */
  appearance?: "checkbox" | "select";
};

export function FilterBoolean({ label, value, onChange, appearance: appearanceProp }: FilterBooleanProps) {
  const ctx = useContext(FilterAppearanceContext);
  const appearance = appearanceProp ?? ctx.boolean ?? "select";

  if (appearance === "select") {
    const strVal = value === true ? "true" : value === false ? "false" : "";
    return (
      <select
        aria-label={label}
        value={strVal}
        onChange={(e) => {
          const v = e.target.value;
          onChange(v === "true" ? true : v === "false" ? false : undefined);
        }}
      >
        <option value="">Any</option>
        <option value="true">Yes</option>
        <option value="false">No</option>
      </select>
    );
  }

  // Checkbox: wrapped in fieldset so .fieldRow fieldset / fieldset label styles apply
  return (
    <fieldset>
      <legend></legend>
      <label>
        <input
          type="checkbox"
          checked={value === true}
          onChange={(e) => onChange(e.target.checked ? true : undefined)}
        />
        {label}
      </label>
    </fieldset>
  );
}

// ── FilterReference ───────────────────────────────────────────────────────────

export type FilterReferenceProps = FilterProps<string> & {
  /**
   * - `"select"` (default) — `<select>` populated from `result.lookups`
   * - `"autocomplete"` — `<input>` paired with a `<datalist>` for browser-native autocomplete
   */
  appearance?: "select" | "autocomplete";
};

export function FilterReference({
  label,
  value,
  field,
  result,
  onChange,
  appearance = "select",
}: FilterReferenceProps) {
  const listId = useId();

  // null = still loading; array = ready (may be empty)
  let options: { id: string; label: string }[] | null = null;
  if (result !== null) {
    options = [];
    if (field.type.kind === "reference") {
      const { entity, displayField } = field.type.reference;
      const lookupMap = result?.lookups?.[entity];
      if (lookupMap) {
        for (const [id, row] of Object.entries(lookupMap)) {
          options.push({ id, label: String((row as Row)[displayField] ?? id) });
        }
      }
    }
  }

  if (appearance === "autocomplete") {
    if (options === null) {
      return <input type="text" aria-label={label} placeholder="Loading…" disabled />;
    }
    return (
      <>
        <input
          type="text"
          aria-label={label}
          placeholder={label}
          value={value}
          list={listId}
          onChange={(e) => onChange(e.target.value)}
        />
        <datalist id={listId}>
          {options.map((opt) => (
            <option key={opt.id} value={opt.id}>{opt.label}</option>
          ))}
        </datalist>
      </>
    );
  }

  // Default: select
  if (options === null) {
    return (
      <select aria-label={label} disabled>
        <option>Loading…</option>
      </select>
    );
  }
  return (
    <select
      aria-label={label}
      value={value}
      onChange={(e) => onChange(e.target.value)}
    >
      <option value="">Any</option>
      {options.map((opt) => (
        <option key={opt.id} value={opt.id}>{opt.label}</option>
      ))}
    </select>
  );
}

// ── FilterFile ────────────────────────────────────────────────────────────────

export function FilterFile({ label, value, onChange }: FilterProps<string>) {
  return (
    <input
      type="text"
      aria-label={label}
      placeholder={label}
      value={value}
      onChange={(e) => onChange(e.target.value)}
    />
  );
}

// ── FilterArray ───────────────────────────────────────────────────────────────
// Always renders a <fieldset> with checkboxes.
// Options source depends on item subtype:
//   Array<Reference> → result.lookups
//   Array<string|number> → distinct values scanned from result.rows

export function FilterArray<T>({
  label,
  value,
  field,
  fieldName,
  result,
  onChange,
}: FilterProps<T>) {
  const selected = (value as unknown[]) ?? [];

  type Option = { id: string; label: string };
  let options: Option[] | null = null;

  if (result && field.type.kind === "array") {
    const item = field.type.item;

    if (item.kind === "reference") {
      const { entity, displayField } = item.reference;
      const lookupMap = result.lookups?.[entity];
      if (lookupMap) {
        options = Object.entries(lookupMap).map(([id, row]) => ({
          id,
          label: String((row as Row)[displayField] ?? id),
        }));
      }
    } else if (item.kind === "primitive") {
      const rows: Row[] = Array.isArray(result.rows)
        ? result.rows
        : result.rows
        ? [result.rows as Row]
        : [];
      const seen = new Set<string>();
      options = [];
      for (const row of rows) {
        const cell = row[fieldName];
        const items = Array.isArray(cell) ? cell : cell != null ? [cell] : [];
        for (const v of items) {
          const id = String(v);
          if (!seen.has(id)) {
            seen.add(id);
            options.push({ id, label: id });
          }
        }
      }
    }
  }

  return (
    <fieldset>
      <legend>{label}</legend>
      {options === null ? (
        <span aria-busy="true">Loading…</span>
      ) : options.length === 0 ? (
        <span>No options</span>
      ) : (
        options.map((opt) => (
          <label key={opt.id}>
            <input
              type="checkbox"
              value={opt.id}
              checked={selected.includes(opt.id)}
              onChange={(e) => {
                const next = e.target.checked
                  ? [...selected, opt.id]
                  : selected.filter((s) => s !== opt.id);
                onChange(next as T);
              }}
            />
            {" "}{opt.label}
          </label>
        ))
      )}
    </fieldset>
  );
}

