import type { ComponentType } from "react";
import { useHyle, FilterContext } from "@tty-pt/hyle-react";
import type { Blueprint, HyleFiltersState } from "@tty-pt/hyle-react";
import { FilterAppearanceContext } from "./filters";

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
    return <div className="hyle-error">No blueprint provided. Pass a blueprint prop or set one on HyleProvider.</div>;
  }

  const fields = hyle.modelFields(blueprint, model);

  return (
    <FilterContext.Provider value="form">
      <FilterAppearanceContext.Provider value={{ boolean: "checkbox" }}>
        <div className="hyle-edit-fields">
          {fields.map(({ key, label, field }) => {
            const Input = Filter[key] as ComponentType | undefined;
            if (!Input) return null;
            const selfLabels = field.type.kind === "array" ||
              (field.type.kind === "primitive" && field.type.primitive === "boolean");
            return (
              <div className="hyle-field-row" key={key}>
                {!selfLabels && <label>{label}</label>}
                <Input />
              </div>
            );
          })}
        </div>
      </FilterAppearanceContext.Provider>
    </FilterContext.Provider>
  );
}
