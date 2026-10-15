# command-line interface

`apps/brahma-cli` provides the following current subcommands:

| command | purpose |
|---|---|
| `convert` | open one document and export it to another format |
| `info` | print document size, mode, depth, and layer information as JSON |
| `run` | open or create a document, execute engine commands, and optionally save |
| `batch` | apply an action list to files in one input directory |
| `droplet` | run a brahma droplet against files or folders |
| `commands` | list command ids and parameter documentation |
| `mcp` | start the MCP server on stdio, optionally bridged to the desktop app |
| `serve` | start the JSON-lines headless server on stdio or loopback TCP |

use `brahma-cli --help` (or `brahma-cli <subcommand> --help`) as the executable source of truth. a typical bounded command run is:

```sh
brahma-cli run input.psd \
  --cmd image.adjustments.invert \
  --out output.png
```

### missing font warnings

if a type layer uses a font family that is not installed, `convert` and `run`
write a warning to standard error before exporting; they do not silently present
the fallback rendering as an exact match. `info --compact` includes the same
warning in its `warnings` JSON array, without writing it to stderr. `run`
also adds `warnings` to the JSON result for the command that first exposes
the missing family. a multi-command `run` reports each missing family once,
even if later commands still use it.

warnings do not change the saved document's font declarations and do not stop
the operation. to list or replace the missing families intentionally, use
`type.resolveMissingFonts` on the document.

### export selected adjustment layers as a .cube LUT

the engine command `file.export.colorLookupTables` creates a 3d LUT from an
identity RGB lattice. it can export all visible top-level adjustment layers
(the default), the current layers panel selection, or a supplied list of
adjustment layer ids. the `scope` and `layers` parameters are mutually
exclusive when `scope` is `selected`:

```json
{"size": 33, "scope": "selected", "path": "chosen-look.cube"}
```

for automation that does not use the layers panel, supply `"layers": [10, 14]`
instead. input id order does not change the final color look: adjustments are
always baked in **document stacking order**, bottom to top. when `path` is
omitted, the command returns the .cube text in `cube`; both forms return
`size` and `layerCount`.

valid LUT dimensions are 2–129 (same maximum as brahma's LUT reader).
selected ids must exist, be visible top-level adjustment layers, and contain
no duplicates; invalid selections report an error rather than silently
exporting a different look. masks, vector masks and clipping are spatial
operations and cannot be represented faithfully by a color-only LUT, so the
baker omits them. group-contained adjustments are not exported as independent
layers because that would lose their group's blending semantics.

paths supplied to the current CLI are ordinary operating-system paths. the CLI does not restrict them to a workspace root. batch output creates the requested output directory, and save operations may create parent directories. `batch` refuses an output directory that is its input directory (the results would replace the originals) unless `--in-place` is given. run untrusted action files only in an appropriately isolated account or environment.

CLI parsing reports malformed flags and JSON as errors, and a flag the subcommand doesn't take (a typo such as `--fromat`) is a usage error (exit status 2) rather than being ignored. it is not a security policy layer. engine commands remain responsible for validating their own parameters.
