import { createContext } from "react";

export enum RunnerState {
  Connected,
  Started,
  Executing,
  Finished,
}

export interface AppState {
  url?: string;
  runnerId?: string;
  runnerState?: RunnerState;
  ws?: WebSocket;
  setAppState?: React.Dispatch<React.SetStateAction<AppState>>;
}

export const AppStateContext = createContext<AppState>({
  url: "ws://localhost:4040/ws",
  runnerId: undefined,
  runnerState: undefined,
  ws: undefined,
  setAppState: undefined,
});
