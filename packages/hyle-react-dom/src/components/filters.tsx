import { useId } from "react";
import type { FilterProps } from "@tty-pt/hyle-react";
import type { Row } from "@tty-pt/hyle-react";

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
   * - `"checkbox"` (default) — unchecked = any/unset (`undefined`), checked = yes (`true`)
   * - `"select"` — three-state `<select>`: Any / Yes / No
   */
  appearance?: "checkbox" | "select";
};

export function FilterBoolean({ label, value, onChange, appearance = "checkbox" }: FilterBooleanProps) {
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

  // Default: checkbox — unchecked = undefined (any), checked = true
  return (
    <input
      type="checkbox"
      aria-label={label}
      checked={value === true}
      onChange={(e) => onChange(e.target.checked ? true : undefined)}
    />
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

  // Extract lookup options from result
  const options: { id: string; label: string }[] = [];
  if (result && field.type.kind === "reference") {
    const { entity, displayField } = field.type.reference;
    const lookupMap = result.lookups?.[entity];
    if (lookupMap) {
      for (const [id, row] of Object.entries(lookupMap)) {
        options.push({ id, label: String((row as Row)[displayField] ?? id) });
      }
    }
  }

  if (appearance === "autocomplete") {
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
