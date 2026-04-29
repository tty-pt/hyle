import { describe, expectTypeOf, it } from "vitest";
import { makeHyleHooks } from "../src/index";
import { field } from "../../../crates/hyle/pkg-src/helpers";

const bp = {
  models: {
    user: {
      fields: {
        name: field.string("Name"),
        active: field.boolean("Active"),
      },
    },
  },
};

const { useList, useData, useFilters } = makeHyleHooks({
  blueprint: bp,
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  adapter: { useSource: null as any },
});

type ListRow  = ReturnType<typeof useList<"user">> extends { rows: (infer R)[] } ? R : never;
type DataRow  = ReturnType<typeof useData<"user">> extends { row: infer R | null } ? R : never;
type FormData = ReturnType<typeof useFilters<"user">>["formData"];

describe("makeHyleHooks row types", () => {
  it("useList rows are typed from the blueprint", () => {
    expectTypeOf<ListRow["name"]>().toEqualTypeOf<string>();
    expectTypeOf<ListRow["active"]>().toEqualTypeOf<boolean>();
  });

  it("useData row is typed from the blueprint", () => {
    expectTypeOf<DataRow["name"]>().toEqualTypeOf<string>();
    expectTypeOf<DataRow["active"]>().toEqualTypeOf<boolean>();
  });

  it("useFilters formData is Partial of the typed row", () => {
    expectTypeOf<FormData["name"]>().toEqualTypeOf<string | undefined>();
    expectTypeOf<FormData["active"]>().toEqualTypeOf<boolean | undefined>();
  });

  it("active is not typed as string (negative case)", () => {
    expectTypeOf<ListRow["active"]>().not.toEqualTypeOf<string>();
  });
});
