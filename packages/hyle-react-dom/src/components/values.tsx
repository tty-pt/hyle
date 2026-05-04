import { useHyle } from "@tty-pt/hyle-react";
import type { ValueProps } from "@tty-pt/hyle-react";

export function BooleanValue({ value }: ValueProps) {
  return (
    <input
      type="checkbox"
      readOnly
      checked={Boolean(value)}
      aria-label={String(Boolean(value))}
    />
  );
}

/** Array values are resolved to display labels via hyle.displayValue. */
export function ArrayValue({ value, column, result, modelName, blueprint }: ValueProps) {
  const hyle = useHyle();
  if (!result) {
    return <span>{Array.isArray(value) ? value.join(", ") : String(value ?? "")}</span>;
  }
  return <span>{hyle.displayValue(blueprint, result, modelName, column.key, value)}</span>;
}

export function ReferenceValue({ value, column, result, modelName, blueprint }: ValueProps) {
  const hyle = useHyle();
  if (!result) return <span>{String(value ?? "")}</span>;
  return <span>{hyle.displayValue(blueprint, result, modelName, column.key, value)}</span>;
}
