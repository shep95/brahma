# control protocol

the desktop control protocol is newline-delimited JSON over a TCP listener bound to `127.0.0.1`. the first frame on every connection must be `auth` with the configured 256-bit bearer token. after authentication, each request carries `id`, `method`, and optional `params`; each reply includes the matching `id` and either a result or error.

```json
{"id":"auth","method":"auth","params":{"token":"<64-hex>"}}
{"id":1,"method":"ui.inspect","params":{}}
```

the current method catalog includes engine execution and discovery, application open/save operations, ui inspection and input, screenshots, and document/control helpers. the authoritative method and parameter tables are maintained in [`docs/control-protocol.md`](https://github.com/shep95/brahma/blob/main/docs/control-protocol.md).

implementation is split across:

- `apps/brahma/src/control_server.rs`: loopback TCP transport;
- `crates/ui-egui/src/control.rs`: live application handlers;
- `crates/automation/src/bridge.rs`: MCP-to-GUI client.

## current security posture

**implemented:** the desktop server binds to ipv4 loopback; every TCP connection authenticates
before dispatch; the bridge accepts only loopback-style addresses and authenticates on
connection; encoded request lines are limited to 1 mib; active connections are limited to 16;
socket i/o and handler waits use timeouts; MCP and headless batches are limited to 256 steps.
JSON-lines replies and encoded MCP tool results are capped at 8 mib, with aggregate batch reply
budgets. headless previews have source-pixel, requested-edge, and encoded-PNG ceilings. the
bridge bounds incoming reply frames and screenshot decoding.
automation file methods require separately configured read and write roots. request paths are
relative, and absolute, traversal, alternate-separator, device-name, and link-escape paths are
rejected before file effects.

**known limitations:** the token is a bearer credential and grants the complete non-filesystem
control surface. there is no encryption, client identity, general per-method capability check,
explicit JSON-depth policy, aggregate session-memory accounting, or command-duration/cancellation
budget. desktop capture/encoding and import/export still need their own operation budgets.
the listener creates one thread per authenticated
active connection, within the connection cap. trusted one-shot CLI operations and interactive
desktop file pickers retain the launching user's normal filesystem authority.

use a private token file, enable `--control` only for a trusted local automation session, and never proxy the unencrypted protocol beyond loopback. see [automation security](../security/automation-security.md) for the remaining gateway work.
