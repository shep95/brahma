# architecture overview

brahma separates the document engine from presentation and transport. the desktop ui, CLI, JSON control channel, and MCP server reach editing behavior through the command registry rather than maintaining separate implementations.

```text
desktop UI       CLI       JSON control       MCP
    |             |              |              |
    +-------------+--------------+--------------+
                          |
                   command registry
                          |
                    engine Session
                          |
       document model + operations + compositors
                          |
             PSD / raster / .pcraft I/O
```

the enforced dependency direction runs from format-independent foundation crates through the document and operation layers to composition, i/o, the engine, and finally ui/automation. `cargo xtask layers` checks this rule.

the complete design narrative and dependency diagram remain in [`docs/architecture.md`](https://github.com/shep95/brahma/blob/main/docs/architecture.md). that document contains historical design material as well as implemented architecture; confirm behavior in current source when security or compatibility depends on it.

## core properties

- `brahma-doc` holds the pure-data document model.
- `brahma-engine::Session` owns open documents, active state, history, and command dispatch.
- `brahma-compose` is the CPU reference compositor; `brahma-gpu` accelerates supported interactive paths.
- `brahma-io` maps between documents and PSD or flat formats.
- `brahma-format` reads and writes the native `.pcraft` bundle.
- `brahma-ui-egui` is a thin shell over engine commands.
- `brahma-automation` exposes the headless engine and the live-GUI bridge.

see [crates and dependency layers](crates.md) for the complete workspace map.
