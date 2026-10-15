# crates and dependency layers

the layer numbers below describe allowed dependency direction, not runtime privilege. a crate may depend only on lower layers. standalone format crates remain independent of other workspace crates where specified.

| layer | crates | responsibility |
|---|---|---|
| l0 | `geom`, `cms`, `color`, `raster` | geometry, ICC processing, color/blend math, pixel formats, and copy-on-write tiles |
| l0 standalone | `psd`, `codecs`, `affinity` | PSD/PSB and flat raster format parsing/encoding; bounded affinity preview extraction |
| l1 | `doc` | pure-data documents, layers, masks, effects, adjustments, text, vectors, and smart objects |
| l2 | `ops`, `paint`, `algo`, `text`, `vector` | history operations, painting, imaging algorithms, type, paths, and shapes |
| l3 | `compose`, `gpu`, `format` | CPU composition, wgpu composition, and `.pcraft` persistence |
| l4 | `io` | import/export between the document model and external formats |
| l5 | `engine` | `Session`, command registry, validation, history integration, and inspection |
| l6 | `ui-egui`, `automation` | egui shell, headless automation, MCP, and the live bridge |
| test support | `testkit` | shared test helpers |

workspace applications are:

| application | path | purpose |
|---|---|---|
| brahma desktop | `apps/brahma` | eframe/wgpu application and loopback control server |
| brahma CLI | `apps/brahma-cli` | convert, inspect, run, batch, JSON-lines serve, and MCP |
| brahma web | `apps/brahma-web` | the ui and engine compiled for WebAssembly |

new crates must be registered in `xtask/src/layers.rs`. nothing below `ui-egui` may depend on egui, eframe, winit, or rfd. the `psd`, `codecs`, and `cms` crates have additional independence requirements documented in [`AGENTS.md`](https://github.com/shep95/brahma/blob/main/AGENTS.md).
