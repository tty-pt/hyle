import type { Field, TypedField } from "./types";

type ShapeOf<F extends Record<string, TypedField<unknown>>> = {
  [K in keyof F]: F[K] extends TypedField<infer T> ? T : never;
};

export const field = {
  string(label: string, options: Field["options"] = {}): TypedField<string> {
    return {
      label,
      type: { kind: "primitive", primitive: "string" },
      options: { input: { kind: "text" }, ...options },
    };
  },

  number(label: string, options: Field["options"] = {}): TypedField<number> {
    return {
      label,
      type: { kind: "primitive", primitive: "number" },
      options: { sort: "numeric", input: { kind: "number" }, ...options },
    };
  },

  boolean(label: string, options: Field["options"] = {}): TypedField<boolean> {
    return {
      label,
      type: { kind: "primitive", primitive: "boolean" },
      options: { sort: "none", input: { kind: "checkbox" }, ...options },
    };
  },

  file(label: string, options: Field["options"] = {}): TypedField<string> {
    return {
      label,
      type: { kind: "primitive", primitive: "file" },
      options: { input: { kind: "file" }, ...options },
    };
  },

  ref(label: string, entity: string, options: Field["options"] = {}): TypedField<string> {
    return {
      label,
      type: { kind: "reference", reference: { entity, displayField: "name" } },
      options: { input: { kind: "select" }, ...options },
    };
  },

  array<T>(label: string, item: TypedField<T>, options: Field["options"] = {}): TypedField<T[]> {
    return {
      label,
      type: { kind: "array", item: item.type },
      options: { ...options },
    };
  },

  shape<F extends Record<string, TypedField<unknown>>>(
    label: string,
    fields: F,
    options: Field["options"] = {},
  ): TypedField<ShapeOf<F>> {
    return {
      label,
      type: { kind: "shape", fields },
      options: { ...options },
    };
  },
};
