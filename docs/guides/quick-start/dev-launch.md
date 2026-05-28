# Dev Launch

Use this launch flow to start the local playable dashboard with one command.

## Start

```powershell
just dev
```

What it does:

1. Kills any existing `civ-watch` processes.
2. Starts `civ-watch` in the background and writes logs to `.process-compose/logs/civ-watch.log`.
3. Waits for `http://localhost:9090/snapshot` to respond.
4. Enters `web/dashboard`, runs `bun install` if `node_modules` is missing, then starts `bun run dev` in the background and writes logs to `.process-compose/logs/web.log`.
5. Prints `Game ready at http://localhost:5173`.

## Stop

```powershell
just dev-stop
```

This stops the background `civ-watch` and dashboard processes started by `just dev`.

## Logs

- `.process-compose/logs/civ-watch.log`
- `.process-compose/logs/web.log`

## Validation

Open the browser at `http://localhost:5173` after the command prints the ready message.
