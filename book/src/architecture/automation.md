# automation architecture

brahma exposes the same engine through several transports:

```text
brahma-cli commands/run/batch
             |
MCP stdio ---+--> brahma-automation::Headless --> engine Session

MCP bridge --> BridgeClient --> 127.0.0.1 control server --> UI ControlRequest

JSON-lines stdio/TCP --> rpc::respond --> Headless::handle --> engine Session
```

headless file operations are implemented in `crates/automation/src/files.rs`; RPC dispatch is in `crates/automation/src/rpc.rs`; MCP tools are registered in `crates/automation/src/server.rs`; and the live client is in `crates/automation/src/bridge.rs`. the desktop TCP listener is `apps/brahma/src/control_server.rs`.

the control and headless TCP listeners are restricted to loopback addresses. stdio MCP avoids a listening network socket, but the MCP process still acts with the filesystem permissions of the user who launched it.

loopback is not authentication. TCP transports require a bearer token and enforce request-byte,
batch-step, and active-connection limits. remote file methods use an `AuthorizedWorkspace` with
independent read and write directory capabilities; paths are relative to a configured root and
fail closed when the applicable root is absent. general per-method scopes, client identity,
encryption, and operation budgets are not implemented. see the
[automation security model](../automation/security-model.md) and
[automation security](../security/automation-security.md) before exposing or extending these
surfaces.
