import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { HashRouter } from "react-router";
import { QueryClientProvider } from "@tanstack/react-query";
import App from "./App";
import { createAppQueryClient } from "./lib/queryClient";
import "./App.css";

const rootEl = document.getElementById("root");
if (!rootEl) {
  throw new Error("AI-Brains desktop: #root element missing from index.html");
}

const queryClient = createAppQueryClient();

createRoot(rootEl).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <HashRouter>
        <App />
      </HashRouter>
    </QueryClientProvider>
  </StrictMode>,
);
