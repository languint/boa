import { useState } from "react";
import { type AppState } from "./hooks/app-state";
import { useAppState } from "./hooks/use-app-state";
import { Editor } from "@monaco-editor/react";
import {
  connect,
  create,
  disconnect,
  execute,
  start,
  stop,
  upload,
  type Log,
} from "./runner";

const buttons = [
  {
    display: "Connect",
    onClick: (state: AppState | undefined, pushLog: Log) =>
      connect(state, pushLog),
  },
  {
    display: "Create",
    onClick: (state: AppState | undefined, pushLog: Log) =>
      create(state, pushLog),
  },
  {
    display: "Start",
    onClick: (state: AppState | undefined, pushLog: Log) =>
      start(state, pushLog),
  },
  {
    display: "Upload",
    onClick: (state: AppState | undefined, pushLog: Log) =>
      upload(state, pushLog),
  },
  {
    display: "Execute",
    onClick: (state: AppState | undefined, pushLog: Log) =>
      execute(state, pushLog),
  },
  {
    display: "Stop (SIGINT)",
    onClick: (state: AppState | undefined, pushLog: Log) =>
      stop("SIGINT", state, pushLog),
  },
  {
    display: "Stop (SIGTERM)",
    onClick: (state: AppState | undefined, pushLog: Log) =>
      stop("SIGTERM", state, pushLog),
  },
  {
    display: "Disconnect",
    onClick: (state: AppState | undefined, pushLog: Log) =>
      disconnect(state, pushLog),
  },
];

export function App() {
  const [logs, setLogs] = useState<[string, boolean][]>([]);

  const state = useAppState();

  const pushLog = (log: string, err?: boolean) => {
    setLogs((prev) => [...prev, [log, err ?? false]]);
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
      <div className="flex grow gap-2 w-full min-h-0">
        <div className="w-1/2 rounded-md border border-neutral-700 flex flex-col p-2 overflow-auto">
          {logs.map(([l, e], i) => (
            <p key={i} className={e ? "text-red-500" : ""}>
              {l}
            </p>
          ))}
        </div>

        <div className="w-1/2 rounded-md border border-neutral-700 flex flex-col min-h-0">
          <Editor
            height="100%"
            width="100%"
            language="python"
            theme="vs-dark"
            value={state.code}
            onChange={(v) => state?.setAppState?.({ ...state, code: v ?? "" })}
            options={{
              minimap: { enabled: false },
              fontSize: 14,
            }}
          />
        </div>
      </div>
    </div>
  );
}
