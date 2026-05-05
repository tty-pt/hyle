import { createElement } from "react";
import { useHyle, componentKeyForField } from "@tty-pt/hyle-react";
import type { ValueProps } from "@tty-pt/hyle-react";
import type { ComponentType } from "react";
import type { Row } from "@tty-pt/hyle-react";

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

/** Array values are resolved to display labels via hyle.displayValue, or through a registered ValueComp for the item's display field type. */
export function ArrayValue({ value, column, result, modelName, blueprint, components }: ValueProps) {
  const hyle = useHyle();
  if (!result) {
    return <span>{Array.isArray(value) ? value.join(", ") : String(value ?? "")}</span>;
  }

  if (
    components &&
    column.field.type.kind === "array" &&
    column.field.type.item.kind === "reference"
  ) {
    const { entity, displayField } = column.field.type.item.reference;
    const displayFieldType = blueprint.models[entity]?.fields[displayField]?.type;
    const compKey = displayFieldType ? componentKeyForField(displayFieldType) : null;
    const ValueComp = compKey ? (components[compKey] as ComponentType<Record<string, unknown>> | undefined) : undefined;

    if (ValueComp) {
      const lookupMap = result.lookups?.[entity] ?? {};
      const ids = Array.isArray(value) ? value : value != null ? [value] : [];
      const nodes = ids.map((id, i) => {
        const refRow = (lookupMap as Record<string, Row>)[String(id)];
        const displayValue = refRow ? refRow[displayField] : id;
        const displayField_ = blueprint.models[entity]?.fields[displayField];
        return (
          <span key={String(id)}>
            {i > 0 && ", "}
            {createElement(ValueComp as ComponentType<Record<string, unknown>>, {
              value: displayValue,
              row: refRow ?? ({ id } as Row),
              field: displayField_ ?? column.field,
              column: { key: displayField, field: displayField_ ?? column.field, label: displayField },
              result,
              modelName: entity,
              blueprint,
              components,
            })}
          </span>
        );
      });
      return <span>{nodes}</span>;
    }
  }

  return <span>{hyle.displayValue(blueprint, result, modelName, column.key, value)}</span>;
}

export function ReferenceValue({ value, column, result, modelName, blueprint, components }: ValueProps) {
  const hyle = useHyle();
  if (!result) return <span>{String(value ?? "")}</span>;

  if (components && column.field.type.kind === "reference") {
    const { entity, displayField } = column.field.type.reference;
    const displayFieldType = blueprint.models[entity]?.fields[displayField]?.type;
    const compKey = displayFieldType ? componentKeyForField(displayFieldType) : null;
    const ValueComp = compKey ? (components[compKey] as ComponentType<Record<string, unknown>> | undefined) : undefined;

    if (ValueComp) {
      const lookupMap = result.lookups?.[entity] ?? {};
      const refRow = (lookupMap as Record<string, Row>)[String(value)];
      const displayValue = refRow ? refRow[displayField] : value;
      const displayField_ = blueprint.models[entity]?.fields[displayField];
      return createElement(ValueComp as ComponentType<Record<string, unknown>>, {
        value: displayValue,
        row: refRow ?? ({ id: value } as Row),
        field: displayField_ ?? column.field,
        column: { key: displayField, field: displayField_ ?? column.field, label: displayField },
        result,
        modelName: entity,
        blueprint,
        components,
      });
    }
  }

  return <span>{hyle.displayValue(blueprint, result, modelName, column.key, value)}</span>;
}
