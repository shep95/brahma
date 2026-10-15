<p align="center">
  <a href="https://github.com/shep95/brahma">
    <img alt="brahma" src="assets/app-icon/brahma-1024.png" width="160">
  </a>
</p>

<h1 align="center">brahma</h1>

<p align="center">
  <b>image editing; an open-source, clean-room reimplementation of adobe photoshop, rebuilt in pure rust.</b><br>
  layers, masks, adjustment layers, layer styles, type, vectors, brushes and real PSD files,<br>
  in a native app written entirely in rust. open source, offline, and yours.
</p>

<p align="center">
  <a href="https://github.com/shep95/brahma"><b>brahma on GitHub</b></a> ·
  <a href="https://github.com/shep95/brahma/releases">releases</a> ·
  <a href="https://github.com/shep95/brahma/discussions">#houseofasher community</a>
</p>

<br>

<p align="center">
  <img src="docs/images/brahma-demo.jpg" alt="brahma editing Hokusai's The Great Wave: a caption card with a drop shadow, Title and Credit type layers, Vibrance and Curves adjustment layers, and the Curves editor drawn over the image's histogram" width="100%">
  <br>
  <sub>a caption card with a drop shadow, live type, and vibrance and curves adjustment layers, with the curves editor open.<br>
  <i>the great wave off kanagawa</i>, katsushika hokusai, c. 1831</sub>
</p>

> [!NOTE]
> **#houseofasher is a community of artists from all walks of life.** digital, generative, music,
> games &mdash; if you make things, you're one of us. **[come say hi in the #houseofasher community](https://github.com/shep95/brahma/discussions).**

<p align="center">
  <a href="#features">features</a> ·
  <a href="#everything-in-the-box">everything in the box</a> ·
  <a href="#psd-without-compromise">PSD</a> ·
  <a href="#built-for-agents">agents</a> ·
  <a href="#under-the-hood">under the hood</a> ·
  <a href="#get-started">get started</a> ·
  <a href="#downloads">downloads</a> ·
  <a href="https://github.com/shep95/brahma/discussions">#houseofasher community</a>
</p>

<br>

<table>
  <tr>
    <td width="25%" valign="top">
      <h3>🎛️ Familiar by design</h3>
      The menus, shortcuts, panels and tools are where your hands expect them, from ⌘J to ⇧⌘D. If you know Photoshop, you already know brahma.
    </td>
    <td width="25%" valign="top">
      <h3>⚡ Native and fast</h3>
      A GPU compositor on wgpu (Metal, Vulkan, DX12, WebGPU), copy-on-write tiles and multithreaded filters. No Electron, no web view, no waiting.
    </td>
    <td width="25%" valign="top">
      <h3>🗂️ Real PSD files</h3>
      Open, edit and save layered Photoshop documents. Re-saving keeps the render of 307 of the 309 psd-tools test files.
    </td>
    <td width="25%" valign="top">
      <h3>🤖 Agent-ready</h3>
      Every action is a command, so you can drive the same engine from the UI, the CLI, a JSON control channel or an MCP server.
    </td>
  </tr>
</table>

<br>

## features

every screenshot here is the real app at work on public-domain art, rendered offscreen through its control channel.

<table>
  <tr>
    <td width="50%" valign="top">
      <img src="docs/images/brahma-adjustments.jpg" alt="Monet's Impression, Sunrise with Levels and Vibrance adjustment layers; the Levels editor and the Histogram panel with mean, standard deviation and median are open on the right" width="100%">
      <br>
      <sub>Levels and Vibrance adjustment layers, with the live Histogram panel.<br><i>Impression, Sunrise</i>, Claude Monet, 1872</sub>
      <h3>Edit without regret</h3>
      Adjustment layers keep every edit live. Stack Levels, Curves, Vibrance, Hue/Saturation and a dozen more, mask them to an area, reorder them, or turn them off, and your original pixels never change.
      <br><br>
      <b>16 adjustment layers</b> that also apply directly to pixels, including Curves with per-channel editing, Levels with a live histogram, Black &amp; White, Channel Mixer, Gradient Map, Photo Filter, Selective Color and Color Lookup (.cube, .3dl, .look). Plus Shadows/Highlights, Replace Color, Match Color, HDR Toning, Desaturate and Equalize.
    </td>
    <td width="50%" valign="top">
      <img src="docs/images/brahma-layer-styles.jpg" alt="The Layer Style dialog editing an Outer Glow on the EARTHRISE type layer, with Stroke also enabled, over the Apollo 8 Earthrise photograph" width="100%">
      <br>
      <sub>Outer Glow and Stroke on a live type layer, in the Layer Style dialog.<br><i>Earthrise</i>, William Anders / NASA, 1968</sub>
      <h3>Styles that sell the shot</h3>
      Drop Shadow, Inner Shadow, Outer and Inner Glow, Bevel &amp; Emboss, Satin, Stroke, and Color, Gradient and Pattern Overlay, live on any layer, including type. Patterns come from a library (built-ins, Edit › Define Pattern, <code>.pat</code> import/export) and PSD <code>Patt</code> blocks.
      <br><br>
      Copy and paste styles between layers, hide all effects at once, and open styles straight from your PSDs, rendered to match Photoshop.
    </td>
  </tr>
  <tr>
    <td width="50%" valign="top">
      <img src="docs/images/brahma-masks.jpg" alt="Vermeer's Girl with a Pearl Earring: an elliptical selection around the face, and a Hue/Saturation layer with an elliptical mask that turns everything outside the face gray" width="100%">
      <br>
      <sub>An elliptical selection becomes the mask of a Hue/Saturation layer, so only the face keeps its color.<br><i>Girl with a Pearl Earring</i>, Johannes Vermeer, c. 1665</sub>
      <h3>Selections that understand your image</h3>
      Marquees, lassos and the Magic Wand for precision; Quick Selection, Object Selection and Select Subject when you want the computer to do the tracing; Select and Mask to refine hair-fine edges.
      <br><br>
      Feather, expand, contract, smooth, grow, reselect. Turn any selection into a layer mask, a vector path or a shape. Smart selection runs on your machine, with no cloud and no account.
    </td>
    <td width="50%" valign="top">
      <img src="docs/images/brahma-type.jpg" alt="Bierstadt's Among the Sierra Nevada with the headline SIERRA NEVADA being edited on the canvas in Georgia, a byline, and a paragraph caption; the Character and Paragraph controls are open" width="100%">
      <br>
      <sub>A headline edited in place, with a byline and a paragraph of body text.<br><i>Among the Sierra Nevada, California</i>, Albert Bierstadt, 1868</sub>
      <h3>Type that sets beautifully</h3>
      Point and paragraph text, edited right on the canvas, with full Character and Paragraph controls: font, weight, size, leading, tracking, alignment and colour. The <a href="docs/color-picker.md">full Type Color Picker</a> also supports screen sampling with a pixel loupe.
      <br><br>
      Type layers stay editable, take layer styles, and round-trip through PSD.
    </td>
  </tr>
  <tr>
    <td width="50%" valign="top">
      <img src="docs/images/brahma-vector.jpg" alt="A lotus badge built from shape layers (Lotus, Water, Sun, Badge and Dotted Ring) over Monet's Water Lilies, with the lotus path's anchor points selected" width="100%">
      <br>
      <sub>A badge made of shape layers: a gradient-filled lotus, a star and a dotted ring.<br><i>Water Lilies</i>, Claude Monet, 1906</sub>
      <h3>Pixel-perfect vectors</h3>
      Rectangle, Ellipse, Triangle, Polygon, Line and the Pen, Freeform Pen and Curvature Pen tools, with resolution-independent shape layers, gradient fills, and dashed, aligned strokes.
      <br><br>
      Combine shapes (unite, subtract, intersect, exclude), keep paths in the Paths panel, use them as vector masks, or stroke and fill them. 116 of 116 shape layers in our PSD corpus match Photoshop's pixels.
    </td>
    <td width="50%" valign="top">
      <img src="docs/images/brahma-filters.jpg" alt="The Twirl filter dialog at a 320 degree angle, previewing live inside an elliptical selection on Van Gogh's The Starry Night" width="100%">
      <br>
      <sub>Twirl previews live on the canvas, only inside the selection.<br><i>The Starry Night</i>, Vincent van Gogh, 1889</sub>
      <h3>See it before you commit</h3>
      Every filter dialog previews live on the canvas, through your selection. Blurs (Gaussian, Box, Motion, Radial, Surface, Smart, Lens, Shape, and the Blur Gallery: Tilt-Shift, Iris, Field, Spin, Path), sharpening, Reduce Noise, distortions (Twirl, Wave, Ripple, Displace, Shear, Zig Zag…), Pixelate, Stylize (Oil Paint, Wind, Extrude…), Render (Clouds, Fibers, Lens Flare, Lighting Effects) and more.
      <br><br>
      Run filters on a smart object and they stay editable: change, hide, reorder or mask them at any time.
      <br><br>
      Large-radius blurs use running-sum box passes across all cores: a radius-180 Gaussian on 3.6 MP takes under a second.
    </td>
  </tr>
  <tr>
    <td width="50%" valign="top">
      <img src="docs/images/brahma-transform.jpg" alt="Free Transform handles around a rotated copy of Ansel Adams' The Tetons and the Snake River, with the History panel listing Open, Duplicate Layer, Free Transform and other steps" width="100%">
      <br>
      <sub>Free Transform on a rotated print, with every step listed in History.<br><i>The Tetons and the Snake River</i>, Ansel Adams, 1942</sub>
      <h3>Shape it any way you like</h3>
      Free Transform with scale, rotate, skew, distort and perspective; exact 90° and 180° rotations and flips; Transform Again. Layers, type, shapes, masks and selections all transform together.
      <br><br>
      Full history, Toggle Last State and the History Brush mean every step can be undone, even one brush stroke at a time.
    </td>
    <td width="50%" valign="top">
      <img src="docs/images/brahma-export-light.jpg" alt="The Export As dialog in the light theme over Klimt's The Kiss: JPG at quality 90, scaled to 50 percent, with a preview and an estimated size of about 684K" width="100%">
      <br>
      <sub>Export As in the light theme, with a preview and a file-size estimate.<br><i>The Kiss</i>, Gustav Klimt, 1907–1908</sub>
      <h3>Ship it anywhere</h3>
      Export As with format, quality, transparency and scale, plus a preview and an instant file-size estimate. Quick Export to PNG in one click.
      <br><br>
      Choose a dark Pro theme, the airy Studio themes, or a Classic look.
    </td>
  </tr>
</table>

<br>

## everything in the box

<table>
  <tr>
    <td width="33%" valign="top">
      <h4>🧰 67 tools</h4>
      Move · Artboard · Rectangular and Elliptical Marquee · Single Row Marquee · Single Column Marquee · Lasso · Polygonal Lasso · Magnetic Lasso · Object Selection · Quick Selection · Magic Wand · Crop · Perspective Crop · Slice · Slice Select · Frame · Eyedropper · Color Sampler · Ruler · Note · Count · Remove · Spot Healing Brush · Healing Brush · Patch · Content-Aware Move · Red Eye · Brush · Pencil · Mixer Brush · Color Replacement · Clone Stamp · Pattern Stamp · History Brush · Eraser · Background Eraser · Magic Eraser · Gradient · Paint Bucket · Blur · Sharpen · Smudge · Dodge · Burn · Sponge · Pen · Freeform Pen · Curvature Pen · Add Anchor Point · Delete Anchor Point · Convert Point · Horizontal and Vertical Type · Horizontal and Vertical Type Mask · Path Selection · Direct Selection · Rectangle · Ellipse · Triangle · Polygon · Line · Custom Shape · Hand · Rotate View · Zoom
    </td>
    <td width="33%" valign="top">
      <h4>🖌️ A real brush engine</h4>
      Shape Dynamics, Scattering, Texture, Dual Brush, Color Dynamics, Transfer, Brush Pose, Wet Edges, Build-up and Smoothing (including Pulled String), driven by pen pressure, tilt, rotation and direction. Brush presets, Define Brush from Selection, and deterministic, replayable strokes.
    </td>
    <td width="33%" valign="top">
      <h4>🗃️ Layers, done properly</h4>
      Groups, clipping masks, pixel and vector masks, fill layers (solid, gradient and pattern), adjustment layers, live smart objects with smart filters and lossless transforms and warps, multi-layer selection with align, distribute and link, alpha channels and Quick Mask, 27 Photoshop blend modes plus six Paint.NET variants, opacity and fill, locks, colour labels, a layer kind filter, merge, flatten, rasterize, Layer via Copy/Cut, Paste Into.
    </td>
  </tr>
  <tr>
    <td width="33%" valign="top">
      <h4>🎨 Any colour, any depth</h4>
      RGB, Grayscale, CMYK and Lab documents at 8, 16 and 32 bits per channel. Bit depth and colour model are runtime data, so every tool works at every depth.
      <br><br>
      Real ICC colour management in pure Rust: embedded profiles, Assign and Convert to Profile with all four rendering intents and black point compensation, soft proofing (⌘Y) and Gamut Warning (⇧⌘Y) on the GPU.
    </td>
    <td width="33%" valign="top">
      <h4>🗂️ Formats</h4>
      PSD and PSB, layered TIFF (Photoshop's layer data in the TIFF, read and written in either byte order), <a href="docs/pdn.md">Paint.NET PDN3 (read, with editable layers and all 14 blend modes)</a>, <a href="docs/ora.md">OpenRaster (read and write, with layers, groups and blend modes, for Krita, MyPaint and GIMP)</a>, <a href="docs/kra.md">Krita .kra (read, with paint layers, groups, blend modes and transparency masks)</a>, SVG (opens as shape layers, places as a vector Smart Object), plus flat PNG, JPEG, TIFF, WebP (lossy and lossless), GIF, BMP, TGA, ICO, QOI, PNM, OpenEXR, Radiance HDR and FITS (astronomy images: 8/16-bit and 32-bit float, gray or RGB, header kept), and the native <code>.pcraft</code> format. Flat formats retain the depths they support: PNG keeps 8/16-bit samples, TIFF keeps 8/16-bit and 32-bit float samples, and OpenEXR keeps half/float samples; JPEG, WebP and the other 8-bit formats reduce higher-depth samples with an export warning. See the <a href="crates/codecs/README.md#capability-matrix-default-build">codec capability matrix</a> for each format. HEIC photos from iPhone and Mac are read-only (included in official builds; optional <code>--features heif</code> for source builds). AVIF is write-only with the optional <code>avif</code> build feature; AVIF decoding and JPEG XL are unavailable. Affinity documents (<code>.af</code>, <code>.afdesign</code>, <code>.afphoto</code>, <code>.afpub</code>) open natively, read only, with artboards, layers, vectors, text, images and masks; effects, adjustments and other unsupported parts are listed in a warning, and Affinity export is unavailable.
    </td>
    <td width="33%" valign="top">
      <h4>🪄 The everyday essentials</h4>
      Auto Tone, Contrast and Color · Equalize · Image and Canvas Size · Crop and Trim · Reveal All · Edit › Fill and Stroke · Copy Merged · Paste in Place · guides, rulers, grid and snapping · Actions record and replay · a command palette (⌘F).
    </td>
  </tr>
</table>

<br>

## bring your own AI, no sign-up

brahma is free and stays free. it has no account, no sign-up and no server of its own. if you want AI
features, bring your own API key: edit › preferences › AI integrations takes a key for anthropic,
openai, google AI or any openai-compatible endpoint (a local server works too). the keys are saved
only on your device, in brahma's preferences file (`localStorage` in the browser), and are sent to
no one but the provider you pick. `ai.status` tells scripts and agents whether a provider is ready
without ever showing the key.

## PSD without compromise

brahma's PSD support is a standalone crate written from adobe's public specification and tested against a corpus of real-world files.

- **faithful round trips:** opening and re-saving a document renders the same for 307 of the 309 files in the psd-tools test set and 169 of 170 in our mixed ag-psd/psd-tools set (`crates/io/tests/corpus.rs`; fetch the psd-tools set with `cargo xtask corpus --psd-tools`), and anything we don't model yet (raw blocks, descriptors, extras) is carried over instead of being dropped. a re-saved file is not byte-identical to its source: brahma rewrites image resources, layer records and the composite. only the standalone `brahma-psd` crate, parsing and writing a file without the document model, reproduces every parseable corpus file byte for byte (`crates/psd/tests/corpus.rs`).
- **pixels that match:** a composite oracle compares our render with photoshop's own merged image, covering gradient interpolation (classic, perceptual and linear), layer effects, shape strokes, clipping and fill opacity.
- **large documents:** PSB, 16 and 32-bit files, and CMYK and lab documents open natively.

## built for agents

every menu item, tool and dialog runs a command from one registry of 500+ commands. the ui, the CLI, the JSON control channel and the MCP server all call the same commands, so anything you can click, a script or an ai agent can do too.

```sh
# Headless: open, edit, save
brahma-cli run wave.psd \
  --cmd filter.sharpen.smartSharpen     --params '{"amount":80}' \
  --cmd layer.newAdjustmentLayer.curves --params '{"points":[[0,0],[64,48],[192,212],[255,255]]}' \
  --out wave-final.png

# Apply one action list to a folder of images
brahma-cli batch --actions grade.json --in ./raw --out ./graded

# Every subcommand explains itself
brahma-cli batch --help

# Let an agent drive it over MCP (headless, or bridged to the running app)
brahma-cli mcp
```

the desktop app also offers an authenticated, loopback-only control channel (`brahma --control`) for inspecting ui state, driving tools with pointer events, and taking offscreen screenshots. every image in this README was rendered that way. see [`docs/control-protocol.md`](docs/control-protocol.md).

## under the hood

- **engine first:** a pure-data document model and a command engine, with a thin egui ui on top. layering is enforced at build time across 24 crates.
- **two compositors:** a CPU compositor serves as the reference oracle, and a wgpu compositor puts the canvas on the GPU. they are tested against each other.
- **copy-on-write tiles:** 256² sparse tiles make undo cheap and huge canvases light, and effect maps are cached per layer state.
- **runs in the browser:** the whole engine and ui compile to WebAssembly.
- **clean-room:** implemented from public specs and observed behaviour only. no proprietary code, shaders or assets.
- **tested:** more than 1,700 tests, including PSD round trips, synthetic generators, compositor oracles and multi-depth checks.

## get started

```sh
git clone https://github.com/shep95/brahma
cd brahma
cargo run --release -p brahma -- image.psd   # the desktop app
cargo test --workspace                           # the test suite
```

japanese fonts for the ui and type tool come from craft-fonts, an optional build input (desktop release builds always include it). without it brahma uses your system's CJK fonts:

```sh
git clone https://github.com/storytold/craft-fonts ../craft-fonts
CRAFT_FONTS_DIR="$PWD/../craft-fonts" cargo run --release -p brahma
```

new contributors and ai agents: start with [`AGENTS.md`](AGENTS.md), then [`docs/`](docs/).

installers for macos, windows, linux, freebsd and the web are attached to each [GitHub release](https://github.com/shep95/brahma/releases). on linux you can pick an AppImage, a `.deb`, an `.rpm`, a tarball or a flatpak bundle. the bundle needs the freedesktop runtime from flathub, which `flatpak` offers to install along with it:

```sh
flatpak install --user brahma-<version>-linux-x86_64.flatpak   # or -linux-aarch64
flatpak run com.houseofasher.brahma
```

the AppImage needs no install: the first run registers its launcher icon and menu entry in `~/.local/share` so the dock shows brahma's icon on wayland. set `BRAHMA_NO_DESKTOP_INTEGRATION=1` to skip that, and see [`docs/releasing.md`](docs/releasing.md) › linux to undo it.

on a wayland session, files dropped on the window don't open yet: winit 0.30, the windowing library under egui, has no wayland drag and drop (#386). use file › open, or copy the image in your file manager and paste it with ctrl+v. to drag and drop, start brahma under xwayland: `WAYLAND_DISPLAY= brahma`, `WAYLAND_DISPLAY= ./brahma-<version>-linux-x86_64.AppImage`, or `flatpak run --nosocket=wayland --socket=x11 com.houseofasher.brahma`. on wayland, brahma shows the command for your install in a notice until you dismiss it. preferences › performance › linux display server = X11 does the same on every launch. with a fractional display scale (125%, 150%, 175%) an xwayland window is drawn at a lower resolution and enlarged by the compositor, so the interface is blurred there; native wayland stays sharp (#1072).

on macos, the command-line tool comes as `brahma-cli-<version>-macos-universal.zip`. the binary is signed with the same developer id as the app and notarized by apple. a bare binary can't carry a stapled notarization ticket the way the DMG does, so the first time you run it macos checks the notarization online. you can confirm it yourself:

```sh
ditto -x -k brahma-cli-<version>-macos-universal.zip .
spctl --assess --type install -vv brahma-cli-<version>-macos-universal/brahma-cli
# ... accepted, source=Notarized Developer ID
```

on freebsd 14 (x86_64), the release has a tarball laid out like `/usr/local`. install the runtime libraries, then unpack it there:

```sh
pkg install libxkbcommon wayland libX11 libXcursor libXrandr libXi libxcb mesa-libs vulkan-loader gtk3 fontconfig freetype2 alsa-lib
tar -xzf brahma-<version>-freebsd-x86_64.tar.gz --strip-components 1 -C /usr/local
brahma
```

maintainers: [`docs/releasing.md`](docs/releasing.md) explains how releases are built, signed and published.

> [!IMPORTANT]
> **status:** brahma is in early alpha, and we want to be straight about where it stands: much of photoshop's feature surface exists in some form, but **it is not yet a photoshop replacement for daily professional work**. the biggest gaps are ai/generative features, 15 missing tools, depth in typography and pro workflows, and plug-in compatibility. every photoshop menu item is wired to a command ([`docs/parity-checklist.md`](docs/parity-checklist.md)), but that measures wiring, not behaviour. the honest, dimension-by-dimension picture and where we're going next are in the [roadmap](ROADMAP.md) and [parity assessment](docs/target-app-parity.md). expect rough edges, and please file issues (include your os, document size, layer count and a screenshot). you can also tell us what broke on [#houseofasher community](https://github.com/shep95/brahma/discussions).

## documentation

developer, architecture, automation, format, and security documentation is maintained in the [brahma documentation book](book/).

to self-host the browser version with docker, see [docker web hosting](packaging/web/README.md#docker).

## security

security architecture, threat modeling, parser hardening, fuzzing, and vulnerability reporting are covered in the [security documentation](book/src/security/) and the repository [security policy](SECURITY.md).

## test corpora

brahma is tested against real files: our own photoshop-authored oracle psds in
brahma-corpus plus the psd-tools, ag-psd and PngSuite sets, pinned and
sha256-verified. fetch them with `cargo xtask corpus --all` and run the tests with
`cargo xtask test-corpus` (details in [docs/development.md](docs/development.md#test-corpora)).

## downloads

**new to brahma?** download it from the [brahma page on github.com/shep95/brahma](https://github.com/shep95/brahma). that's the easiest way to install it.

**want a specific build or format?** on GitHub, the [latest release](https://github.com/shep95/brahma/releases/latest) has every build listed below, and [all releases](https://github.com/shep95/brahma/releases) has earlier versions and their notes. `<ver>` in the file names is the version number, and `SHA256SUMS.txt` lists a checksum for every file.

### windows

| build | installer | portable |
|---|---|---|
| x64 (64-bit intel/AMD) | `brahma-<ver>-windows-x64.msi` | `brahma-<ver>-windows-x64-portable.zip` |
| arm64 (snapdragon and other ARM pcs) | `brahma-<ver>-windows-arm64.msi` | `brahma-<ver>-windows-arm64-portable.zip` |
| x86 (32-bit) | `brahma-<ver>-windows-x86.msi` | `brahma-<ver>-windows-x86-portable.zip` |

installers and executables are code-signed.

### macos

| build | file | notes |
|---|---|---|
| app, universal (apple silicon + intel) | `brahma-<ver>-macos-universal.dmg` | signed and notarized |
| command-line tool, universal | `brahma-cli-<ver>-macos-universal.zip` | signed and notarized |

### linux

| format | x86_64 | aarch64 (ARM64) | notes |
|---|---|---|---|
| AppImage | `brahma-<ver>-linux-x86_64.AppImage` | `brahma-<ver>-linux-aarch64.AppImage` | runs anywhere; updates itself with AppImageUpdate (`.zsync` files) |
| flatpak | `brahma-<ver>-linux-x86_64.flatpak` | `brahma-<ver>-linux-aarch64.flatpak` | sandboxed; `flatpak install --user <file>` |
| debian/ubuntu | `brahma-<ver>-linux-x86_64.deb` | `brahma-<ver>-linux-aarch64.deb` | |
| fedora/RHEL/opensuse | `brahma-<ver>-linux-x86_64.rpm` | `brahma-<ver>-linux-aarch64.rpm` | |
| tarball | `brahma-<ver>-linux-x86_64.tar.gz` | `brahma-<ver>-linux-aarch64.tar.gz` | unpack anywhere |

RISC-v (riscv64): `brahma-<ver>-linux-riscv64.tar.gz` only, built against ubuntu 24.04 (needs glibc 2.39+).

### freebsd

| build | file |
|---|---|
| x86_64 | `brahma-<ver>-freebsd-x86_64.tar.gz` |

### web (WebAssembly)

| build | file | notes |
|---|---|---|
| static site | `brahma-web-<ver>.zip` | runs in a modern browser; host it on any static server |

## #houseofasher

brahma is made by #houseofasher: free, open-source creative tools, each written from scratch in rust and each
able to stand on its own. the conventions are the same everywhere: clean-room and pure rust, native on macos,
windows and linux, in the browser via WebAssembly, and fully drivable by agents.

<br>

<h3 align="center">come make things with us</h3>

<p align="center">
  our #houseofasher community is where artists of every kind hang out: people who paint, shoot, draw, cut film,
  set type, and people still figuring out what they like to make. share what you're working on,
  ask for help, tell us what's broken, or tell us what you wish these tools could do.
  whatever your medium and however long you've been at it, you're welcome here.
</p>

<p align="center">
  <a href="https://github.com/shep95/brahma/discussions"><b>github.com/shep95/brahma/discussions</b></a> ·
  <a href="https://github.com/shep95/brahma">github.com/shep95/brahma</a> ·
  <a href="https://github.com/shep95/brahma">the #houseofasher apps</a> ·
  <a href="https://github.com/shep95/brahma">brahma</a>
</p>

---

## license and credits

brahma is dual-licensed under [MIT](LICENSE-MIT) or [apache-2.0](LICENSE-APACHE), at your option.
copyright (c) 2026 #houseofasher and the brahma contributors. required notices are in [NOTICE](NOTICE).

bundled fonts, icons, images and other assets keep their own open licenses; each one is listed
with its author, source and license in [ATTRIBUTION.md](ATTRIBUTION.md).

every artwork shown is in the public domain (wikimedia commons, NASA, u.s. national archives); sources are listed in [`docs/images/SOURCES.md`](docs/images/SOURCES.md).

<sub>adobe, photoshop, illustrator, premiere pro, lightroom, acrobat, after effects and InDesign are trademarks or registered trademarks of adobe inc. in the united states and/or other countries. brahma is an independent, open-source project and is not affiliated with, sponsored by or endorsed by adobe inc.; these names are used only to describe the workflows it is compatible with.</sub>

<p align="center">
  <sub>made by <a href="https://github.com/shep95/brahma">#houseofasher</a> and the community.</sub>
</p>

## star history

