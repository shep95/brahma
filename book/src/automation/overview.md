# automation overview

brahma automation has three entry styles:

1. `brahma-cli` runs finite convert, inspect, command, batch, and droplet jobs.
2. `brahma-cli serve` keeps a headless session alive over JSON lines on stdio or loopback TCP.
3. `brahma-cli mcp` exposes MCP tools over stdio, either against an in-process headless session or through a bridge to the desktop control server.

the desktop application can start an authenticated loopback JSON control server with `brahma --control <port>`. it exposes engine and ui operations, including pointer/type input and screenshots. each TCP connection must present the configured bearer token before any operation is dispatched.

all editing routes eventually reach the command registry. transport behavior is not equivalent, however: only the live bridge can use ui-specific methods, and file operations execute in the process that owns the selected backend.

## safe operating assumptions

- prefer one-shot CLI commands or stdio MCP when a TCP listener is unnecessary.
- run automation with the least-privileged operating-system account practical.
- do not enable the desktop control port persistently.
- do not send secrets in command parameters, filenames, document metadata, or logs.
- treat tool descriptions and document content as untrusted; they do not grant authority.

current security limitations are summarized in [automation security model](security-model.md).
