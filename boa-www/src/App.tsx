export function App() {
  return (
    <div className="App flex flex-col gap-2 p-8">
      <h1>boa-www</h1>

      <div className="flex flex-row gap-2">
        <button className="bg-neutral-800 p-2 rounded-md border-neutral-700 border">
          Connect
        </button>
        <button className="bg-neutral-800 p-2 rounded-md border-neutral-700 border">
          Create Runner
        </button>
        <button className="bg-neutral-800 p-2 rounded-md border-neutral-700 border">
          Start Runner
        </button>
        <button className="bg-neutral-800 p-2 rounded-md border-neutral-700 border">
          Upload Code
        </button>
        <button className="bg-neutral-800 p-2 rounded-md border-neutral-700 border">
          Execute Code
        </button>
        <button className="bg-neutral-800 p-2 rounded-md border-neutral-700 border">
          Stop Runner (SIGINT)
        </button>
        <button className="bg-neutral-800 p-2 rounded-md border-neutral-700 border">
          Stop Runner (SIGTERM)
        </button>
      </div>
    </div>
  );
}
