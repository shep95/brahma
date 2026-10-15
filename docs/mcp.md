# MCP conventions

brahma follows the same conventions as #houseofasher #28. start a headless session:

```sh
brahma-cli mcp --automation-read-root /work/project --automation-write-root /work/project
```

paths are relative to the corresponding root, or absolute paths beneath it; without a root that
filesystem authority is absent. bridge mode uses `--bridge` and `--control-token-file` as
described in [development.md](development.md). ui tools require a running desktop app.

## tools and resources

- `command_list`: discover engine ids, parameter docs and current enabled state; accepts
  `filter` and `enabled_only`. returns a bare JSON array of
  `{id, label, menu, shortcut, params, enabled, disabledReason, needsDocument}` objects (see
  [command list replies](control-protocol.md#command-list-replies)).
- `command_run`: `id`, optional `params` and `wait`. command parameters remain engine-defined.
- `command_batch`: ordered `steps`, optional `stop_on_error` (default true); stops at the first
  error unless false. not an atomic transaction: earlier successful steps stay applied, with
  no rollback. existing per-step results and reply budgets are preserved.
- `doc_inspect`: layer tree, history and selection; an empty session returns `document:null`.
  every layer (including groups and nested children) reports raster-only `hasMask`,
  `hasVectorMask` for vector mask presence, and `vectorMaskEnabled` (`true`/`false` when
  present, `null` when absent). a disabled vector mask still has `hasVectorMask:true`.
- `render_preview`: bounded PNG content; `index` and `max_side` (default 1024, maximum 2048).
- `ui_inspect` and `ui_screenshot`: live app state/window in bridge mode.

existing document/session/job/ui tools remain listed, including the documented
`doc_render_preview` spelling. there are no hidden legacy aliases. every tool has a title
and all four annotation hints. generic command/control tools conservatively advertise writes;
file saves may replace existing targets. hints do not grant permission.

after a failed batch, `completed` and per-step `results` report successful steps, not a
rolled-back transaction. inspect the document and its history with `doc_inspect`; commands
do not necessarily add one history entry each. for undoable edits, use `command_run` with
`{"id":"edit.undo","params":{}}` once per undoable history entry added, inspecting history
as you recover. undo does not roll back files written to disk. if a reply-budget error says
the current step may have completed, inspect its effects before retrying or undoing it.

before experiments, use `command_run` with `image.duplicate` and `{}` params, or
`history.newDocument` and `{}` params (copies the current state by default). both create a
new active document, leaving the source unchanged; run experimental edits on that copy.
the optional `state` parameter of `history.newDocument` selects a retained historical state.

unknown tool arguments return JSON-RPC `-32602`, naming the key, before execution. batch
step envelopes are strict too. nested engine `params` retain their command-specific handling.
tool execution failures return `isError:true`. a worker panic becomes a tool error and the
existing poisoned-session recovery keeps serving. invalid JSON lines receive `-32700` with
`id:null`; the next line is still processed.

embedded smart objects support `command_run` with `layer.smartObjects.editContents` and
`layer.smartObjects.convertToLayers`, optionally targeting `params.layer`. use
`layer.smartObjects.saveContents` in an opened contents document to update its parent in memory;
save nested contents from the inside outward, then `doc_save` the parent to a scoped path.
`doc_save` on a contents document exports that document rather than updating its parent.
no filesystem root is needed for the in-memory commands. external linked sources that require
a file read remain refused. see the [control policy](control-protocol.md#engine-commands)
for the shared restrictions.

`brahma://document` and `brahma://commands` return live JSON matching `doc_inspect`
and `command_list`. resource reads are uncached; tool/resource catalogs are private and cached
for ten minutes. list/read responses include the MCP 2026-07-28 result/cache fields.

## progress and cancellation

a direct headless `command_run` of `file.export.renderVideo` reports frame counts when
`_meta.progressToken` is present (a string or number). notifications increase strictly,
are throttled to at most ten per second except completion, and include `total`. no token
means no progress notifications. ping remains responsive while rendering.

```json
{"jsonrpc":"2.0","id":20,"method":"tools/call","params":{"name":"command_run","arguments":{"id":"file.export.renderVideo","params":{"dir":"frames","format":"png"}},"_meta":{"progressToken":"export-20"}}}
{"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":20}}
```

create a document and timeline first (`doc_new`, then `timeline.create`). export uses the
configured write root and creates `dir` beneath it. filenames must be single components;
absolute paths, traversal and escaping symlinks are rejected. existing target filenames
are refused, including symlinks, so a cancelled or failed job cannot replace an earlier export.
only files actually created by this job are removed on cancellation, error or an escaped panic;
unrelated files and the destination directory remain. PNG sequences and animated GIF share
the engine renderer. the document and playhead are unchanged.

cancellation suppresses the request response and stops at a frame boundary (the current
frame/encoding may finish first). unknown request ids are ignored. render video stays a
synchronous command even with `wait:false`; other job-capable commands keep their existing
`jobs_list`/`jobs_cancel` behavior. batch exports do not emit MCP progress or support per-step
MCP cancellation. nested engine actions and bridge render video remain denied by the existing
ambient-filesystem policy; the new scoped writer is available for direct headless commands.
