import { makeHyleHooks, makeRestAdapter } from "@tty-pt/hyle-react";
import { hyleDomComponents } from "@tty-pt/hyle-react-dom";
import { blueprint } from "./demo-data";

const API = "http://localhost:3001";

const adapter = makeRestAdapter(`${API}/api`);

export const { useList, useFilters, useForm, useMutation } = makeHyleHooks({
  blueprint,
  adapter,
  components: hyleDomComponents,
});
