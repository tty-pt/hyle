import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { fileURLToPath, URL } from "node:url";

// SSR build config — react/react-dom/react-router-dom are external so the
// SSR bundle shares one React instance with them at runtime (no duplicate).
export default defineConfig({
  resolve: {
    alias: {
      "@tty-pt/hyle-react": fileURLToPath(new URL("../../packages/hyle-react/src", import.meta.url)),
      "@tty-pt/hyle-react-dom": fileURLToPath(new URL("../../packages/hyle-react-dom/src", import.meta.url)),
      "@tty-pt/hyle-react-query": fileURLToPath(new URL("../../packages/hyle-react-query/src", import.meta.url)),
      "@tty-pt/hyle/hyle.css": fileURLToPath(new URL("../../crates/hyle/assets/hyle.css", import.meta.url)),
      "@tty-pt/hyle": fileURLToPath(new URL("../../crates/hyle/pkg-src", import.meta.url)),
      // NOTE: no react/react-dom aliases here — they must stay external
    },
  },
  plugins: [react()],
  ssr: {
    external: [
      "react",
      "react-dom",
      "react-dom/server",
      "react-router-dom",
      "@tanstack/react-query",
    ],
  },
  build: {
    ssr: true,
    rollupOptions: {
      input: fileURLToPath(new URL("./src/entry-server.tsx", import.meta.url)),
      external: [
        "react",
        "react-dom",
        "react-dom/server",
        "react-router-dom",
        "@tanstack/react-query",
      ],
    },
    outDir: "dist/server",
  },
});
