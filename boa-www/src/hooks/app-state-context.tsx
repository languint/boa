import { useState, type ReactNode } from "react";
import { AppStateContext, type AppState } from "./app-state";

export function AppStateProvider({ children }: { children: ReactNode }) {
  const [appState, setAppState] = useState<AppState>({
    url: "ws://localhost:4040/ws",
  });

  return (
    <AppStateContext.Provider value={{ ...appState, setAppState }}>
      {children}
    </AppStateContext.Provider>
  );
}
