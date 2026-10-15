# MCP

`brahma-automation` implements an MCP server over stdio with two backends:

- **headless:** an in-process `Headless` session owns the engine and performs file i/o.
- **bridge:** `BridgeClient` forwards tools to a desktop application started with `brahma --control <port>`.

start the headless server with:

```sh
brahma-cli mcp --automation-read-root /work/project --automation-write-root /work/project
```

bridge to a running desktop application with the same private token file used by the app:

```sh
brahma-cli mcp --bridge 127.0.0.1:7878 --control-token-file /private/path/control.token
```

tools cover session/document operations, command discovery and execution, batching, and—when bridged—ui inspection and control. tool schemas improve correctness but are not authorization boundaries.

## from an installed release

the release packages ship `brahma-cli` alongside the desktop app, so no build is needed:

| install | CLI |
|---|---|
| windows (MSI) | `C:\Program Files\brahma\brahma-cli.exe` by default (wherever you installed it otherwise), not on `PATH` |
| linux (deb, rpm) | `/usr/bin/brahma-cli` |
| macos | the separate `brahma-cli-<version>-macos-<arch>.zip` release asset (the `.app` holds only the desktop app) |

```sh
# Windows, default install folder
claude mcp add brahma -- "C:\Program Files\brahma\brahma-cli.exe" mcp --automation-read-root <dir> --automation-write-root <dir>
# Linux, or macOS with the CLI unzipped onto PATH
claude mcp add brahma -- brahma-cli mcp --automation-read-root <dir> --automation-write-root <dir>
```

## security notes

stdio MCP does not open a network listener. its client receives only the read and write roots
explicitly granted at launch; omitting a root omits that authority. bridge mode authenticates its
loopback control connection with a bearer token before invoking any control method and uses the
roots configured on the desktop process.

the current server does not issue scoped sessions or distinguish trusted tools from high-impact tools. authentication proves possession of the token; it does not restrict an authenticated client to document-only access. do not expose either transport through an untrusted broker, shell, proxy, or remote service without adding an external policy boundary.

filesystem capabilities are implemented. general tool and method capabilities remain
[proposed](../security/automation-security.md).

the five shared core tools are `command_list`, `command_run`, `command_batch`, `doc_inspect`
and `render_preview`. existing documented tools, including `doc_render_preview`, remain
listed. `brahma://document` and `brahma://commands` expose live JSON. direct headless
render video supports progress and cancellation using the granted write root; see
[`docs/mcp.md`](../../../docs/mcp.md) for arguments, cleanup and transport behavior.
