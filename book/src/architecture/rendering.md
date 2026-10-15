# rendering

brahma has two compositors with distinct roles:

- `brahma-compose` is the CPU compositor and correctness oracle used by export and tests.
- `brahma-gpu` plans and executes supported canvas composition through wgpu.

the interactive canvas uses the GPU path when the planner can represent the document. unsupported cases fall back to CPU composition. current documentation identifies multichannel documents and regions above the GPU texture limit as fallback cases.

pixel storage in `brahma-raster` supports runtime sample depth rather than a public 8-bit-only path. color mode and profiles are document data; color conversion belongs in `brahma-cms` rather than being treated as implicit srgb.

## verification

GPU changes require parity checks against the CPU oracle. ui changes require an actual rendered image, using either the offscreen snapshot example or a control-channel screenshot:

```sh
cargo run -p brahma-ui-egui --example snapshot
cargo test -p brahma-gpu
```

performance-sensitive work should avoid full-surface work per frame, operate per tile, skip empty tiles, and cache by revision. the current commands and benchmark entry points are listed in [`docs/development.md`](https://github.com/shep95/brahma/blob/main/docs/development.md).

GPU processing is not a security sandbox. dimensions, decoded buffers, shaders, and command parameters must be validated before resource creation or dispatch.
