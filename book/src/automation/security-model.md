# automation security model

automation requests are untrusted input with potential effects beyond the active document. opening, saving, rendering to a path, preferences changes, application control, pointer/type injection, and arbitrary engine commands have different risk levels but currently share broad transport access.

| control | current status |
|---|---|
| loopback-only TCP binding | **implemented** for desktop control and headless TCP serve |
| stdio transport | **implemented** for MCP and headless JSON lines |
| command-level error handling | **implemented**, with command-specific tests and `panic_hunt` coverage |
| control-channel authentication | **implemented:** 256-bit bearer token required before TCP method dispatch |
| MCP capability scopes | **partial:** filesystem read/write capabilities only; general tool scopes are absent |
| allowed read/write roots | **implemented:** separate launch-time roots, with absent authority failing closed |
| symlink-safe capability filesystem | **implemented:** relative operations use held directory capabilities and reject link escapes |
| request-byte and JSON-depth limits | **partial:** 1 mib request-line limit on TCP and headless JSON-lines stdio (MCP stdio input is not byte-capped); no explicit JSON-depth policy |
| batch-step limit | **implemented:** 256 steps for headless and MCP batches |
| reply-size limit | **implemented:** 8 mib encoded JSON/MCP result; aggregate batch reply budget |
| headless preview budgets | **implemented:** 2048-pixel requested edge, 67,108,864 source pixels, 5 mib encoded PNG |
| connection/worker limit | **implemented:** 16 active TCP connections; one worker thread per accepted active connection |
| security audit events | **proposed** |

## gateway status and remaining work

```text
MCP or control client
          |
 authenticated session
          v
 security gateway
   - general method capabilities (proposed)
   - path handles/read-write roots (implemented)
   - request budgets
   - command policy
   - audit events
          |
          v
 command engine / UI shell
```

the current token proves possession of a secret but does not provide general method authorization.
filesystem handles and a defensive command-path policy are implemented; explicit non-filesystem
capabilities, session-memory/command-duration budgets, and audit events remain. preview and
reply ceilings do not bound all document operations or desktop screenshot capture. private token handling, process
isolation, and least-privileged execution are still practical containment measures.
