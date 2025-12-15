import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App.tsx";
import "./index.css";

import init from "@boa-wasm";
import { WasmProvider } from "./hooks/wasm-provider.tsx";

await init();

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <WasmProvider>
      <App />
    </WasmProvider>
  </StrictMode>,
);
