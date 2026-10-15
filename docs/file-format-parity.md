# file-format parity

> **last reviewed:** 2026-10-12 · **last updated:** 2026-10-13 · **change:** minor (CMYK hue/saturation and color balance keep the black ink; CMYK posterize on the stored values, psd-tools oracle floor 242/309; CMYK adjustment layers composite on the document's inks; photoshop-oracle floor 165/258; CMYK brightness/contrast on the inks, CMYK threshold's luma; modern brightness/contrast's brightness curve; photoshop-oracle floor 151/258; lab documents composite on their stored l, a, b; photoshop-oracle floor 152/258; levels on whole levels at every step with photoshop's shadow toe; photoshop-oracle floor 146/258, psd-tools 238/309; 32-bit adjustments unclipped; photoshop-oracle floor 149/258; mask feathers blur with their exact variance; threshold splits on photoshop's luminosity; a pass-through clipping base with an adjustment renders isolated; psd-tools oracle floor 240/309; 16-bit posterize as photoshop bins it; psd-tools oracle floor 241/309; gradient map luminosity; photoshop-oracle floor 147/258; PSD: `Mt16`/`Mt32` merged transparency read; palette pngs open in indexed color mode, #4804; earlier: CMYK invert, posterize and threshold on the inks; 16-bit lab invert; photoshop-oracle floor 145/258; `.atn` import maps 112 events; DDS textures open, #1327; DPX read and write, #1049; #2465 fixed: CS2 placed layers with only a `PlLd` block open their contents; photoshop actions `.atn` read and write; `.shc` contour files read and write; earlier the same day: indexed BMP palette mode; save keeps a JPEG's quality and a webp's encoding) · **target:** adobe photoshop 2026

every format photoshop 2026 (27.11.0) reads or writes, with brahma's support, fidelity and
tests. photoshop's list comes from its `Info.plist` `CFBundleDocumentTypes`, the pipl read/write
flags of `Contents/PlugIns/Required/File Formats`, and format names in the main binary (open/save
as behaviour from adobe's documentation where the bundle doesn't say). ours comes from
`crates/codecs/src/format.rs` (capability table), `crates/io/src/lib.rs` (import/export
dispatch), `crates/psd`, `crates/raw`, `crates/heif`, `crates/affinity`. more detail on our side:
[raster formats](../book/src/formats/raster-formats.md), [PSD/PSB](../book/src/formats/psd-psb.md),
[OpenRaster](ora.md), [paint.NET](pdn.md).

**summary.** by count, 13 of photoshop's 31 formats are fully supported in the directions photoshop supports them, 4 partially (48%,
measured). weighted by use (PSD/PSB 40%, JPEG 15%, PNG 15%, TIFF 10%, camera raw 8%, webp/AVIF/
HEIF/JXL 6%, PDF 3%, everything else 3%), presence is ~80% and ready-for-real-work ~60%: PSD
is broad but not yet reliable against photoshop (gap g1), and photoshop PDF, AVIF read and JPEG
xl are missing (gap G10). remaining: 300–500 h including PSD fidelity.

## photoshop's formats

| format | photoshop | brahma read | brahma write | fidelity and tests |
|---|---|---|---|---|
| PSD | r/w | yes | yes | 1/8/16/32-bit; bitmap, gray, indexed, RGB, CMYK, multichannel, lab, duotone; layers, groups, masks, vector masks, adjustment and fill layers, effects, text, shapes, smart objects (SoLd/lnk2; CS2 PlLd/lnkd read with their warp, kept as is or re-saved as SoLd once edited), smart filters (14 mapped plus filter gallery with chrome, rest verbatim), slices, comps, advanced blending. oracle floors: photoshop-authored 185/258, io 148/170, psd-tools 249/309; round trip ≥ 169/170, 308/309, 258/256. known failures: #1281, #2469 (#2374, text layers, closed 2026-10-10; #2465, CS2 `PlLd`-only smart objects, fixed 2026-10-12) |
| PSB | r/w | yes | yes | auto-switches above 2 gb (REL-212-2); 4–5 gb fixtures generated (FILE-216-2); > 4 gb fails on windows (#375) |
| cloud PSD (.psdc) | r/w | no | no | out of scope (adobe cloud) |
| TIFF | r/w | yes | yes | 8/16/32f, gray/RGB/CMYK, bigtiff, every IFD, banded parallel decode; layered TIFF both byte orders (FILE-215-7: extra alpha channels not written) |
| JPEG | r/w | yes | yes | 8-bit gray/RGB/CMYK, ICC/EXIF/XMP; save keeps the opened file's quality (estimated from its quantisation tables, #2399); no JPEG options dialog (quality 0–12, baseline/progressive) (FILE-215-19) |
| PNG | r/w | yes | yes | 8/16-bit, ICC/text; APNG first frame only; alpha channels on open (#2225); palette pngs open in indexed color mode with their transparent entry (#4804) |
| GIF | r/w | yes | yes | file › open keeps the first frame; new video layer from file and video frames to layers take every frame, with the timeline rate from the delays (#572). per-frame delays and keeping the frames in `.pcraft` are not done |
| BMP | r/w | yes | yes | 8-bit; palette bmps open in indexed color mode |
| webp | r/w | yes | yes | lossless and our own lossy VP8 encoder; ICC/EXIF/XMP; save keeps an opened file lossy (at its quality) or lossless (#2399) |
| AVIF | r/w | no (#463) | optional feature, off by default | |
| JPEG xl | r/w | inside DNG only | no | |
| HEIF/HEIC | r/w(HEIF) | yes (pure-rust heic-rs, `heif` feature, on in releases) | no | 8/16-bit |
| openexr | r/w | yes | yes | f16/f32; deep EXR and cryptomatte readers (`corpus/exr`) |
| radiance HDR | r/w | yes | yes | f32 |
| portable bit map (PBM/PGM/PPM/PNM/PFM/PAM) | r/w | yes | yes | 8/16/32f |
| FITS | r/w (FITS liberator plug-in) | yes | yes | 8/16/32f gray or RGB, header cards kept; no fpack, no debayer |
| targa | r/w | yes | yes | 8-bit |
| photoshop PDF | r/w | no (#2541) | raster only (print, artboards to PDF) | no vector/text PDF, no multi-page save, no PDF presentation (FILE-215-10) |
| photoshop EPS, DCS 1.0/2.0, EPS TIFF/PICT preview | r/w | no | no | |
| generic EPS / ai | r (rasterize) | no | no | |
| SVG | r (rasterize) | yes, as editable shape layers or a vector smart object | no | better than photoshop for editing; gradients approximated (FILE-215-9) |
| JPEG 2000 | r/w | no (#1047) | no | |
| cineon / DPX | r (DPX), r/w (CIN) | DPX: luma, RGB, RGBA at 8/10/12/16 bits, both byte orders (#1049); cineon no | DPX: 8/10/12/16 bits (`dpxBits`, `--dpx-bits`); cineon no | writes DPX, which photoshop only reads; header fields (creator, project, copyright, transfer, frame rate, time code…) round-trip; no cineon, no RLE |
| DICOM | r/w | no | no | |
| PCX | r/w | no | no | |
| pixar | r/w | no | no | |
| amiga IFF | r/w | no | no | |
| wireless bitmap | r/w | no | no | |
| MPO / JPS (stereo) | r/w | no | no | |
| PICT | r | no | no | legacy |
| scitex ct | r/w | no | no | legacy |
| photoshop raw (.raw) | r/w | no | no | |
| camera raw (via camera raw 18.7) | r | partial | – | see below |

## camera raw

photoshop opens raws through camera raw 18.7 (the bundle registers cr2, crw, dcr, dng, erf, mos,
fff, 3fr, mef, mfw, iiq, nrw, rwl, srw, mrw, nef, orf, pef, raf, srf, x3f; camera raw itself
supports many more, including CR3).

| camera format | brahma | evidence |
|---|---|---|
| DNG (incl. LJPEG and JPEG xl tiles) | yes; lossy DNG no | FILE-215-11 |
| canon CR2 | yes | |
| canon CR3 | detected, unsupported | `crates/raw/src/lib.rs:219` |
| nikon NEF/NRW (lossless, lossy) | yes; "lossy after split" no | #50; D5100 image area measured on three 14-bit lossy nefs; [validation](nikon-nef-development.md) |
| sony ARW/SR2/SRF (incl. compressed) | yes | |
| panasonic RW2, pentax PEF | yes | |
| olympus ORF | uncompressed only | |
| fujifilm RAF (bayer and x-trans) | uncompressed only; no fuji colour calibration | |
| others (CRW, 3fr, IIQ, X3F, SRW, MRW, ERF…) | embedded preview only | |
| .xmp sidecars, highlight modes, raw preferences | missing | FILE-215-11 |
| camera raw as a PSD smart filter | partial | 2026-10-07: wb, light/presence, curves, HSL, grading, detail, grain, vignette mapped (FILE-215-12) |

2026-10-11: on #2135's embedded-JPEG rendering work, open-time camera raw tuning
preserves the fitted rendering while changing exposure, temperature and tint. default
tuned import matches normal import exactly; output remains full-resolution 16-bit ProPhoto.
this improves appearance when a usable preview is present, without adding a measured
D5100 colour profile. counts and the G14 estimate are unchanged.

## formats photoshop doesn't have (ours only)

| format | read | write | notes |
|---|---|---|---|
| `.pcraft` (native) | yes | yes | incremental, autosave, crash recovery, atomic save |
| affinity `.af`, `.afphoto`, `.afdesign`, `.afpub` | yes | no | see below |
| paint.NET `.pdn` | yes | no | [pdn.md](pdn.md) |
| OpenRaster `.ora` | yes | yes | [ora.md](ora.md) |
| ICO, QOI | yes | yes | |
| DirectDraw surface `.dds` | yes | no | top mip level of a 2d texture: uncompressed RGB, RGBA or luminance, BC1–BC3 (DXT1/3/5), legacy or DX10 header. cubemaps, arrays, volumes and BC4–BC7 are refused (#1327). photoshop needs a plug-in |
| krita `.kra`, GIMP `.xcf`, procreate | no | no | not in photoshop either |

### affinity import notes (moved from `roadmap.md`)

2026-10-08: affinity `.af`, `.afdesign`, `.afphoto` and `.afpub` documents (container versions
8–12) open **natively** ([format notes](../crates/affinity/README.md)): pages and artboards, layers
and groups, curves and geometric shapes with fills, gradients and strokes, text as type layers,
placed images as smart objects, pixel layers and masks, as an 8-bit RGB document without a source
save path. measured against the thumbnail affinity embeds in each file, 21 pinned public documents
(`cargo xtask corpus --affinity`) differ by 0–4.6 of 255 on average. layer effects, adjustments,
live filters, brush strokes, special shapes, master pages and CMYK/lab/16-bit document colour are
approximated or left out, each with a warning; damaged or unknown files fall back to the embedded
preview. affinity writing is not implemented: no affinity installation was available to check
written files, so `.af` export stays unsupported.

2026-10-10: current `.af` artboard properties (`phrp`/`aprp`) are recognized alongside the
legacy flag, including converted curve boards. synthetic two-board regressions check separate
bounds, overflow clipping, `.pcraft`/PSD board and pixel round trips, and moving one board with its
children through undo/redo. nested boards remain masked groups; rotated/curved board outlines
still import at their bounding rectangle.

2026-10-09: `corpus/affinity/` now has 39 pinned documents and 21 pngs (20 rendered references and
one bitmap-fill texture): the prior 21 public documents plus 18 CC0 samples for #1606. the new
samples compare against their exported pngs;
mean differences range from 0.44/255 (conical gradients) to 86.05/255 (RGB/32 reduced to 8-bit).
clouds, hearts, cogs, callouts, arrows, double stars, tears, crescents, diamonds and circular segments
now import as parametric shapes; the special-shapes sample measures 5.68/255 against its PNG. the
remaining gaps in #1606 stay tracked with per-sample corpus ceilings.

## presets and data files

| format | photoshop | brahma | notes |
|---|---|---|---|
| brushes `.abr` | r/w | r/w | `psd/src/abr.rs` (v6/v12) |
| gradients `.grd` | r/w | r/w | |
| patterns `.pat` | r/w | r/w | |
| swatches `.aco`, `.ase` | r/w | r/w | FILE-215-14 |
| luts `.cube` / `.3dl` / `.look` | r/w (export color lookup) | r/w / r / r | `cms/src/lutfile.rs` |
| keyboard shortcuts `.kys` | r/w | r | `engine/src/kys.rs` |
| ICC profiles | r | r | own CMS |
| actions `.atn` | r/w | r/w | `psd/src/atn.rs` (version 16 sets, exact round trip), `engine/src/atn_cmds.rs` (112 photoshop events ↔ commands; other steps kept verbatim). panel menu, CLI `batch --actions set.atn`, `actions.load`/`actions.save`. photoshop 5-era version 12 files unsupported (AUTO-219-2) |
| styles `.asl` | r/w | no | #1693 |
| contours `.shc` | r/w | r/w | `psd/src/shc.rs`; contour editor load…/save… (FILE-215-15). no user contour library; not yet checked against a photoshop-written file |
| custom shapes `.csh`, tool presets `.tpl` | r/w | no | |
| curves `.acv`, levels `.alv`, other adjustment presets | r/w | not audited | |
| C2PA content credentials | r/w | no | #1050 |

## revision history

| date | change | summary |
|---|---|---|
| 2026-10-14 | minor | corpus floors raised to the measured results of the CMYK, lab, 32-bit, levels, threshold, posterize, feather and pass-through fixes landed together: photoshop-authored 165 → 185 of 258, io 146 → 148 of 170, psd-tools 242 → 249 of 309 |
| 2026-10-13 | minor | CMYK hue/saturation (not colorize) and color balance work on c, m and y as inverted RGB with black kept, on the inks for display, export and destructive edits; cmyk8/16 hue-saturation 100% → 0.4–0.7% bad pixels, color-balance 74% → 25% |
| 2026-10-13 | minor | CMYK posterize works on the stored values (1 − ink), as photoshop does: psd-tools posterize_cmyk passes (13.8 % → 0 of pixels off on the ink path); psd-tools floor → 242 of 309 |
| 2026-10-13 | minor | CMYK documents whose layers are plain raster layers plus invert, posterize, threshold, brightness/contrast, levels or curves layers composite on their own inks for display, export and flattening: the cmyk8/16 oracles for those seven adjustments pass; photoshop-oracle floor 151 → 165 of 258 |
| 2026-10-13 | minor | CMYK threshold compares the rec. 601 luma of the inks' plain complement, as photoshop does (every pixel of cmyk8/16 threshold-128.psd); oracle bad pixels 1.6% → 0.2% |
| 2026-10-13 | minor | CMYK brightness/contrast (modern and legacy) curves each ink's stored value (1 − ink), exact on destructive edits; cmyk8/16 oracle bad pixels 71% → 48–57% (the RGB composite's separation still differs) |
| 2026-10-13 | minor | modern brightness/contrast: the brightness curve refit to photoshop, so the rgb and gray 8/16/32 brightness-contrast oracles pass; photoshop-oracle floor 145 → 151 of 258 |
| 2026-10-13 | minor | lab documents of plain layers with invert, posterize, threshold, brightness/contrast or lab levels/curves adjustment layers composite on the stored l, a, b instead of clipped display RGB, and flattening keeps them: lab16 invert, posterize-4, levels, curves and legacy brightness/contrast and lab8 invert and posterize-4 pass; photoshop-oracle floor 145 → 152 of 258 |
| 2026-10-13 | minor | levels round the gamma and output steps to whole levels too (8-bit), and the gamma above 1 starts along a toe of slope 2^gamma that joins the power curve: psd-tools levels_rgb and rgb8 stack-levels-curves-hue-posterize pass, rgb8/gray8 levels.psd exact; photoshop-oracle floor 145 → 146 of 258, psd-tools 236 → 238 of 309 |
| 2026-10-13 | minor | 32-bit documents: legacy brightness/contrast, channel mixer, photo filter and the black & white tint keep values above 1 and below 0 as photoshop does (rgb32/gray32 brightness-contrast-legacy, rgb32 channel-mixer and black-white-tint pass); photoshop-oracle floor 145 → 149 of 258 |
| 2026-10-13 | minor | mask feathers blur with exactly the feather's variance (three extended boxes; whole-pixel boxes rounded 6 px to 6.48): psd-tools mask-parameters-no-real-channel passes, layer_mask_data 27 % → 4 % off; psd-tools floor → 240 of 309 (main 239) |
| 2026-10-13 | minor | threshold splits on the luminosity 0.30 r + 0.59 g + 0.11 b of whole samples, as photoshop does (8-bit: rounded to a level, halves down; 16-bit: against the level in 16 bits), not the rec. 601 grey: psd-tools threshold_rgb and threshold_16bits_rgb pass; psd-tools floor → 241 of 309 (main 239) |
| 2026-10-13 | minor | a pass-through group that is the base of a clipping group and holds an adjustment layer renders isolated, as photoshop does (its blend modes still reach the backdrop): psd-tools passthrough_clipping_mask_adjustment passes (was 50 % of pixels off); psd-tools floor → 240 of 309 (main 239) |
| 2026-10-13 | minor | 16-bit posterize bins the 16-bit sample and lands on 8-bit levels through a 16-bit step, as photoshop does (RGB, grayscale and the CMYK/lab channel path): psd-tools posterize_16bits_rgb and posterize_16bits_grayscale pass; psd-tools floor → 241 of 309 (main 239) |
| 2026-10-13 | minor | gradient map adjustment layers are driven by the W3C luminosity (0.3, 0.59, 0.11), not the rec. 601 grey: rgb8 and rgb16 gradient-map.psd pass; photoshop-oracle floor 145 → 147 of 258 |
| 2026-10-13 | minor | PSD: a "saving merged transparency" global block (`Mtrn`, `Mt16`, `Mt32`; photoshop's 32-bit documents keep a positive layer count) marks the merged image's first extra channel as its transparency: opening such a file no longer adds a "transparency" alpha channel, and the oracle compares psd-tools transparentbg.psd's alpha (83.7 % → 0.9 % of pixels off; its shape edges remain) |
| 2026-10-13 | minor | palette pngs open in indexed color mode (#4804) |
| 2026-10-12 | minor | `.atn`: 112 events mapped (select and mask, filter gallery spatter, ripple, crystallize, pattern fill layers, mask targeting, step backward, points, bitmap) |
| 2026-10-12 | minor | DDS textures open, read-only (#1327) |
| 2026-10-12 | minor | #2465 fixed: photoshop CS2 placed layers (`PlLd` only, file in `lnkD`) open as editable smart objects and save their edits |
| 2026-10-12 | minor | CMYK invert, posterize and threshold work on the inks and 16-bit lab invert mirrors a and b around neutral (oracle bad pixels: CMYK 71–77% → 9–10%, 37% → 1.6%; lab16 invert 52% → 1.2%); photoshop-oracle floor 133 → 145 of 258 |
| 2026-10-12 | minor | DPX read and write (#1049); cineon still missing |
| 2026-10-12 | minor | photoshop actions `.atn`: read and write, 90 events mapped, the rest kept verbatim (AUTO-219-2) |
| 2026-10-12 | minor | presets table: contours `.shc` read and write through the contour editor (FILE-215-15); `.csh` and `.tpl` stay missing |
| 2026-10-12 | minor | palette bmps open in indexed color mode (#3802) |
| 2026-10-12 | minor | save writes a JPEG at its own estimated quality and a webp lossy or lossless as opened (#2399) |
| 2026-10-11 | minor | PSD smart filters: filter gallery (`GEfc`) maps to `filter.filterGallery` when every effect is modelled (chrome so far), editable from the layers panel |
| 2026-10-11 | minor | D5100 image area and fitted-look preservation in camera raw tuning |
| 2026-10-10 | major | first version: photoshop 2026 27.11.0's formats from its bundle, against our codecs; affinity notes moved from `roadmap.md` |
