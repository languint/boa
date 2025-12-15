import { createContext } from "react";
import * as wasm from "@boa-wasm";

export type WasmContextValue = {
  wasm: typeof wasm | null;
  loading: boolean;
  error: Error | null;
};

export const WasmContext = createContext<WasmContextValue | undefined>(
  undefined,
);
