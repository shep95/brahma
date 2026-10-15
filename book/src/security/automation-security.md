# automation security

automation is a privilege boundary because requests can cause filesystem access, application control, ui input, preferences changes, rendering, and command execution.

## current implementation

- desktop control binds to `127.0.0.1` and uses one JSON request/reply per line.
- headless TCP refuses a successfully bound non-loopback address.
- every desktop-control and headless-TCP connection must authenticate with a 256-bit bearer token before method dispatch.
- encoded request lines are limited to 1 mib on desktop and headless TCP, the headless
  JSON-lines stdio server (`brahma-cli serve`), and MCP stdio. the MCP line limit is
  enforced before rmcp's line buffer; an oversized MCP line closes that stdio session because
  its request id cannot safely be parsed. active TCP connections are limited to 16, and
  headless/MCP batches to 256 steps.
- capability-scoped automation file opens read in chunks without reserving from the file's
  metadata length. a read rejects data beyond the length observed on the opened file, so growth
  after the metadata check cannot expand the read. large PSB files have no fixed automation file
  size ceiling; their materialized bytes and decoded document still require available memory.
- JSON-lines replies and encoded MCP tool results are limited to 8 mib; retained batch replies
  have an aggregate budget and stop later steps when exhausted. the headless stdio JSON-lines
  transport enforces the same request/reply ceilings as TCP.
- headless previews reject requested edges above 2048 pixels and source documents above
  67,108,864 pixels before compositing. full-size requests must fit the edge ceiling. rendered
  pngs are limited to 5 mib before base64 encoding or file writes. trusted-local CLI rendering
  retains its existing behavior.
- the MCP bridge bounds incoming replies and does not retry an operation after an oversized
  reply. edits are never automatically resent after a transport failure (#1007); the server's
  pure-read tools opt into a single retry on a fresh connection, which applies nothing.
  screenshots are decoded under separate dimension, pixel, and allocation ceilings before
  downscaling; the 5 mib PNG limit applies to the returned image.
- MCP normally uses stdio and can optionally bridge to loopback control TCP.
- the bridge and ui request paths use timeouts.
- engine commands are expected to reject invalid parameters without panicking.
- remote filesystem methods use separately granted read and write directory capabilities.
- a save without a `path` (MCP `doc_save`, `serve` `doc.save`, the control channel's `app.save`)
  writes back only to the document's own PSD, PSB or `.pcraft` file in its own format; any other
  save needs an explicit `path`, so automation never flattens or converts over the file it opened.
- automation paths must be relative beneath the applicable root. absolute paths, parent
  traversal, alternate separators, windows device names, malformed components, and link escapes
  are rejected before file effects.
- engine commands that still use ambient paths fail closed at the automation boundary. synthetic
  ui input reaches the same command policy, and automation-triggered file hooks do not run
  user-configured script-event paths. the policy also covers every command a command runs on its
  own behalf (an action's steps, `file.automate.conditionalModeChange` running `image.mode.*`), so
  an allowed command can't reach a denied one.
- `prefs.set` applies the engine's dotted-path normalization before checking for filesystem-bearing
  preference sections, including paths with leading or repeated dots. it refuses whole-preferences
  updates and those sections; individual safe preference keys remain available.
- applying the preferences dialog with `ui.dialog.apply` is deliberately refused over automation.

## known limitations

- no per-client or per-tool capabilities;
- no general per-client or per-tool capability model beyond filesystem read/write authority;
- no automated bulk apply of the preferences dialog; agents can change individual safe keys with
  `prefs.set`;
- no application-specific JSON-depth policy (serde_json's parser has its default finite recursion
  limit), aggregate document/session-memory accounting, compositor scratch-space accounting,
  or command-duration/cancellation budget;
- one thread per accepted TCP connection;
- no structured security audit event stream;
- desktop screenshot capture/encoding and document import/export have no automation-specific
  operation budgets; the desktop transport limits replies after their creation;
- expensive engine commands are not charged to a session budget.

reply rejection can happen after a command has changed state. such errors report that the
operation may have completed. exhausting a batch reply budget stops subsequent steps, including
when normal command errors would allow the batch to continue; it does not roll back prior edits.
see [control protocol](../automation/control-protocol.md) for the authoritative protocol reference.

## proposed session model

the transport now establishes an authenticated connection from a cryptographically random credential before method discovery. a future security gateway should turn that authenticated connection into a short-lived, capability-scoped session.

remaining capabilities should be explicit and composable, for example:

```text
DocumentRead
DocumentWrite
UiInspect
UiControl
FilesystemRead (implemented as a root capability)
FilesystemWrite (implemented as a separate root capability)
PreferencesWrite
ApplicationControl
```

names and granularity remain a design proposal. the important property is that a client granted document inspection is not implicitly granted arbitrary file writes, pointer injection, preferences changes, or application shutdown.

## proposed request budgets

the gateway should enforce and test finite values for:

- request bytes and JSON nesting;
- batch steps and nested/recursive calls;
- concurrent connections and per-client in-flight requests;
- render dimensions and returned payload bytes;
- document pixels, decoded bytes, and aggregate session memory;
- command duration or cancellation where the engine supports it.

limits should fail with stable errors, be applied before expensive work, and be configurable only inside safe global ceilings. audit events should capture session identity, capability decision, method/command id, outcome, duration, and redacted path handle—not file contents, tokens, or secrets.

## deployment guidance today

prefer stdio, start control TCP only when needed, grant the narrowest read and write roots, use a
private token file, keep the listener on loopback, use a least-privileged os account, and isolate
untrusted agents from sensitive files. do not commit or log bearer tokens. do not tunnel or proxy
the unencrypted loopback protocol to another host.
