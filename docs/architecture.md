# brahma architecture

> **last reviewed:** 2026-10-10 · **last updated:** 2026-10-13 · **change:** minor (workspace size and test counts recounted; current-state summary added; the design plan below is kept, with its deviations listed) · **target:** adobe photoshop 2026

## current state (2026-10-10)

what exists on `main` today. the numbered sections below are the original design plan of
2026-09-30; where the code differs, this summary wins.

- **workspace:** 25 library crates under `crates/`, 3 apps (`brahma` desktop, `brahma-cli`,
  `brahma-web`), `xtask`, an example plug-in and 5 fuzz crates; ~539,000 lines of rust in
  ~1,310 files; ~8,040 `#[test]` functions (ui-egui 2,809, engine 1,796, algo 572, io 539,
  codecs 426, psd 341), counted 2026-10-13. crate map and layers: [`AGENTS.md`](../AGENTS.md) §2 and
  `xtask/src/layers.rs` (enforced by `cargo xtask layers`).
- **ui toolkit:** decided: egui/eframe on wgpu (no webview, no JavaScript). the same rust builds
  to WebAssembly for the browser (trunk + wasm-bindgen).
- **engine:** ~700 commands registered in `crates/engine` (`CommandSpec` in `commands.rs`); the
  ui, CLI, JSON control channel and MCP server all dispatch by command id. 628 photoshop menu items
  map to them (`crates/ui-egui/src/menu_catalog.rs`).
- **rendering:** CPU compositor (`compose`, the oracle) and wgpu compositor (`gpu`, ≤ 1/255 of the
  CPU); colour through our own ICC engine (`cms`), display transform folded into a GPU 3d LUT.
- **files:** own PSD/PSB crate (`psd`), flat codecs (`codecs`), camera raw (`raw`), HEIF (`heif`),
  affinity (`affinity`), document mapping (`io`), native `.pcraft` (`format`, atomic saves).
  coverage: [`file-format-parity.md`](file-format-parity.md).
- **not built (yet) from the plan:** the `ml` crate (no ml runtime; see gap G15), plug-in ABI v2.
- **deviations from the plan below:** `brahma-tablet` is the one crate allowed `unsafe`
  (AppKit pen events); `affinity`, `heif` and `raw` crates were added; the phased roadmap in §15
  is superseded by [`roadmap.md`](roadmap.md).

---

# design plan (2026-09-30)

this plan sets out:
- the cargo workspace,
- the rules that keep the engine independent of the ui,
- the interfaces a ui has to implement,
- how the PSD, compositing, history and automation parts fit together.

---

## 1. goals and principles

**goals**
- a native, portable (macos / windows / linux), fully open-source layered image editor, roughly matching photoshop's feature surface over time.
- **the ui can be swapped** without touching the core: egui today, maybe slint, qt or something else later.
- **the core compiles to `wasm32`,** which keeps a browser demo possible.
- PSD read and write lives in its **own crate that can be published separately.**

**principles**

1. **engine-first, headless-first.** every feature is reachable without a GUI: from tests, the CLI and MCP. the GUI is one client of the engine.
2. **everything is a command.** each user-visible action has a stable `CommandId` (such as `filter.blur.gaussianBlur`) with typed, serializable parameters. the menu, command palette, shortcuts, recorded actions/macros, CLI, MCP and plugins all dispatch the same commands.
3. **data describes the ui; toolkits draw it.** tools produce *overlay primitives* (lines, handles, marching-ants paths) as data. filter and adjustment dialogs are generated from *parameter schemas*. a new toolkit implements one renderer for each, not about 150 bespoke dialogs.
4. **one algorithm, one parameter struct.** every algorithm has a CPU reference implementation (rust + rayon, deterministic). some also get a GPU implementation (WGSL). both read the same `#[repr(C)]` `bytemuck::Pod` parameter struct. we never keep hand-synced copies of an algorithm in several languages.
5. **immutable snapshots, copy-on-write tiles.** pixel data is stored in `Arc`-shared 256² tiles. taking a document snapshot is o(layers), which makes undo, background jobs, autosave and ui reads cheap and lock-free.
6. **determinism.** the same input gives the same output across CPU and GPU (within tolerance), across thread counts, and between preview and export. noise is hashed from document coordinates.
7. **pure rust by default.** c/c++ dependencies are allowed only behind cargo features, in i/o-edge crates (for example an optional LibRaw).
8. **clean-room.** we study other editors' *behaviour* only, and we must **not copy proprietary source** (rust, WGSL, c++ or js), even where it ships as readable source. specs come from public format docs (adobe PSD spec, ISO/ICC), academic papers (PatchMatch, poisson blending, ARAP) and observed behaviour. never paste code from another product.

### 1.1 avoiding GIMP's hole

the criticism of GIMP is historically true, and much of it still applies in 2026.

**bit depth.**
- GIMP's original core hard-coded 8-bit-per-channel tiles.
- high bit depth needed a whole new engine, GEGL. GEGL began around 2000, was introduced as optional in 2.6 (2008), and the port only finished in **GIMP 2.10 (2018)**, "the result of six years of work" on that port alone.
- only then did GIMP get 16/32-bit integer and float processing.

**non-destructive editing.**
- photoshop has had adjustment layers since about 1994.
- GIMP got non-destructive *filters* in **3.0 (march 2025)**, about seven years after 2.10 because of the GTK3 port.
- it got non-destructive *layer types* (link layers, vector layers) in **3.2 (march 2026)**.

**CMYK.**
- GIMP 3.x is still "a 100% RGB editor" internally.
- CMYK exists only as export (JPEG/TIFF/PSD) and soft-proofing.

**PSD.**
- import is still lossy. photoshop adjustment layers and most layer styles are not carried over faithfully, and text often needs re-typing.
- 3.2 converts legacy drop and inner shadows into GIMP filters, and added PSB export.

**root cause:** fundamental properties were **baked into the core as assumptions instead of being data.** pixel depth, colour model, and "a layer is a pixel buffer" were all hard-coded. every later feature that touched them needed a rewrite of everything built on top.

sources: GIMP 2.10 release notes, GEGL on wikipedia, GIMP 3.2 release notes, LWN on GIMP 3.0 colour.

**rules we adopt from day one** (cheap now, and each would force a rewrite later):

| property | rule | where |
|---|---|---|
| **bit depth** | every surface carries a runtime `PixelFormat { depth: U8\|U16\|F16\|F32, model, alpha }`. algorithms are written generically over a `Sample` trait and monomorphized per depth, or run on f32 working buffers and convert at tile edges. no `u8` pixel type appears in any public engine API. ci runs the golden suite at 8, 16 and 32f. | `raster`, `color`, `algo` |
| **colour model** | `ColorMode` (RGB, gray, CMYK, lab, indexed, bitmap, plus multichannel/spot) is in `doc` from the start. channel count is dynamic (1–n plus alpha plus spot). only RGB/gray *rendering* ships first, but the storage, PSD round-trip and conversion paths never assume 3 or 4 channels. | `doc`, `raster`, `psd` |
| **colour management** | every document has an ICC profile. the compositor takes a working-space descriptor. the display transform is a separate final stage. blending space (gamma vs linear) is a per-document option. | `color`, `compose` |
| **non-destructive by construction** | `LayerContent` is an enum that includes adjustment, fill, text, shape and smart from v1, with smart filters as a stack on any layer. the compositor evaluates a *graph*, not a list of flattened buffers. destructive ops are just "apply and bake". | `doc`, `compose` |
| **file formats** | formats plug in through `trait ImageDecoder/Encoder` plus a registry with capability flags (depths, modes, layers, alpha, metadata, ICC). adding a format is one crate feature. it must never need a change in core types. lossless PSD round-trip preserves unknown blocks. | `codecs`, `io`, `psd` |
| **size** | tiled, sparse storage behind a `TileStore` trait, with 64-bit coordinates for PSB-scale canvases (300k × 300k). nothing assumes a document fits in one allocation or one GPU texture. | `raster` |
| **ui toolkit** | the core never depends on the toolkit (§3–§4). GIMP spent years porting GTK2 to GTK3, and our ui crate can be replaced without touching the engine. | layering rule 2 |

the test is simple: **adding 64-bit float, a new colour model, or a new file format should touch a leaf crate, not the foundation.** we check this in phase 2 by adding a (hidden) CMYK and a 32f code path early, even before they are exposed in the ui.

---

## 2. workspace layout

```text
brahma/
├─ Cargo.toml                  # [workspace], shared deps, lints, profiles
├─ crates/
│  │  ── foundation (no_std-friendly where practical, wasm-safe) ──
│  ├─ geom/                    brahma-geom      points, rects, affine/perspective, tile coords, bezier (kurbo)
│  ├─ cms/                     brahma-cms       pure-Rust ICC colour management: profiles, transforms, intents, BPC, soft proof, 3D LUTs
│  ├─ color/                   brahma-color     pixel formats, color spaces, blend-mode math (scalar reference); conversions via cms
│  ├─ raster/                  brahma-raster    tiled COW surfaces, masks, mip pyramids, damage regions, pixel iterators
│  │  ── document ──
│  ├─ doc/                     brahma-doc       document model: layer tree, masks, effects, channels, paths, guides, metadata (pure data + serde)
│  ├─ ops/                     brahma-ops       reversible operations on a doc + transactions + history/undo
│  │  ── algorithms (CPU reference impls, rayon, wasm-safe) ──
│  ├─ algo/                    brahma-algo      adjustments, filters, selection, inpaint/heal, warp/transform, resampling
│  ├─ paint/                   brahma-paint     brush engine: dabs, dynamics, stroke smoothing, brush presets (.abr import later)
│  ├─ text/                    brahma-text      font DB trait, shaping + layout (parley/skrifa/harfrust), text-layer rasterization
│  ├─ vector/                  brahma-vector    shapes, paths, strokes → coverage (vello_cpu / kurbo)
│  │  ── rendering ──
│  ├─ compose/                 brahma-compose   layer tree → DrawOp plan; CPU compositor (reference + export + wasm fallback)
│  ├─ gpu/                     brahma-gpu       wgpu backend for the DrawOp plan + GPU kernels (WGSL), residency cache, memory budget
│  ├─ viewport/                brahma-viewport  camera (zoom/pan/rotate), visible tiles/mips, renders a doc into a wgpu texture; toolkit-agnostic
│  │  ── I/O ──
│  ├─ psd/                     brahma-psd       PSD/PSB read + write; its OWN format-level model; depends on nothing in this workspace
│  ├─ adobe-assets/            brahma-adobe-assets  .abr .asl .aco/.ase .grd .pat .csh .atn .cube/.3dl, ACR .xmp presets (standalone, like psd)
│  ├─ codecs/                  brahma-codecs    flat raster decode/encode at format-supported depths; AVIF encode only (feature `avif`), HEIF/HEIC decode only (feature `heif`); no JPEG XL
│  ├─ heif/                    brahma-heif      optional HEIF/HEIC decoder (heic-rs, pure Rust); used only by codecs behind its `heif` feature (standalone)
│  ├─ affinity/                brahma-affinity  bounded native Affinity reader: archive, object stream, layers, vectors, text, pixels (standalone; read only)
│  ├─ raw/                     brahma-raw       clean-room camera RAW decode (DNG, CR2, TIFF/EP) + develop pipeline (standalone, like psd)
│  ├─ format/                  brahma-format    native document format (.pcraft bundle): manifest + content-addressed tiles
│  ├─ io/                      brahma-io        import/export orchestration; doc ⇄ PSD mapping; PDF/SVG import (features)
│  │  ── intelligence ──
│  ├─ ml/                      brahma-ml        model registry + InferenceBackend trait (ort native / burn|candle web); SAM2, matting, depth, sky
│  ├─ plugins/                 brahma-plugins   sandboxed WebAssembly filter plug-ins (wasmi host, limits, ABI v1, registry)
│  │  ── the façade every frontend talks to ──
│  ├─ tools/                   brahma-tools     tool state machines (move, marquee, lasso, brush, clone, gradient, crop, transform, pen, text…)
│  ├─ engine/                  brahma-engine    Session: open docs, command registry + dispatch, jobs, events, view-models, preferences
│  │  ── platform + frontends ──
│  ├─ platform/                brahma-platform  traits: file dialogs, clipboard, fonts, tablet input, menus, storage; native + web impls
│  ├─ ui-egui/                 brahma-ui-egui   the (first) GUI shell: panels, dialogs, canvas widget, theme
│  ├─ automation/              brahma-automation  MCP server (rmcp) + JSON-RPC over the command registry
│  └─ testkit/                 brahma-testkit   golden images, perceptual diff, fixtures, PSD corpus helpers
├─ apps/
│  ├─ brahma/              desktop binary (winit + wgpu + ui-egui + platform-native)
│  ├─ brahma-cli/          headless batch CLI (open → commands → export), also hosts `mcp` subcommand
│  └─ brahma-web/          wasm32 binary (ui-egui + platform-web), demo
├─ shaders/                    (or inside gpu/) WGSL, with generated struct headers
├─ assets/                     icons, bundled fonts, default brushes/swatches/presets
├─ fuzz/                       cargo-fuzz targets (psd, codecs, format)
├─ xtask/                      build tasks: layer-check, shader-gen, corpus fetch, release
└─ plan/                       this directory
```

**what exists today.** this layout is the target design. built so far: `geom`, `cms`, `color`, `raster`, `psd`, `codecs`, `doc`, `ops`, `algo`, `paint`, `text`, `vector`, `compose`, `gpu`, `format`, `raw`, `io`, `plugins`, `engine`, `ui-egui`, `automation`, `testkit`, and the three apps. not yet split out: `viewport` and `tools` live inside `ui-egui` and `engine`; `platform` services are function hooks injected by each app (`ui_egui::Services`); `adobe-assets` and `ml` are not started.

**crate granularity:** start with the crates above. split `algo` into `-adjust`, `-filters`, `-select`, `-inpaint` and `-warp` once any module passes about 10k lines, or once compile times hurt. its internal module boundaries should already follow those lines.

**naming:** directories are short and packages are prefixed `brahma-`. `brahma-psd` gets a neutral, publishable name (such as `psd-rw`) if we release it to crates.io.

---

## 3. dependency layering (enforced)

```text
 L7  apps/*                         (binaries: wire everything together)
 L6  ui-egui · automation · platform
 L5  engine
 L4  tools · viewport · io · ml · plugins
 L3  compose · gpu · format
 L2  ops · algo · paint · text · vector
 L1  doc
 L0  geom · cms · color · raster           psd, codecs, raw, adobe-assets (standalone, no workspace deps)
```

**rules** (checked by `cargo xtask layers` in ci, which parses `cargo metadata`):

1. a crate may depend only on crates in **lower** layers. no cycles and no sideways dependencies, except where listed.
2. **nothing below l6 may depend on any ui toolkit, winit, or a `platform` implementation.** platform services reach the engine through traits defined in `engine` (or in `platform`'s trait-only core), and are injected at startup.
3. **`brahma-psd` depends on no workspace crate.** the doc ⇄ PSD mapping lives in `io`. this keeps the PSD crate publishable and reusable by other projects. the same holds for every standalone crate, with one documented exception: `codecs` → `heif` (both standalone and publishable; `STANDALONE_EXCEPTIONS` in `xtask/src/layers.rs`).
4. **wasm gate:** every crate in l0–l5 (except feature-gated native backends) must build for `wasm32-unknown-unknown`. ci runs `cargo build -p brahma-engine --target wasm32-unknown-unknown --no-default-features --features web`.
5. `gpu` is optional for `engine`. engine features are `gpu` (default on) and `cpu-only`, and `cpu-only` builds are what the headless CLI and ci tests use.
6. **c dependencies** (optional LibRaw, pdfium) only behind features, only in `codecs`, `raw` or `io`, and never on by default for the web target.

---

## 4. the ui-swap seam (the most important interface)

a frontend needs exactly five things from the core. keeping these small and data-oriented is what makes the ui replaceable.

### 4.1 `Session`: commands in, events out

```rust
// brahma-engine
pub struct Session { /* docs, registry, job pool, prefs, services */ }

impl Session {
    pub fn new(services: Services) -> Self;                         // platform traits injected
    pub fn dispatch(&self, doc: Option<DocId>, cmd: CommandInvocation) -> Result<JobHandle, CommandError>;
    pub fn registry(&self) -> &CommandRegistry;                     // metadata for menus/palette/shortcuts
    pub fn snapshot(&self, doc: DocId) -> Arc<DocSnapshot>;         // lock-free read (arc-swap)
    pub fn events(&self) -> EventReceiver;                          // DocChanged{damage}, JobProgress, Toast, …
    pub fn view_models(&self, doc: DocId) -> ViewModels;            // layers, history, channels, tool options…
}

pub struct CommandInvocation { pub id: CommandId, pub params: serde_json::Value /* validated vs schema */ }
```

### 4.2 `CommandRegistry`: the menu, palette and automation catalogue

each command registers the following:
- `id` (`"filter.blur.gaussianBlur"`), `label`, `menu_path` (`["Filter","Blur"]`), and a `default_shortcut`.
- `params: schemars::Schema`, plus optional `ui_hints` (slider ranges, units, log scale, grouping).
- `enabled(&DocSnapshot, &ToolState) -> bool` and `checked(...) -> Option<bool>`.
- `preview: PreviewMode` (none / live / proxy). this lets dialogs show on-canvas preview generically.
- `kind`: instant / dialog / interactive (hands control to a tool, e.g. free transform).

**consumers:**
- the GUI builds the menu bar, the ⌘f palette and the shortcut editor from it.
- `automation` exposes the registry as MCP tools.
- `brahma-cli` exposes `brahma-cli run in.psd --cmd filter.blur.gaussianBlur --params '{"radius":4}' --out out.png`.
- action recording is simply `Vec<CommandInvocation>`.

### 4.3 tools: pointer events in, ops and overlays out

```rust
// brahma-tools
pub struct PointerEvent { pub pos_doc: Point, pub pressure: f32, pub tilt: Vector, pub twist: f32,
                          pub time_us: u64, pub buttons: Buttons, pub mods: Modifiers, pub kind: PointerKind /* mouse|pen|eraser|touch */ }

pub trait Tool {
    fn id(&self) -> ToolId;
    fn options_schema(&self) -> Schema;                       // options bar generated from schema
    fn on_pointer(&mut self, ev: &PointerEvent, cx: &mut ToolCx) -> ToolResponse;   // ToolCx gives snapshot + op sink
    fn on_key(&mut self, key: Key, cx: &mut ToolCx) -> ToolResponse;
    fn overlay(&self, view: &ViewTransform) -> Vec<OverlayPrim>;                     // drawn by any UI
    fn cursor(&self) -> CursorSpec;
}

pub enum OverlayPrim { Path{path: BezPath, style: StrokeStyle, space: Space}, MarchingAnts{path: BezPath},
                       Handle{pos: Point, kind: HandleKind}, BrushOutline{center: Point, radius: f32, ..},
                       Text{pos: Point, text: String}, Guide{..}, Grid{..} }
```

coalesced high-rate pen samples come from `platform` and are handed to tools as a batch, so the brush sees every sample even when the ui renders at 60–120 hz.

### 4.4 viewport: document to pixels

```rust
// brahma-viewport
pub struct Viewport { pub camera: Camera /* zoom, pan, rotation, dpr */, /* tile/mip cache */ }
impl Viewport {
    pub fn render_gpu(&mut self, snap: &DocSnapshot, gpu: &GpuContext, target: &wgpu::TextureView, damage: &Region);
    pub fn render_cpu(&mut self, snap: &DocSnapshot, target: &mut RgbaImageMut, damage: &Region); // wasm fallback, tests
    pub fn doc_to_screen(&self) -> Affine;  pub fn screen_to_doc(&self) -> Affine;
}
```

- any toolkit that can host a `wgpu::TextureView` (egui paint callbacks, slint's wgpu texture import, bevy, a raw winit surface) embeds the canvas the same way.
- a toolkit without wgpu can fall back to `render_cpu`.

### 4.5 view-models and schemas

- panels (layers, history, channels, paths, properties, navigator, histogram, swatches) read plain structs produced from a snapshot, such as `LayersModel { rows: Vec<LayerRow{ id, name, kind, depth, visible, locked, thumb: ThumbHandle, blend, opacity, has_mask, effects: Vec<..> }> }`. thumbnails are `ThumbHandle`s that the ui resolves to textures through an engine thumbnail cache.
- dialogs are generated from command `params` schemas. a toolkit may register *custom* dialog widgets for a few complex commands: curves, levels, camera raw, liquify, layer style.

**litmus test for the seam:** `apps/brahma-cli` and the automation server must be able to do *everything* the GUI does, except pointer painting, which they do by replaying `PointerEvent` streams through tools.

---

## 5. document model (`brahma-doc`)

```rust
pub struct Document {
    pub id: DocId, pub size: Size, pub resolution_dpi: f32,
    pub mode: ColorMode /* Rgb | Gray | Cmyk | Lab | Bitmap | Indexed */, pub depth: Depth /* U8 | U16 | F32 */,
    pub profile: IccProfileRef,
    pub root: LayerGroup,            // tree
    pub channels: Vec<AlphaChannel>, // saved selections/spot
    pub paths: Vec<NamedPath>, pub guides: Guides, pub slices: .., pub metadata: Metadata /* XMP, EXIF */,
    pub selection: Option<Mask>,     // current selection (a raster mask + optional vector outline)
}

pub struct Layer { pub id: LayerId, pub name: String, pub visible: bool, pub locks: Locks,
                   pub blend: BlendMode, pub opacity: f32, pub fill_opacity: f32, pub clipped: bool,
                   pub mask: Option<RasterMask>, pub vector_mask: Option<VectorMask>,
                   pub effects: Effects, pub blend_if: BlendIf, pub content: LayerContent }

pub enum LayerContent {
    Raster(TiledSurface),
    Group(LayerGroup /* children, pass-through or isolated */),
    Adjustment(Adjustment /* Curves, Levels, HueSat, … as params */),
    Fill(Fill /* solid | gradient | pattern */),
    Text(TextLayer /* runs, paragraph styles, warp; cached raster */),
    Shape(ShapeLayer /* vector paths + fill/stroke */),
    Smart(SmartObject /* embedded or linked source + transform + smart filters */),
}
```

- **pixels.** `raster::TiledSurface<F>` is a sparse map from `TileCoord` to `Arc<Tile<F>>`, with 256×256 tiles. missing tiles are transparent, or take a fill value. mutation calls `Arc::make_mut`, so it is copy-on-write.
- **tile generation counters** feed the GPU residency cache and the thumbnail cache.
- **mip pyramids** are derived on demand per layer, for zoomed-out views.
- **big documents.** the tile store is behind a `TileStore` trait: in-memory now, disk-spilling ("scratch disk") later, with LRU eviction under a global memory budget.
- **the first milestone supports RGB and gray, at 8/16/32f.** CMYK and lab are modelled in the types from day one (so PSD round-trips don't lose them) but are rendered by converting to RGB until later phases.

---

## 6. operations and history (`brahma-ops`)

- **every mutation is an `Op`**, such as `SetLayerProps`, `PaintTiles{layer, tiles_before, tiles_after}`, `AddLayer`, `MoveLayer` or `ApplyFilter`.
- **a command produces a `Transaction` of ops** with a label ("gaussian blur").
- **history stores transactions.** because tiles are `Arc`-shared, "before" states cost only the tiles that actually changed. undo means swapping `Arc`s, so no pixel copies are made.
- **each state remembers its targeted layers** (the active layer and the layer selection, as they were when the state was created: when the document was opened, or right after the step that made it). selecting layers is not a step and doesn't change any state's target, but undo and redo target the restored state's layers again, as photoshop does.
- **brush strokes** accumulate into one transaction per stroke. tiles are published incrementally with damage regions, so the viewport updates live.
- **snapshots for readers.** after each committed op the engine publishes a new `Arc<DocSnapshot>` through `arc-swap`. the ui, background jobs (filters, ai inference, export) and autosave read snapshots without locks.
- **background jobs** compute from snapshot n and commit as a transaction. if the doc changed meanwhile, the job rebases, meaning it re-targets the same layer id if it still exists, or reports a conflict.
- **history features:** a history panel, a memory cap with oldest-first eviction, and named snapshots. the history brush comes later, reading tiles from a past state.

---

## 7. rendering

### 7.1 planner (`compose`)

`plan(&DocSnapshot, region) -> Vec<DrawOp>` flattens the layer tree into a linear instruction stream:
- `PushGroup{isolated, blend, opacity}` / `PopGroup`
- `BeginClipBase` / `Clipped`
- `DrawTiles{layer, blend, opacity, mask}`
- `Adjust{program}`
- `Effect{kind, params}`
- `Fill{...}`

both backends consume the same plan. this is the only place that encodes photoshop layer semantics: pass-through groups, clipping groups, fill vs opacity, blend-if and knockout.

### 7.2 backends

- **CPU (`compose`).** the reference implementation, done per tile with rayon. it is used for export, tests, the CLI, and the web fallback when webgpu is unavailable.
- **GPU (`gpu`).** wgpu compute and render pipelines that run the same plan. they use:
  - `rgba16float`/`rgba32float` accumulators,
  - specialized pipelines per blend family and adjustment (WGSL `override` constants, not one giant `switch`),
  - a residency cache keyed by `(TileId, generation)`,
  - cached group and effect results that are invalidated by generation,
  - one global `Budget` for GPU memory.
- **parity tests.** for every blend mode, adjustment and effect, the CPU and GPU results must match within a tolerance of ≤1/255 for 8-bit output. this runs in ci on a software adapter (wgpu with lavapipe or WARP).

### 7.3 color

- blending happens in document space by default, which is photoshop-compatible. a per-document "linear light blending" option is also available.
- **display transform** (`engine/src/display_color.rs`): the canvas is always colour-managed, document profile → monitor profile (relative colorimetric + BPC), cached per (document profile, mode, monitor). on the GPU canvas the transform, plus proof colors / gamut warning / 32-bit preview, is baked into a 33³ 3d LUT the canvas shader's final pass applies; the CPU canvas runs an 8-bit `brahma-cms` transform on the composite. when the document profile matches the monitor (srgb on srgb) there is no LUT and no transform. linear composites (EXR/HDR, tagged linear srgb on import) are stored srgb-encoded in the 8-bit canvas texture. CMYK documents are read through their embedded CMYK profile (`brahma_color::convert::with_cmyk_space`, entered by the compositors and composite exports).
- **monitor profile:** edit › color settings › monitor profile: `auto`, a built-in RGB profile or an `.icc` path. in `auto` each window uses the profile of the display it is on (#569): the desktop app reads every display's id, name, frame and ICC profile at launch and again when the app comes back to the front, when any window is on an unknown or resized display (at most every 30 s; a trigger inside that window is deferred) and when edit › color settings opens (`ui-egui/src/monitor_status.rs`). macos reads AppKit `NSScreen` through `osascript` (no FFI); X11 reads randr outputs and the default colord profile matched by exact `XRANDR_name`; native wayland and xwayland windows from a wayland session stay on srgb because eframe/winit cannot safely set the app surface's wayland image description. the macos helper is stopped after 10 s. there is deliberately no periodic re-read (#569 decision: a read costs about 0.2 s of CPU), so a profile reassigned while brahma stays in front with no window moving is picked up at the next return to the front or color settings. X11 selects the root window's randr output by winit's current-monitor name and other windows by their overlapping randr frame; macos follows `NSWindow.screen`. elsewhere `auto` is srgb. the GPU canvas keeps one texture per document and a display LUT per (document, display); CPU canvas textures are per (document, display). `ColorState::monitor_status_for` says what is applied per display: `auto`, `manual`, or `fallback` to srgb with the reason (missing, unreadable, non-RGB or unusable as a destination), and whether the reading is an earlier one kept after a failed re-read; `edit.colorSettings` (`monitorStatus`, `displays`), help › system info and the color settings dialog show it, and a fallback in `auto` posts a notice. letting macos colour-match the canvas instead is #581 (an architecture decision).
- **HDR/EDR output** (an `rgba16float` surface with an extended-range colorspace) is a later-phase feature. the interfaces already carry `f32` pixels.

### 7.4 oracle

**PSD files contain photoshop's own merged composite.** running our compositor on a PSD's layers and diffing against its embedded composite gives a free, large conformance test suite. `testkit` automates this over a PSD corpus. the corpora are fetched at pinned commits into `corpus/` (never committed) by `cargo xtask corpus --all`; our own photoshop-authored oracles live in https://github.com/storytold/brahma-corpus (see `docs/development.md` › test corpora).

---

## 8. `brahma-psd`: standalone PSD/PSB crate

**scope:** faithful, format-level read and write of PSD (v1) and PSB (v2, 64-bit lengths). it has no dependency on our document model.

```text
psd/src/
  header.rs         color mode, depth, size
  color_mode_data.rs
  resources/        image resources (1005 resolution, 1036 thumbnail, 1039 ICC, 1060 XMP, 1058 EXIF, slices, guides, …)
  layers/           layer records, channel image data (raw | RLE/PackBits | ZIP | ZIP+prediction), masks, blending ranges
  tagged/           additional layer info blocks, one module per key:
                    luni lsct lyid lfx2/lrFX (effects) TySh (text) SoLd/PlLd/lnk2 (smart objects) vmsk/vsms/vogk (vector)
                    adjustment keys (curv levl hue2 blnc selc mixr grdm phfl brit thrs post nvrt expA vibA clrL blwh)
                    SoCo GdFl PtFl (fills), Patt, Txt2, …
  descriptor.rs     ActionDescriptor / OSType parser+writer (used by many tagged blocks)
  engine_data.rs    text "EngineData" (PostScript-like) parser+writer
  image_data.rs     merged composite
  write.rs          serializer (+ RLE/ZIP encoders)
```

- **lossless round-trip.** unknown or unsupported tagged blocks are kept as raw bytes and re-emitted unchanged. that way files keep features we don't model yet, like 3d or video.
- **deferred decode.** the parser indexes channel data and decodes it only on request. this makes opening a 2 gb PSB fast and memory-bounded.
- **the merged composite on write is supplied by the caller** as an image buffer. `io` renders it with `compose`, so `psd` never needs a compositor.
- **mapping lives in `io::psd`.** it converts `psd::File` into `doc::Document` and back, with fidelity levels:
  1. flattened image.
  2. raster layers, masks, blend/opacity/fill, visibility, names.
  3. groups (including pass-through), clipping, blend-if, adjustment and fill layers.
  4. layer effects (lfx2).
  5. text layers: shaped with `text`; the original descriptors are kept for round-trip.
  6. smart objects: embedded and linked, live both ways (`io::smart_map`). import reads the
     placed-layer data (`SoLd`: file id, transform quad, warp), the smart filter stack
     (`filterFX`: modelled filters become their brahma command and params; any other filter
     stays verbatim and is listed as not editable) and the filter mask (global `FEid`). Export
     writes `PlLd` + `SoLd`, embeds the source in `lnk2` (a `.pcraft` source becomes a PSB of the
     nested document, written by our own PSD writer) and a filter cache with the unfiltered
     pixels and the mask in `FEid`; unedited imported smart objects stay byte-identical.
  7. vector masks and shape layers.
- **testing:**
  - round-trip byte-stability for unmodified files.
  - open → save → open equality.
  - composite-oracle diffs (§7.4).
  - `cargo-fuzz` on the parser.
  - a corpus from MIT/BSD-licensed test sets (ag-psd, psd-tools), fetched by `xtask`, not committed.
  - our own photoshop-authored oracle psds (smart filters, effect shapes, type, adjustments in every mode and depth) from https://github.com/storytold/brahma-corpus, fetched at a pinned commit the same way.
- **existing crates:** `psd` (read-only, limited) and the new `ag-psd` rust port. we evaluate them in spike week 1, then either depend on one, fork it, or write our own. given how central PSD is, writing our own behind a stable API is likely.

---

## 9. native file format (`brahma-format`)

- **format:** a `.pcraft` file is a zip (store mode) or directory bundle.
  - `manifest.json`: a versioned document tree with serde, including `format_version` and migrations.
  - `tiles/<blake3>.zst`: content-addressed tiles, zstd-compressed.
  - `thumb.png` and `composite/` mips for quick look and previews.
- **saving is incremental.** tiles are content-addressed, so a save writes only new tiles plus the manifest, and garbage-collects unreferenced ones.
- **autosave and crash recovery** reuse the same writer into a recovery directory, reading from a snapshot on a background thread.
- **PSD is the interchange format,** and `.pcraft` is the lossless native one.

---

## 10. algorithms (`algo`, `paint`, `text`, `vector`, `ml`)

| area | contents (first pass) | GPU kernel? |
|---|---|---|
| adjustments | levels, curves, brightness/contrast, exposure, vibrance, hue/sat, color balance, b&w, photo filter, channel mixer, gradient map, selective color, invert/posterize/threshold, 3d LUT | yes (LUT + small kernels) |
| filters | gaussian/box/motion/radial/surface/lens blur, unsharp/smart sharpen, high pass, noise add/reduce/median/dust, distort family (twirl, wave, polar, spherize, displace), stylize, render (clouds, lens flare), pixelate, color to alpha | many |
| selection | marquee/lasso geometry → mask, magic wand (flood fill tolerance), color range, quick select (graph cut / superpixels), feather, grow/shrink, refine edge | some |
| inpaint/heal | spot heal, healing brush (poisson), content-aware fill (PatchMatch + multiscale em) | later |
| warp | free transform (affine/perspective/warp mesh), liquify, puppet (ARAP), lens correction, resampling (lanczos/bicubic) | yes |
| paint (`paint`) | dab generation (hardness, roundness, angle, spacing, scatter), dynamics from pressure/tilt/velocity, smoothing (pulled string / lazy mouse), wet edges, airbrush, eraser, clone/heal sources, blend modes per stroke, `.abr` import | CPU first, GPU later |
| text (`text`) | font db trait (system fonts native, bundled fonts on web), parley layout, per-run styles, paragraph options, warp; renders to a raster cache | — |
| vector (`vector`) | shapes, bezier paths, pen tool geometry, strokes (dash, align, caps), boolean ops → coverage via vello_cpu | — |
| ml (`ml`) | `InferenceBackend` trait; tasks: `segment_point/box` (SAM2), `matte_subject` (background removal), `depth`, `sky`; model registry with SHA-256 verification and on-demand download | backend-dependent |

**rules for every algorithm:**
- **signature:** a pure function `fn(input: &SurfaceView, params: &P, region: Rect, cancel: &CancelToken) -> Surface`.
- **params:** `P: Pod + JsonSchema + Serialize + Deserialize`.
- **registration:** registered with the command registry by the engine, not by the algo crate itself, so `algo` stays ui- and engine-agnostic.
- **tiles:** tile-aware, requesting an input halo of `radius` pixels.
- **testing:** golden-image tests in `testkit`, plus benchmarks with criterion.

---

## 11. engine runtime model

```text
 UI thread ──dispatch()──► Engine thread (owns mutable docs, applies ops, publishes snapshots)
     ▲                         │  spawns
     │ events (damage, progress)│
     │                         ▼
     │                  Job pool (rayon): filters, AI, export, autosave — read snapshots, return transactions
     │
 Render (UI thread or dedicated): Viewport::render_gpu(snapshot) into toolkit texture
```

- **pen input path for low latency:** platform coalesced samples → tool (on the engine thread) → dab rasterization into COW tiles → damage published → viewport re-composites only the damaged tiles. target: the stroke appears within the next frame.
- **cancellation:** every job takes a `CancelToken`, and the ui can cancel any job with progress.
- **services injected into `Session`:**
  - `FileSystem` (native fs / web file system access / in-memory for tests)
  - `Clipboard`, `FontSource`, `ModelStore`
  - `Clock`, `Notifier`

---

## 12. automation and extensibility

- **`automation`:** an MCP server built on `rmcp`. its tool names use underscores (the dotted
  names such as `doc.open` belong to the JSON control protocol, `docs/control-protocol.md`). it exposes:
  - `session_list`, `doc_open`, `doc_new`, `doc_place` (place from the read root as a smart object),
    `doc_export_layers`, `doc_export_data_sets` (write beneath the write root), `doc_import_data_sets` (a CSV from the read root),
    `doc_save`, `doc_export`, `doc_inspect` (layer tree as JSON), `doc_render_preview` (PNG), `doc_select`, `doc_close`.
  - `command_list`, `command_run` and `command_batch`, generated from the registry; `jobs_list` and `jobs_cancel` for background jobs.
  - bridge only (a running app): `ui_inspect`, `ui_screenshot`, `ui_pointer`, `ui_menu_invoke`, `ui_set`, `control_call`. the full list is in `docs/development.md` › MCP.
  - stdio for agent clis, and optionally loopback TCP with a token so it can attach to a running GUI.
  - `AuthorizedWorkspace`, which holds independent read and write directory capabilities. remote
    paths are validated relative names; engine commands that still require ambient filesystem
    access fail closed at the automation boundary.
- **actions:** recorded `Vec<CommandInvocation>`, replayable in batch (file → automate → batch).
- **scripting (later):** embed a scripting language over the same registry. options are rhai, or lua via mlua (c). js via quickjs is possible if we want photoshop-script familiarity.
- **plugins:** sandboxed WebAssembly filter plug-ins (`brahma-plugins`, l4) run by `wasmi`, a pure-rust interpreter, with fuel, memory, stack and wall-time limits and no host imports. they are driven by the `plugin.*` commands and listed under filter › plug-ins; the ABI is in [`plugins.md`](plugins.md). native photoshop `.8BF`/CEP/UXP hosting is out of scope (it needs unsafe FFI and can't run on the web). panel plug-ins are later.

---

## 13. testing and quality

| layer | what |
|---|---|
| unit | per crate; algorithms against small hand-computed fixtures |
| golden images | `testkit::assert_image_eq(actual, "golden/name.png", tol)`; goldens regenerated by `xtask bless` |
| CPU/GPU parity | every GPU kernel vs its CPU reference, software adapter in ci |
| PSD | round-trip, composite oracle, fuzzing |
| engine | command-level tests: open fixture → dispatch commands → assert snapshot/render; action replays |
| ui | headless egui tests (egui_kittest) for panels; kept thin because the ui is thin |
| perf | criterion benches + budget checks in ci (composite 4k² 20 layers, brush dab throughput, open 100 mp) |
| portability | ci matrix: macos arm64, windows x64, linux x64, wasm32 build check |

**performance targets for v1:**
- stroke-to-pixel within one frame at 120 hz.
- pan and zoom at display refresh on a 100 mp / 30-layer document.
- open a 100 mp JPEG in under 1.5 s.
- 20k×20k 16-bit documents on a 16 gb machine.

---

## 14. cargo workspace sketch

```toml
[workspace]
resolver = "3"
members = ["crates/*", "apps/*", "xtask"]

[workspace.package]
edition = "2024"
license = "MIT OR Apache-2.0"        # decision pending, see §16
rust-version = "1.95"

[workspace.dependencies]
# foundation
bytemuck = { version = "1", features = ["derive"] }
kurbo = "*"          # pin at scaffold time
glam = "*"
smallvec = "1"
arc-swap = "1"
rayon = "1"
thiserror = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
schemars = "1"
blake3 = "1"
zstd = "*"           # native; ruzstd for wasm decode
# color / codecs
# ICC: our own brahma-cms (pure Rust)
image = { version = "*", default-features = false }
jxl-oxide = "*"
# gpu
wgpu = "*"           # one version shared by gpu, viewport, ui
# text / vector
parley = "*"
skrifa = "*"
vello_cpu = "*"
# ml
ort = { version = "*", optional = true }
# automation
rmcp = "*"

[workspace.lints.rust]
unsafe_code = "deny"   # allowed per-crate with justification (gpu, platform)

[profile.release]
lto = "thin"
codegen-units = 1

[profile.dev.package."*"]
opt-level = 2          # image code is unusable at opt-level 0
```

versions get pinned when we scaffold, using the numbers in `rust-framework-options.md`.

---

## 15. phased roadmap

each phase ends with a demoable build and green ci on all targets. phases 8–10 (colour/print pro, photographer/designer pro, frontier) and the per-feature phase assignments are in [`target-app-parity.md`](target-app-parity.md).

| phase | scope | done when |
|---|---|---|
| **0. skeleton** (1–2 wk) | workspace, all crates stubbed, layering check, ci matrix incl. wasm, testkit, xtask | `cargo xtask ci` green on 3 os + wasm |
| **1. viewer** | raster doc, tiles, codecs (png/jpeg/tiff/webp), viewport pan/zoom/rotate on GPU + CPU fallback, `.pcraft` save/open, CLI `convert` | open a 100 mp JPEG, pan at refresh rate, save/reopen losslessly |
| **2. layers** | layer tree, all 27 blend modes, opacity/fill, masks, groups, clipping; planner + CPU/GPU compositors + parity tests; history; **PSD read** fidelity 1–3 | composite-oracle suite ≥ 95% within tolerance on the corpus |
| **3. paint and select** | platform tablet input, brush engine + presets, eraser, marquee/lasso/wand, move, free transform, crop | draw with pen pressure at <1 frame latency; selection-restricted painting |
| **4. adjust and filter** | adjustment layers (first 10), first 20 filters with schema-generated dialogs + live preview; **PSD write**; actions recording | PSD open → edit → save opens correctly in photoshop/photopea |
| **5. text, vector, styles** | text layers, shapes/pen/paths, layer effects (drop shadow, stroke, glows, overlays, bevel) | PSD text/shape/effects round-trip at fidelity 4–5 |
| **6. smart features** | ml backend + SAM2 select subject/object, background removal, content-aware fill, healing, RAW develop | select subject + remove background on a portrait in <3 s |
| **7. automation and web** | MCP server + CLI parity, batch; `brahma-web` demo | an agent edits a PSD through MCP; the web demo opens a PSD, paints and filters |

---

## 16. open decisions

1. **ui toolkit.** the leading option is egui on winit + wgpu. bevy, slint and qt are the alternatives (see `rust-framework-options.md`). this plan works with any of them.
2. **license.** MIT/apache-2.0 is maximally reusable. GPL-3.0 is the krita/GIMP model and protects against proprietary forks. note that some candidate deps are LGPL (rawler) or GPL-or-commercial (slint), and MIT/apache would constrain which of them we can use.
3. **project name.** `brahma` (the repo name) is assumed.
4. **PSD crate.** build our own, or adopt or fork `ag-psd`/`psd`. decide after the week-1 spike.
5. **default working depth and blending.** 8-bit + gamma-space blending matches photoshop. 16-bit/float + linear is the modern default. the proposal is to follow photoshop by default and make the other a per-document option.
6. **scope of photoshop plugin compatibility.** decided: no native `.8BF` hosting; a sandboxed WebAssembly plug-in API instead (see [`plugins.md`](plugins.md)).

## revision history

| date | change | summary |
|---|---|---|
| 2026-10-13 | minor | workspace size and `#[test]` counts recounted (393k → 539k lines, 5,460 → 8,040 tests); other current-state figures unchanged |
| 2026-10-10 | minor | current-state summary and deviations from the plan; status line; broken link to `photoshop-parity.md` repointed to `target-app-parity.md` |
| 2026-09-30 | major | design plan v1 |
