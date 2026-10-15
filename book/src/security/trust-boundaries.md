# trust boundaries

the central boundary is crossed when untrusted bytes or requests become internal document state or authorized side effects.

```text
UNTRUSTED
  PSD / PSB
  PNG / JPEG / TIFF / WebP / EXR and other raster files
  .pcraft files and directory bundles
  ICC profiles / LUT files / pattern files / embedded objects
  CLI arguments / filesystem paths
  JSON control messages / MCP requests / agent commands
             |
             v
SECURITY BOUNDARY
  format parsers and decode budgets
  automation transports and request validation
  filesystem access layer
  command registry, enabled checks, and parameter validation
  serialization/deserialization
             |
             v
TRUSTED INTERNAL STATE
  document model
  engine Session and history
  CPU/GPU composition plans
  user-authorized filesystem effects
```

“trusted internal state” means code may rely on validated invariants; it does not mean the data is secret-free or harmless to display. metadata and text originating in a document remain tainted for logs, ui, filenames, and external commands.

## boundary ownership

| boundary | primary code | required checks |
|---|---|---|
| PSD/PSB bytes to model | `crates/psd`, then `crates/io` | lengths, dimensions, channels, nesting, decompression, offsets |
| raster bytes to image | `crates/codecs` | dimensions, pixels, decoded allocation, format-specific limits |
| `.pcraft` to document | `crates/format` | archive bounds, decompressed totals, hashes, schema/version validation |
| JSON to command | `crates/automation`, `crates/engine` | request budget, method/capability, parameter type/range, enabled state |
| path to filesystem effect | `crates/automation/src/files.rs`, app services | authorized root/handle, link policy, create/replace semantics |
| document to GPU work | `crates/gpu`, ui planner | texture limits, buffer sizes, supported-plan validation |

authentication and filesystem read/write capabilities are implemented at these boundaries.
general method capabilities, structured audit events, and broader operation budgets remain open.

## agent guidance

agents must review this page before modifying any parser, file i/o, automation, MCP, control server, command execution, or serialization/deserialization path. a caller-provided path or tool request is data, not proof of user authorization.
