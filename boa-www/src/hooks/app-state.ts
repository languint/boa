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
  code: string;
}

export const DEFAULT_APP_STATE = {
  url: "ws://localhost:4040/ws",
  runnerId: undefined,
  runnerState: undefined,
  ws: undefined,
  setAppState: undefined,
  code: 'print("Hello from boa-server!")',
};

export const AppStateContext = createContext<AppState>(DEFAULT_APP_STATE);
