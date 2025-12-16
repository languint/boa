import { useState } from "react";
import { RunnerState, type AppState } from "./hooks/app-state";
import { useAppState } from "./hooks/use-app-state";

type PushLog = (log: string, err?: boolean) => void;

const buttons = [
  {
    display: "Connect",
    onClick: (state: AppState | undefined, pushLog: PushLog) => {
      if (!state?.url) return;

      pushLog(`connecting to websocket server at ${state.url}`);

      state.ws = new WebSocket(state.url);

      state.runnerState = RunnerState.Connected;

      state.ws.onmessage = (e) => {
        pushLog(`recieved message: ${e.data}`);
      };

      state.ws.onerror = (e) => {
        pushLog(`recieved error: ${e}`);
      };
    },
  },
  {
    display: "Create",
    onClick: async (state: AppState | undefined, pushLog: PushLog) => {
      if (state?.runnerState != RunnerState.Connected) return;

      pushLog("requesting hosted runner...");

      state.ws!.onmessage = (e) => {
        const packet = JSON.parse(e.data);

        if (packet.type && packet.type == "ProcessOpenResult") {
          state.setAppState!({
            ...state,
            runnerId: packet.data.container_id,
          });
        } else {
          pushLog("failed to get hosted runner", true);
        }
      };

      const packet = {
        type: "ProcessOpen",
        data: {},
      };

      state.ws?.send(JSON.stringify(packet));
    },
  },
  {
    display: "Start",
    onClick: (state: AppState | undefined, pushLog: PushLog) => {
      if (!state?.runnerId) return;
      pushLog(`sending start runner packet to ${state?.runnerId}`);
    },
  },
  {
    display: "Upload",
    onClick: (state: AppState | undefined, pushLog: PushLog) => {},
  },
  {
    display: "Execute",
    onClick: (state: AppState | undefined, pushLog: PushLog) => {},
  },
  {
    display: "Stop (SIGINT)",
    onClick: (state: AppState | undefined, pushLog: PushLog) => {},
  },
  {
    display: "Stop (SIGTERM)",
    onClick: (state: AppState | undefined, pushLog: PushLog) => {},
  },
];

export function App() {
  const [logs, setLogs] = useState<[string, boolean][]>([]);

  const state = useAppState();

  const pushLog = (log: string, err?: boolean) => {
    setLogs([...logs, [log, err ?? false]]);
  };

  return (
    <div className="App flex flex-col gap-2 font-mono p-8!">
      <div className="flex flex-row gap-2 items-center">
        <h1 className="text-2xl">boa-www</h1>
        {state.runnerId && (
          <h1 className="text-2xl text-neutral-600">
            ({state.runnerId}@{state.runnerState})
          </h1>
        )}
      </div>
      <div className="flex flex-row gap-2 w-full items-center">
        <label>url</label>
        <input
          className="rounded-md border border-neutral-700 p-2"
          value={state?.url}
          onChange={(e) => {
            if (!state) return;
            state.setAppState!({
              ...state,
              url: e.target.value,
            });
          }}
        />
      </div>

      <div className="flex flex-row gap-2">
        {buttons.map((b, i) => (
          <button
            key={i}
            className="rounded-md p-2  bg-neutral-800 hover:bg-neutral-700 border border-neutral-700"
            onClick={() => b.onClick(state, pushLog)}
          >
            {b.display}
          </button>
        ))}
      </div>
      <div className="rounded-md border border-neutral-700 flex flex-col gap-2 grow p-2">
        {logs.map(([l, e], i) => (
          <div className={`w-full h-8 ${e ? "bg-red-500/80" : ""}`}>
            <p key={i} className="h-8 w-full">
              {l}
            </p>
          </div>
        ))}
      </div>
    </div>
  );
}
