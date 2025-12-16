import { useState, type ReactNode } from "react";
import { AppStateContext, DEFAULT_APP_STATE, type AppState } from "./app-state";

export function AppStateProvider({ children }: { children: ReactNode }) {
  const [appState, setAppState] = useState<AppState>(DEFAULT_APP_STATE);

  return (
    <AppStateContext.Provider value={{ ...appState, setAppState }}>
      {children}
    </AppStateContext.Provider>
  );
}
