import { WasmContext } from "./wasm-context";
import { useContext } from "react";

export function useWasm() {
  const ctx = useContext(WasmContext);

  if (!ctx) {
    throw new Error("useWasm must be used inside a WasmProvider");
  }

  return ctx;
}
