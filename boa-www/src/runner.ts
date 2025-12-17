import { RunnerState, type AppState } from "./hooks/app-state";

export type Log = (log: string, err?: boolean) => void;

export async function connect(state: AppState | undefined, log: Log) {
  if (!state) return;
  if (!state.url) {
    log("cannot to connect to no url!", true);
    return;
  }

  if (state.runnerState === RunnerState.Connected) {
    log("already connected!", true);
  }

  state.ws = new WebSocket(state.url);

  state.runnerState = RunnerState.Connected;

  log("successfully connected to remote");
}

export async function create(state: AppState | undefined, log: Log) {
  if (state?.runnerState !== RunnerState.Connected) {
    log("cannot create new runner, not connected to remote!", true);
    return;
  }

  state.ws!.onmessage = (e) => {
    const packet = JSON.parse(e.data);

    if (packet.type && packet.type == "ProcessOpenResult") {
      log(`connected to runner \`${packet.data.container_id}\``);
      state.setAppState!({
        ...state,
        runnerId: packet.data.container_id,
      });
    } else {
      log("failed to get runner!", true);
    }
  };

  const packet = {
    type: "ProcessOpen",
    data: {},
  };

  state.ws?.send(JSON.stringify(packet));
}

export async function start(state: AppState | undefined, log: Log) {
  if (!state) return;
  if (state?.runnerState !== RunnerState.Connected) {
    log("cannot request runner to start, not connected to remote!", true);
  }

  state.ws!.onmessage = (e) => {
    const packet = JSON.parse(e.data);

    if (packet.type && packet.type == "ProcessEvent") {
      if (packet.data === "Started") {
        log(`hosted runner \`${state.runnerId}\` is started`);
        state.setAppState!({
          ...state,
          runnerState: RunnerState.Started,
        });
      } else {
        log("recieved unhandled server packet", true);
      }
    } else {
      log("failed to start runner", true);
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
}

export async function upload(state: AppState | undefined, log: Log) {
  if (!state) return;
  if (
    state.runnerState !== RunnerState.Started &&
    state?.runnerState !== RunnerState.Finished
  ) {
    log("cannot upload code to runner if runner is not started");
  }

  const startPacket = {
    type: "UploadStart",
    data: {
      container_id: state!.runnerId,
      path: "main.py",
      size: state!.code.length,
    },
  };

  log("starting upload");

  state!.ws?.send(JSON.stringify(startPacket));

  const textEncoder = new TextEncoder();

  const codePacket = textEncoder.encode(state!.code);

  state!.ws?.send(codePacket);

  const finishPacket = {
    type: "UploadFinish",
    data: {
      container_id: state!.runnerId,
    },
  };

  state!.ws?.send(JSON.stringify(finishPacket));

  log("upload finished");
}

export async function disconnect(state: AppState | undefined, log: Log) {
  if (
    state?.runnerState !== RunnerState.Connected &&
    state?.runnerState !== RunnerState.Finished &&
    state?.runnerState !== RunnerState.Executing
  )
    return;

  log(`disconnecting from remote`);

  state!.ws!.close();
}

export async function execute(state: AppState | undefined, log: Log) {
  if (
    state?.runnerState !== RunnerState.Started &&
    state?.runnerState !== RunnerState.Finished
  ) {
    log("cannot execute runner if runner is not started!", true);
    return;
  }

  state!.ws!.onmessage = (e) => {
    const packet = JSON.parse(e.data);
    console.log(packet);
    switch (packet.type) {
      case "ProcessEvent":
        if (packet.data === "Started") {
          log(`runner ${state!.runnerId} is starting execution`);
        } else if (packet.data === "TimedOut") {
          log("runner timed out!", true);
        } else if (packet.data.Finished) {
          log(
            `runner finished execution with exit code \`${packet.data.Finished.exit_code}\``,
          );
          state!.runnerState = RunnerState.Finished;
        }
        break;
      case "ProcessOutput":
        if (packet.data.StdOut) {
          packet.data.StdOut.split("\n").forEach((o: string) => log(o));
        } else if (packet.data.StdErr) {
          packet.data.StdErr.split("\n").forEach((o: string) => log(o, true));
        }
        break;
      default:
        log(`unhandled packet type: ${packet.type}!`, true);
    }
  };

  const execPacket = {
    type: "ProcessControlSignal",
    data: {
      container_id: state!.runnerId,
      control_signal: { Exec: "main.py" },
    },
  };

  state!.ws?.send(JSON.stringify(execPacket));

  log(`requested execution`);
  log(`---`);
}

export async function stop(
  signal: string,
  state: AppState | undefined,
  log: Log,
) {
  const controlSignal = signal === "SIGINT" ? "Interrupt" : "Terminate";

  state!.ws?.send(
    JSON.stringify({
      type: "ProcessControlSignal",
      data: {
        container_id: state?.runnerId,
        control_signal: controlSignal,
      },
    }),
  );

  log(`sent ${signal} to runner`);
}
