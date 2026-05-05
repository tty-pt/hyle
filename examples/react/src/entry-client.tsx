import { StrictMode } from "react";
import { hydrateRoot } from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { HyleProvider, createHyleClient } from "@tty-pt/hyle-react";
import { SsrDataContext } from "./ssr-context";
import { blueprint } from "./demo-data";
import App from "./App";
import "./style.css";

const hyleClient = createHyleClient(() => import("./wasm-pkg/hyle"));
const queryClient = new QueryClient();

// Pick up any SSR data injected by the server into window.__SSR_DATA__
const ssrData = (window as unknown as Record<string, unknown>).__SSR_DATA__ as import("./ssr-context").SsrData | undefined ?? null;

hydrateRoot(
  document.getElementById("root")!,
  <StrictMode>
    <BrowserRouter>
      <QueryClientProvider client={queryClient}>
        <HyleProvider client={hyleClient} blueprint={blueprint}>
          <SsrDataContext.Provider value={ssrData}>
            <App />
          </SsrDataContext.Provider>
        </HyleProvider>
      </QueryClientProvider>
    </BrowserRouter>
  </StrictMode>,
);
