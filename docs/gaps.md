# where brahma falls short of photoshop

> **last reviewed:** 2026-10-10 · **last updated:** 2026-10-13 · **change:** minor (g1: CMYK hue/saturation and color balance keep the black ink; g1: CMYK posterize on the stored values, psd-tools oracle floor 242/309; g1: CMYK adjustment layers composite on the document's inks, photoshop-oracle floor 165/258; g1: CMYK brightness/contrast on the inks, CMYK threshold's luma; g1: modern brightness/contrast's brightness curve matches photoshop, photoshop-oracle floor 151/258; g1: lab documents composite on their stored l, a, b, photoshop-oracle floor 152/258; g1: levels on whole levels at every step, photoshop-oracle floor 146/258, psd-tools 238/309; g1: 32-bit brightness/contrast legacy, channel mixer, photo filter and black & white tint unclipped as in photoshop, photoshop-oracle floor 149/258; g1: mask feathers blur with their exact variance, psd-tools oracle floor 240/309; g1: threshold splits on photoshop's luminosity, psd-tools oracle floor 241/309; g1: pass-through clipping base with an adjustment renders isolated, psd-tools oracle floor 240/309; g1: 16-bit posterize as photoshop bins it, psd-tools oracle floor 241/309; g1: gradient map driven by the W3C luminosity, photoshop-oracle floor 147/258; g7: field blur's on-canvas pins, #3373; g4: hue jitter amount track, #3825 partial; g4: minimum field interaction, #3825 partial; g1: CMYK invert, posterize and threshold on the inks, 16-bit lab invert; photoshop-oracle floor 145/258; G11: `.atn` import maps 112 events: select and mask, filter gallery spatter, ripple, crystallize, pattern fill layers, mask targeting, step backward, nudges in points, bitmap mode; g5: free transform h/v skew fields; g9: toolbar tool shortcut editing; refreshed against main: g2, g5, g7, g9, G10, G13, G14, G17 items that landed; G10: DPX read and write, #1049; g4: checkbox-only brush settings rows; G11: load/save actions `.atn` landed, AUTO-219-2 partial; g5: hold-a-key spring-loaded tools landed, #2672; g1: #2465 fixed; stale missing-tool rows corrected: color replacement, red eye and add/delete anchor point are live tools, so only art history brush is missing; no estimate changed) · **target:** adobe photoshop 2026

every known shortfall against photoshop 2026 (27.11.0), one entry each, ranked by user impact
on the way to beta. this is the work list agents pick from: take the highest-ranked entry that
nobody is working on, and when you close part of one, update its evidence and estimate here and
the matching parity doc. numbers come from [`target-app-parity.md`](target-app-parity.md);
checklist ids (`TOOL-…`, `FILE-…`, `UI-…`) are rows in [`scorecard.md`](scorecard.md); `#n` are
GitHub issues.

no gap blocks alpha: the [alpha gate](roadmap.md#alpha-gate) passes. "blocks beta" marks what
stands between alpha and beta.

hours are opus 5.5 agent-hours, one agent working sequentially, including tests. **human** marks
what an agent can't finish alone.

## summary

| # | gap | kind | impact | estimate | parity doc |
|---|---|---|---|---:|---|
| g1 | PSD fidelity and photoshop acceptance | file format | blocks beta | 150–250 h | [file-format-parity](file-format-parity.md) |
| g2 | interactive performance | performance | blocks beta | 200–350 h | [scorecard](scorecard.md#performance) |
| g3 | user-reported bug backlog | stability, ui | blocks beta | 250–400 h | [target-app-parity](target-app-parity.md) |
| g4 | brush feel | ui / painting | high | 100–180 h | [brush-parity](brush-parity.md) |
| g5 | transform, handles and modifiers | ui | high | 60–110 h | [ui-parity](ui-parity.md) |
| g6 | missing tools (12) | features | high | 60–100 h | [ui-parity](ui-parity.md#tools) |
| g7 | workspaces shown as parameter dialogs | features | high | 80–140 h | [target-app-parity](target-app-parity.md#feature-areas) |
| g8 | type depth | features | high | 120–200 h | [target-app-parity](target-app-parity.md#feature-areas) |
| g9 | panels, preferences and workspace | ui | medium-high | 150–250 h | [ui-parity](ui-parity.md#panels-and-workspace) |
| G10 | missing and partial file formats | file format | medium | 120–220 h | [file-format-parity](file-format-parity.md) |
| G11 | actions and scripting | ecosystem | medium | 120–200 h | [target-app-parity](target-app-parity.md) |
| G12 | pen tablets and display hardware | hardware | medium | 80–150 h | [hardware-parity](hardware-parity.md) |
| G13 | languages: hindi, arabic, vietnamese, RTL, engine messages | localization | medium | 80–140 h + human | [localization-parity](localization-parity.md) |
| G14 | camera raw depth | features, formats | medium | 120–200 h | [file-format-parity](file-format-parity.md#camera-raw) |
| G15 | ai and generative features | ai | medium (high for some users) | 300–600 h + human | [target-app-parity](target-app-parity.md) |
| G16 | plug-in compatibility (.8bf, UXP) | ecosystem | medium | 250–450 h + human | [target-app-parity](target-app-parity.md) |
| G17 | platforms and distribution | platforms | low-medium | 30–60 h + human | [scorecard](scorecard.md#distribution) |

## feature gaps

### g1. PSD fidelity and photoshop acceptance
- **missing:** psds we write are not always accepted by photoshop, and photoshop-authored files
  don't always render as photoshop renders them.
- **evidence:** corpus floors (`crates/io/tests/corpus.rs`, scorecard): photoshop-authored
  oracles 185/258 (72%; CMYK and lab adjustment layers, 32-bit unclipped adjustments, levels, threshold, posterize and mask feathers 2026-10-13; gradient map luminosity 2026-10-13; CMYK invert, posterize and threshold on the inks 2026-10-12), io corpus 148/170 (87%), psd-tools 249/309 (81%); round trips ~100%
  within the corpus. smart-filter re-render matches 5/30. user reports: a PSD with text layers
  re-saved unmodified would not reopen (#2374, closed 2026-10-10), photoshop 26.6 cannot reopen our files (#1281), "PSD not
  opening in photoshop" (#2469), a CS2-era smart object unavailable (#2465, fixed 2026-10-12), PSB smart object
  blank (#1764), perspective placed layers flattened (#579, fixed by #426), PSB > 4 gb fails on windows (#375).
  no test opens our exports in a third-party editor (FILE-216-4).
- **impact:** PSD is the format professionals exchange. beta requires it to be reliable.
- **estimate:** 150–250 h. **human:** opening exports in photoshop on the owner's mac (or a
  scripted check there) is the only real oracle.

### g6. missing tools
- **missing (1 of 68):** art history brush. convert point landed (#3204). freeform
  pen and curvature pen landed (#1054); color sampler landed (#1046, `view.colorSamplers.*`); horizontal and vertical type mask landed (#1055,
  `type.createSelection`); perspective crop landed (#1043, `image.perspectiveCrop`); frame landed
  (#2914).
- **evidence:** `Tool` enum (`crates/ui-egui/src/state.rs`) against the 67 `<tool>` entries of
  photoshop's `Default Keyboard Shortcuts.kys` plus remove, triangle and curvature pen.
- **impact:** users look for them by name and shortcut; several are daily tools for designers.
- **estimate:** 60–100 h (2–9 h each, per the calibration).

### g7. workspaces shown as parameter dialogs
- **missing:** select and mask workspace layout
  (TOOL-214-1, refine hair #2671; its view modes preview live since #1754, refine edge brush #2972); content-aware fill workspace with a paintable sampling area
  (TOOL-214-15); vanishing point plane editor (TOOL-214-14); content-aware move's live preview
  of the result (TOOL-213-4; transform on drop and content-aware scale through the transform box,
  TOOL-214-7, are done).
  blur gallery: field, iris, tilt-shift and spin blur are edited on the image with photoshop's pins
  and handles (2026-10-11, #3373); tilt-shift has one band (the engine takes one), and path blur
  (#1759) is still a parameter dialog.
- **evidence:** scorecard rows above, all partial or missing.
- **impact:** these are the tools retouchers and compositors use daily; a slider dialog is not a
  substitute.
- **estimate:** 80–140 h.

### g8. type depth
- **missing:** type on a path (TYPE-218-1), paragraph direction ui (TYPE-218-2), complex-script
  fallback fonts and golden tests (TYPE-218-3), OpenType numerals and stylistic sets submenus
  (TYPE-218-5; the character panel buttons landed 2026-10-11, #3752), live warp text preview
  (TYPE-218-7, #2689), "rasterize and continue" for distort/perspective on type
  (TYPE-218-9), text free transform (#2630). done since the re-measure: manual and optical kerning
  with PSD round trip (TYPE-218-6, #206), variable-font axes (TYPE-218-4, #2985).
- **evidence:** scorecard type: 3 done, 2 partial, 4 missing. 2026-10-11: 2/3/4/5 clicks select
  word, line, paragraph and all text, with tests (TYPE-218-8, #3493). the type tool froze the app for
  60 s on one macos machine (#2518).
- **estimate:** 120–200 h.

### G11. actions and scripting
- **2026-10-13 evidence:** batch supports [output naming templates](batch-naming.md) using
  `{name}`, `{serial}` and a captured UTC `{date}`, with a starting serial number. colliding
  names remain errors; custom output names cannot replace inputs. AUTO-219-5 stays partial.
- **missing:** document-independent action recording (AUTO-219-1), batch recursion, open-document inputs, progress/cancel and an error-log file
  (AUTO-219-5), ExtendScript/UXP compatibility. `.atn` steps outside the
  mapped events (paths, scripts, generative fill, filter gallery effects other than spatter,
  duplicate layers into a new document, bitmap halftone screens…) load as kept photoshop steps
  that fail at playback by name.
- **done 2026-10-12:** load/save actions `.atn` in the panel menu, `batch --actions set.atn`
  and `actions.load`/`actions.save`; 112 photoshop events map to commands both ways (transform,
  apply image, save for web, smart-object edits, gradient map, quick mask, polygon selections,
  place, open, image size in percent, the paint bucket and insert conditional since the third
  and fourth prs; select and mask, filter gallery spatter, ripple, crystallize, pattern fill
  layers, targeting the layer mask or composite channel, loading the targeted channel, step
  backward/forward, removing a layer from the selection, nudges in points, layer from background,
  grayscale and bitmap mode objects and image size with resample keeping the print size since the
  fifth), play, stop and conditional steps included; layer steps replay by layer name in any
  document. four TrueGrit texture supply sets (debaser, distress press, halftone zine machine,
  transparency) import with 3 of 533 steps unmapped and play end to end. filter gallery spatter's smoothness
  now smooths the spray's displacement, so high values tear an edge without detaching specks, as
  photoshop's does (calibrated by eye on the distress press guide's screenshots). crystallize
  fills each cell with the colour at its point instead of the cell's average, so a black-and-white
  mask keeps hard edges (not yet checked against a photoshop render). the panel groups actions
  in set folders, has the dialog toggle column (playback pauses on the step's dialog and resumes
  on ok) and insert stop…; `actions.insert`, `actions.dialog`, `actions.assignSet` for agents
  (AUTO-219-2 done).
- **ahead of photoshop:** CLI, MCP and the JSON control channel drive every command.
- **estimate:** 120–200 h. **human:** whether to support a JavaScript scripting dialect at all
  (AGENTS.md forbids js in the app).

### G14. camera raw depth
- **missing:** CR3 (detected, unsupported), compressed RAF, lossy DNG (FILE-215-11), .xmp
  sidecars, masking and calibration in the camera raw dialog, per-camera colour for non-DNG
  cameras, lens profiles.
- **evidence:** `crates/raw/src/lib.rs:228` (CR3), scorecard FILE-215-11/12, #50.
- **progress (2026-10-11):** on the embedded-JPEG rendering work in #2135, camera raw
  tuning retains the fitted look and applies user gains afterwards. three D5100 nefs
  establish a guarded 4928 × 3264 image area. this is an estimated rendering, not a
  measured D5100 colour calibration; [validation](nikon-nef-development.md).
- **estimate:** 120–200 h.

### G15. ai and generative features
- **missing:** generative fill and expand, generative upscale, neural filters, sky replacement,
  super zoom, ml select subject/sky. there is no ml runtime (AUTO-219-7). classical substitutes
  exist (GrabCut/max-flow selection, remove tool by non-local patch completion).
- **evidence:** photoshop's `Required/UXP` ships the neural filters gallery and its `.kys` has
  a neural filters taskspace; users ask for local models (#2313, #1996, #2682).
- **estimate:** 300–600 h. **human:** owner decision #41 (model licences, local vs remote).

### G16. plug-in compatibility
- **missing:** hosting photoshop .8bf filters (#2647), UXP panels, CEP. brahma has its own
  sandboxed WebAssembly plug-in ABI v1 (`docs/plugins.md`), but ABI v2 (panels, progress, other
  layers) is missing (AUTO-219-6).
- **estimate:** 250–450 h. **human:** .8bf needs native code under the isolated-`unsafe` rule.

## ui / ux gaps

### g3. user-reported bug backlog
- **missing:** fixes for 579 open issues (2026-10-10), filed mostly by 0.5/0.6 users. by title:
  ~150 ui/panels/menus/shortcuts, 94 file/format, 87 layers, 54 painting, 54 selection,
  46 type, 41 performance, 34 transform/move, 25 linux/wayland, 24 windows, 20 macos.
- **impact:** each is a place where a photoshop user hits something different from what they
  expect; together they are why ready-for-real-work is ~45% while breadth is ~76%.
- **estimate:** 250–400 h (many overlap with other gaps; count each fix once).

### g4. brush feel
- **progress (2026-10-13, #3825 partial):** hue jitter shows a standard themed
  percentage-amount track rather than a hue spectrum. other brush settings layout
  and brush-behaviour gaps remain open.
- **progress (2026-10-13, #3825 partial):** shape dynamics' minimum diameter / minimum
  roundness and transfer's four minimum rows disable editing when their control is off,
  retaining the configured values. the other layout and control gaps in #3825 remain open.
- **progress (2026-10-12, #4283):** the five checkbox-only brush settings rows keep the open
  controls selected; smoothing can be disabled without discarding its amount or mode. this
  closes that panel interaction gap, not the stroke-quality or performance gaps below.
- **missing:** lag with big brushes (#2619, #1994) and under the cursor (#2566, #1343); jumpy
  pressure on windows (#2046) and tap sensitivity (#1933); stair-stepped line weights (#1776),
  interpolation and smoothing problems (#2138), edge falloff (#2149), pixelated cursor vs
  photoshop (#1433), brush rotation not driven by pen rotation (#1301); live preview only for
  brush and eraser (TOOL-213-7); no image file as a brush tip (TOOL-213-12); per-control pressure
  curves (TOOL-213-11); 4 of photoshop's symmetry modes.
- **detail:** [`brush-parity.md`](brush-parity.md).
- **estimate:** 100–180 h. **human:** checking feel on real tablets.

### g5. transform, handles and modifiers
- **missing:** the full interpolation list; blend mode/opacity edits during
  a transform (#1384); path-point nudge; autoscroll at the edge while selecting (#2231);
  wheel and pinch behaviour (#2485, #2463, #1989);
  toolbar press-and-hold flyouts (#2301).
- **done since the re-measure (2026-10-10 to 12):** right-click menu during free transform
  (TOOL-214-5); commit/cancel buttons in the options bar (#2442); transform keeps the layer's
  place in the stack (#2439, #2480); hand tool no longer interrupts a transform (#2302, #2327);
  locked layers refused before the drag (#2585, #2609); ⌘-drag temporary move with box and
  snapping (#2681, #2684); auto-select box follows the picked layer (#2666, #2675); ⌘-arrow
  layer nudge (#2474, #2556); polygonal lasso shift angles (#2352, #2396); double-click zoom tool
  to 100% (#2508, #2527); tool-key memory in a group (#2608, #2625); hold-a-key spring-loaded
  tools (#2672); h/v skew fields in the free transform options bar.
- **detail:** [`ui-parity.md`](ui-parity.md).
- **estimate:** 60–110 h.

### g9. panels, preferences and workspace
- **progress (2026-10-12):** toolbar tool shortcuts can be reassigned, removed and reset (#4086),
  with saved bindings, matching toolbar hints and regression coverage (`ui-egui/src/tool_shortcuts.rs`).
- **missing:** 27 of 156 preferences do nothing (scorecard prefs audit); panel groups can't
  float or tear off into os windows (ui-217-5, #1828, #1213, #2305); color panel lacks wheel and
  slider modes (ui-217-17, #2569); more themes (ui-217-11, #2661); layer clipboard
  (ui-217-23, #2670); tool-specific context menus beyond brushes, selections and pen
  ([context-menu-parity.md](context-menu-parity.md)); libraries, comments, contextual task bar
  panels.
- **estimate:** 150–250 h.

### g2. interactive performance
- **measured progress (2026-10-10):** tilt-shift's existing per-level halo/copy
  optimization measures 1.07–1.49× faster across nine local 24 mp cases, with exact
  finite output bits. the filtered proxy improves for medium/large blur; its
  default small blur and the 36 mp saturated-field gain are inconclusive.
  [method and full ranges](tilt-shift-performance.md). native GUI latency remains
  unmeasured; no scorecard budget or parity percentage is changed by this work.
- **missing:** 22 of 25 budgeted scenarios are over budget (baseline 2026-10-05, a heavily
  loaded machine): layer move with styles 321 ms p50 against 25 ms, opacity change 276 ms
  against 20 ms, paste 12 mp 1.2 s against 100 ms, layer move on 15000×10000 16-bit 2.7 s,
  content-aware scale 24 mp 954 s against 3 s, select subject 559 ms against 100 ms; the
  150-layer nudge crashes the GPU compositor (P12); menus slow with thousands of layers (#2460); old machines lag (#2687).
- **estimate:** 200–350 h. serial in places (compositor architecture).

## file-format gaps

### G10. missing and partial file formats
- **missing:** JPEG options dialog (FILE-215-19), photoshop PDF read and write and multi-page PDF
  (FILE-215-10, #2541), AVIF read (#463) and default-build AVIF write, JPEG xl, JPEG 2000 (#1047),
  cineon (DPX landed, #1049), DICOM, EPS/DCS, PCX, pixar, IFF, WBMP, MPO/JPS, PICT, scitex ct, photoshop
  raw; layered TIFF extra alpha channels (FILE-215-7); spot plates (FILE-215-17); .atn, .asl,
  .shc, .csh presets (FILE-215-15); C2PA (#1050).
- **detail:** [`file-format-parity.md`](file-format-parity.md).
- **estimate:** 120–220 h.

## hardware gaps

### G12. pen tablets and display hardware
- **missing:** tilt and rotation on windows (pressure only, via WM_POINTER; no WinTab); native
  wayland tablet-v2 (#79; the pen does not click on GNOME 50, #2622, KDE #2297); eraser end on
  flipped wacom pens (#2285); HDR/EDR output (photoshop has "precise color management for HDR
  display"); monitor profile detection off macos and live profile changes; touch bar (macos
  intel only, low value); 3d mice.
- **detail:** [`hardware-parity.md`](hardware-parity.md).
- **estimate:** 80–150 h. **human:** tablets and HDR displays to test on.

## localization gaps

### G13. hindi, arabic, vietnamese, RTL and engine messages
- **missing:** catalogs for hindi, arabic and vietnamese (3 of the 12 key languages); right-to-
  left ui layout; devanagari fallback fonts (thai #1909 and khmer, lao and myanmar #3259 landed); engine error and status messages in
  english in every language; browser-locale detection and CJK fonts on the web; japanese IME on
  windows (#590); installed japanese fonts missing from the font list (#2330); arabic layer names
  as tofu (#1408, #1693).
- **evidence (2026-10-11):** czech dynamic ui wrappers, hardware labels and composed messages
  translate at display sites; engine errors remain english. this does not close G13.
- **detail:** [`localization-parity.md`](localization-parity.md).
- **estimate:** 80–140 h. **human:** native-speaker review of every catalog (none is recorded).

## platform gaps

### G17. platforms and distribution
- **missing:** windows code signing (material not obtained), in-app update check (DIST-220-1,
  #2650, #2375), in-app bug report (DIST-220-3), flathub (#173),
  starting without a GPU adapter (#2519, #2412), windows 7 (#2310, out of scope), intel macs with
  OpenCore (#1859).
- **estimate:** 30–60 h. **human:** signing certificates.

## revision history

| date | change | summary |
|---|---|---|
| 2026-10-14 | minor | corpus floors raised to the measured results of the CMYK, lab, 32-bit, levels, threshold, posterize, feather and pass-through fixes landed together: photoshop-authored 165 → 185 of 258, io 146 → 148 of 170, psd-tools 242 → 249 of 309 |
| 2026-10-13 | minor | g1: CMYK hue/saturation and color balance work on c, m and y as inverted RGB with black kept, as photoshop does; cmyk hue-saturation 100% → 0.4–0.7% bad pixels, color-balance 74% → 25%; floor unchanged |
| 2026-10-13 | minor | g1: CMYK posterize works on the stored values (1 − ink); psd-tools posterize_cmyk passes; psd-tools floor → 242 of 309 |
| 2026-10-13 | minor | g1: CMYK stacks with invert, posterize, threshold, brightness/contrast, levels or curves layers composite on the document's own inks (14 CMYK oracles pass); photoshop-oracle floor 151 → 165 of 258 |
| 2026-10-13 | minor | CMYK threshold compares the rec. 601 luma of the inks' plain complement, as photoshop does (every pixel of cmyk8/16 threshold-128.psd); oracle bad pixels 1.6% → 0.2% |
| 2026-10-13 | minor | g1: CMYK brightness/contrast (modern and legacy) curves each ink's stored value as photoshop does; oracle bad pixels 71% → 48–57%, floor unchanged |
| 2026-10-13 | minor | g1: modern brightness/contrast's brightness curve refit to photoshop (six oracles pass); photoshop-oracle floor 145 → 151 of 258 |
| 2026-10-13 | minor | g1: lab documents composite invert, posterize, threshold, brightness/contrast and lab levels/curves on their stored l, a, b (no clipped RGB round trip); seven lab oracles pass; photoshop-oracle floor 145 → 152 of 258 |
| 2026-10-13 | minor | g1: levels round every step to whole levels and use photoshop's shadow toe (slope 2^gamma); levels_rgb and stack-levels-curves-hue-posterize pass; photoshop-oracle floor 145 → 146 of 258, psd-tools 236 → 238 of 309 |
| 2026-10-13 | minor | g1: 32-bit legacy brightness/contrast (pivot 127), channel mixer (1/1024 steps, constant ×255/512), photo filter and black & white tint unclipped as photoshop computes them, CPU and GPU; photoshop-oracle floor 145 → 149 of 258 |
| 2026-10-13 | minor | g1: mask feathers blur with exactly the feather's variance; psd-tools mask-parameters-no-real-channel passes, layer_mask_data 27 % → 4 % off; psd-tools floor → 240 of 309 |
| 2026-10-13 | minor | g1: threshold splits on photoshop's luminosity of whole samples (8- and 16-bit rules exact on four oracles); psd-tools threshold_rgb and threshold_16bits_rgb pass; psd-tools floor → 241 of 309 |
| 2026-10-13 | minor | g1: a pass-through group that is a clipping base and holds an adjustment layer renders isolated; psd-tools passthrough_clipping_mask_adjustment passes; psd-tools floor → 240 of 309 |
| 2026-10-13 | minor | g1: 16-bit posterize bins the 16-bit sample, as photoshop does (exact on two oracles); psd-tools floor → 241 of 309 |
| 2026-10-13 | minor | g1: gradient map driven by the W3C luminosity (0.3, 0.59, 0.11), as photoshop is; rgb8/rgb16 gradient-map pass; photoshop-oracle floor 145 → 147 of 258 |
| 2026-10-13 | minor | g6: stale "missing tool" list corrected against `main` — color replacement (`paint.colorReplacement`), red eye (`paint.redEye`) and add/delete anchor point (p flyout, #4144) are live tools, so the toolbox is 67 of 68 and art history brush is the only one missing; no percentage or estimate changed |
| 2026-10-13 | minor | g4: hue jitter amount-track appearance (#3825, partial); no readiness percentage or estimate changed |
| 2026-10-13 | minor | g4: minimum inputs in shape dynamics and transfer follow their control (#3825, partial); no readiness percentage or estimate changed |
| 2026-10-12 | minor | g5: free transform options bar h/v skew fields done |
| 2026-10-12 | minor | G11: `.atn` import maps 112 events (select and mask, spatter, ripple, crystallize, pattern fill, mask targeting, step backward, points, bitmap) |
| 2026-10-12 | minor | g9: toolbar tool shortcut editing (#4086); the remaining workspace gaps stay open |
| 2026-10-12 | minor | g1: #2465 (CS2 `PlLd`-only smart objects) marked fixed |
| 2026-10-12 | minor | g1: CMYK invert, posterize and threshold work on the inks, 16-bit lab invert keeps greys neutral; photoshop-oracle floor 133 → 145 of 258 (145 already passed on main) |
| 2026-10-12 | minor | g5: spring-loaded tool letters done (#2672) |
| 2026-10-12 | minor | G10: DPX read and write (#1049); cineon remains open |
| 2026-10-12 | minor | g4: checkbox-only brush settings rows preserve the open controls and smoothing settings (#4283); no readiness percentage or estimate changed |
| 2026-10-12 | minor | G11: load/save actions `.atn` landed (AUTO-219-2 partial); remaining panel work listed |
| 2026-10-12 | minor | refreshed against main: g5 items fixed by #2480, #2327, #2609, #2684, #2675, #2556, #2396, #2527, #2625 and TOOL-214-5/#2442; g7 TOOL-214-7 and transform on drop done; g2 #2377 fixed (#2417); g9 ui scale (ui-217-9) and stamp visible (ui-217-22) done, prefs 27 of 156; G10 #2225 done (#2482); G13 khmer, lao, myanmar fallback (#3259); G14 CR3 line; G17 windows ARM64 done (DIST-220-2) |
| 2026-10-11 | minor | g7: tilt-shift's band (lines, turn dots) and spin blur's pins (outline handles, blur-angle ring) on the image; path blur remains a dialog |
| 2026-10-11 | minor | g7: iris blur's pins, outline, roundness and feather handles on the image too; tilt-shift, spin and path blur remain dialogs |
| 2026-10-11 | minor | g7: field blur pins are placed, dragged and tuned on the image (#3373); the other blur gallery filters remain dialogs |
| 2026-10-11 | minor | D5100 image area and camera raw tuning validation; G14 remains open |
| 2026-10-11 | minor | G13: czech dynamic ui text localized; engine messages and native review remain open |
| 2026-10-11 | minor | g8: triple/quadruple/five-click type selection done (TYPE-218-8, #3493); type evidence recounted |
| 2026-10-11 | minor | g8: character panel OpenType buttons landed (TYPE-218-5, #3752); numerals and stylistic sets submenus still missing |
| 2026-10-10 | minor | g2: recorded tilt-shift computation/filtered-proxy measurements and limits; gap and estimate remain open |
| 2026-10-10 | minor | checked against the alpha gate (no alpha blockers); #2374 marked closed |
| 2026-10-10 | major | first version, from the full re-measure against photoshop 2026 27.11.0 and the open issues |
