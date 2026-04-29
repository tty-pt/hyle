import { defineConfig } from "vitest/config";
import { fileURLToPath } from "node:url";

const r = (p: string) => fileURLToPath(new URL(p, import.meta.url));

export default defineConfig({
  test: {
    globals: true,
    environment: "jsdom",
  },
  resolve: {
    alias: [
      { find: "react/jsx-dev-runtime", replacement: r("node_modules/react/jsx-dev-runtime.js") },
      { find: "react/jsx-runtime",     replacement: r("node_modules/react/jsx-runtime.js") },
      { find: "react-dom/client",      replacement: r("node_modules/react-dom/client.js") },
      { find: "react-dom",             replacement: r("node_modules/react-dom/index.js") },
      { find: "react",                 replacement: r("node_modules/react/index.js") },
    ],
  },
});
