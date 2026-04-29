import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { HyleProvider, createHyleClient } from "@tty-pt/hyle-react";
import { blueprint } from "./demo-data";
import App from "./App";
import "./style.css";

const hyleClient = createHyleClient(() => import("./wasm-pkg/hyle_wasm"));
const queryClient = new QueryClient();

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <BrowserRouter>
      <QueryClientProvider client={queryClient}>
        <HyleProvider client={hyleClient} blueprint={blueprint}>
          <App />
        </HyleProvider>
      </QueryClientProvider>
    </BrowserRouter>
  </StrictMode>,
);
