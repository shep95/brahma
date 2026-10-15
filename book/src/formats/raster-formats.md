# raster formats

`brahma-codecs` provides format detection, capability declarations, decode limits, metadata handling, and encoding for flat images. the default build declares read/write support for PNG, JPEG, TIFF, webp, GIF, BMP, TGA, ICO, netpbm/PFM, QOI, openexr, and radiance HDR. HEIF/HEIC (iPhone and mac photos) is read-only, decoded in pure rust by `heic-rs` through the optional `brahma-heif` crate. it is a build-time choice behind the `heif` cargo feature (`cargo build -p brahma --features heif`): official builds include it; without it HEIC files are still recognised and opening one reports that HEIC support isn't included in that build. AVIF encoding is optional; AVIF decoding is not implemented in the current crate.

## layered TIFF

a TIFF saved by photoshop with its layers carries them in two private tags: 37724 (`ImageSourceData`, the same layer section a PSD has, preceded by the string `Adobe Photoshop Document Data Block`) and 34377 (the PSD image resources). `brahma-codecs` passes both through as opaque bytes on `Metadata`; `brahma-io` rebuilds a PSD model from them plus the TIFF's pixels and hands it to the PSD importer, so groups, masks, layer styles, type, shapes, smart objects and adjustment layers open from a TIFF exactly as from a PSD, and are written back the same way. in intel-order (`II`) files photoshop stores the whole block structure byte-swapped (reversed keys such as `MIB8` and `ryaL`, little-endian integers and doubles) while pixel samples stay big-endian; `brahma-psd`'s `tiff` module converts between the two orders field by field, and drops, with a warning, any block whose layout it does not know rather than pass byte-swapped data to other readers. limits: a TIFF holds one transparency sample, so extra alpha channels are not written; lab documents are saved flat. save as asks "including layers will increase file size" (with "discard layers and save a copy") while *ask before saving layered TIFF files* is on in preferences › file handling. scripted and agent saves write a flat TIFF unless they ask for layers: `--tiff-layers` for `brahma-cli convert`, `run` and `batch`, and `"tiffLayers": true` in the save, batch and export commands and MCP `doc_save`.

## webp

lossless webp (VP8L) is encoded and decoded by `image-webp`. lossy webp is written by brahma's own VP8 key-frame encoder in `crates/codecs/src/codecs/vp8`, a clean-room implementation of RFC 6386: bt.601 4:2:0 conversion, 16×16 and 4×4 intra prediction chosen per macroblock by reconstruction error, the forward DCT and walsh-hadamard transforms paired with the bit-exact inverse transforms, quantisation on the RFC's `dc_qlookup`/`ac_qlookup` tables, token coding with per-frame probability updates, and skipped macroblocks for flat areas. `EncodeOptions::webp_lossless` picks the encoder and `webp_quality` (0–100, the libwebp scale) sets the quantiser; alpha travels losslessly in an `ALPH` chunk and ICC/EXIF/XMP in their own chunks of the extended (`VP8X`) container. the decoder applies the frame's loop filter; no c library is involved on either side.

capability details such as sample depth, channel layout, alpha, ICC, EXIF, XMP, DPI, and animation behavior are defined by `crates/codecs/src/format.rs`. do not infer fidelity from the filename extension alone.

## exporting documents

every route that writes a document to a flat format (save as, the CLI, MCP `doc_export`, layers to files, layer › export as) flattens it the same way:

- formats without transparency (JPEG, radiance HDR) get the image composited over white, as export as and quick export do; in CMYK, white is no ink.
- formats that can't embed an ICC profile (GIF, BMP, TGA, QOI, ICO, netpbm/PFM) get RGB documents in another profile (linear light, display p3, …) converted to srgb through the colour engine (perceptual intent), since an untagged file is read as srgb. openexr and radiance HDR get linear srgb instead (linear gray for a grayscale openexr), which is how they are tagged when opened.
- TGA, as photoshop's targa: an RGB targa is 24 or 32 bits per pixel. save as and save a copy ask (targa options, starting at 32 with an alpha channel, else 24); export as has a resolution choice (auto, 24 or 32 bits/pixel); scripts pass `tgaBits` (`--tga-bits` on the CLI). at 32 bits the document's alpha channel is the file's alpha when it has exactly one (spot channels don't count; with several, none is used), and the colours are written exactly, never premultiplied or cleared where the alpha is 0, so a game texture's mask and its colours both survive. without a usable alpha channel the layers' transparency is the alpha (opaque without any). 24 bits write no alpha, compositing transparency over white. without a choice: 32 with an alpha channel or transparency, else 24. a 32-bit targa opens as an opaque background with its alpha as "alpha 1", so it round-trips. grayscale targas are 8-bit.

flat TIFF and JPEG exports of composited CMYK documents convert the RGB composite back through
the document's CMYK ICC profile (relative colorimetric with black point compensation) and embed
that exact profile. untagged composites use and embed brahma's synthetic coated CMYK profile;
invalid, non-CMYK or unusable embedded profiles are rejected before compositing. TIFF retains
8/16/32f depth and straight alpha; JPEG reduces to 8-bit with a warning and mattes transparency
over zero-ink white. a plain single raster layer retains its native samples. RGB-only formats use
the RGB composite directly. this RGB composite roundtrip does **not** preserve original ink
separations, black generation or native CMYK blending, and is not a claim of photoshop parity.

## implemented decode limits

`DecodeOptions::default()` applies `Limits` before decoded pixel allocation:

| limit | default |
|---|---:|
| maximum width | 262,144 pixels |
| maximum height | 262,144 pixels |
| maximum pixel count | 1,073,741,824 pixels |
| maximum decoded allocation | 8 gib on 64-bit targets; 2 gib on 32-bit targets (including wasm32) |

format adapters pass compatible allocation limits into PNG, TIFF, webp, and `image`-based decoders, while other implementations perform explicit checked dimension/buffer validation. tests cover bomb-like headers, truncation, bit flips, header overwrites, and random data.

`Limits::none()` exists for callers with a deliberate reason to relax policy. it remains bounded by representable memory sizes, but it removes the default resource policy and should not be used for untrusted input.

animated containers and multi-page tiffs are represented as a single decoded frame or page by the current codec API; opening one adds an import warning such as "only the first of 3 frames was imported". a JPEG cut off inside its image data opens with a warning that its data ends early (an error when it holds no image data at all). refer to [`crates/codecs/README.md`](https://github.com/shep95/brahma/blob/main/crates/codecs/README.md) and source capability declarations for current behavior.
