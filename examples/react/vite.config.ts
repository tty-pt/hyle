import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { fileURLToPath, URL } from "node:url";

export default defineConfig({
  resolve: {
    alias: {
      "@tty-pt/hyle-react": fileURLToPath(new URL("../../packages/hyle-react/src", import.meta.url)),
      "@tty-pt/hyle-react-dom": fileURLToPath(new URL("../../packages/hyle-react-dom/src", import.meta.url)),
      "@tty-pt/hyle-react-query": fileURLToPath(new URL("../../packages/hyle-react-query/src", import.meta.url)),
      "@tty-pt/hyle/hyle.css": fileURLToPath(new URL("../../crates/hyle/assets/hyle.css", import.meta.url)),
      "@tty-pt/hyle": fileURLToPath(new URL("../../crates/hyle/pkg-src", import.meta.url)),
      react: fileURLToPath(new URL("./node_modules/react", import.meta.url)),
      "react/jsx-runtime": fileURLToPath(
        new URL("./node_modules/react/jsx-runtime.js", import.meta.url),
      ),
      "@tanstack/react-query": fileURLToPath(
        new URL("./node_modules/@tanstack/react-query", import.meta.url),
      ),
    },
  },
  plugins: [
    react(),
    {
      name: "wasm-mime-fix",
      configureServer(server) {
        server.middlewares.use((req, res, next) => {
          if (req.url?.endsWith(".wasm")) {
            res.setHeader("Content-Type", "application/wasm");
          }
          next();
        });
      },
    },
  ],
});
