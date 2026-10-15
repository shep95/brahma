# brahma-codecs

codecs for flat raster image formats, written in pure rust. the crate is standalone: it depends on no other
workspace crate and defines its own `Image` type. it also builds for `wasm32-unknown-unknown` and
does no i/o: decoders take `&[u8]` and encoders return `Vec<u8>`.

it follows the "avoiding GIMP's hole" rules in `docs/architecture.md` §1.1:

* **symmetric.** every format we write, we can also read. the only exceptions are listed in
  `ASYMMETRIC_EXCEPTIONS`, each with a reason. the test suite checks this.
* **depth-preserving.** an image can hold u8, U16, F16 or F32 samples, and each format keeps
  what it can hold.
* **metadata-preserving.** ICC, EXIF, XMP, DPI and text are kept wherever the format supports
  them.
* **honest.** `caps(format)` says what each format holds, and
  `fidelity_warnings(&image, format)` lists what an export will lose before you write it. both
  functions use the same encode plan as `encode`, so a warning appears exactly when the file
  changes. on the way in, `Image::warnings` lists what the decoded image doesn't show: frames or
  pages left out, or data that ended early.

```rust
use brahma_codecs::*;

let img = decode(&bytes)?;                               // detect + decode (default Limits)
for w in fidelity_warnings(&img, Format::Jpeg) {         // e.g. "16-bit will be reduced to 8-bit"
    println!("{w}");
}
let out = encode(&img, Format::Tiff, &EncodeOptions::default())?;
```

## capability matrix (default build)

| format | read | write | depths (native) | layouts (native) | alpha | ICC | EXIF | XMP | DPI | text | lossy write | backend |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| PNG | yes | yes | u8, U16 | gray, graya, RGB, RGBA | yes | yes (iccp) | yes (exif) | yes (itxt `XML:com.adobe.xmp`) | yes (phys) | yes (text/ztxt/itxt) | no | `png` |
| JPEG | yes | yes | u8 | gray, RGB, CMYK | no | yes (multi-segment APP2) | yes (APP1) | yes (APP1) | yes (JFIF) | no | yes | `zune-jpeg` / `jpeg-encoder` |
| TIFF | yes | yes | u8, U16, F32 | all six (CMYK, CMYK+a included) | yes | yes (tag 34675) | no | yes (tag 700) | yes | yes (description, make, model, software, DateTime, artist, copyright) | no | `tiff` |
| webp | yes | yes | u8 | RGB, RGBA | yes | yes | yes | yes | no | no | yes (VP8, when `webp_lossless` is off) | `image-webp` / built-in VP8 |
| GIF | yes | yes | u8 | RGBA | 1-bit | no | no | no | no | no | yes (256-colour palette) | `image` |
| BMP | yes | yes | u8 | RGB, RGBA | yes | no | no | no | no | no | no | `image` |
| TGA | yes | yes | u8 | gray, graya, RGB, RGBA | yes | no | no | no | no | no | no | `image` |
| ICO | yes | yes | u8 | RGBA (at most 256×256) | yes | no | no | no | no | no | no | `image` |
| DDS | yes | **no** | u8 | gray, graya, RGB, RGBA | yes | no | no | no | no | no | n/a | built in |
| netpbm (PBM/PGM/PPM/PAM/PFM) | yes | yes | u8, U16, F32 (float: PFM gray/RGB only) | all six (PAM `CMYK`/`CMYK_ALPHA`) | yes (PAM) | no | no | no | no | no | no | built in |
| QOI | yes | yes | u8 | RGB, RGBA | yes | no | no | no | no | no | no | `image` |
| openexr | yes | yes | F16, F32 | y, ya, RGB, RGBA | yes | no | no | no | no | no | no | `exr` |
| radiance HDR | yes | yes | F32 | RGB | no | no | no | no | no | no | yes (RGBE) | `image` |
| FITS | yes | yes | u8, U16, F32 | gray, RGB | no | no | no | no | no | yes (header cards) | no | built in |
| DPX | yes | yes | u8, U16 (10/12-bit code values read and written exactly through U16) | gray, RGB, RGBA | yes | no | no | no | no | DPX header fields only (creator, project, copyright, creation time, transfer, colorimetric, frame position, frame rate, time code) | no | built in |
| AVIF | **no** | only with feature `avif` | u8 | RGB, RGBA | yes | no | no | no | no | no | yes | `image` + `ravif` |
| HEIF/HEIC | with feature `heif` | **no** | u8, U16 (10/12-bit decodes to U16) | RGB, RGBA | yes (auxiliary alpha) | yes (`colr` prof) | yes | yes | no | no | n/a | `heic-rs` |

"native" means the data is stored and read back without conversion. anything else is converted by
the encode plan, and `fidelity_warnings` reports the conversion when it loses information:

* a float image written to an integer format goes to U16 where the format supports it. values
  outside 0..1 raise `RangeClipped`.
* integer data written to a float-only format (EXR, HDR) becomes F32 without loss.
* CMYK is converted to RGB with a naive formula that is not colour-managed. `CmykConverted` is
  raised and the ICC profile is dropped, because a CMYK profile does not describe RGB data.
* gray written to an RGB-only format is expanded to RGB without loss, so no warning is raised.

`Image::convert` applies the same ICC rule: it keeps the profile only when the colour model stays
the same.

palette BMP and PNG pixels decode as RGB(a) in `Image`; `bmp_palette` and `png_palette` expose the
embedded table (and a PNG's transparent entry) so `brahma-io` can open them in indexed color mode.

## documented asymmetries and limitations

* **DDS.** the top mip level of a 2d texture: uncompressed pixels described by channel masks
  (8, 16, 24 or 32 bits, RGB or luminance, with or without alpha) or BC1/BC2/BC3 (DXT1/DXT3/DXT5),
  with the legacy header or the DX10 one. everything opens as u8; the other mip levels are
  ignored. cubemaps, texture arrays, volume textures, premultiplied DXT2/DXT4 and other formats
  (BC4–BC7, float) are refused with `CodecError::Unsupported`. writing is not provided.

* **FITS.** the image in the primary HDU, or the first `IMAGE` extension, is read; `NAXIS3 = 3`
  opens as RGB and any other plane count opens the first plane with `DecodeWarning::MorePages`.
  rows follow `ROWORDER` (bottom-up by default, as siril and DS9 do). physical values that fit
  stay exact as u8/U16 (a camera's `BITPIX = 16`, `BZERO = 32768`); the rest become F32, taken as
  normalised when at most 2 and scaled by 65535 (or the maximum) otherwise. header cards are
  kept as text (`HIERARCH` for long names). tile-compressed (fpack) files are refused with
  `CodecError::Unsupported`; bayer mosaics open undebayered.

* **DPX.** SMPTE 268m frames in either byte order (`SDPX`, `XPDS`): the first image element,
  descriptor 6 (luma), 50 (RGB) or 51 (RGBA), unsigned and unencoded, at 8, 10, 12 or 16 bits.
  10-bit data may be packed (0) or filled three to a word (1, method a; 2, method b), 12-bit data
  packed (0) or one to a 16-bit word (1, method a). 10- and 12-bit code values open as U16
  `round(c × 65535 / (2^b − 1))`, so a frame written back at its depth keeps them exactly. no
  colour transform is applied: `DecodeWarning::DpxEncoding` names the transfer characteristic and
  colorimetric specification, so a log frame is reported as log. more elements than the first
  give `DecodeWarning::MoreElements`; the image orientation is applied on import. writing is
  big-endian, one unencoded element, at `EncodeOptions::dpx_bits` (8/10/12/16; by default 8 for
  u8 and 16 otherwise), packing 1 for 10 and 12 bits; gray with alpha is written as luma.
  other descriptors, signed or run-length-encoded data and other bit depths return
  `CodecError::Unsupported` naming the field. cineon (`.cin`) is not supported.

* **camera raw files** (DNG, CR2, NEF, ARW… which are TIFF-structured) are recognised and refused
  with `CodecError::Unsupported`: they are sensor data, not flat images. `brahma-raw` decodes
  and develops them, and `brahma-io` routes them there.

* **HEIF/HEIC (read-only, feature `heif`).** decoding lives in the optional `brahma-heif` crate
  behind the non-default `heif` feature (official brahma builds enable it; distributors choose).
  without it, HEIF is still detected and decoding returns `CodecError::Unsupported` ("HEIC/HEIF
  support isn't included in this build"). iPhone and mac photos decode with `heic-rs` (pure rust, no `unsafe`):
  single pictures and grid-tiled photos, 8 to 12 bits, auxiliary alpha, ICC, EXIF and XMP. the
  container's `irot`/`imir`/`clap` are applied and EXIF orientation is rewritten to 1 (HEIF's EXIF
  tag only repeats the container's, and applying both would turn the photo twice). grayscale
  decodes to RGB. image sequences, overlays and identity derivations, multilayer HEVC and some
  4:2:2/4:4:4 streams return `CodecError::Unsupported` or `Malformed`, never wrong pixels. writing
  needs an HEVC encoder and every mature one is c, so HEIF is listed in `ASYMMETRIC_EXCEPTIONS`.
* **AVIF.** encoding uses `ravif`, which is pure rust. decoding
  needs `dav1d`, which is c. AVIF is therefore read-unsupported, and write support is gated
  behind the non-default `avif` feature. in a default build it is neither readable nor writable,
  so the symmetric guarantee holds. it is listed in `ASYMMETRIC_EXCEPTIONS`.
* **lossy webp.** lossless (VP8L, via `image-webp`) is the default. `webp_lossless: false`
  writes a lossy file with our own clean-room VP8 key-frame encoder (`codecs::vp8`, from
  RFC 6386; `webp_quality` 0–100 on libwebp's scale sets the quantizer). alpha travels losslessly
  in an `ALPH` chunk and ICC/EXIF/XMP in their own chunks of the extended (`VP8X`) container. the
  encoder makes no rate-distortion decisions yet, and a side longer than 16383 px (the VP8 frame
  header's limit) is refused with `CodecError::Encode`. we read both lossy and lossless files.
* **animation and multi-page files** (APNG, animated GIF/webp, multi-page TIFF): only the first
  frame or page is decoded, and a single frame is written. `FormatCaps::animation` marks
  containers that can hold more frames. the decoded image then carries a
  `DecodeWarning::MoreFrames` / `MorePages` in `Image::warnings`, with the total when the file
  states it (frames and TIFF directories are counted without decoding them; reduced-resolution
  TIFF directories and transparency masks are not pages). any TIFF page can still be opened
  with `decode_tiff_page` (and `brahma_io::import_tiff_page`). every frame of a GIF comes
  from `decode_gif_frames`: each frame composited onto the logical screen with the disposal
  methods applied, as RGBA8 with its delay in milliseconds, within `Limits` and
  `MAX_GIF_FRAMES`. the video-layer commands use it (`brahma_io::import_gif_frames`).
* **EXIF in TIFF** is stored as a sub-IFD rather than a blob, so it is not preserved yet.
  `caps.exif = false` for TIFF, and a warning is raised.
* **orientation** (EXIF tag 274 in JPEG, PNG `eXIf` and webp; the decoded page's tag in TIFF and
  bigtiff) is applied on
  decode, like photoshop: the pixels come back upright and the EXIF/XMP orientation is rewritten
  to 1 (`DecodeOptions::keep_orientation` opts out). encoders always write orientation = 1
  (`upright_exif`, `upright_xmp`, which change only the tag's value, kept in its own type), so upright pixels
  are never rotated twice. parsing is strict: an IFD0 inside the 8-byte header or cut off before
  its next-IFD pointer, an orientation that is not exactly one SHORT or LONG, or an out-of-range value
  reads as 1; with duplicate entries the first wins. `Limits` are checked again on the upright
  size (5–8 swap width and height), and the rotated buffer is allocated fallibly.
  `Image::oriented` turns any layout and depth in parallel bands (about 10 ms for 24 mp RGB8 in
  release).
* **resolution** (#1691, module `resolution`): a file without a resolution of its own format
  (no PNG `pHYs`, a webp, HEIF, …) takes it from its XMP `tiff:XResolution` / `YResolution` /
  `ResolutionUnit`, else from EXIF IFD0 (RATIONAL x/yresolution, unit 2 inch — the default when
  absent — or 3 cm; unit 1 or anything malformed counts as no resolution). a JPEG uses, like
  photoshop, the first usable of: APP13 photoshop ResolutionInfo (0x03ed), XMP, EXIF, and only
  then the JFIF density (cameras write EXIF only, many tools write a default JFIF 72). on export
  the EXIF and XMP resolution are rewritten to the image's (`export_exif`, `export_xmp`): the
  RATIONAL values in place, ResolutionUnit set to 2, an entry that can't be rewritten removed
  from IFD0 without moving any other data. IFD1 (the thumbnail's) is left as it is.
* **JPEG**
  * 8-bit only. neither decoder backend supports 12-bit.
  * a file cut off inside its image data decodes leniently (a baseline JPEG's missing rows come
    out grey) but never silently: the image carries `DecodeWarning::Truncated`. A file that ends
    before any scan data is an error.
  * CMYK is always written 4:4:4 (subsampled CMYK is not portable) as adobe-inverted CMYK with an
    APP14 marker.
  * text is not written because there is no COM-segment support.
  * EXIF and XMP must each fit in one APP1 segment (65 533 bytes with its header). metadata that
    doesn't fit is dropped and the export still succeeds, with a
    `MetadataTooLarge` warning ("EXIF (… bytes) is too large for the format; it will be dropped").
    Before dropping XMP, the encoder first removes the layered-document properties
    (`DocumentAncestors`, `TextLayers`) that mean nothing in a flat JPEG. Extended XMP is not
    written.
* **TIFF decode**
  * classic TIFF and bigtiff (version 43, 8-byte offsets), both byte orders, strips or tiles,
    planar configuration 1 or 2.
  * supports 1, 2 and 4-bit gray, 8/16/32/64-bit integers (32 and 64-bit are reduced to U16),
    F16, F32 and F64 (converted to F32), WhiteIsZero (#691: the tones are no longer inverted),
    palette images (opened as RGB), associated alpha (made straight) and extra samples (dropped
    unless the first is alpha).
  * every directory is reachable: `tiff_info` lists the main IFD chain and its subifds (tag 330)
    with each page's size, kind (page, reduced-resolution copy, mask) and storage, walked with a
    seen-set, a depth limit and a 10 000-directory cap; `decode_tiff_page` opens any of them.
    `decode` opens the first full-resolution page, like Photoshop (thumbnails are skipped).
  * banded decoding: the image is allocated once at its final size and each strip or row of
    tiles is decoded straight into it, in parallel on native targets, with at most one
    strip/tile-sized scratch buffer per worker (30 MP RGB8: Deflate 345 → 32 ms, LZW 428 → 43
    ms in release). Image data cut off by the end of the file decodes with
    `DecodeWarning::Truncated`; a file far too small for its declared size (more than any
    compression could expand) is refused before allocating.
  * ycbcr, CIE lab, signed integers and compressions that the `tiff` crate lacks (CCITT,
    JPEG-in-TIFF) return `Unsupported`.
* **TIFF encode** writes classic TIFF, or bigtiff when `EncodeOptions::tiff_bigtiff` is set or
  the file could pass 4 gib.
* **EXR**
  * multi-part files (a maya/arnold render writes one part per AOV) open the part that
    looks most like a colour image — RGBA beats RGB beats YA beats a single channel, so a
    depth-only part no longer fails the file — with `DecodeWarning::MoreParts` naming how
    many parts exist. `exr_info` lists every part (name, view, size, channels, deep/tiled)
    without decoding pixels, and `decode_exr_part` decodes a chosen one. Deep parts stay
    with the deep decoder: a file that has any is refused here with a pointer to it.
  * reads at full resolution, from the data window.
  * channel names are matched by suffix, so `layer.R` counts as `R`.
  * subsampled channels are unsupported.
  * cryptomatte layers (specification 1.2, as written by arnold, v-ray, redshift, mantra/karma
    and Cycles) are listed and decoded structurally: `cryptomatte_layers` reads the header
    attributes, channels and embedded manifest without pixels, `decode_cryptomatte` returns the
    per-pixel (ID, coverage) pairs, and `cryptomatte_id`/`cryptomatte_key`/`cryptomatte_preview_color`
    hash names to IDs the way the reference implementation does (official test vectors in the
    test suite).
  * deep data (scanlines or single-level tiles, compressed NONE, RLE or ZIPS) opens as a flat
    image: each pixel's samples are composited into one, with
    `DecodeWarning::DeepFlattened` because per-pixel depth is not kept. Deep ZIP (16-line
    blocks) and deep mip-map or rip-map tiles return `Unsupported`.
  * only lossless compressions are offered for writing: none, RLE, ZIP1, ZIP16 and PIZ.
* **netpbm writing** picks the subtype from the image: float gives PFM, gray gives p5, RGB gives
  p6, and anything with alpha or CMYK gives p7 (PAM). writing a `.pbm` file therefore produces
  whichever of these fits the data.
* **colour conversions** (CMYK↔RGB, RGB→gray) are naive. colour-managed conversion belongs in
  `brahma-color`.

## limits (decompression bombs)

`DecodeOptions { limits: Limits { max_width, max_height, max_pixels, max_alloc }, .. }` guards
decoding. header dimensions are checked before the pixel buffer is allocated, and the budget is
also passed to the underlying decoders. a violation returns `CodecError::LimitExceeded`. the
defaults are 262144 px per side, 2^30 pixels and 8 gib (2 gib on 32-bit targets such as wasm).
PNG also bounds the aggregate decoded text and XMP (UTF-8 keywords and values) by `max_alloc`,
including chunks after IDAT. this budget is separate from pixels, not a total process-memory cap.

## tests

`cargo test -p brahma-codecs` covers:

* the full round-trip matrix (every writable format × 4 sample types × 6 layouts), exact for
  lossless formats and above a PSNR threshold for lossy ones;
* metadata preservation per format, checked against `caps`;
* magic-number detection, including negatives;
* fidelity warnings, and a cross-check that the warnings match the actual round-trip result;
* the symmetric-capability guarantee;
* malformed input: truncation, bit flips and proptest random bytes;
* adam7 PNG against progressive PNG and against the `image` crate as an oracle;
* limit enforcement.

if `corpus/pngsuite/*.png` exists at the repo root, every file in it is compared against the
`image` crate's decoder. without it, that test is skipped.
