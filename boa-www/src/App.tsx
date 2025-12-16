import { useState } from "react";
import { RunnerState, type AppState } from "./hooks/app-state";
import { useAppState } from "./hooks/use-app-state";
import { Editor } from "@monaco-editor/react";

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

      state.ws.onerror = () => {
        pushLog("recieved server error", true);
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
          pushLog(`connected to hosted runner \`${packet.data.container_id}\``);
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
      if (state?.runnerState !== RunnerState.Connected) return;
      pushLog(`requesting runner start...`);

      state.ws!.onmessage = (e) => {
        const packet = JSON.parse(e.data);

        if (packet.type && packet.type == "ProcessEvent") {
          if (packet.data === "Started") {
            pushLog(`hosted runner \`${state.runnerId}\` is started`);
            state.setAppState!({
              ...state,
              runnerState: RunnerState.Started,
            });
          } else {
            pushLog("recieved invalid server packet", true);
          }
        } else {
          pushLog("failed to start hosted runner", true);
        }
      };

      const packet = {
        type: "ProcessControlSignal",
        data: {
          container_id: state.runnerId,
          control_signal: "Start",
        },
      };

      state.ws?.send(JSON.stringify(packet));
    },
  },
  {
    display: "Upload",
    onClick: (state: AppState | undefined, pushLog: PushLog) => {
      if (state?.runnerState !== RunnerState.Started) return;
      pushLog(`requesting runner upload...`);

      const startPacket = {
        type: "UploadStart",
        data: {
          container_id: state.runnerId,
          path: "main.py",
          size: state.code.length,
        },
      };

      state.ws?.send(JSON.stringify(startPacket));

      const textEncoder = new TextEncoder();

      const codePacket = textEncoder.encode(state.code);

      state.ws?.send(codePacket);

      const finishPacket = {
        type: "UploadFinish",
        data: {
          container_id: state.runnerId,
        },
      };

      state.ws?.send(JSON.stringify(finishPacket));

      pushLog(`runner upload finished`);
    },
  },
  {
    display: "Execute",
    onClick: (state: AppState | undefined, pushLog: PushLog) => {
      if (state?.runnerState !== RunnerState.Started) return;

      state.ws!.onmessage = (e) => {
        const packet = JSON.parse(e.data);
        console.log(packet);
        switch (packet.type) {
          case "ProcessEvent":
            if (packet.data === "Started") {
              pushLog(`runner ${state.runnerId} is starting execution`);
            } else if (packet.data === "TimedOut") {
              pushLog(`runner executed timed out`, true);
            } else if (packet.data.Finished) {
              pushLog(
                `runner finished execution with exit code \`${packet.data.Finished.exit_code}\``,
              );
            }
            break;
          case "ProcessOutput":
            if (packet.data.StdOut) {
              packet.data.StdOut.split("\n").forEach((o: string) => pushLog(o));
            } else if (packet.data.StdErr) {
              packet.data.StdErr.split("\n").forEach((o: string) =>
                pushLog(o, true),
              );
            }
            break;
          default:
            pushLog(`unhandled packet type: ${packet.type}!`, true);
        }
      };

      const execPacket = {
        type: "ProcessControlSignal",
        data: {
          container_id: state.runnerId,
          control_signal: { Exec: "main.py" },
        },
      };

      state.ws?.send(JSON.stringify(execPacket));

      pushLog(`requested execution`);
      pushLog(`---`);
    },
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
