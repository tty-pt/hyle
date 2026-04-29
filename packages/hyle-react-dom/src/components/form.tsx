import type { ComponentType } from "react";
import { useHyle } from "@tty-pt/hyle-react";
import type { Blueprint, HyleFiltersState } from "@tty-pt/hyle-react";

// ── HyleFormFields ────────────────────────────────────────────────────────────
//
// Renders a labeled input row for every field in a model, using the Filter
// components from a HyleFiltersState.  Skips fields with no Filter component.

export function HyleFormFields({
  blueprint: blueprintProp,
  model,
  Filter,
}: {
  blueprint?: Blueprint;
  model: string;
  Filter: HyleFiltersState["Filter"];
}) {
  const hyle = useHyle();
  const blueprint = blueprintProp ?? hyle.blueprint;

  if (!blueprint) {
    return <div className="hyle-form-error">No blueprint provided. Pass a blueprint prop or set one on HyleProvider.</div>;
  }

  const fields = hyle.modelFields(blueprint, model);

  return (
    <div className="hyle-form-fields">
      {fields.map(({ key, label }) => {
        const Input = Filter[key] as ComponentType | undefined;
        if (!Input) return null;
        return (
          <div className="hyle-field-row" key={key}>
            <label className="hyle-field-label">{label}</label>
            <Input />
          </div>
        );
      })}
    </div>
  );
}
