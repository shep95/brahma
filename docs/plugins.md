# plug-ins: the WebAssembly plug-in API (ABI v1)

brahma does not host native photoshop plug-ins (`.8BF`, CEP, UXP): that needs unsafe FFI into
arbitrary machine code and cannot work in the browser build. instead, plug-ins are **WebAssembly
modules** run in a sandbox. one `.wasm` file works on macos, windows, linux and the web.

this page is the contract for plug-in authors and the reference for the host
(`crates/plugins`, the `plugin.*` commands in `crates/engine/src/plugin_cmds.rs`).

## using plug-ins

| command | params | what it does |
|---|---|---|
| `plugin.install` | `{"path": "/x/y.wasm"}` (native) or `{"data": "<base64>"}`, `"replace": bool = true` | validates and installs a module; returns its description. |
| `plugin.list` | `{}` | `{"plugins": [{id, name, version, kind, author, description, params, paramsSchema, overlap, area, size, source}], "folder": report}` |
| `plugin.run` | `{"id": "…", "params": {…}}` (plug-in params may also be top-level keys) | runs a filter plug-in on the active pixel layer, targeted alpha channel / quick mask, or smart object. one undo step. |
| `plugin.remove` | `{"id": "…"}` | uninstalls. |
| `plugin.reload` | `{"path": "folder"}` (default: the preference folder) | loads every `*.wasm` in a folder; a bad module is reported and skipped. |

in the app, installed filters appear under **filter › plug-ins** (with a generated dialog and live
preview when they have parameters), next to **install plug-in…**.

**preference folder.** edit › preferences › plug-ins › *use additional plug-ins folder* plus
*additional plug-ins folder*: when both are set, every `*.wasm` in that folder (not recursive) is
installed at startup and whenever the preference changes (native builds only; the web build installs
from bytes).

plug-ins are installed **per process**, not per document. a smart filter recorded from a plug-in
(`{"command": "plugin.run", "params": {"id", "params"}}`) re-runs on re-render while that plug-in is
installed; if it is missing, the filter is skipped (the cached pixels stay as they were), like other
filters brahma does not implement.

## the sandbox

the host is `wasmi`, a pure-rust WebAssembly interpreter
(MIT or apache-2.0). it builds for `wasm32-unknown-unknown` too, so plug-ins also run in the web
build. limits (`brahma_plugins::Limits`, defaults):

| limit | default | on violation |
|---|---|---|
| host imports | none: no WASI, file system, network, clock or randomness | module rejected at install |
| module size | 32 mib | rejected |
| linear memory | 512 mib per instance | `memory.grow` returns -1; oversized initial memory fails |
| instructions (fuel) | 50 m per call + 4,000 per sample for `pc_filter` | `Limit("instruction budget")` |
| wall clock | 60 s per run (native; checked every 20 m instructions) | `Limit("time budget")` |
| call depth | 1,024 frames | trap |
| module shape | wasmi's strict limits (≤ 10,000 functions, 1 memory, …) | rejected |

every band runs in a **fresh instance**, so no state survives between calls, and parallel bands
never share memory. traps, exhausted budgets, malformed modules, bad manifests, out-of-range
pointers and non-zero return codes all become errors; the document is left untouched (the edit is
transactional). when a failure stops sibling bands, those bands retain the originating error
instead of replacing it with a generic cancellation message. output samples that are nan or
infinite keep their original value; integer-depth
documents and alpha are clamped to 0–1.

wasmi is built with its *portable* (loop) dispatcher: the default tail-call dispatcher relies on LLVM
sibling-call optimisation, and when that doesn't happen a long-running module overflows the host
stack, which aborts the process instead of returning an error.

## ABI v1

a plug-in is a core WebAssembly module (MVP + the usual post-MVP features: bulk memory, sign
extension, multi-value, saturating conversions; no SIMD, no memory64) with **no imports** and these
exports. all integers are `i32` unless noted; pointers are offsets into the module's memory.

| export | signature | meaning |
|---|---|---|
| `memory` | memory | the module's linear memory. |
| `pc_abi_version` | `() -> i32` | must return `1`. |
| `pc_manifest` | `() -> i64` | `(len << 32) \| ptr` of the UTF-8 manifest JSON (≤ 64 kib). |
| `pc_alloc` | `(size) -> ptr` | a block of `size` bytes the host may write, or `0` on failure. never freed: each band gets a fresh instance. |
| `pc_filter` | `(buf, buf_len, width, height, channels, format, params, params_len) -> i32` | filters the pixels in place; `0` = success, anything else is an error code. |

### manifest

```json
{
  "id": "org.example.tone",
  "name": "Tone…",
  "version": "1.0.0",
  "kind": "filter",
  "author": "Example",
  "description": "What it does.",
  "params": {
    "amount": {"type": "number", "min": 0, "max": 100, "default": 50},
    "steps":  {"type": "int", "min": 1, "max": 16, "default": 4},
    "mono":   {"type": "bool", "default": false},
    "mode":   {"type": "choice", "options": ["soft", "hard"], "default": "soft"}
  },
  "overlap": 0,
  "area": "content"
}
```

- `id`: 1–64 characters from `A-Z a-z 0-9 . _ -` (reverse-DNS recommended). installing a module
  with the same id replaces the old one.
- `name`: the menu label (1–64 printable characters). any script works, but control characters and
  unicode format characters (bidi overrides and marks, zero-width spaces, soft hyphens) are rejected,
  except the zero-width joiner and non-joiner, and at least one character must be visible.
  `kind`: `"filter"` (the only kind in v1).
- `params`: in dialog order. names are `[A-Za-z0-9_]`, not starting with `_`, not `id`/`layer`.
  the host validates the user's values before calling: missing values take the default, numbers are
  clamped to `min..max` (ints rounded), a wrong type or an unknown choice is an error.
- `overlap` (0–256): pixels of context the filter needs around each output pixel (a blur radius).
  the host pads each band by `overlap` on every side, repeating the edge pixels of the canvas (or of
  off-canvas layer content), and keeps only the inner rectangle of the result.
- `area`: `"content"` (default) filters the layer's non-transparent bounds grown by `overlap`;
  `"canvas"` filters the whole canvas, including transparent pixels (for generators).

### pixels

`buf` holds `width × height × channels` **little-endian `f32` samples**, interleaved, row-major,
normalised to 0–1 at every bit depth (32-bit documents may hold values above 1). the buffer is
`width × height` including the overlap margins. channels are the document's own colour model, with no
conversion:

`format` = `depth | alpha << 8 | mode << 16`

- `depth` (bits 0–7): the document's bit depth, 8, 16 or 32 (informational: samples are always `f32`).
- `alpha` (bit 8): 1 when the last channel is (straight, not premultiplied) alpha.
- `mode` (bits 16–23): the colour mode, numbered as in the PSD file format: 0 bitmap, 1 grayscale,
  2 indexed (stored as RGB), 3 RGB, 4 CMYK, 7 multichannel, 8 duotone (stored as gray), 9 lab.

alpha channels and quick mask targets arrive as one grayscale channel.

the host converts the layer into bands (tile-aligned blocks of at most ~4 mib of `f32`), runs them in
parallel on native builds, writes the results back in the layer's format, then blends with the
selection: `out = original + (filtered − original) × coverage`, exactly as built-in filters do. a
plug-in never sees the selection and needs nothing special to support it.

### parameters

`params` points at `params_len` bytes of UTF-8 JSON: the validated parameters, plus `_image` with
the band's geometry:

```json
{"amount": 50, "mode": "soft",
 "_image": {"x": 0, "y": 256, "width": 1024, "height": 256, "overlap": 0,
            "canvasWidth": 6000, "canvasHeight": 4000,
            "mode": "rgb", "depth": 8, "alpha": true, "channels": 4}}
```

`x, y` is the document position of the buffer's top-left sample (including the margins), so
position-dependent filters (gradients, vignettes) are seamless across bands.

## writing a plug-in in rust

[`examples/plugins/invert-rs`](../examples/plugins/invert-rs) is a complete `no_std` plug-in
(no allocator, under 1 kib built). build it with:

```sh
rustup target add wasm32-unknown-unknown
examples/plugins/invert-rs/build.sh      # cargo build --release --target wasm32-unknown-unknown
```

the script also refreshes `crates/plugins/tests/fixtures/invert.wasm`, the test fixture built from
that source. any language that targets core WebAssembly works (c, zig, AssemblyScript, WAT…), as
long as the module has no imports: in rust, that means `no_std` or a `std` build that never touches
WASI.

## performance

the interpreter costs roughly 10–100× native speed per instruction, so plug-ins suit per-pixel and
small-neighbourhood filters. because bands run in parallel, the example invert on a 6000 × 4000
RGBA 8-bit layer (24 mp, release, 14-core apple silicon under heavy load) took 1.1–1.6 s through
`plugin.run`, versus 3.6–5.1 s for the (single-threaded) built-in image › adjustments › invert
(`cargo run --release -p brahma-engine --example bench_plugin_invert`; plug-in alone:
`cargo run --release -p brahma-plugins --example bench_invert`). peak extra memory is about three bands per worker thread (input,
original for the selection blend, and the instance's memory), i.e. ~12 mib per thread.

## not in v1

- panel / ui plug-ins, plug-ins that read other layers or documents, host callbacks (progress,
  cancel), plug-in-provided file formats.
- filter › last filter does not repeat plug-in runs yet.
