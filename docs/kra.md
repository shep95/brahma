# krita documents

brahma opens krita's native documents (`.kra`, and the `.krz` archive variant) with their
layers. file › open, recent files, drops, the CLI (`convert`, `info`, `batch`) and automation
(`doc.open`, MCP) all use the reader in `brahma-io`; the browser build accepts them too. a
file is recognised by its `mimetype` entry (`application/x-krita`). krita files are import-only:
save suggests OpenRaster (`.ora`), which krita opens with its layers, and `.pcraft` keeps
everything.

## what is read

| krita | brahma |
|---|---|
| paint layers, 8 and 16-bit RGBA, 8 and 16-bit gray | pixel layers, at the document's depth (a gray layer in an RGB document is converted) |
| layer position (`x`, `y`), tiles anywhere, also off-canvas | kept |
| groups, nested; pass-through groups; collapsed state | groups; pass-through; expanded state |
| names, visibility, opacity, lock | kept |
| blend modes | krita's ids, mapped as for OpenRaster (`docs/ora.md`); a few are approximations, reported |
| transparency masks | the layer mask (the first one; more are reported and left out) |
| selection masks | ignored (a layer's local selection, not part of its look) |
| the image's ICC profile, resolution | kept |

left out with a warning per layer: vector, filter, fill, file and clone layers, filter, transform
and colorize masks, layers in another colour space than those above, inherit alpha, per-channel
blending flags (channel locks), and animation beyond the frame saved in the layer. drawing
assistants and reference images are reported as left out. krita's merged image still shows
the original look, so the result can differ where those were used.

documents whose image colour space we don't read yet (CMYK, lab, XYZ, ycbcr, 16 and 32-bit
float) open as krita's merged image (`mergedimage.png`), flagged as a preview, with a warning.

## the format, as observed

a ZIP whose first, stored entry is `mimetype`; `maindoc.xml` holds the node tree (topmost first)
and points each paint layer and mask at a file under `<image name>/layers/`. those files are a
text header (`VERSION 2`, `TILEWIDTH`, `TILEHEIGHT`, `PIXELSIZE`, `DATA <tiles>`) and per tile a
`x,y,LZF,<bytes>` line followed by a compression flag and the tile's pixels in byte planes, LZF
compressed. RGB pixels are b, g, r, a; 16-bit samples are little-endian. a `.defaultpixel` file
next to each holds the colour outside its tiles. a group's own `x`/`y` does not move its
children (checked against krita's merged image).

## limits and safety

the reader uses the bounded ZIP reader of `brahma-format` (no ZIP64, CRC-checked), parses
`maindoc.xml` with `roxmltree` (32 mib cap; the DTD is declared by public id and not fetched),
and decodes tiles with a bounds-checked LZF decoder that must produce exactly one tile. tiles
are at most 1024 px a side; canvases at most 300,000 px a side and 1 gib of decoded pixels;
8,192 nodes; nesting at most `MAX_GROUP_DEPTH`. version 1 tile files (krita 1.x) are refused.
malformed archives, XML, numbers, tile headers and tile data return errors; truncated and
corrupted files are covered by tests.

## verification

unit tests (`crates/io/src/kra/tests.rs`) build krita archives in code (raw and LZF tiles) and
check layers, groups, pass-through, offsets, opacity, visibility, locks, masks, default pixels,
16-bit and gray conversion, the merged-image fallback, LZF edge cases and malformed input.
`crates/automation/tests/krita.rs` opens a `.kra` through the agent surface, edits it and saves
it as OpenRaster. `crates/engine/tests/kra.rs` (opt-in) opens a krita-authored 16-bit file in a
session, checks it is read-only, edits with undo and saves as `.ora` and `.pcraft`.

real-file checks use the krita-authored oracles in
brahma-corpus (`krita/`, made by
`tools/krita-oracles/generate.py`, which removes krita's embedded third-party ICC profiles so
the files are srgb):

```sh
KRITA_TEST_DIR=/path/to/brahma-corpus/krita cargo test -p brahma-io --test krita_corpus -- --ignored --nocapture
KRITA_TEST_DIR=/path/to/brahma-corpus/krita cargo test -p brahma-engine --test kra -- --ignored
```

2026-10-09, krita 5.2.9: 31 of 35 `.kra` files render within 3/255 of krita's merged image; the
others differ for known reasons (the three blend-mode tie-breaks as for OpenRaster, and the
file with a vector layer, which is left out). before the profiles were stripped, a linear-light
16-bit document with its embedded profile matched krita's merged image within 0.5/255 after
flattening and converting to srgb.

the reader is written from files krita writes; no krita code was used.
