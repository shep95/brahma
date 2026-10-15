# brahma-raw

a clean-room, pure-rust camera raw decoder and developer. the crate is standalone (no workspace
dependencies; its decoding dependencies are `flate2` for deflate DNG and `jxl-oxide`, a pure-rust
JPEG xl decoder, for DNG 1.7, native builds only), has no `unsafe`, does no i/o (`&[u8]` in), builds for `wasm32-unknown-unknown`
(sequential there, rayon-parallel on native) and never panics on hostile input: every offset is
bounds-checked and sizes are checked against `Limits` before allocating.

```rust
use brahma_raw::{develop, DevelopOptions, Demosaic};

let dev = develop(&bytes, &DevelopOptions { demosaic: Demosaic::Ahd, ..Default::default() })?;
// dev.rgb: interleaved 16-bit RGB in ProPhoto RGB (ROMM primaries, D50, gamma 1.8)
// dev.warnings: anything approximated or not applied
```

`brahma-io` uses it so opening a raw file yields a normal 16-bit RGB document tagged with the
built-in ProPhoto-compatible profile.

## sources (clean-room)

implemented only from public specifications, papers and observation of files:

* TIFF 6.0, TIFF/ep (ISO 12234-2) and the adobe DNG specification 1.7.
* ITU-t t.81 (ISO 10918-1) annex h: lossless JPEG, process 14 ("LJ92").
* the published description of canon's CR2 container (header, raw IFD, slice tag 0xc640).
* publicly documented maker-note / private tags (ExifTool's tag tables): canon modelid (0x0010),
  SensorInfo (0x00e0) and ColorData (0x4001), nikon wb_rblevels (0x000c) and BlackLevel (0x003d), sony
  BlackLevel (0x7310), wb_rggblevels (0x7313), SonyRawFileType (0x7000) and SonyToneCurve
  (0x7010), the PanasonicRaw IFD0 tags, olympus ImageProcessing (0x2040) and CameraSettings
  (0x2020) preview tags, the fujifilm RAF header and metadata records (RawImageCropTopLeft /
  RawImageCroppedSize, FujiLayout, xtranslayout, wb_grgblevels, RawExposureBias) and fujiifd
  (RawImageFullWidth / height, BitsPerSample, StripOffsets / ByteCounts, BlackLevel,
  wb_grblevels).
* nikon compressed NEF: laurent clévy's NEF structure notes and bill claff's "NEF compression"
  (prose: the `0x0096` table layout, a lossy curve followed by a fixed-table huffman stage),
  ExifTool's nikon tag-name documentation and ITU-t t.81 annex f / h. the huffman tables are not
  stored in the files; they were recovered by black-box analysis of CC0 raw.pixls.us samples (the
  same scenes shot compressed and uncompressed) for #houseofasher (#houseofasher issue 86) and
  verified there on ~45 files from ~35 bodies.
* sony craw (ARW 2): h. dietz, "sony ARW2 compression: artifacts and credible repair"
  (electronic imaging 2016) and the RawDigger / diglloyd write-ups of the 11 + 7-bit scheme;
  the exact bit layout and tone-curve scale were established by observation of sample files.
* panasonic RW2 RawFormat 5, uncompressed olympus ORF and uncompressed fujifilm RAF: established
  by observation of sample files (bit packing, page layout, sample justification; for RAF the
  three sample storages and the reversed order of the x-trans / bayer layout records, each
  verified by developing with every candidate phase against the camera's preview).
* panasonic RW2 RawFormat 3/4 (vendor-compressed): the fixed-rate 16 bytes per 14 pixels, the
  predictor groups with per-parity predictors, the 2/4/8-bit field widths, the page-permuted
  downward bit order and the black level sitting 15 counts above the recorded tags were all
  established by observation of sample files.
* demosaicing: malvar, he & cutler (ICASSP 2004); hirakawa & parks, "adaptive
  homogeneity-directed demosaicing" (IEEE TIP 2005); for x-trans and other non-bayer cfas,
  gradient-weighted green interpolation (lu & tan, "color filter array demosaicking: new method
  and performance measures", IEEE TIP 2003) with edge-aware colour differences.
* McCamy's CCT approximation (1992); the bradford chromatic adaptation transform.
* per-file colour from the camera's embedded JPEG (`fit_look`, `src/look.rs`): the method follows
  the prose description in #houseofasher's `docs/camera-preview-colour.md` (#houseofasher issue 165,
  apache-2.0, same organisation); no #houseofasher code was used. pool-adjacent-violators isotonic
  regression (ayer et al., 1955), tukey's biweight (beaton & tukey, 1974), ridge regression
  (hoerl & kennard, 1970) and the CIE 1976 l\*a\*b\* colour difference.

no code from dcraw, LibRaw, rawspeed, rawler, rawloader or darktable was read or used, and no
camera colour tables were copied.

## support matrix

| format | status |
|---|---|
| DNG | uncompressed (8–16 bit, packed or not), deflate (compression 8, predictors 1 and 2), lossless JPEG and JPEG xl (DNG 1.7, e.g. samsung expert RAW; decoded by `jxl-oxide`; not in wasm32 builds, where it is `Unsupported`); strips and tiles; CFA (bayer) and LinearRaw (three-sample colour, or one-sample monochrome such as leica monochrom and film-scanner dngs, developed as neutral grey with no white balance or colour matrix); LinearizationTable, BlackLevel (+ repeat, deltah/v), WhiteLevel, ActiveArea, DefaultCrop, ColorMatrix1/2, CameraCalibration, ForwardMatrix, AnalogBalance, AsShotNeutral / asshotwhitexy, BaselineExposure, orientation, OpcodeList2 GainMap (lens shading) |
| DNG (lossy JPEG, floating point; opcodes other than GainMap; ProfileGainTableMap) | unsupported / not applied (reported) |
| CR2 | lossless JPEG with slices, borders and as-shot white balance from the maker note, black measured on the masked border. CR2 has no CFA tag and the row phase varies by model, so it is measured from the data (the green diagonal), with a canon model-id table as the fallback (see `src/cr2.rs`) |
| CR2 sraw / mraw | unsupported |
| NEF / NRW, ARW, PEF and other TIFF/ep raws | uncompressed and lossless-JPEG (incl. sony lossless ARW) CFA data |
| nikon compressed NEF (compression 34713): lossless and lossy type 1 / 2, 12 and 14 bit | decoded: fixed huffman tables + maker-note `0x0096` seeds and curve (see `src/nefc.rs`). BlackLevel `0x003d` is read in 14-bit units. "lossy after split" files are unsupported (preview fallback) |
| sony compressed ARW ("craw", SonyRawFileType 2) | decoded: 11-bit min/max + 7-bit delta blocks, SonyToneCurve to 14 bits, the curve's 512 black level when no BlackLevel tag is written (the first-generation bodies, e.g. the ILCE-7) |
| panasonic / leica RW2, RawFormat 5 (12- and 14-bit packed) | decoded, with PanasonicRaw black / white / wb / sensor borders |
| panasonic / leica RW2, RawFormat 3 and 4 (vendor-compressed, 12-bit) | decoded; the black level is read 15 counts above the recorded tag; PanasonicRaw white / wb / sensor borders |
| olympus ORF, uncompressed 16-bit (e-1, e-400…) | decoded, with ImageProcessing black / wb / ValidBits / crop |
| nikon "lossy after split" NEF, sony "compressed RAW 2", pentax compressed PEF, RW2 RawFormat 6 and newer, olympus compressed ORF | unsupported: no public description of these codes was found apart from GPL decoder source, which this crate may not use (clean-room). `brahma-io` opens the embedded JPEG preview instead |
| fujifilm RAF, uncompressed (x-trans i–v, GFX and bayer bodies since about 2010) | decoded: 16-bit containers (either byte order), 12-bit LSB-first packing, 14-bit packing in 32-bit words; x-trans 6×6 or bayer 2×2 layout, black, as-shot wb, crop, RawExposureBias, orientation from the preview's EXIF |
| fujifilm compressed RAF, early FinePix / superccd RAF (no CFA TIFF) | unsupported (no public description of the compression); preview fallback |
| CR3 | native CRX unsupported; bounded JPEG-track/PRVW preview fallback with CMT1 orientation. i/o labels the result as an 8-bit camera preview |
| x-trans and other periodic non-bayer cfas (up to 16×16) | demosaiced by one pattern-agnostic edge-directed method, whatever `Demosaic` is chosen |

nikon maker-note compression modes 6/10 select checked little-endian packed 12/14-bit
rows even when TIFF compression is 34713. the modern lossy `0x44 0x40` curve uses
its smaller encoded domain while preserving sensor adus. valid nikon CropArea
coordinates take precedence over model fallbacks. he/he* and lossy-after-split
remain unsupported. four pinned CC0 z6 files cover both bit depths and both paths.

## development pipeline

1. linearization table, per-position black level, scale to the white level.
2. white balance (as shot; or grey-world when the file has none; or explicit multipliers),
   normalized so the smallest multiplier is 1, then clip to 1 so blown highlights stay white.
3. demosaic: `Bilinear`, `Mhc` (malvar–he–cutler) or `Ahd` (default) for bayer; non-bayer cfas
   (x-trans) always use the edge-directed generic method (`demosaic_cfa`).
4. camera → XYZ (D50) per the DNG specification (ColorMatrix interpolated by the white's
   correlated colour temperature, or ForwardMatrix), or a camera look fitted to the embedded JPEG,
   → linear ProPhoto; exposure (BaselineExposure + user ev); the look's tone curve; gamma 1.8;
   16 bits.
5. orientation.

development validates public sensor/crop/CFA metadata before allocation. the working-memory
limit checks peak live development buffers instead of summing disjoint phases. AHD limits
simultaneous scratch bands only when required; normal load balancing and pixels are preserved.
the default 2 gib limit is unchanged and excludes caller-owned sensor/document storage.

files without colour calibration (CR2, NEF, ARW, RW2, ORF, RAF…) have no camera matrix. every one of
them carries a JPEG the camera rendered from the same sensor data, so `fit_look` estimates the
camera's rendering per file: both the sensor data (white-balanced as `develop` does) and the
decoded JPEG are reduced to a common grid of at most 256 cells on the long edge, and a 3×3
camera → ProPhoto matrix plus one monotone tone curve (applied per channel after exposure) are fitted
so that the developed raw matches the JPEG, by alternating isotonic regression for the curve and
weighted, ridge-regularised least squares for the matrix (residuals scaled to approximate l\*,
tukey biweights against misaligned edges and local tone mapping; clipped and near-black cells left
out). one cell in three is held out: the look is used only when it shows the same picture in colour,
the matrix is plausible and the held-out mean CIE76 δe drops to at most 85 % of the fallback's.
pass it as `DevelopOptions::camera_look`; `develop` then notes "colour fitted to the camera's
embedded JPEG (mean δe …)". the crate cannot decode JPEG itself: `brahma-io` decodes the
smallest preview at least 512 pixels long, fits and develops in one go.
camera raw's tuned import fits at the default exposure and as-shot white balance first,
then applies the user's exposure, temperature and tint to that same rendering.
the D5100's measured image area is 4928 × 3264 at (8, 8) in a 4992 × 3280
sensor plane; it removes the border without changing levels, wb or calibration.
measured image areas preserve an explicit file crop.

for a slightly tighter camera JPEG, fitting can align the reference proxy using a bounded
90–100% centred search on training cells. it requires a strong correlation improvement and
retains the held-out colour gates; the developed crop and camera calibration are unchanged.

without a usable fit (no preview, an unrelated, monochrome or differently shaped one, or a fit that
does not clearly help) the documented neutral fallback stays: the white-balanced camera channels are
treated as linear srgb primaries, with no tone curve (plausible but flat, greyish colours; converting
to DNG gives calibrated colour), and `develop` says so.

measured on 8 nikon D3200 nefs (12-bit lossy compressed, indoor scenes) against each file's
embedded JPEG (mean CIE76 δe, 300-pixel grid, full develop output in srgb): fallback 11.6–26.0,
fitted 1.0–7.5; all 8 fits were accepted. the highest remaining errors are shots where the camera
brightened shadows locally (active d-lighting), which one global curve cannot follow. the fit
(preview decode included) takes about 50–70 ms on a 24 mp file. it is an estimate of the camera's
look, not a measured calibration: picture styles and hue-dependent rendering are approximated.

cameras with a profile measured against photoshop's camera raw (`src/cameras.rs`: the nikon d4) carry
a calibration, so no look is fitted for them. on 6 d4 nefs against photoshop 25.4's default camera
raw rendering (quarter size, medians over the frames) the measured profile is 0.048 stop from its
luminance with a mean CIE76 δe of 1.8; the look fitted to the embedded JPEG would be 0.30 stop and
δe 6.4, as it follows the camera's picture style instead. the open-time re-develop
(`brahma-io`'s `import_raw_tuned`) and the tone curve its previews use (`tone_curve`) take the
same look as the default open, so changing a slider does not change the colour rendering.

## tools

`cargo run --release -p brahma-raw --example rawinfo -- [--dump] [--demosaic ahd] [--png DIR] FILE...`
prints what was decoded, times decode and develop, and can write srgb PNG previews and the
embedded JPEG previews.

`cargo run --release -p brahma-io --example rawlook -- [--png DIR] FILE...` times the
camera-JPEG colour fit, reports δe against the embedded JPEG with the neutral fallback and with the
fit, and can write 1200-pixel srgb pngs of both and of the JPEG; `--synthetic` times it on a
generated 24 mp file.
