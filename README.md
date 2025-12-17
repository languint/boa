# boa

A lightweight containerized python code runner.

## Running

This requires the Rust programming language, cargo, and nodejs to be installed and present in the `PATH`.

First, clone the git repository
```sh
git clone https://github.com/languint/boa.git && cd boa
```

Then, set the boa environment variables

```sh
export BOA_SERVER_PORT=4040
export BOA_CONTAINER_PREFIX="boa-runner"
```

Next, run the server with
```sh
cargo run --
```

and run the frontend with

```sh
cd boa-www && npm install && npm run dev
```

### Alternative

If you have bash installed in your terminal, you can run the apps with

```sh
./dev-server.sh
```

and 

```sh
./dev-web.sh
```

Navigate to [localhost:5173](http://localhost:5173), and write your code in the editor.

## Managing runners

After you have the frontend and server instances running, you can click the connect button to open a connection to the server.

Then you can click the start button to request a new hosted code runner.

Next, hit the upload button when you are done writing code.

Then you should be able to hit the execute button to run your code.

Then disconnect when you are finished.
