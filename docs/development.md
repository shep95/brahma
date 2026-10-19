# development guide

## prerequisites

- rust stable (1.95+). add the web target with `rustup target add wasm32-unknown-unknown`.
- macos, windows or linux. on linux, install the windowing, gl and GTK development packages:
  - debian/ubuntu: `sudo apt install libxkbcommon-dev libwayland-dev libx11-dev libxrandr-dev libxi-dev libgl1-mesa-dev libgtk-3-dev`
  - fedora/RHEL: `sudo dnf install libxkbcommon-devel wayland-devel libX11-devel libXrandr-devel libXi-devel mesa-libGL-devel gtk3-devel`
  - arch: `sudo pacman -S libxkbcommon wayland libx11 libxrandr libxi mesa gtk3`
  - opensuse: `sudo zypper install libxkbcommon-devel wayland-devel libX11-devel libXrandr-devel libXi-devel Mesa-libGL-devel gtk3-devel`

### windows source builds

use the rust MSVC toolchain and install visual studio or microsoft c++ build tools with the
**desktop development with c++** workload. follow microsoft's
rust setup guide,
then open a new terminal and check `cargo --version` and `rustc --version`.

on windows 11, smart app control can block cargo, rustc or executables generated during a build.
if a build reports `An Application Control policy has blocked this file. (os error 4551)`
([#1585](https://github.com/shep95/brahma/issues/1585)), check:

- **windows security → app & browser control → smart app control settings** for its status.
- **event viewer → applications and services logs → microsoft → windows → CodeIntegrity →
  operational** for the blocked executable and policy, especially on managed machines.

this error indicates an application-control restriction; moving `CARGO_TARGET_DIR` or
reinstalling rust does not establish that the blocked executable is trusted. see microsoft's
smart app control FAQ
for the available controls; there is no per-app exception. on a managed device, ask your
administrator about an approved development environment. if you only want to run brahma,
use the [packaged windows release](https://github.com/shep95/brahma/releases) to avoid
building locally; the downloaded app is still subject to windows application-control checks.

## build and run

```sh
cargo run --release -p brahma -- path/to/image.psd      # desktop app
cargo run --release -p brahma -- --control 7878 --control-token-file .private/control.token \
  --automation-read-root work --automation-write-root work
cargo test --workspace                                     # everything
cargo xtask ci                                             # fmt + clippy + tests + layers + wasm
cargo xtask stats                                          # tests and lines per crate
cargo xtask parity                                         # Photoshop menu coverage -> docs/parity-checklist.md
```

image code is slow at `opt-level 0`, so the workspace profile builds dependencies at `opt-level 2`. use `--release` for anything interactive.

the dev profile builds with `debug = "line-tables-only"`: panic backtraces keep file and line, and test binaries link about twice as fast. to inspect local variables in a debugger, build with `CARGO_PROFILE_DEV_DEBUG=true`.

### faster test loops

most of a `cargo test` run is linking test binaries and running them one after another. while iterating:

```sh
cargo test -p <crate> --lib --tests        # skip examples and doctests (ui-egui has 11 examples)
cargo test -p <crate> --test <file>        # one integration-test file
cargo nextest run --workspace              # parallel across binaries (cargo install cargo-nextest --locked)
cargo test --doc --workspace               # nextest does not run doctests
```

cargo-nextest runs each test in its own process. `.config/nextest.toml` puts the GPU tests (`brahma-gpu`, and ui-egui's integration and canvas/GPU tests) in a group of at most 4 at a time, because each process opens its own wgpu device and the in-process `gpu_lock()` no longer serializes them. on a 32-thread windows machine the full workspace went from 297 s (`cargo test`) to about 120 s (#2204). ignored tests run with `cargo nextest run --run-ignored only`. ci still runs `cargo test`, and a full `cargo test` stays the final check before a pr.

## fonts on demand (google fonts catalogue)

every family of google fonts (about 1,900, all under the OFL, apache or UFL) is listed in the font
menus of both the desktop app and the web build without shipping a byte of font data: the
catalogue `crates/text/data/google_fonts.tsv` (family, category, repository directory, the files
to fetch, fallback scripts) is compiled into `brahma-text` (`served::google_catalog`), and a family
is fetched from the pinned commit `served::GOOGLE_FONTS_COMMIT` on `raw.githubusercontent.com` the
first time it is picked or a layout needs it (`ui-egui/src/served_fonts.rs` › `serve`, through
`Services::fetch_url`: a browser fetch on the web, cached by the service worker; a thread with a
disk cache under the settings directory on the desktop). the noto sans families lead the catalogue
so they are the fallbacks for arabic, hebrew, devanagari, thai, khmer, lao, burmese, japanese,
chinese and korean text. regenerate the catalogue with
`python3 -I crates/text/scripts/google_fonts_catalog.py <dir of METADATA.pb files> crates/text/data/google_fonts.tsv`
and bump the commit constant together. a host's own `fonts/manifest.txt` (craft-fonts format)
is still read first on the web and wins for the families it lists.

## fonts (craft-fonts)

font assets shared by the #houseofasher apps live in storytold/craft-fonts, never in this repo: don't commit font files here (inter and JetBrains mono in `assets/fonts/` are the only exceptions; new fonts go to craft-fonts). the rules are in `craftrules/standards/fonts.md` in a sibling `craftrules` checkout (see below); that repository is not public, so outside contributors can ask a maintainer for the rules that apply to their change.

craft-fonts is an **optional build input**, never a cargo dependency:

```sh
git clone https://github.com/storytold/craft-fonts ../craft-fonts
CRAFT_FONTS_DIR="$PWD/../craft-fonts" cargo run --release -p brahma
CRAFT_FONTS_DIR="$PWD/../craft-fonts" cargo test --workspace     # runs the Japanese font tests too
```

- use an absolute path (`$PWD/...`): `build.rs` runs in `crates/text`, so a relative `CRAFT_FONTS_DIR` would resolve from there.
- `crates/text/build.rs` reads `$CRAFT_FONTS_DIR/fonts/manifest.txt` and embeds the fonts as `brahma_text::CRAFT_FONTS` (`crates/text/src/craft_fonts.rs`). unset, `CRAFT_FONTS` is empty and the build is unchanged. a bad path is a build warning, or an error with `CRAFT_FONTS_REQUIRED=1` (release builds set both).
- **ui:** the japanese fonts (BIZ udpgothic regular first) are the first japanese fallback in the on-demand CJK loader (`crates/ui-egui/src/cjk_fonts.rs`), ahead of the system japanese fonts and in the same locale script order, appended last to every egui family with the usual baseline alignment.
- **type tool:** the text engine registers them in `FontDb::new` (so also with no system fonts) and puts them first in the japanese slot of the locale-ordered fallback list: BIZ udpgothic for sans runs, shippori mincho / BIZ udmincho for serif runs.
- **web:** the wasm32 build never embeds craft-fonts, even with `CRAFT_FONTS_DIR` set: measured on 2026-10-06, the ui face alone took the release wasm from 24.19 mb to 28.87 mb, over the 24 mib gate in `packaging/web/package.sh`. instead it loads fonts served next to it, on demand: `fonts/manifest.txt` beside `index.html`, in craft-fonts' manifest format (`crates/text/src/served.rs`; hosting: `packaging/web/README.md` › fonts). `BRAHMA_WEB_FONTS_DIR="$PWD/../craft-fonts" packaging/web/package.sh` puts craft-fonts there. served families are in the font menus and are fetched when picked or when a document's text needs them; they aren't in the script fallback list yet, so japanese text set in a latin font still needs a japanese font picked (#1615).
- tests that need the fonts skip with a message when `CRAFT_FONTS` is empty; ci's linux job runs the tests a second time with `CRAFT_FONTS_DIR` set. desktop releases check out craft-fonts at the commit pinned in `.github/workflows/release.yml` (`CRAFT_FONTS_REF`; ci.yml pins the same commit) and ship each font's `OFL.txt` as `OFL-<family>.txt`.

## graphics startup and device loss

`--safe-gpu` starts with the CPU renderer for one launch (no GPU canvas; a software adapter for the window where the platform has one: WARP on windows, llvmpipe over gl on linux). before creating the wgpu device the app writes and locks `gpu-starting.json` in the config directory; it clears it once the first frames have rendered. a launch that finds an unlocked marker knows the previous start died inside the graphics driver (#4) and uses the next safer backend (windows: vulkan → DX12 → CPU; linux: vulkan → gl → CPU; macos: metal → CPU), remembering a working hardware backend in `performance.gpuBackend` (preferences › performance › GPU backend, with **reset GPU backend**). CPU recovery is temporary until **keep using CPU** is chosen in the recovery dialog; dismissing the dialog or quitting without a choice preserves the previous GPU preference (#3483). with `auto`, intel adapters on windows use DX12. help › system info shows the adapter, backend, driver and fallback state.

on DX12 the shader compiler is FXC (`d3dcompiler_47.dll`, part of windows), or a `dxcompiler.dll` placed beside `brahma.exe`, loaded by its full path. wgpu's default looks `dxcompiler.dll` up by name, which reaches the current directory and `PATH` and loaded other programs' incompatible builds (#712).

handled windowing and app initialization errors clear this launch's startup marker, restoring any
prior crash evidence. for example, a linux launch without `DISPLAY` or `WAYLAND_DISPLAY` does not
change the next launch's graphics backend. renderer initialization errors and driver crashes still
keep the marker for recovery.

if the device is lost while running (#243), every GPU entry point checks the device's health flag first, the canvas switches to the CPU compositor for the rest of the session and a notice says "GPU device was lost; using the CPU renderer." `ui.gpu.simulateLoss` triggers this path from the control channel.

## logs

the desktop app writes its `log` records to standard error and to `logs/brahma.log` in the settings directory (linux `~/.config/brahma/logs/`, macos `~/Library/Application Support/Brahma/logs/`, windows `%APPDATA%\Brahma\logs\`, or under `BRAHMA_CONFIG_DIR` / the portable data folder). a start launched from a desktop menu or the dock has no terminal, so this file is what to attach to a bug report: GPU startup fallbacks, a lost device and the crash guard's panic report all land there. each launch moves the previous log to `brahma.1.log` (and that one to `brahma.2.log`), so the log of a run that crashed survives the next start. the file stops growing at 16 mib. `--version` and command-line errors write no file.

by default brahma's own crates log at `info` and everything else at `warn`. `RUST_LOG` replaces that with env_logger-style directives, for example `RUST_LOG=debug`, `RUST_LOG=warn,brahma_gpu=trace` or `RUST_LOG=info,wgpu_core=warn`; a directive ending in `*` covers every target starting with it (`brahma*=debug`). the logger is `apps/brahma/src/logging.rs`; the web build logs to the browser console instead.

## environment variables

| variable | effect |
|---|---|
| `BRAHMA_CONTROL_PORT` | same as `--control <port>` |
| `BRAHMA_NO_PRINT` | set to anything but `0`: `file.print` / `file.printOneCopy` refuse to spool (`dryRun` still works). ci sets it, with `LPDEST=brahma-no-such-printer` as a second net; set both when running the panic hunt or fuzzers locally |
| `RUST_LOG` | log levels for standard error and the log file (see [logs](#logs)) |
| `BRAHMA_CONTROL_TOKEN` | 64-hex bearer token for control TCP (avoid on shared systems where environment inspection is possible) |
| `BRAHMA_CONTROL_TOKEN_FILE` | read, or create for a server, the control bearer-token file |
| `BRAHMA_AUTOMATION_READ_ROOT` | desktop app only: automation read root. headless CLI modes require the `--automation-read-root` flag |
| `BRAHMA_AUTOMATION_WRITE_ROOT` | desktop app only: automation write root. headless CLI modes require the `--automation-write-root` flag |
| `BRAHMA_CPU_CANVAS=1` | force the CPU canvas path instead of the wgpu shader canvas |
| `BRAHMA_NATIVE_WAYLAND=1` | linux: stay on native wayland when a pen is attached (by default the window then opens through xwayland, because wayland gives the app no pen input; #639). a pen is an input device with `BTN_TOOL_PEN` and the `ABS_X`/`ABS_Y` axes; virtual keyboards that claim every key are not pens (#1072) |
| `WGPU_BACKEND=dx12` | pick the wgpu backend(s) (`vulkan`, `dx12`, `metal`, `gl`); overrides `performance.gpuBackend` and the startup fallback |
| `WGPU_DX12_COMPILER=fxc` | DX12 shader compiler (`fxc`, `dxc`, `auto`); `dxc` and `auto` look `dxcompiler.dll` up through the DLL search path |
| `BRAHMA_GPU_TILE=2048` | force GPU canvas tiling (tests tile seams) |
| `BRAHMA_FX_NOCACHE=1` | bypass the CPU layer-effect map cache (`compose::effect_maps`) |
| `BRAHMA_CPU_COMPOSE=1` | keep the wgpu canvas but composite on the CPU (compare GPU vs CPU renders, e.g. with the snapshot example) |
| `BRAHMA_LOCALE` | override auto's native ui language for this launch; unsupported tags use english. see [system language detection](localization.md#first-launch-and-system-language). |
| `BRAHMA_FX_TRACE=1` | print the CPU time spent on GPU effect shapes and distance fields per rebuild |
| `BRAHMA_STROKE_TRACE=1` (or `=<file.csv>`) | time every frame of each brush stroke; print a summary per stroke, and with a path also append each frame to that CSV (see [stroke latency trace](#stroke-latency-trace)) |
| `BRAHMA_THEME_FILE=tokens.json` | **debug builds only:** live design-token overrides, re-read on change |

### live design tokens

```json
{ "card": "#323232", "tab_strip": "#262626", "accent": "#378ef0", "radius": 4 }
```

keys are the field names of `theme::Tokens` (`crates/ui-egui/src/theme.rs`). edit and save the file while the app runs to see changes immediately, with no recompile. this is compiled out of release builds.

### stroke latency trace

to find where a stroke's input-to-pixel time goes on a given machine (#1343), launch with `BRAHMA_STROKE_TRACE=1` (a summary per stroke on standard error) or `BRAHMA_STROKE_TRACE=strokes.csv` (also every frame, appended to that file). paint a few strokes with the mouse or pen. each summary gives p50/p95 in ms:

```text
stroke trace: 62 frames on vulkan NVIDIA GeForce RTX 5090 (p50/p95 ms): interval 8.33/8.53 moves/frame 1.00/2.00 stroke 0.80/1.53 canvas 0.40/0.75 ui 0.91/1.41 cpu 2.20/2.84 wait 6.04/6.64
```

| column | what it is |
|---|---|
| `interval` | time between frame starts while drawing (the display's refresh period when the app keeps up) |
| `moves/frame` | pointer moves the os delivered per frame (control-channel `ui.pointer` input bypasses raw input and counts 0) |
| `stroke` | rendering the stroke's new points (the live stroke; on the last frame, the commit) |
| `canvas` | canvas refresh: composite and upload |
| `ui` | building the ui, the two above included |
| `cpu` | eframe's CPU time per frame: ui, tessellation and GPU submission, without the vsync wait |
| `wait` | `interval − cpu`: waiting on vsync or the swap chain, or idle |

what the app can't time is the rest of input-to-photon: the os input queue before a frame starts (up to one `interval`), frames queued for the display (the surface keeps up to two; the gl backend ignores that setting), and the compositor. so the first line names the backend. the last summary is also in the control channel's `perf.strokeTrace` (`ui.inspect`). the code is `crates/ui-egui/src/stroke_trace.rs`.

## driving the app programmatically

start the app with a private token file. it then accepts authenticated JSON lines on `127.0.0.1:7878`:

```sh
TOKEN=$(tr -d '\r\n' < .private/control.token)
printf '%s\n' "{\"id\":\"auth\",\"method\":\"auth\",\"params\":{\"token\":\"$TOKEN\"}}" \
              '{"id":1,"method":"engine.execute","params":{"command":"layer.newAdjustmentLayer.hueSaturation","params":{"hue":30}}}' \
              '{"id":2,"method":"ui.screenshot","params":{"path":"evidence/shot.png"}}' | nc 127.0.0.1 7878
```

see `docs/control-protocol.md` for every method. tips:

- **macos does not render occluded windows.** `ui.screenshot` raises the window first (`focus: true` by default). use `ui.focus` before other visual checks.
- **capture only our own window.** for native captures (to see the real title bar and traffic lights), capture by window id (`screencapture -l <CGWindowID>`), never a screen region, which can grab other apps.
- **pointer gestures:** `ui.pointer` takes events in *document* coordinates and runs them through the same tool state machine as the mouse.
- **seeing tile storage:** `ui.set {"tileGrid": true}` overlays the active layer's stored tiles, which shows how sparse a layer is and which tiles share pixels (`ui-egui/src/tile_overlay.rs`).

## headless CLI

`apps/brahma-cli` builds the binary `brahma-cli`:

```sh
cargo run -p brahma-cli -- convert in.psd out.pcraft               # any supported format -> any
cargo run -p brahma-cli -- info out.pcraft                          # JSON: size, mode, depth, layer tree
cargo run -p brahma-cli -- run in.png --cmd layer.new.layer --params '{"name":"Ink"}' \
                                          --cmd filter.blur.gaussianBlur --params '{"radius":3}' --out out.psd
cargo run -p brahma-cli -- run --new '{"width":800,"height":600}' --cmd document.inspect
cargo run -p brahma-cli -- batch --actions actions.json --in photos/ --out done/ --format jpg
cargo run -p brahma-cli -- commands --filter blur                    # the command registry
```

`actions.json` holds the steps of a recorded action: `[["<id>", {…}], …]`, `[{"command": "<id>", "params": {…}}, …]` or bare ids, as a list or wrapped in `{"actions": …}` or `{"steps": …}`; a droplet file works too. it can also be an actions set (the actions panel's `actions.json`, `{"actions": [{"name": …, "steps": […]}, …]}`): `--action <name>` picks the action to play (optional when the set holds one), and the whole set is loaded for each file, so a step that plays another action of the set works. `batch` refuses an `--out` folder that is the `--in` folder, since the results would replace the originals, unless `--in-place` is given. `run` prints one JSON line per command result.

## native format (.pcraft)

`crates/format` defines the lossless native bundle. it is either a ZIP of STORE entries or a directory with the same layout:

- `manifest.json`: the versioned document tree, with migrations in `format/src/migrate.rs`.
- `tiles/<blake3>.zst` and `blobs/<blake3>.zst`: zstd-compressed objects, content-addressed by the BLAKE3 hash of their uncompressed bytes.
- `thumb.png` and `composite/preview.png`: previews.

keep one `PcraftWriter` per open document: re-saving then only compresses and writes tiles that changed. directory bundles verify objects on first encounter in a folder; later saves reuse them while their file size and modification time are unchanged, and re-verify changed objects, repair missing or damaged objects, and garbage-collect unreferenced ones. `format::Autosaver` writes snapshots into a recovery directory on a background thread. `list_recovery` / `recover` / `discard_recovery` implement crash recovery, and `format::RecoveryStore` is the lifecycle the desktop app uses: new documents autosave under per-launch keys (document ids restart every launch, so they never overwrite an older entry), and a recovered document adopts the entry it came from. that entry is replaced in place by the next autosave and removed only when the document is saved or closed, never just because it was recovered, so a second crash loses nothing. the web build has no crash recovery (no autosave services).

at startup the desktop lists recovery metadata, then decodes one document at a time through the existing background-job system. the window stays usable, with recovery progress and cancel in the status bar. cancel skips the rest of that launch's recovery queue and prevents the current result from opening; the decoder may finish in the background. failed or cancelled loads leave their recovery files intact for a later launch. turning off preferences › file handling › recover on launch skips discovery altogether.

`brahma-io` routes `.pcraft` through this crate in `import`/`export`, detecting it by magic or by extension.

## MCP (agents)

`brahma-cli mcp` serves MCP on stdio using `crates/automation`, which is built on `rmcp`:

- **headless:** `brahma-cli mcp --automation-read-root <dir> --automation-write-root <dir>`. it drives an in-process engine session and has no file authority when a root is omitted.
- **live app:** start `brahma --control 7878 --control-token-file <private-path> --automation-read-root <dir> --automation-write-root <dir>`, then run `brahma-cli mcp --bridge 127.0.0.1:7878 --control-token-file <private-path>`. the desktop process owns the roots. see `docs/control-protocol.md#mcp-bridge`.

for headless MCP clients, pass absolute paths to deliberately chosen trusted workspace directories in the launch arguments. desktop `BRAHMA_AUTOMATION_READ_ROOT` and `BRAHMA_AUTOMATION_WRITE_ROOT` environment variables do **not** grant headless CLI access. omitting a root flag deliberately denies that direction of access.

tools:

- `session_list`
- `doc_open`, `doc_new`, `doc_place`, `doc_export_layers`, `doc_export_data_sets`, `doc_import_data_sets`, `doc_save`, `doc_export`, `doc_inspect`, `doc_render_preview` (returns a PNG image), `doc_select`, `doc_close`
- `command_list`, `command_run`, `command_batch` (several commands per call)
- `jobs_list` (running background jobs with progress, then recently finished ones with their result or error), `jobs_cancel` (one job by id, or every running job)
- bridge only: `ui_inspect`, `ui_screenshot`, `ui_pointer`, `ui_menu_invoke`, `ui_set`, `control_call`

claude code (`.mcp.json` in the repo root, or `claude mcp add`):

```json
{
  "mcpServers": {
    "brahma": {
      "command": "/path/to/brahma/target/release/brahma-cli",
      "args": ["mcp", "--automation-read-root", "/absolute/path/to/trusted/workspace", "--automation-write-root", "/absolute/path/to/trusted/workspace"]
    },
    "brahma-live": {
      "command": "/path/to/brahma/target/release/brahma-cli",
      "args": ["mcp", "--bridge", "127.0.0.1:7878", "--control-token-file", "/private/path/brahma-control.token"]
    }
  }
}
```

```sh
cargo build --release -p brahma-cli
mkdir -p "$PWD/brahma-work"
claude mcp add brahma -- "$PWD/target/release/brahma-cli" mcp \
  --automation-read-root "$PWD/brahma-work" \
  --automation-write-root "$PWD/brahma-work"
```

`doc_inspect` (and the engine command `document.inspect`) reports the layer tree with kinds,
bounds, masks, selection, effects (`effects.items[].kind`), smart filters (`smartFilters[]`), smart
object sources (`smartSource`: `embedded` with its file name, or `linked` with a file path or, for a
PSD placed layer, the `Idnt` uuid its duplicates share), shared contents (`smartContents`: an `id`
every copy of the same contents has, and the number of `instances`; replace contents changes them
all and lists them in its result), type
text, adjustment settings, channels and history, so agents can verify what they did without a
screenshot. `crates/automation/tests/agent_tasks.rs` is the reference: ten realistic edit tasks
(title card, colour grade, undo/redo, editable smart blur, masks, saved selections, align,
capability-scoped export, resize/crop, CMYK + native save) driven purely over MCP.

without MCP, `brahma-cli serve [--port N]` keeps a headless session open and answers JSON lines
(see `docs/control-protocol.md#headless-server`).

a typical agent loop (place inputs under the configured read root and outputs under the write root):

1. `doc_open {path}`
2. `command_list {filter:"blur"}`
3. `command_run {id:"filter.blur.gaussianBlur", params:{radius:4}}`
4. `doc_render_preview` to check the result
5. `doc_save {path:"out.pcraft"}`

## colour management

`crates/cms` is our own pure-rust ICC engine (v2/v4 parsing, matrix/TRC and LUT profiles, all four
intents, black point compensation). it ships CC0 built-in profiles, including a synthetic
"brahma coated CMYK", because adobe's CMYK profiles are proprietary (see `crates/cms/README.md`).

- documents carry an optional embedded ICC profile (`Document::icc_profile`); `edit.assignProfile`
  and `edit.convertToProfile` change it. mode changes (`image.mode.*`) convert through cms.
- **proof colors** (⌘y), **proof setup** and **gamut warning** (⇧⌘y) bake a 3d LUT
  (`cms::Lut3d`) that the canvas shader applies; the document pixels never change.
- convert colours with `brahma_cms::transform::cached(src, dst, opts)`: transforms are cached
  process-wide and integer buffers use precomputed tables or a device link.

## menu parity

`cargo xtask parity` compares photoshop's menu tree (`crates/ui-egui/src/menu_catalog.rs`) with
the live command registry (`menus::is_live`) and rewrites [`docs/parity-checklist.md`](parity-checklist.md). the test
`parity::tests::parity_does_not_regress` fails if the live count drops below `parity::FLOOR`.

`cargo xtask i18n-coverage` prints the ui translation coverage for each language in the
`crates/ui-egui/src/i18n/mod.rs` registry, in stable language-code order. its english-key set is
derived from ui `tl!` literals, the menu catalog and ui command table, and menu command labels in
the engine source. catalog rows are validated for malformed and duplicate keys; unused legacy
translations do not affect the denominator. unregistered locale TSV files are ignored.

## testing strategy

- **unit and property tests** in every crate. proptest is used for tile COW, regions and codecs.
- **format crates:**
  - synthetic generators (`psd::testgen`)
  - byte-exact round trips
  - malformed-input sweeps (truncate at every offset)
  - fuzz targets (`crates/*/fuzz`)
- **real-file corpora** in `corpus/` (gitignored, fetched at pinned commits, sha256-verified): opt-in locally through the `corpus` cargo feature, always run in ci. see [test corpora](#test-corpora).
- **composite oracle:** a PSD's embedded merged image is compared with our compositor's output. the pass rate is tracked in the roadmap.
- **ui:** unit tests for widgets and state, plus screenshot checks through the control channel.

## performance notes

- the canvas is presented by a custom WGSL shader (`ui-egui/src/gpu_canvas.rs`): mip-mapped/nearest sampling, procedural checkerboard, pixel grid, tiling. brush strokes upload only their damage rect.
- **the canvas composites on the GPU** (`brahma-gpu`, driven from `gpu_canvas.rs`), layer effects included. what the planner can't express returns `Unsupported` and the canvas falls back to the CPU compositor (`brahma-compose`, also the reference for export and tests): multichannel documents, and documents or effect regions over the texture limit. pieces the GPU can't derive itself are rasterised once on the CPU and cached (`compose::masks` for vector masks, `compose::shape_split` for stroked shapes with clipped layers, effect distance fields). timings of the interactive paths: `cargo run --release -p brahma-ui-egui --example interactive_bench`; effects: `--example fx_bench` (`--compare files…` for GPU vs CPU); the cost of changing a chisel bevel's size (a dragged slider, GPU and CPU, ellipse and text): `--example bevel_bench`; large documents (open, refresh, thumbnails, a filter, a stroke, saves; one operation per run so `/usr/bin/time -l` gives its peak memory): `--example large_image_bench -- --size 14000x14000 --op psd`. the app requests the adapter's own texture limit (egui's default is 8192 px; see `gpu_canvas::use_adapter_limits`); beyond it the CPU fallback composites and uploads in bands (`compose::render_bands`), and exports, thumbnails and flattening stream bands too, so no full-size float composite is ever held. rendering fidelity: `cargo run --release -p brahma-io --example oracle_diff -- corpus/psd` (the whole PSD oracle table in seconds). `ui.inspect` → `perf.timings.gpuFallback` names the reason (`null` on the GPU path).
- **layer effects on the GPU** (`gpu/src/fx.rs`, kernels in `gpu/src/compose.wgsl`). every enabled effect becomes a *map program* over the layer's effect region (shift, dilate, gaussian blur, glow ramp, bevel height and shading, contour, stroke band), mirroring `compose::effects` step by step; the chunked composite then paints through the maps, clipped to that region, and copies the result back into the backdrop in place, so a small text layer costs only its own pixels. the layer's shape (`compose::layer_shape`) and its distance fields (`compose::effects::distance_field`: a transform whose tie-breaking a parallel GPU pass can't reproduce bit for bit) come from compose on the CPU: bands computed in parallel, except the euclidean fields of precise glows and chiselled bevels (a halo as tall as the bevel size would make most of a band's work redundant), which are one window whose passes run on all threads, the CPU compositor's own way. everything is cached per layer state: an unrelated edit, or an effect's colour or opacity, rebuilds nothing; a brush dab recomputes the touched 256² tiles grown by the effect reach; changing one effect's geometry rebuilds that effect only. cache budget `gpu::FX_BUDGET` (1.5 gb, least recently drawn layers evicted first). `BRAHMA_FX_TRACE=1` prints the CPU time of each rebuild.
- the canvas grows a stroke's damage rect by the effect reach of the layers around it (`canvas::effect_reach`), so effects beyond the dab refresh too (on both paths).
- **numbers** (7360 × 4912, 8 text layers + one painted layer with drop shadow + stroke + bevel, hue/saturation on top; m4 pro; `cargo run --release -p brahma-ui-egui --example fx_bench`), CPU fallback → GPU: full refresh with warm effect maps 5.7 s → 47 ms; hue/saturation tweak above the effects 4.7 s → 31 ms; brush dab on a plain layer 32 → 1.7 ms; brush dab on the effect layer (its maps rebuilt around the dab) 659 → 3.7 ms; first refresh (all maps built) 6.4 s → 0.39 s. with the CPU ~14× oversubscribed by parallel builds (min of 7 runs): 8.1 s → 0.33 s, 9.9 s → 0.28 s, 28 → 2.2 ms, 1.7 s → 89 ms; moving a text layer 7 px 336 → 13 ms. `fx_bench --compare corpus/psd/…/*.psd` reports the GPU vs CPU difference on real files (30 of the 31 corpus files with effects render on the GPU, worst 0.12/255).
- on the CPU path `compose::effect_maps` caches shadow, glow, bevel and satin maps per layer state (LRU, 768 mb budget).
- live adjustment previews on large documents use a downsampled proxy (`ui-egui/src/proxy.rs`).
- **content-aware tools** (`algo::inpaint`, `algo::content_aware`: fill, spot healing, patch, content-aware move). `cargo run --release -p brahma-engine --example content_aware_bench -- bench caf|heal|cancel` times the P21, P22 and P49 rows without the ui, one sample per line, so two builds can be run in turn and pooled. `-- hash` prints a hash of every tool's output at 8, 16 and 32 bits and in gray, CMYK and lab: the fill is fitted to photoshop, so a speed-up must leave both builds' lists equal. the solver works only over the hole's bounds grown by a patch radius (nothing else changes or is matched). PatchMatch sweeps its bands in wavefronts when there are fewer bands than threads; each cell sees the same neighbours and random numbers as in raster order, so the result does not depend on the thread count. cancellation is checked per row: a 24 mp fill stops within tens of milliseconds, most of it spent freeing its buffers.
- `ui.inspect` returns `perf` timings (ui ms per frame, composite ms, upload ms).
- never scan full surfaces per frame. cache per document revision (`BrahmaApp::cached_bounds`). an uncached `content_bounds()` on a 36 mp layer once cost 77 ms per frame.

## scorecard and performance budgets

[`docs/scorecard.md`](scorecard.md) says, with numbers, where brahma stands per area. it is
generated; never edit it by hand.

```sh
cargo xtask scorecard            # regenerate docs/scorecard.md
cargo xtask scorecard --check    # what CI runs: fails if the committed file is stale
cargo xtask perf                 # full benchmark run (tens of minutes, release build)
cargo xtask perf --quick         # small synthetic documents (about a minute once built)
cargo xtask perf --update-baseline   # also write perf/baseline.json from this run
```

**sources.** the scorecard reads only committed files, so it is deterministic:
- `scorecard/*.toml`: one checklist per area (tools, file compatibility, ui, type, automation,
  reliability, distribution, open bugs). each item has an `id`, a short `target`, a `status`
  (`done`, `partial` or `missing`), the `issue` and a `note` with the evidence (file:line).
  statuses must be true on `main`: verify against the code, and say `partial` with a note when
  unsure. flip a row in the pr that changes it.
- `perf/budgets.toml`: the benches `xtask perf` runs and the scenarios p1…pn, each mapped to one
  bench row, with a budget (targets from #209, #210, #211). `enforce = false` marks a target the
  code doesn't meet yet (reported as over budget; only regressions fail); flip it to `true` in the
  pr that meets it. scenarios the code can't support yet carry `not_measurable` and a reason, never
  a number.
- `perf/baseline.json`: the numbers the scorecard shows, written only by
  `cargo xtask perf --update-baseline`, with the machine (CPU, RAM, GPU adapter, os), its machine
  class, commit, date and load average.
- `crates/io/tests/corpus.rs`: the corpus floors (every `Source { .. }` constant).
- the prefs audit: `Preferences` fields that no code reads (target 0, #204), plus counts taken
  from the tree (never-crash attribute coverage, `docs/parity-checklist.md`).

**perf runs.** `xtask perf` builds the benches in release (`perf_scenarios`, `interactive_bench`,
`fx_bench`, `bevel_bench`, `type_bench`, `large_image_bench`, and `layout_bench` once it exists), runs each with
`--json`, and merges the reports into `target/perf/results.json` keyed by scenario id, with p50,
p95 and max (nearest rank over the samples), peak RSS measured in-process (`brahma-testkit`'s
`perf::RssSampler`), GPU bytes held by the canvas, and the load average before and after each
bench. `target/perf/summary.md` is the markdown table. it exits non-zero when an enforced budget
breaks, when a scenario's p50 regresses more than 15 % (`--threshold`, or `regression_pct`)
against a baseline from the same machine class and mode, or when a bench fails. runs from
different machine classes are never compared, and `--quick` numbers are only compared with a
quick baseline. machines that run other work give noisy numbers: check the load average in the
summary before trusting a regression, and record it next to any number you quote.

**adding a scenario.** add a row to a bench (keep its name stable: it is the key), give the
bench `--json` support through `brahma_testkit::perf::{row, report, write_report}`, then add a
`[[scenario]]` to `perf/budgets.toml` and run `cargo xtask scorecard`.

**ci.** `ci.yml` runs `cargo xtask scorecard --check`. `perf-nightly.yml` runs `cargo xtask perf`
on a fixed macos runner every night (and on demand), posts the table as the job summary, uploads
`results.json`, and fails on a broken budget or regression. it never runs on pull requests: the
release build of the benches alone takes longer than pr ci should. to give the nightly runner
its own baseline, run the workflow by hand with `update_baseline` and commit the `perf-baseline`
artifact as `perf/baseline.json`.

## web build

`apps/brahma-web` runs the same `BrahmaApp` in the browser through eframe's web runner. the renderer is wgpu: webgpu where the browser has it, webgl2 otherwise. it is rust only. the only JavaScript is the glue that wasm-bindgen generates.

```sh
brew install trunk                 # or: cargo install trunk --locked
cd apps/brahma-web
trunk build --release              # writes ../../dist/web (index.html, .js glue, .wasm)
trunk serve --release              # dev server on http://127.0.0.1:8765
```

any static file server works for `dist/web`, for example `python3 -m http.server 8765` run inside that directory. trunk downloads the matching `wasm-bindgen` and `wasm-opt` itself. `trunk build --release` uses the `wasm-release` cargo profile (`data-cargo-profile` in `index.html`: fat LTO, opt-level "z" for the ui, engine, i/o and web shells and for large crates off the hot paths (document model, file parsers, JSON, SVG parsing, shader translation), "s" for other crates except pixel processing, which keeps level 3). the ui override reduces browser code size while retaining the speed-oriented settings for filters and compositing. serve the `.wasm` with compression; its raw size varies with the source and compiler. keep it under 24 mib (`packaging/web/package.sh` enforces this; cloudflare's per-file cap is 25 mib). pull-request ci also builds the actual browser app with the same pinned trunk version as the release job and runs `packaging/web/package.sh`, including the relative-URL and 24 mib checks. the `web build and wasm size gate` check runs on main and on prs that change something the web build compiles or packages (`apps/brahma-web`, `packaging/web`, crate sources and manifests, `Cargo.toml`/`Cargo.lock`, the ci workflow), and reports the resulting wasm size in the actions job summary. the web build never embeds craft-fonts (see fonts above). to see where the bytes go, run `twiggy top -n 40` on `target/wasm32-unknown-unknown/wasm-release/brahma-web.wasm` (before wasm-opt strips the names).

URL flags: `?webgl` forces the webgl2 backend, and `?cpu` forces the CPU canvas path.

to build and serve the web app entirely in docker, run from the repository root:

```sh
docker build --load -t brahma-web:local .
docker run --rm -p 8080:8080 brahma-web:local
```

open http://localhost:8080/. see [docker hosting](../packaging/web/README.md#docker) for
HTTPS/reverse-proxy deployment and browser limitations.

how the web shell (`apps/brahma-web/src/web.rs`) differs from desktop:

- **open** uses `rfd::AsyncFileDialog`. the bytes arrive asynchronously in `Services::inbox`, which the app drains every frame.
- **save / save as / export** trigger a browser download of the encoded bytes. the shell does this with a blob, an object URL and a temporary `<a download>`, all created from rust. there is no save dialog, so the suggested name becomes the download name.
- **drag-and-drop:** `WebShell` takes the frame's `dropped_files` before the app sees them. it reads each file with `DroppedFile::bytes_async` and pushes the bytes into the inbox.
- **startup failures:** `index.html` shows "loading brahma…" until the app removes it, or replaces it with the error when the web runner fails to start. if the `.wasm` never downloads or doesn't match the page's `.js` (a stale cached `index.html` after a deploy), no rust runs at all, so the page itself (CSS only, no script) adds a hint with a reload link after 20 seconds.
- **no control server:** browsers can't listen on TCP. to automate the web build, drive headless chrome with `--remote-debugging-port`. `Page.setInterceptFileChooserDialog` plus `DOM.setFileInputFiles` covers open, `Input.dispatchDragEvent` with `files` covers drops, and `Browser.setDownloadBehavior` captures downloads.
- headless chrome on macos (`--headless=new --enable-unsafe-webgpu`) gets a real webgpu adapter.


## indexed color timings

`cargo run --release -p brahma-engine --example indexed_color_perf -- image.png` measures
palette construction, floyd–steinberg diffusion, and the complete full-resolution CPU preview
path (proxy copy, engine command, result composition; excludes GPU upload/presentation).
it prints JSON lines for 8, 16, 32, 64, 128 and 256 colours, with one warmup and three measured
runs per count, plus palette and pixel hashes for exact before/after comparisons. input files
stay local; do not publish personal images, palettes, or metadata with benchmark reports.

the synthetic 24 mp lookup comparison is reproducible with
`cargo test --release -p brahma-algo indexed_diffusion_24mp_release_comparison -- --ignored --nocapture`.
it alternates linear/accelerated lookup order and checks identical indices and pixels.
for the 2026-10-09 local timings both binaries used `CARGO_PROFILE_RELEASE_LTO=false` and
`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16`; compare binaries built with the same settings.

on native desktop sessions, indexed color live previews calculate and flatten on one worker.
the last preview remains visible while dragging; intermediate settings are coalesced, and only
a result matching the current document revision, active layer, dialog and parameters is uploaded.
GPU upload/display conversion still run on the ui thread. web and `BRAHMA_INLINE_JOBS=1`
sessions retain synchronous previews. closing the dialog invalidates its result; an already
running calculation finishes without editing the session.

## offscreen ui snapshots (no window)

render the full ui headlessly with the real wgpu canvas, e.g. for design reviews or when the app
window would be occluded (macos doesn't render occluded windows, so live `ui.screenshot` must raise
the window and steal focus):

```sh
cargo run --release -p brahma-ui-egui --example snapshot -- \
    --out ui.png --size 1440x900 --scale 2 --open photo.jpg \
    --script '[["ui.set", {"tool": "type"}], ["ui.menu.invoke", {"id": "image.canvasSize"}]]'
```

`--script` is a list of control-protocol calls (`[method, params]`), applied in order.
input calls (`ui.key`, `ui.type`, `ui.click`) now reply only after the app has processed the events,
so a following `ui.inspect` observes their effect.


## photoshop actions (.atn)

`brahma-psd` reads and writes photoshop actions files (`atn`, version 16: photoshop 6 and
later); `brahma-engine`'s `atn_cmds` maps their events onto commands (`actions.load`,
`actions.save`; the actions panel menu's load/save actions…; `brahma-cli batch --actions
set.atn`). to see what a set holds (event ids, panel names, the parameter keys photoshop
recorded) when adding an event to the table:

```sh
cargo run -p brahma-psd --example atn_dump -- "Default Actions.atn"
```

events outside the table load as `actions.photoshopStep` steps that keep the descriptor
verbatim (so save actions writes them back) and fail at playback with the step's photoshop name.

## remove tool quality

`cargo run --release -p brahma-engine --example remove_quality -- <dir> [out dir]` fills
holes (discs and thin lines over background) in five photos with `brahma_algo::remove` and
prints, per hole, the PSNR against the original, `texture` (the fill's mean gradient over the
original's: below 1 it is smoother than what it replaces) and `seam` (the same on the band along the
hole's edge: above 1 the edge shows). with an out dir it writes original | hole | fill crops. the
photos are public domain, not committed; download them at 1920 px wide into `<dir>` as:

| file | photo |
|---|---|
| `01.jpg` | the tetons and the snake river, ansel adams, 1942 (u.s. national archives) |
| `02.jpg` | ansel adams, national archives 79-AAB-01 (u.s. national archives) |
| `03.jpg` | hiking at crane meadows national wildlife Refuge.jpg) (USFWS) |
| `04.jpg` | meadow anemone, lake andes national wildlife Refuge.jpg) (USFWS) |
| `06.jpg` | line5070, NOAA photo library (NOAA) |

`https://commons.wikimedia.org/wiki/Special:FilePath/<file name>?width=1600` serves each at the
size the cases were placed for (commons rounds it to 1920). timing on a 24 mp document through the
command: `cargo run --release -p brahma-engine --example remove_bench`.

## test corpora

real files are our best oracles, but they are large binaries, so they never go into this
repository. they live in `corpus/` (gitignored) and are fetched at **pinned commits** and checked
against committed **sha256 manifests**. all pins are in one place:
[`xtask/src/corpus_pins.rs`](../xtask/src/corpus_pins.rs).

| directory | what | source | manifest |
|---|---|---|---|
| `corpus/photoshop/` | 256 psds we authored with photoshop: smart filters, layer-style effect shapes, the text engine, adjustments in every mode and depth | https://github.com/storytold/brahma-corpus (ours, MIT or apache-2.0) | `xtask/photoshop-corpus.sha256` |
| `corpus/psd/` | 170 small psd-tools and ag-psd files, the mix most PSD tests use | psd-tools and ag-psd upstreams (MIT) | `xtask/psd-corpus.sha256` |
| `corpus/psd-tools/` | the complete psd-tools test set (309 files) | psd-tools upstream (MIT) | `xtask/psd-tools-corpus.sha256` |
| `corpus/heif/` | 9 small HEIC/HEIF files (checkerboards, RGB strips, a grid-tiled photo with EXIF/XMP, each with apple's decode as `.ref.png`; a 10-bit RGBA file with its source PNG), for the `heif` feature | heic-rs (MIT or apache-2.0) and pillow-heif (BSD-3-clause) upstreams | `xtask/heif-corpus.sha256` |
| `corpus/exr/` | the 5 deep openexr test images (scanline deep data with half colour and u32 id channels; 2.3 mb), checked against the id manifests of their upstream sidecars | openexr upstream at v3.5.2 (BSD-3-clause) | `xtask/exr-corpus.sha256` |
| `corpus/affinity/` | 21 public affinity documents and the 18 affinity 3 samples for #1606, with exported PNG references | vector-art and affinity-samples (CC0), afdesignload, jac21/branding and AssetStoreTemplate (MIT) upstreams | `xtask/affinity-corpus.sha256` |
| `corpus/pngsuite/` | PngSuite | schaik.com release archive (public domain) | (fixed archive) |

```sh
cargo xtask corpus                 # where each corpus lives, its pin, present or missing
cargo xtask corpus --all           # fetch everything missing or stale (cold: about 15 s; verified copies are left alone)
cargo xtask test-corpus            # fetch, then cargo test --release --features corpus (+ heif on codecs, io) on psd, codecs, io, engine
cargo xtask test-corpus -p io      # narrow to one crate (repeat -p for more)
cargo xtask test-corpus --changed  # only if psd, io, codecs, compose, gpu, text, format or affinity changed vs origin/main
cargo xtask test-corpus -- --nocapture   # pass arguments to the test binaries (per-file tables)
scripts/fetch-corpus.sh            # the same as cargo xtask corpus --all
```

**opt-in locally, strict when opted in, always in ci.**

- the corpus tests sit behind the `corpus` cargo feature of `brahma-psd`, `brahma-codecs`,
  `brahma-io` and `brahma-engine`, so plain `cargo test` neither compiles nor needs them.
  the HEIF ones also need the `heif` feature of `brahma-codecs`/`brahma-io` (test-corpus
  turns it on).
- with the feature on, a missing corpus is a failure ("run `cargo xtask corpus --all`"), never a
  silent skip, and every floor is enforced.
- if you touch psd, io, codecs, compose, gpu, text or format, run `cargo xtask test-corpus` before
  you finish.
- the ci `corpus` job runs `cargo xtask corpus --all` and then `cargo xtask test-corpus` on every pr
  and on main. `corpus/` is cached with `actions/cache`, keyed on the hash of `corpus_pins.rs` and
  the manifests. a download happens only when a pin changes; otherwise the cache restores in seconds
  and is re-verified (sha256) before the tests run. fork prs read the base branch's cache.

**our own corpus: brahma-corpus.**

- **authoring clone:** a normal git clone, `git clone git@github.com:storytold/brahma-corpus.git`,
  placed next to this checkout (`../brahma-corpus`). its generators write there, and you commit
  and push there.
- **consuming copy:** `corpus/photoshop/` here. it is a plain directory: the pinned snapshot
  (downloaded as the codeload tarball of the pinned commit, then verified).
- **local mode:** `cargo xtask corpus --photoshop --local` copies `photoshop/` from
  `../brahma-corpus` (or `BRAHMA_CORPUS_REPO=<path>`) instead of downloading. it warns when
  the clone's HEAD isn't the pin or its files differ from the manifest. use it to test regenerated
  files before pushing: `cargo xtask test-corpus --local` copies them and runs every corpus test.
- **bumping the pin:** commit and push in brahma-corpus, then in a pr here set
  `BRAHMA_CORPUS_COMMIT` in `corpus_pins.rs`, run
  `cargo xtask corpus --photoshop --update-manifest`, commit the manifest diff, run
  `cargo xtask test-corpus`, and raise floors that improved.
- **not a submodule or a subtree:** a subtree would put the binaries back into this repository's
  history, and submodules cause init and detached-HEAD friction for every contributor. a pin plus a
  manifest gives the same reproducibility.
- the third-party sets (psd-tools, ag-psd, PngSuite) are fetched from their upstreams the same way,
  and are never copied into brahma-corpus.

## rendering fidelity (PSD oracle)

`cargo xtask test-corpus -p io -- --nocapture` compares our composite of every corpus PSD with
photoshop's own merged image (PASS ≤ 2/255) and checks that export → re-import renders the same,
per source with its own floors: `corpus/psd`, the psd-tools set (also a truncation/corruption sweep
over every file that must never panic) and our photoshop set `corpus/photoshop` (per-feature-group
totals: `smart-filters`, `effects`, `text`, `adjustments/<mode><bits>`). smart objects and type
layers composite photoshop's cached pixels there; `cargo xtask test-corpus -p engine -- --nocapture`
(`crates/engine/tests/photoshop_oracles.rs`) re-renders them with our smart-filter stack and text
engine and is the failure map for both. a panic is reported as `CRASH` and fails the run. files without a real merged image (maximize compatibility off)
are judged against their embedded thumbnail instead (`PASS (thumbnail)`, a strict low-resolution
check); a file only skips when it has no oracle at all. raise the floors in `crates/io/tests/corpus.rs` when they
improve; never lower them. synthetic reproductions of corpus findings live in
`crates/io/tests/corpus_regressions.rs` (corpus files are never committed). to dig into one file:

```sh
cargo run --release -p brahma-io --example oracle_diff -- corpus/psd/<file>.psd 0 png /tmp/diff.png
```

writes ours | photoshop | a diff heatmap side by side; `col`, `row`, `worst [n]`, `grid x0 y0 x1 y1 [ch]`,
`layerpx x y` and `DUMP_FX=1` print samples, the worst pixels, value grids, per-layer pixels and raw
effect descriptors. findings so far: fill-layer gradients are framed by the layer's mask bounds;
photoshop's gradient smoothness is a catmull-rom blend and "perceptual" interpolation is oklab
(baked into dense stops on import, `crates/io/src/gradient_bake.rs`); a shape layer's vector
stroke is drawn above its clipped layers; linked effect patterns tile from the layer's `fxrp`
reference point; stroke distances follow a 5 × 5 chamfer metric (1, √2, √5) seeded at sub-pixel
edge offsets; interior effects keep the layer's alpha; outside strokes blend onto the backdrop with
their own modes, an upper stroke covering lower ones.

## system printing

`file.print` and `file.printOneCopy` keep PDF layout, scaling, marks and ICC conversion in
brahma and submit the rendered bytes through `craft-print` at the git revision pinned in
`Cargo.toml`. CUPS streams the PDF on macos/linux; windows uses the shared windows PDF backend.
the driver receives exact physical sheet size so it does not scale brahma's layout twice.
windows raster printing does not establish physical-printer colour fidelity; test critical
photo output with the actual printer and its profile.

desktop commands run composition and submission on a background worker. the ui reports success
only when the spooler accepts the job; remove submitted jobs in the system print queue. headless
`Session::execute` stays synchronous. the returned `pdf` is `null` for a spool-only job, and
`command` is a diagnostic preview, not an executable command with a PDF path. supplying `output`
keeps a PDF at that path and defaults `send` to false. `dryRun` renders the PDF and reports the
backend command without submitting; it writes a PDF only when `output` is explicit. `send: false`
without an output requires `dryRun: true`. remembered print settings omit `output`, `dryRun` and
`send`, so print one copy derives submission from its own output choice. shared spooler transport
files are automatically removed on windows; CUPS needs none.
