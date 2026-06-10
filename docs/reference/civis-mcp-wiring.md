# Wiring the Civis MCP server

The `civis-mcp` crate (crates/civis-mcp) is a stdio MCP server exposing the Civis
verify/build/screenshot/census operations as native MCP tools, so agents (and
Claude) verify the game without the fragile shell-autoshot dance.

## Build it

```pwsh
$env:CARGO_TARGET_DIR = 'E:/civis-axdx-target'
cargo build -p civis-mcp --release
# binary -> E:/civis-axdx-target/release/civis-mcp.exe
```

## Wire it into settings.json

Add a `civis` entry alongside the existing servers (e.g. `dinoforge-http`) under
`mcpServers`. Do NOT replace the whole block — merge this key in:

```jsonc
{
  "mcpServers": {
    "civis": {
      "type": "stdio",
      "command": "E:/civis-axdx-target/release/civis-mcp.exe",
      "args": [],
      "env": {
        "CIVIS_REPO": "C:/Users/koosh/Dev/civis-game"
      }
    }
    // ... keep dinoforge-http and any other existing servers here ...
  }
}
```

Notes:
- stdio transport: the server speaks MCP on stdout, logs to stderr only.
- After editing settings.json, restart the Claude Code session to load the server.

## Tools exposed

| Tool | Params | Returns |
|------|--------|---------|
| `civis_build` | `target_dir?` | exit code + build log path |
| `civis_screenshot` | `out` | screenshot path + bytes |
| `civis_census` | — | parsed world census (dims, non_air%, chunks, seed) |
| `civis_verify` | `out?`, `target_dir?` | screenshot + panicked flag + panic tail + census |
| `civis_spawn` | `kind, x, y, z` | STUB (not yet wired to a running instance) |
| `civis_brush_paint` | `material, x, y, z, radius` | STUB (not yet wired) |

The first four wrap the `civis-cli` logic directly. The last two are honest stubs
pending a control channel into a running civ-standalone instance.
