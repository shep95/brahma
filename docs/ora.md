# OpenRaster documents

brahma opens and saves OpenRaster (`.ora`), the layered format krita, MyPaint, GIMP, pinta
and others share. file › open, recent files, drops, save and save as, the CLI (`convert`, `info`,
`batch`) and automation (`doc.open`, `doc.save`, MCP) all use the same reader and writer in
`brahma-io`. the browser build accepts `.ora` too. a file is recognised by its `mimetype`
entry, so an OpenRaster file with another extension still opens as one.

## what is kept

| | open | save |
|---|---|---|
| pixel layers (any size and position, also off-canvas) | yes | yes, cropped to their pixels |
| groups, nested | yes | yes |
| pass-through groups | `isolation="auto"` without a blend mode | `isolation="auto"` |
| names, visibility, opacity | yes | yes |
| blend modes | the `svg:*` names, `svg:plus`, and krita's `krita:*` names | `svg:*` where the spec has the mode, else `krita:*` (read back by krita, normal elsewhere) |
| locks | `edit-locked`, `alpha-preserve` | the same |
| 8 and 16-bit | yes (the deepest layer sets the depth) | 8-bit documents as 8-bit pngs, 16 and 32-bit as 16-bit pngs |
| resolution | `xres` | `xres` and `yres` |
| merged image and thumbnail | not read (we render our own) | written (`mergedimage.png`, `Thumbnails/thumbnail.png`) |

saving reports what OpenRaster can't hold, layer by layer: pixel masks are applied to the
layer's pixels, fill opacity is folded into the pixels, text, shape, fill and smart object layers
are written as their rendered pixels, and adjustment layers, layer styles, vector masks,
clipping, alpha channels, metadata and a colour profile other than srgb are left out. xor and paint.NET's
color burn and color dodge variants have no OpenRaster name and are saved as normal. CMYK, lab
and other modes are refused with a message to convert first; grayscale saves as RGB.

an OpenRaster file is the document's own file only while it holds everything in it (#3806).
`brahma_io::ora_losses` lists what a save would leave out, from the same checks the writer
warns with, without encoding anything. when that list isn't empty, file › save doesn't overwrite
the `.ora`: save as opens on a PSD of the same name, with a notice listing what OpenRaster would
drop. a save to an `.ora` path then writes a copy, as photoshop does for formats that can't keep
everything: the document keeps its file and its unsaved changes. automation follows the same rule
(`doc.save` and `app.save` without a path refuse, saying why).

OpenRaster elements other than `stack` and `layer` (MyPaint's text, for example), missing
layer images and unknown composite operations open with a warning instead of failing the file.
krita's W3C soft light (`svg:soft-light`, `krita:soft_light_svg`) and its own hard mix open as
photoshop's soft light and hard mix, which differ slightly, also with a warning.

## limits and safety

the reader takes the archive with the bounded ZIP reader of `brahma-format` (no ZIP64,
CRC-checked), parses `stack.xml` with `roxmltree` (16 mib cap), and decodes each layer image
through `brahma-codecs` with size limits. canvases are at most 300,000 px a side, at most
8,192 layers and groups, nesting at most `MAX_GROUP_DEPTH` deep, and 1 gib of decoded pixels in
total. malformed archives, XML, numbers and images return errors; truncated and corrupted files
are covered by tests.

## verification

unit tests (`crates/io/src/ora/tests.rs`) round-trip layers, groups, pass-through, offsets,
opacity, visibility, locks and every blend mode at 8 and 16 bit, check masks, rendered layer
kinds, the archive layout and malformed input. `crates/automation/tests/openraster.rs` saves,
reopens, edits and saves in place through the agent surface, and checks that a document with an
adjustment layer never overwrites its `.ora`; the CLI test converts both ways.

real-file checks use krita-authored oracles from
brahma-corpus (`krita/`, made by
`tools/krita-oracles/generate.py`). each krita file carries krita's rendering of itself in
`mergedimage.png`:

```sh
KRITA_TEST_DIR=/path/to/brahma-corpus/krita cargo test -p brahma-io --test krita_corpus -- --ignored --nocapture
```

2026-10-09, krita 5.2.9: 32 of 35 `.ora` files render within 3/255 of krita's merged image; the
other three differ for known reasons listed in `crates/io/tests/krita_corpus.rs` (krita's
tie-break in darker and lighter color, and hard mix at a channel sum of exactly 1). in the other
direction, krita opened every brahma re-export of the first 34 of those files and rendered 33
of them identically to its original; the exception is photoshop's soft light, which is written
as `svg:soft-light`. MyPaint and GIMP files have not been checked yet.

format reference: the OpenRaster specification. the reader and
writer are written from the specification and from files krita writes; no application code
was used.
