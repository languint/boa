import { useEffect, useState, type ReactNode } from "react";

import init, * as wasm from "@boa-wasm";
import { WasmContext } from "./wasm-context";

export function WasmProvider({ children }: { children: ReactNode }) {
  const [wasmModule, setWasmModule] = useState<typeof wasm | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    let cancelled = false;

    (async () => {
      try {
        await init();
        if (!cancelled) {
          setWasmModule(wasm);
          setLoading(false);
        }
      } catch (err) {
        if (!cancelled) {
          setError(err as Error);
          setLoading(false);
        }
      }
    })();

    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <WasmContext.Provider
      value={{
        wasm: wasmModule,
        loading,
        error,
      }}
    >
      {children}
    </WasmContext.Provider>
  );
}
