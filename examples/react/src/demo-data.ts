import { field, type TypedBlueprint, type Query } from "@tty-pt/hyle-react";

const bp = {
  models: {
    user: {
      label: "Users",
      fields: {
        name: field.string("Name", { metadata: { required: true, minLength: 2 } }),
        email: field.string("Email"),
        role: field.ref("Role", "role"),
        active: field.boolean("Active"),
      },
    },
    role: {
      label: "Roles",
      fields: {
        name: field.string("Role name"),
      },
    },
  },
} satisfies TypedBlueprint<any>;

export const blueprint = bp;

export const query: Query = {
  model: "user",
  select: ["name", "email", "role", "active"],
  sort: { field: "name", ascending: true },
  perPage: 5,
};
