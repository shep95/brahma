# brahma-affinity

an independent, bounded rust reader for native affinity documents: `.af` from affinity 3 and
`.afdesign`, `.afphoto` and `.afpub` from affinity 1 and 2 (container versions 8 to 12). it reads
the archive, the tagged object stream in `doc.dat` and the part of the document model listed below,
and hands `brahma-io` a document in pixels with every transform composed. it does not write
affinity files.

## what is imported

`brahma-io` maps the model to an 8-bit RGB document at the affinity document's resolution:

| affinity | brahma |
|---|---|
| pages, publisher spreads | one artboard per spread, spreads side by side 64 px apart |
| artboards (legacy `ABEn`, or current `.af` `phrp`/`aprp` properties, including converted curves), with name, solid background, any transform | artboards (nested ones as groups with a vector mask; rotated/curved boards use their bounding rectangle) |
| layers (`Scop`), groups (`Grup`), pass-through or isolated | groups (pass-through or normal) |
| curves (`PCrv`): cubic subpaths, closed flags, live corners (`CnrD`) | shape layers (multi-subpath curves fill even-odd) |
| rectangles with corner radii (relative or absolute, locked or per corner) and round or straight corners, ellipses, polygons (smooth too), stars, square stars, pies, triangles, trapezoids, clouds, hearts, cogs, rounded and elliptical callouts, arrows, double stars, tears, crescents, diamonds and circular segments | shape layers |
| compound shapes (`Comp`, add and subtract) | one even-odd shape |
| a shape or curve with children clips them | group with the outline as vector mask, the shape at its bottom |
| vector masks (`AdCh` curves and shapes) | vector masks |
| pixel masks (`MRst`, attached or as a layer in a group) | layer masks |
| solid fills: RGBA, HSLA, CMYK, grey, lab (D50) | shape fill (converted to RGB) |
| linear, elliptical, radial and conical gradients with affinity's midpoint bias | gradient fills |
| strokes: weight, scale with object, dash pattern, alignment, caps, joins, miter limit | shape strokes |
| fill layers (`FRst`) | filled rectangles |
| artistic and frame text: characters, font family/weight/style, size, tracking, fixed leading, colour, paragraph alignment, first baseline | type layers (rendered with the fonts installed) |
| placed images (`ImgN`): the original JPEG/PNG embedded in the file | embedded smart objects |
| pixel layers (`Rstr`): RGBA 8/16-bit and CMYK 8-bit tiles, cropped to their content | pixel layers |
| embedded documents and symbols (`EmbN`) | the picture affinity cached of them |
| opacity, visibility, lock, names, blend modes | the same (add as linear dodge) |

everything else is reported in the import warnings, one line per kind with a count, never
dropped silently: layer effects, adjustment layers and live filters, brush and pressure strokes,
transparency gradients, fill opacity, bitmap fills, master pages, unrecognized
parametric shapes (imported as their bounding ellipses), corner types other than round or straight, stars with
rounded points, several fills or strokes on one object (one of each is used), strokes behind the
fill, outlined or scaled text, text fields such as page numbers, frames with columns or a curved
outline, grey/lab/32-bit pixels, CMYK and lab colour (converted to RGB without the document's
profile), facing pages and the average, negation, reflect, glow and erase blend modes (as normal).

opened affinity documents start without a save path: save asks for a new copy and never writes
back over the affinity file. when the native document can't be read at all, `brahma-io` opens
the embedded PNG preview instead, as one pixel layer named “affinity preview”, with a warning
that names the reason. that preview can't be placed or used by imports that can't show the
warning (smart-object decoding, stacks, batch, displacement maps, variables).

## embedded preview

`preview(bytes)` reads only the thumbnail record the header's offset 24 points to (the same in
container versions 8 to 12); it never scans for PNG signatures. all integers are little-endian.

| record offset | size | observed value |
|---|---:|---|
| 0 | 8 | `ffffffff` followed by `Thmb` |
| 8 | 4 | record version 1 |
| 12 | 4 | PNG byte length + 13 |
| 16 | 4 | PNG start offset 29 |
| 20 | 4 | zero |
| 24 | 4 | PNG byte length |
| 28 | 1 | PNG encoding 1 |
| 29 | declared length | exactly one PNG |

the PNG is limited to 16 mib and 4096 pixels a side. IHDR must come first and be unique, IDAT
chunks contiguous, an optional PLTE unique and before IDAT, every chunk CRC must match and IEND must
end the file; compressed metadata (`zTXt`, `iTXt`, `iCCP`), APNG chunks and unknown critical chunks
are rejected. the i/o adapter decodes it with a 128 mib allocation limit.

## provenance and clean-room

affinity has no published file-format specification, so this reader is built only from a public
description of the format and from public files, never from affinity itself. this is how
brahma already treats formats that are only partly documented: `brahma-psd` fills the gaps
in adobe's specification from the MIT-licensed psd-tools and ag-psd, and `brahma-raw` recovered
nikon's compression tables by black-box analysis of CC0 sample files (see their readmes).

* **no affinity software.** affinity was never downloaded, installed, run, scripted, screenshotted or
  disassembled for this work, and nothing from an affinity installation (program code, resources,
  presets, fonts, colour profiles) was read. no file was made with affinity for it: every sample
  is a document its author published.
* **a public, permissively licensed description.** the container and object-stream layout was
  learned from vmdevcpp/afread (MIT, at
  `04b672334a43e3e37ded6b5ffc57af231d589774`, written for container versions 7–11) and re-described
  in our own words before this rust code was written; no code was translated. no GPL/AGPL code (such
  as inkscape's affinity extension) was read.
* **every structure checked against public files.** afread doesn't say how its author learned the
  format, so nothing here rests on it alone: each structure the reader relies on is confirmed by
  the public documents. every archive entry carries a CRC-32 that must match, and every field in
  the object stream names its own type, so a misread layout fails loudly instead of producing
  plausible wrong data.
* **meaning fitted to affinity's own pictures.** what shapes, paints, text and pixel data mean was
  worked out by comparing our render with the thumbnail every affinity document embeds (affinity's
  render of itself), on 176 public documents saved by affinity 1.x, 2.x and 3.0/3.1 on windows,
  macos and iPad. the collection has since grown to 189 distinct documents, and the reader opens
  all of them. most are published under CC0, MIT, BSD, apache-2.0 or cc by (-sa); the others
  (no licence stated, or GPL or non-commercial terms) were only opened locally to compare
  pictures, never committed or redistributed. the corpus test keeps the comparison for the pinned
  files, which are CC0, MIT or apache-2.0 only.
* **nothing of affinity's is in this repository**: no program code, assets or documents. the test
  files are fetched at pinned commits and sha256-verified, never committed.
* **read-only.** nothing is written in affinity's format; export waits until someone can check
  written files in affinity itself.

the same reader is shared, as an independent copy, with #houseofasher's `houseofasher-affinity`.

## validation

* unit tests on synthetic containers built by the `synth` feature (stored, zlib and zstd entries,
  checksums, budgets, cycles, hostile streams, every truncation) and property tests of random
  mutations, here and in `brahma-io`'s `tests/affinity.rs`.
* `cargo xtask corpus --affinity` fetches 21 public CC0/MIT documents, including the four
  affinity 3 `.af` files of samuel-etver/vector-art
  (CC0), at pinned commits, each checked against `xtask/affinity-corpus.sha256`. `tests/real_files.rs`
  parses them; `brahma-io`'s `tests/affinity_corpus.rs` imports and flattens each one and
  compares it with the thumbnail affinity saved in it (mean difference 0–4.6 of 255, with a
  ceiling per file).
* `fuzz/` has `cargo-fuzz` targets for the whole reader (`preview`), the object stream (`stream`)
  and the i/o import (`import_preview`); ci's fuzz job runs `preview` and `import_preview`:

  ```sh
  cd crates/affinity/fuzz
  cargo +nightly fuzz run stream -- -max_total_time=600
  cargo +nightly fuzz run import_preview --features io -- -max_total_time=600
  ```

what has not been verified: files from affinity builds or platforms outside the public set, files
written by other applications, rotated or skewed images against affinity's render, mask polarity
against a render with and without the mask, and anything listed as a warning above. the `synth`
builders write only what this reader accepts; affinity has never opened their output, so they are
test fixtures, not an exporter.

## safety limits

every size is checked before it is allocated: 256 mib per archive entry, 1 gib extracted per import
and 1 gib of decoded pixels per import by default (`Limits`: extracted bytes and materialized pixels
are separate budgets, since a bitmap's declared size costs memory whatever its tiles hold), a 64 mib
zstd window, 4096 saved revisions, array lengths no longer than the
bytes left, 16 777 216 decoded values per document stream (fields and array elements together, counted before
anything is allocated: a value takes about 40 bytes in memory however few it took in the file; the largest of
those 189 public documents uses 3.3 million), 384 levels of object nesting, 128 levels of layers, 500 000 layers, four million curve
nodes per curve, 64 megapixels per pixel layer (cropped to its content first) and 1024 gradient
stops; `brahma-io` adds 300 000 pixels a side per document and 64 megapixels per image on the
canvas. every archive entry's CRC-32 and size must match. malformed input returns an `Error`; the
crate has no panics outside tests.

## privacy

affinity documents can hold the folder they were saved in, user names, original image paths and
XMP metadata. this reader never imports those fields.
