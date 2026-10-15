# brahma-heif

the optional HEIF/HEIC decoder of brahma: iPhone and mac photos. a thin, panic-guarded wrapper
around `heic-rs`, a pure-rust HEVC still-picture decoder
(no `unsafe`, MIT or apache-2.0): single pictures and grid-tiled photos, 8- and 10-bit (10-bit
decodes to 16-bit), alpha auxiliary images, the container's rotation/mirror/crop, ICC, EXIF and XMP.
read-only: writing would need an HEVC encoder.

```rust
let info = brahma_heif::probe(&bytes)?;             // size, depth, alpha: check limits first
let img = brahma_heif::decode(&bytes, &brahma_heif::Options::default())?;
// img.width, img.height, img.has_alpha, img.sixteen_bit, img.data (RGB/RGBA), img.icc, img.exif, img.xmp
```

errors are `Error::Unsupported` (image sequences, overlays, …), `Error::Limit` or
`Error::Malformed`. it never panics: every call into heic-rs runs under `catch_unwind`, and a panic
inside it (fuzzing found two in 0.1.1) becomes `Error::Malformed`.

## why it is a separate, optional crate

- **a young decoder.** heic-rs is new and has a single maintainer. it is pinned exactly
  (`=0.1.1`); moving the pin is a reviewed change (re-run `cargo fuzz run decode_heif` in
  `crates/codecs` and `cargo xtask test-corpus`).
- **HEVC patents are a distributor's call.** HEVC is patent-encumbered in some jurisdictions, so
  whether a build includes an HEVC decoder is a build-time choice. `brahma-codecs` uses this
  crate only behind its non-default `heif` feature; the apps forward it
  (`cargo build -p brahma --features heif`). official brahma builds and ci enable it.
  without it, HEIC files are still recognised and opening one is a clear "HEIC/HEIF support isn't
  included in this build" error.

the crate depends on no other brahma crate and knows nothing of `brahma-codecs`' types;
`codecs` → `heif` is the one allowed dependency between standalone crates (`cargo xtask layers`).
tests on real files live in `brahma-codecs` (`tests/heif.rs`, features `corpus,heif`, files
fetched by `cargo xtask corpus --heif`).
