# brahma vs adobe photoshop: target-app parity

> **last reviewed:** 2026-10-12 · **last updated:** 2026-10-13 · **change:** minor (on-canvas corner radius widgets, TOOL-214-12; `.atn` import maps 112 events; free transform h/v skew fields; stale rows corrected against `main` and the scorecard, estimates unchanged; photoshop actions `.atn` load and save; 2026-10-10: tilt-shift measurements added; earlier the same day: major, full re-measure against photoshop 2026 27.11.0; merged `parity-estimate.md` and the roadmap's honest assessment; stale missing-tool rows corrected: color replacement and red eye are live toolbar tools, frame landed; percentages unchanged) · **target:** adobe photoshop 2026

the authoritative answer to "how close is brahma to photoshop, and how much work is left". the
one-page summary is [`ROADMAP.md`](../ROADMAP.md); the ranked work list is [`gaps.md`](gaps.md);
the measured checklists are [`scorecard.md`](scorecard.md) (`cargo xtask scorecard`) and
[`parity-checklist.md`](parity-checklist.md) (`cargo xtask parity`). detail by dimension:
[ui](ui-parity.md), [brushes and painting](brush-parity.md), [context menus](context-menu-parity.md),
[file formats](file-format-parity.md), [hardware](hardware-parity.md),
[localization](localization-parity.md).

## headline (2026-10-10, brahma 0.6.0, `main` at 3e000bc1)

| number | value | kind |
|---|---|---|
| **feature breadth** (does each photoshop feature exist?) | **~76%** | partly measured (menus, tools, panels, formats), weights below |
| **ready for real work** (could a professional replace photoshop with it?) | **~45%** (range 40–50%) | estimated, weights and evidence below |
| **mainstream practitioner** (typical pro, weekly areas only) | **~43%** (38–48%) | estimated, [method](#mainstream-practitioner-43) |
| **essentials user** (core features only) | **~61%** (55–67%) | estimated, [method](#essentials-user-61) |
| **stage** | **alpha** | ready-for-real-work is inside the ~40–75% band and the [alpha gate](roadmap.md#alpha-gate) passes (six core workflows end to end); PSD exchange with photoshop is not yet reliable enough for beta |
| remaining to **beta** (~75% ready, PSD reliable) | **~1,100–1,800 opus 5.5 agent-hours** | estimated, calibrated below |
| remaining to **full parity** (~95–100%) | **~2,300–4,100 opus 5.5 agent-hours** | estimated; includes ai and ecosystem, which need owner decisions |

these numbers replace the 2026-10-03 estimate (82% "of the way", 140–220 agent-hours to 95%,
kept below for the record). that estimate counted feature surface and was far too optimistic:
about 1,230 issues have been filed since the first release (579 open on 2026-10-10), most of them
against features already counted as present. the 2026-10-05 honest assessment (feature surface 60–70%,
ready for real work 25–35%) was closer; ~700 merged prs since then moved both numbers up.

### readiness by audience

| audience | ready % | opus 5.5 agent-hours to ~95% | work that dominates |
|---|---:|---:|---|
| full target (ready for real work) | ~45% (additive weighted sum: 46%) | 2,300–4,100 | everything below, plus ai tier, plug-in/scripting ecosystem, video, localization, hardware |
| mainstream practitioner | ~43% | 1,700–2,800 | depth in the 11 weekly areas (1,180–2,000), performance budgets, stability on real machines, PSD exchange with photoshop |
| essentials user | ~61% | 400–700 | startup and GPU robustness, brush and transform responsiveness, type tool, discoverability (flyouts, commit buttons), AVIF/PDF open |

hours calibrated as in [remaining effort](#remaining-effort-and-calibration) (2–9 h per checklist
item, 8–20 h per format or subsystem, from the repo's 1,004 merged prs); each tier is a subset of
the one above. about 80% parallelizes (independent crates and command modules); performance
architecture and the PSD writer are the serial parts.

## target measured

- **adobe photoshop 2026, version 27.11.0** (build 20260929.r.26, macos universal), installed at
  `/Applications/Adobe Photoshop 2026`, with camera raw 18.7. inspected without launching it:
  `Info.plist` document types, the file-format plug-ins' pipl read/write flags, `.lproj` and
  `Locales/` language packs, `Default Keyboard Shortcuts.kys` (67 toolbox tools, 107 menu commands
  with default shortcuts), `Required/` plug-ins (85 filter entries, 12 format plug-ins), presets,
  and strings in the main binary (81 "… tool" titles; HDR/EDR, metal, touch bar, pen features).
- brahma's own photoshop menu tree (`crates/ui-egui/src/menu_catalog.rs`, 628 items on 2026-10-10, 631 on 2026-10-12) was
  written from photoshop's menus; it is the denominator of the menu number. photoshop's binary
  `Default Menus.mnu` was not parsed, so items added in 27.x that the catalog lacks would not show
  (generative and cloud items are the likely ones).
- user reports on GitHub (`shep95/brahma`, 579 open issues on 2026-10-10) as evidence of
  depth and feel. adobe's documentation and release notes for behaviour not visible in the bundle.

## feature breadth: ~76% (weighted)

| component | weight | ours / photoshop | % | kind |
|---|---:|---|---:|---|
| menu commands | 30% | 631 / 631 catalog items dispatch a live command (628 / 628 on 2026-10-10) | 100% | measured (`cargo xtask parity`) |
| toolbox tools | 15% | 59 / 68 (photoshop's 67 `.kys` tools plus remove, triangle, curvature pen; minus legacy rounded rectangle and targeted adjustment) | 87% | measured (`Tool` enum in `ui-egui/src/state.rs`, `TOOL_SECTIONS` in `panels.rs`) |
| panels | 10% | 30 / 35 window-menu panels | 86% | measured (`view_cmds.rs`, panel modules) |
| filters and adjustments | 10% | every filter and image › adjustments item live; filter gallery 47 looks | 100% | measured (menu catalog) |
| file formats (usage-weighted) | 15% | 13 of 31 photoshop formats supported in photoshop's directions, 4 partial; the common ones (PSD/PSB, TIFF, JPEG, PNG, webp) all present | ~80% (48% by count) | measured list, estimated weights ([file-format-parity.md](file-format-parity.md)) |
| ai / generative | 10% | classical select subject/sky, remove tool, content-aware; no generative fill/expand, neural filters, sky replacement, super zoom | ~15% | estimated |
| ecosystem (plug-ins, scripting, libraries) | 10% | WebAssembly plug-ins, CLI, MCP, actions, `.atn` load/save (112 events); no .8bf, UXP/ExtendScript, libraries | ~20% | estimated |
| **weighted** | 100% | | **~76%** | |

## ready for real work: ~45% (weighted)

weights reflect what a working photoshop user depends on (photographers, retouchers, designers).

| dimension | weight | % ready | remaining (agent-h) | evidence | detail |
|---|---:|---:|---:|---|---|
| features in depth (tools, layers, selections, type, filters, transforms) | 25% | 55% | 600–1,000 | scorecard checklists: tools 20 done / 15 partial / 6 missing, type 0 / 3 / 6, automation 3 / 2 / 6. 14 tools missing; select and mask, vanishing point and content-aware fill are parameter dialogs, not workspaces; type on a path, manual kerning, RTL paragraphs missing | [gaps.md](gaps.md) |
| ui / ux fidelity and feel | 15% | 40% | 350–600 | ~150 open ui issues; workspace checklist 13 / 5 / 5 (2026-10-12; 5 / 7 / 11 on 2026-10-10); 27 of 156 preferences do nothing (was 46 of 149); no floating panel groups, thin color panel, brush lag and pressure jumps reported | [ui-parity.md](ui-parity.md), [brush-parity.md](brush-parity.md) |
| file formats and fidelity | 15% | 60% | 300–500 | PSD oracle on photoshop-authored files 133/256 (52%); io corpus 146/170 (86%); psd-tools 236/309 (76%); round trips ~100% of corpus; but users report photoshop refusing our files (#1281, #2469; a text-layer case, #2374, was fixed 2026-10-10); no photoshop PDF, EPS, JPEG 2000, JPEG xl, AVIF read, CR3 | [file-format-parity.md](file-format-parity.md) |
| performance | 10% | 25% | 200–350 | 3 of 25 budgeted scenarios meet budget (baseline 2026-10-05); 150-layer nudge crashes the GPU path (P12); layer move on 15000×10000 16-bit 2.7 s p50; users report brush and cursor lag (#2619, #2566, #1994) | [scorecard.md](scorecard.md#performance) |
| stability | 10% | 45% | 150–250 | never-crash lints on 28/28 crates, `panic_hunt` in the gate, atomic saves; but field reports of freezes (#2518 type tool 60 s on macos), GPU panics (#2020, #1215), no-GPU start failures (#2519, #2412) | [gaps.md](gaps.md) |
| hardware | 5% | 50% | 80–150 | GPU compositor (wgpu), pen pressure on all desktops but tilt/rotation not on windows, wayland pen broken (#2622), no HDR/EDR output, monitor profile auto-detected on macos only | [hardware-parity.md](hardware-parity.md) |
| localization | 5% | 55% | 80–140 | 7 of the 12 key languages at 100% of measured ui keys; hindi and arabic absent, vietnamese catalog added 2026-10-12 (#2636, coverage not re-measured); no RTL ui; engine messages english | [localization-parity.md](localization-parity.md) |
| platforms | 5% | 80% | 30–60 | macos (notarized), windows (unsigned), linux (AppImage/deb/rpm/flatpak), web, freebsd partial; windows ARM64 is packaged and install-checked by `.github/workflows/windows-arm64.yml` (DIST-220-2); no in-app updates | [scorecard.md](scorecard.md#distribution) |
| ecosystem (plug-ins, scripting, presets, libraries) | 5% | 15% | 250–450 | no .8bf hosting (#2647), no UXP/ExtendScript; .atn load/save (2026-10-12, 112 events, the rest kept verbatim), .abr/.grd/.pat/.aco/.ase/.cube supported | [gaps.md](gaps.md) |
| ai features | 5% | 10% | 300–600 | no ml runtime at all (scorecard AUTO-219-7); classical substitutes only. needs an owner decision and openly licensed models (#41) | [gaps.md](gaps.md) |
| **weighted** | 100% | **~46%** | **2,340–4,100** | | |

## mainstream practitioner: ~43%

the same question for the typical professional photoshop user (photographer, retoucher,
designer), not the whole product. method: craftrules `standards/progress-docs.md`. areas a
professional uses weekly (the [alpha-gate](roadmap.md#alpha-gate) workflows plus their everyday
tools), depth from the [feature areas](#feature-areas) table. left out: ai, plug-ins and
scripting, libraries/cloud, video, specialist hardware (tilt, touch bar), languages.

| area | weight | depth |
|---|---:|---:|
| layers, masks, blend modes, smart objects, styles | 20% | 65% |
| adjustments and filters | 15% | 70% |
| selections and masking | 15% | 55% |
| retouching | 12% | 50% |
| transform and warp | 8% | 55% |
| type | 8% | 40% |
| painting and brushes | 7% | 55% |
| vector shapes and paths | 5% | 55% |
| camera raw | 4% | 45% |
| colour management | 3% | 70% |
| export, save for web | 3% | 50% |
| **weighted depth** | 100% | **57%** |

discounts for what still stops real work (estimated, each with evidence):

| discount | factor | evidence |
|---|---:|---|
| interaction fidelity | ×0.92 | ~150 open ui issues; spring-loaded tools, panel tear-off missing ([ui-parity.md](ui-parity.md)); brush lag and pressure reports ([brush-parity.md](brush-parity.md)) |
| stability and speed on real machines | ×0.88 | 3 of 25 performance budgets met; 150-layer GPU crash (P12); type tool freeze (#2518); GPU start failures (#2519, #2412, #2020); 41 open performance issues |
| exchanging files with photoshop users | ×0.92 | photoshop-authored oracle 133/256; exports refused by photoshop (#1281, #2469); smart-filter re-render 5/30 |

**57% × 0.92 × 0.88 × 0.92 ≈ 43%** (range 38–48), estimated.

it is not higher than ready-for-real-work (~45%) because the areas it leaves out carry little
weight in the full number (ai, ecosystem, localization and hardware together 20%, averaging
~33%), while the discounts apply to every workflow. the two numbers agree within their error;
the full number stays where its written weights put it.

**user evidence** (GitHub, 2026-10-10; search hits include maintainer replies, so they are upper
bounds): 0 issues mention switching from or replacing photoshop; praise words appear in ~19
("love"), 10 ("amazing"), 6 ("awesome"), 4 ("great work") issues. of 579 open issues, 136 are
feature requests and 443 bugs or reports, of which ~244 are on core paths (layers, selection,
brush, transform, type, save/open, performance) by title. users are trying it for real work,
and most of what they report is core-path friction, not niche requests.

## essentials user: ~61%

an occasional user who touches only the essentials: open or create a file, the most used tools
and commands, undo, save and export, default settings. excluded: advanced options, pro
workflows, PSD exchange edge cases, and everything excluded above.

| core feature | weight | depth | notes |
|---|---:|---:|---|
| save, save as, export JPEG/PNG | 12% | 85% | |
| layers (add, reorder, hide, opacity) | 12% | 80% | |
| new / open a photo | 10% | 90% | |
| select (marquee, lasso, magic wand, object) | 10% | 75% | |
| move and free transform | 10% | 70% | commit and cancel buttons in the options bar (#2442, closed) |
| brush and eraser | 10% | 70% | lag with big brushes (#2619) |
| undo, history | 8% | 90% | |
| basic adjustments (brightness, levels, curves, hue/saturation) | 8% | 80% | |
| crop and straighten | 6% | 85% | |
| text | 6% | 55% | freeze report (#2518) |
| spot healing, remove | 5% | 65% | |
| blur and sharpen filters | 3% | 85% | |
| **weighted depth** | 100% | **78%** | |

discounts: launch and stability ×0.90 (start failures without a usable GPU, #2519, #2412,
#1859; type tool freeze, #2518); discoverability and ui clarity ×0.92 (language option not
found, #2532; flyouts only on right-click, #2301; toolbar hover issue, #2603); opening files
people send ×0.95 (JPEG, PNG, HEIC, webp, PSD open; AVIF and PDF don't, #463, #2541).
**78% × 0.90 × 0.92 × 0.95 ≈ 61%** (range 55–67), estimated.

## feature areas

percent is ready-for-real-work for that area (depth and fidelity, not presence). hours are to full
parity for the area and are already contained in the dimension rows above.

| area | % | hours | notes |
|---|---:|---:|---|
| layers, groups, masks, blend modes, smart objects, layer styles | 65% | 150–250 | broad and GPU-accelerated. open: layer clipboard between documents (ui-217-23: `layer.copyToDocument` exists, no copy/paste), smart-object perspective (FILE-215-6), a user contour library (FILE-215-15). multi-layer smart object conversion (#2052, #2651), stamp visible (ui-217-22) and mask targeting by command (#2458) are done |
| selections, select and mask, channels | 55% | 120–200 | all selection tools, single row/column marquee included (#2837). select and mask is a dialog with photoshop's view modes, a refine edge brush and six outputs, not a workspace (TOOL-214-1), no refine hair (#2671); select subject/sky classical, 559 ms p50 on 24 mp vs a 100 ms budget |
| painting and brush engine | 55% | 150–250 | every brush settings section exists; feel is the gap: lag on big brushes, pressure jumps, smoothing and edge quality reports ([brush-parity.md](brush-parity.md)) |
| retouching (healing, clone, patch, remove, content-aware) | 50% | 100–180 | remove tool non-generative; content-aware fill has no workspace (TOOL-214-15); live stroke preview only for brush/eraser (TOOL-213-7) |
| transform, warp, puppet, perspective, liquify | 55% | 100–180 | free transform modifiers right, with a right-click transform menu (TOOL-214-5) and h/v skew fields; puppet warp mesh follows the object outline (TOOL-214-6); liquify called "useless" by a user (#2606) |
| adjustments and filters | 70% | 120–200 | all present; smart-filter re-render matches photoshop on 5 of 30 files; filter gallery looks approximate; no neural filters |
| type | 40% | 120–200 | engine works; no type on a path, manual kerning, RTL paragraphs, OpenType panel ui, live warp text preview; type tool freeze on macos (#2518) |
| vector shapes and paths | 55% | 80–140 | pen, direct/path selection, shapes, shape strokes. freeform and curvature pen (#1054). add/delete anchor point and convert point tools exist (`Tool` enum); on-canvas corner radius widgets on live rectangles (TOOL-214-12, 2026-10-13); missing path-point nudge; shape parity tracking issue #2496 |
| colour management and modes | 70% | 60–100 | own ICC engine, soft proof, CMYK/lab/indexed/duotone/multichannel; no adobe CMYK profiles, no spot plates (FILE-215-17), monitor profile only at launch |
| camera raw and computational photo (photomerge, HDR, lens) | 45% | 120–200 | raw for DNG/CR2/NEF/ARW/RW2/ORF/RAF(uncompressed)/PEF; no CR3; ACR dialog partial (no masking, calibration); generic lens profiles only |
| automation, actions, batch, scripting | 45% | 120–200 | CLI/MCP/control channel ahead of photoshop; actions panel and recording not document-independent (AUTO-219-1/2); `.atn` load/save since 2026-10-12 (112 events mapped, the rest kept verbatim); no ExtendScript/UXP |
| file i/o | 60% | 300–500 | see [file-format-parity.md](file-format-parity.md) |
| print, export, save for web | 50% | 60–100 | JPEG options dialog missing (FILE-215-19), no photoshop PDF, no separations, save for web settings not remembered |
| video / timeline, data-driven graphics | 30% | 100–200 | menu items live; frame model persists in `.pcraft`; no PSD video-layer mapping |
| workspace, panels, preferences | 40% | 150–250 | 46 of 149 prefs do nothing; no floating/os-window panel groups; libraries, comments, contextual task bar missing |
| ai / generative | 10% | 300–600 | see the dimension row |

batch supports [output naming templates](batch-naming.md); AUTO-219-5 remains partial.

## remaining effort and calibration

**calibration.** the repository's history from the first commit (2026-09-30) to 2026-10-10: 1,152
commits on `main` and 1,004 merged prs in 10.5 days. median pr open-to-merge time is 3.4 h
(p75 6.4 h, p90 14.3 h; that includes ci and review queueing). comparable features:

| feature | prs | lines added | pr hours |
|---|---|---:|---:|
| crop tool parity set (#1919: nudge/swap/readout, overlays, straighten, shield) | #1958, #2026, #2127, #2128 | ~2,050 | 6.5 |
| crop rotation (#1792) | #1879 | 710 | 2.8 |
| remove tool with non-local patch completion | #2193 | 2,005 | 9.0 |
| OpenRaster open and save | #2257 | 1,331 | 8.9 |
| paint.NET import | #1998 | 1,461 | 7.0 |
| shape stroke controls | #2218 | 1,506 | 9.0 |
| red eye tool | #1084 | 1,232 | 17.9 |
| korean catalog and live switching | #582, #604 | ~3,800 | 13.0 |

so one checklist item of tool or ui depth costs about 2–9 agent-hours including tests, and a new
format or subsystem 8–20. with roughly 2–4 agent-hours per merged pr, the ~1,000 prs so far
represent about **2,000–4,000 agent-hours**, which took ready-for-real-work from 0 to ~45%. the
remaining points are harder per point (fidelity against photoshop's unpublished maths, feel,
performance on large documents, user-reported long tail), so the remaining estimate is not lower
than the work already done.

**totals.**

| milestone | scope | opus 5.5 agent-hours |
|---|---|---:|
| **beta** (~75% ready, PSD reliable) | PSD fidelity and photoshop re-open check, the performance budgets, the user bug backlog, the 15 missing tools, select and mask / content-aware fill workspaces, type depth, ui feel (brush, transform, panels), JPEG/PDF/AVIF/JXL formats, hindi/arabic/vietnamese and RTL | **1,100–1,800** |
| **full parity** | the above plus the ai tier, plug-in/scripting ecosystem, video depth, long-tail fidelity | **2,300–4,100** |

**parallelism.** about 80% parallelizes: areas are independent crates and command modules, and
the repository has sustained 10–20 concurrent agents. at ~12 agents, beta is roughly 100–150
wall-clock hours. serial parts: GPU compositor and performance architecture, the PSD writer.

**needs a human.** owner decisions on the ai backend (#41: which openly licensed models, local
vs remote) and plug-in hosting (.8bf needs native code under the isolated-`unsafe` rule; UXP is a
JavaScript runtime, which AGENTS.md forbids in the ui); windows code-signing material; tablet
hardware checks (wacom/xp-pen on windows, wayland, macos); native-speaker review of every
catalog; checking exports by opening them in photoshop (the owner's machine has it installed).

## how to re-measure

1. `cargo xtask parity` and `cargo xtask scorecard` for the measured rows.
2. tools: diff `Tool` in `crates/ui-egui/src/state.rs` against the `<tool>` entries of
   photoshop's `Locales/en_US/Support Files/Shortcuts/Mac/Default Keyboard Shortcuts.kys`.
3. formats: photoshop's `Info.plist` `CFBundleDocumentTypes` and the pipl flags of
   `Contents/PlugIns/Required/File Formats`, against `crates/codecs/src/format.rs` and
   `crates/io/src/lib.rs`.
4. languages: `cargo xtask i18n-coverage`.
5. re-grade the estimated rows from the open issues by area and the corpus floors. move a number
   only with new evidence, and add a revision-history row.

---

# earlier assessments (kept for the record)

the two sections below are superseded by the tables above. they are kept because they hold dated
evidence that later numbers build on.

## feature-parity estimate of 2026-10-03 (from the former `docs/parity-estimate.md`)

how close brahma is to adobe photoshop 2026, and how much opus 5.5 work remains. generated by
assessment, not a tool; revisit when a major arc lands. menu coverage is from `cargo xtask parity`;
fidelity is from the PSD oracle (`cargo xtask corpus` / `oracle_diff`).

### headline

> **update 2026-10-03:** menu/command breadth reached **100% (625/625)** — every photoshop
> menu item now dispatches a live command (incl. variables/data sets, trap, the timeline panel and
> layer › video layers). remaining parity work is depth and fidelity, not breadth: PSD oracle
> (~66%), camera raw/video-layer depth, and the ai/neural tier. see the table below.


- **menu / command breadth:** 625 / 625 photoshop menu items live = **100%** (every menu item has a live command).
- **PSD rendering fidelity (oracle):** ~111 / 170 corpus files pixel-match = **~65%**.
- **overall functional parity (weighted, mainstream workflows):** **~86%** (menu breadth complete; depth + fidelity are the remaining gap).
- **overall including adobe's ai / firefly features:** **~75%** (these are mostly cloud/ml and the
  hardest to reach clean-room).

menu coverage (96%) overstates true parity because many live commands are faithful-but-approximate
(camera raw without masking/calibration, vanishing point without its ui, heuristic select subject/sky
vs adobe's ml, content-aware fill basic). the weighted figure accounts for depth and fidelity, not
just presence.

### where we stand, by area

| area | parity | notes |
|---|---:|---|
| layers, masks, groups, blend modes, smart objects, layer styles | ~92% | broad + GPU-accelerated; bevel/satin shapes and a few blend edge cases remain |
| selections, paths, vector shapes, pen | ~90% | select menu 100% of items; select subject/sky are heuristic, not ml |
| paint / brush engine, retouch (clone, healing) | ~85% | solid; no mixer brush depth, no content-aware healing |
| adjustments + adjustment layers | ~90% | all kinds present; camera raw depth and a few toe-curve models approximate |
| filters (incl. filter gallery ×47, liquify, blurs, render) | ~88% | gallery looks are approximations; no neural filters |
| type / text engine, character & paragraph styles, glyphs | ~90% | vertical type laid out and rendered (#199; no mojikumi or upright-roman option yet); OpenType alternates thin |
| color management (ICC v2/v4, intents, soft-proof) | ~90% | own pure-rust CMS; no adobe CMYK profiles (synthetic) |
| file i/o: PSD, native .pcraft, flats (png/jpg/tiff/webp/exr/…) | ~85% | PSD oracle ~65% pixel-exact; most structure round-trips |
| transform, warp, puppet, perspective, liquify | ~88% | no face-aware liquify (needs a landmark model) |
| automation: CLI, MCP, headless server, control channel | ~95% | ahead of photoshop here (see below) |
| image modes (RGB/gray/CMYK/lab/indexed/duotone/multichannel) | ~85% | multichannel display-only; no ink-plate model |
| print / export / save for web / slices / generator | ~85% | no separations/bleed; droplets basic |
| computational photo (photomerge, HDR, lens, camera raw, wide angle) | ~70% | classical implementations; generic lens profiles only |
| data-driven graphics (variables, data sets) | ~0% | not started (5 menu items) |
| video / timeline / animation | ~0% | not started (13 menu items) — a whole subsystem |
| ai / neural / generative (firefly, sensei, neural filters) | ~5% | mostly absent; needs licensed/trained models |
| prepress niche (trap, measurement scale pro) | ~30% | trap, ink spread not modelled |

### the remaining 25 menu items, grouped

- **video & timeline (13):** video layers (new/insert/duplicate/delete/replace/interpret/restore/
  reload/rasterize), rasterize › video, timeline panel, render video, import video frames to layers.
  one cohesive subsystem: a frame/time model, a video decoder/encoder (clean-room rust, no ffmpeg),
  onion-skinning, and a timeline ui.
- **data-driven graphics (5):** variables › define / data sets, apply data set, export data sets as
  files, import variable data sets. a template-binding model over layers + a CSV/data-set engine.
- **niche / platform (7):** trap (prepress ink spread), frame from layers, migrate presets, WIA
  support (windows scanners), and two already-adjacent items.

### effort to parity (opus 5.5)

two clocks: **agent-hours** (total model work) and **wall-clock** (with this session's observed
~6–8× parallelism across independent arcs). this session landed ~70 menu items + major subsystems
in a handful of wall-clock hours using 8 parallel agents, so wall-clock ≈ agent-hours ÷ 6.

| milestone | scope | agent-hours | wall-clock |
|---|---|---:|---:|
| **to 90% functional** | remaining non-ai/non-video menu items (variables, trap, frame, migrate, WIA), camera raw depth, vanishing point ui, selection quality, polish | 60–90 | 10–15 h |
| **to 95% functional** | the above + PSD oracle 65% → ~90% (bevel/satin shapes, lab blending, noise-gradient RNG, toe curves), broad ui-parity pass, perf pass | +80–130 | +15–22 h |
| **video / timeline subsystem** | frame model, clean-room decode/encode, timeline, animation, render video | 50–80 | 10–14 h |
| **credible ai tier** | ml select subject/sky, content-aware fill (PatchMatch++), basic generative fill/expand — needs model integration; some features may be infeasible fully clean-room without licensed models | 120–250+ | 25–45 h |

**totals:**
- to **~95% functional parity** (everything except video and ai): **~140–220 agent-hours ≈ 25–40
  wall-clock hours** of opus 5.5 with parallel agents.
- to **~98%** incl. the video subsystem: **+50–80 agent-hours ≈ +10–14 wall-clock hours**.
- to **~99%** incl. a credible ai tier: **+120–250 agent-hours ≈ +25–45 wall-clock hours**, with the
  caveat that pixel-for-pixel firefly/sensei parity is not achievable clean-room without trained
  models; we target functional equivalents.

**best single number:** brahma is **~82% of the way** to full photoshop parity, and reaching a
**95%, ships-for-most-users** state is roughly **25–40 wall-clock hours** of parallel opus 5.5 work.
closing the last 5% (video + ai) is the long, uncertain tail.

### assumptions & caveats

- "parity" = functional + visual equivalence of what a user can do, not byte-identical output. clean-
  room means no adobe code, profiles, shaders, or trained models; ai features target equivalents.
- wall-clock assumes 6–8 independent arcs in parallel with their own target dirs, as this session ran.
  serial work is ~6× longer.
- adobe removed 3d/substance from photoshop, so 3d is intentionally out of scope.
- fidelity hours are the least certain: the remaining oracle failures are proprietary rngs and
  unpublished curve models fitted by observation.

## honest parity assessment of 2026-10-05, with dated evidence to 2026-10-10 (from the former section of `docs/roadmap.md`)

this is the reference answer to "how close are we to photoshop parity, really". agents: read it
before picking work. update it (with dated measurements) when the numbers move; don't restate the
menu-parity number in its place.

**measured numbers live in the [scorecard](scorecard.md)** (`cargo xtask scorecard`, kept current
by ci): performance scenarios against their budgets, corpus floors, per-area checklists with
done / partial / missing counts, and the number of settings that do nothing. where the scorecard
and the estimates below disagree, the scorecard's numbers supersede them. as of 2026-10-05 it
counts 67 of 129 preferences that nothing reads; 3 of the 25 budgeted performance scenarios meet
their #209–#211 targets (first baseline, taken on a heavily loaded machine), and the 150-layer
nudge scenario crashes the GPU compositor; the open area issues' checklists (#203–#220) are
almost entirely missing or partial.

2026-10-07: ui font size now applies to interface text, including CJK fallbacks, independently
of display and canvas zoom (#532). the current preference audit drops from 59 to 58 unread
settings out of 135; see the regenerated scorecard.

2026-10-09: output-preserving radial blur direct sampling measured on a local
i7-9750h, 12 workers, 24 mp RGBA8, good quality, amount 1: spin 66.10 → 16.43 s
(4.02×), zoom 19.19 → 5.35 s (3.59×), one paired run each, exact output equality.
these are local measurements at amount 1; see [method and limits](radial-blur-performance.md).

2026-10-10: native selection distance transforms measured 7.59–10.57x faster on six
24–36 mp synthetic masks on an AWS c7i.4xlarge (16 workers, three paired release runs).
the transform also corrects f32 envelope errors past coordinate 4096, including nonzero
distances at selected pixels. the parallel path needs an additional four bytes per pixel;
see [measurements, correctness and limits](selection-distance-performance.md).

**bottom line.** two days after 0.2.0 we had merged ~96 prs and closed ~48 issues, but **real
photoshop parity is still well below 50%**. the biggest gaps are ai, missing tools, professional
workflow depth and the plug-in ecosystem. most fixes since 0.2.0 have passed our tests but have
**not yet been validated by users**, and this week proved our tests miss what users hit:
on first public use they found broken basics (text selection offset, 214 shortcut failures, panels
resizing themselves, an immovable crop frame, folders that wouldn't collapse, lag on layout psds)
that all counted as "live" in `parity-checklist.md`.

**overall.** feature surface: roughly **60–70%** of what a typical photoshop user touches exists in
some form. "a professional could switch for daily work": roughly **25–35%**. true 1:1 parity is
**many months** of focused work, and some areas need product decisions rather than effort.
confidence: moderate — the next users of 0.2.x will move these numbers either way.

### by dimension

2026-10-10: tilt-shift reuses the established contiguous premultiplication and
per-level trailing-halo trimming. nine local 24 mp RGB u8/U16/F32 cases improve
1.07–1.49× with separated observed ranges (ryzen 7 9700x, windows, eight workers):
default blur 15, 609.5 → 451.7 ms; rotated blur 80, 1729.3 → 1241.4 ms. at 36 mp,
rotated blur 80: 2458.7 → 1668.0 ms; the saturated-field gain is inconclusive.
the existing filtered preview proxy at document blur 80/300 improves from
66.9/179.4 → 55.9/132.8 ms; its default blur 15 gain is inconclusive.
finite pixel bits match the frozen starting kernel exactly. these are computation
timings with two baseline rounds, not GUI frame latency or a photoshop parity
increase; see [method, upstream SIMD/rustfft investigation and full ranges](tilt-shift-performance.md).

2026-10-10: iris blur pin preparation, saturated-distance evaluation and trailing
gaussian-level halo trimming measure 1.18–1.76× faster across eight local 24 mp
RGB u8/U16/F32 cases (ryzen 7 9700x, windows, eight workers). default blur 15:
626.8 → 426.9 ms; two pins: 1789.9 → 1016.2 ms. the default 1500×1000 filtered
proxy for 24 mp, document blur 15/80: 28.7/51.8 → 24.9/42.6 ms. at 36 mp,
two pins: 2595.8 → 1822.8 ms; small/medium u8 gains are inconclusive.
differential tests preserve exact finite pixel bits.
these are computation timings, not GUI frame latency or new photoshop parity;
see [measurements, upstream SIMD findings and limits](iris-blur-performance.md).

2026-10-10: quick selection object strokes through raster surfaces measured
3.46-3.70x faster at 8-bit, 16-bit and 32f on 24-36 mp documents (AWS
c7i.4xlarge, five paired release runs), with identical region bounds and mask bytes.
hybrid FIFO push/relabel and sparse edge enumeration preserve the fitted energy.
see [stroke measurements and reproduction](quick-selection-performance.md).

2026-10-10: pixelate's facet shares exact 3x3 quadrant statistics and color halftone
shares dot radii within each rotated screen grid. full tiled CPU application at 24 mp
on a 16-vcpu m7i.4xlarge improves facet from 0.684 s to 0.312 s in RGB8
(2.20x), and from 7.888 s to 0.574 s in CMYK8 (13.74x).
color halftone improves from 1.137 s to 0.604 s in RGB8 (1.88x)
and from 3.589 s to 0.718 s in lab8 (5.00x).
three alternating paired release runs also cover 36 mp RGB, 16/32-bit storage,
and screen radii 2/8/64. all 36 complete output comparisons match exactly.
see [method, raw timings and checks](pixelate-performance.md).

2026-10-10: native CMYK solid and gradient fills share depth-quantized samples between
rendering and PSD export. the uncached 16-bit CMYK gradient now round-trips with zero
rendered error; psd-tools round trips rise from 307 to 308. mixed and photoshop round
trips remain 169 and 258; rendering oracle counts remain 146, 237 and 133.
exact 8-bit CMYK row reuse reduces 1 mp conversion-read time by 77% for smooth ramps,
33% with dithering and 10% for random colours (m4 max, release optimization level 3).

2026-10-10: [shape stroke controls](shape-strokes.md) expose existing solid/dashed/dotted,
custom dash/gap, offset, caps, joins, alignment, miter-limit and opacity capabilities in
shape/pen options and properties. properties stages a cached canvas preview before one
undoable edit. gradient fill controls from #1053 remain open; the line tool still draws a
filled bar. this extends ui reachability and does not claim photoshop-authored stroke parity.

| dimension | measured / evidence (2026-10-05) | grade | notes |
|---|---|---|---|
| menu wiring | 626/626 menu items dispatch a command (`parity-checklist.md`) | high but shallow | says nothing about behaviour. |
| PSD fidelity (rendering) | corpus oracle 115/170 (68%): 30 differ, 26 have no usable reference, 1 import error. 2026-10-07: io corpus 146/170, psd-tools corpus 236/309 (was 229: advanced blending knockouts), photoshop oracles 132/258 | medium | push to 170/170 under way (effects/strokes, multi-instance effects, 16/32-bit and colour modes, references for skipped files). |
| PSD round trip | 169/169 re-import identically; every adjustment layer and blend mode round-trips | high (within corpus) | floors in `crates/io/tests/corpus.rs`; raise, never lower. corpora: `cargo xtask corpus --all` (ours: https://github.com/storytold/brahma-corpus). |
| OpenRaster read/write | 2026-10-09: 34 krita 5.2.9-authored `.ora` files (27 blend modes, groups, pass-through, offsets, opacity, visibility, masks, 16-bit, gray) render within 3/255 of krita's merged image (30) or differ for known reasons (4: krita's lighter/darker-colour tie-break, hard mix at exactly 1, a linear-light 16-bit document); krita re-opens our exports and renders them identically to its originals (33/34; soft light is written as `svg:soft-light`) | medium (one authoring app) | MyPaint and GIMP files untested. masks are applied to pixels and layer styles dropped on save (reported). see [OpenRaster](ora.md). |
| paint.NET import | 2026-10-09: 14/14 local PDN3 textures preserve editable layers exactly through `.pcraft` and render within 2.142/255 of full-size previews; a supplied 5.x document (1280×720, 11 layers) passes editing/history/native saves and matches its thumbnail within 0.418/255 mean; all 14 blend modes exercised synthetically | medium (limited corpus) | import only; metadata omitted. more current-version files and full-size exports needed. see [PDN support](pdn.md). |
| smart filters / text / effect shapes in psds | measured on our photoshop-authored set (https://github.com/storytold/brahma-corpus, `corpus/photoshop`, 258 files): see the per-group floors in `crates/io/tests/corpus.rs` and `crates/engine/tests/photoshop_oracles.rs`. smart objects and smart filters now survive PSD save and open (41 corpus files, 206 smart objects, round trip strict; photoshop opens our exports with live filters); re-rendering photoshop's smart filters with ours matches 5/30 (was 1/30) | low–medium | remaining re-render gaps are filter maths (gaussian/motion blur, unsharp mask, emboss, add noise RNG) and bicubic placement. |
| core editing (layers, masks, selections, adjustments, filters, transforms) | broad engine coverage; many interaction bugs fixed after 0.2.0 (adjustment dialogs, curves, crop, move/transform modifiers, gesture origin) | medium | fixes not yet user-validated. |
| ui / ux polish | shortcut audit 214 → 0 failures; dock, layers rows and menus reworked; first visual-qa sweep found 14 defects (#147–#157). 2026-10-07: the keyboard-only shortcuts with no menu item (⌥[ ⌥] ⌥, ⌥. layer navigation, ⇧⌥[ ⇧⌥] to extend the selection, 1–0 for opacity and ⇧ for flow or fill, ⇧[ ⇧] hardness, ⌥⌘t to transform a copy and ⌥⇧⌘t to step and repeat), ⌥-click colour sampling with painting tools, double-click a layers row for layer style, file › new from clipboard, a centred main window and remembered liquify settings (#352, #417, #350, #368, #419, #418). 2026-10-09: canvas zoom is physical at any display scale — 100% is one document pixel per physical display pixel, and every screen-space overlay, cursor, scrollbar, navigator and fit follows it (#1943) | low–medium | needs recurring visual qa with realistic documents. |
| tools | 2026-10-10: pencil, mixer brush, color replacement, patch, content-aware move, vertical type, pattern stamp and rotate view are toolbar tools. the only tool still missing is art history brush: frame landed (#2914), red eye is a live toolbar tool (`paint.redEye`, TOOL-213-17) and perspective crop landed (2026-10-10, #1043, `image.perspectiveCrop`) | low–medium | patch has a live healing preview. content-aware move is partial: move/extend, structure/color and sample all layers exist; transform on drop and a live result preview remain missing (`TOOL-213-4` in the scorecard). magic/background eraser added; live gradients in progress (#180). magnetic lasso added 2026-10-08 (live-wire edge tracing, width/contrast/frequency, `select.magneticLasso`). pattern stamp added 2026-10-08 (s flyout, `paint.patternStamp`, aligned / impressionist). rotate view (2026-10-09) turns the canvas camera around its centre without rewriting pixels; reset view and match rotation copy the angle. crop rotation (2026-10-09, #1792): dragging outside the crop frame turns it (⇧ 15° steps, angle readout), and ↵ rotates the document and crops in one undo step (`image.crop` `angle`); the frame turns over the image as in photoshop's classic mode, and the options-bar straighten is still missing. |
| painting | brush model and brush settings panel near photoshop; .abr/.grd import; persistent presets; pen pressure/tilt on windows, web, macos and X11 | medium | native wayland pen input open (#79; a pen opens the window through xwayland meanwhile); X11 pressure confirmed by a user, macos not yet verified on tablet hardware. |
| text / typography | engine works; caret placement and size editing fixed; OpenType features, text-on-path editing, composer parity partial | medium-low | measure with the photoshop-authored set. |
| colour management | colour-managed canvas (document → monitor), embedded CMYK profiles, linear EXR/HDR, 16-bit float canvas | medium-high | monitor profile follows only at launch. |
| performance | 14k+ px on the GPU at ~⅓ the memory; adjustment preview 285 ms → 4–9 ms; font-size edits 297 ms → 4.6 ms; 2026-10-07: 30 mp TIFF open (banded, parallel strip/tile decode) deflate 345 → 32 ms, LZW 428 → 43 ms, bigtiff and every IFD readable; 2026-10-09: 4.2 mp indexed color, 256 colours, full-resolution CPU preview p50 3018 → 1061 ms (ryzen ai 7 350, three measured runs; GPU upload excluded); 2026-10-10: liquify 3500×2200, 800 px brush, visible 1:1 crop 3.20 → 3.63 ms p50 (5 runs) while refreshing ~9× as many pixels | medium-high on rasters | complex layout documents still laggy (#125/#128); >16384 px GPU tiling in progress (#49). liquify now displays source-resolution detail above 100% zoom; its ≤1 ms update target is not met (3.73 ms p95 on a windows run; GPU upload excluded). native indexed color previews run on one background worker with stale-result rejection; large palettes can still take about a second to compute. web previews remain synchronous. |
| stability | never-crash lint series, crash guard, `panic_hunt` fuzzing in the gate | medium-high | no field crash data yet. |
| camera RAW | DNG, CR2, sony ARW (lossless + compressed), nikon NEF (lossless + lossy compressed), RW2, uncompressed ORF; 2026-10-09: uncompressed fujifilm RAF, bayer and x-trans (edge-directed x-trans demosaic; 26 mp x-trans develop 306 ms on 12 cores) | medium | CR3, compressed RAF and nikon "lossy after split" still open (#50). non-DNG cameras have no calibration: their colour (matrix + tone curve) is fitted per file to the camera's embedded JPEG, with the neutral srgb-primaries fallback when the fit doesn't clearly help (2026-10-08: 8 D3200 nefs, mean δe vs the camera JPEG 11.6–26.0 → 1.0–7.5); measured per-model calibration still open. |
| ai / generative | none | ~0% | deferred by decision (#41). |
| ecosystem | sandboxed WebAssembly plug-ins instead of .8bf; `.atn` actions load and save (2026-10-12); no ExtendScript/UXP; no adobe fonts/libraries/cloud docs | low | by design for 8bf; scripting compatibility open. |
| platforms | macos (notarized), windows, linux (AppImage/deb/rpm/flatpak bundle), web | medium-high | flathub later (#173); windows signing material pending. |
| localisation | 2026-10-07: 10 ui languages; menu, `tl!`, blend mode, preference and brush-section coverage enforced by tests; live switching and scoped preferences previews | medium | engine errors/status messages still partly english; CJK web fonts, browser-locale detection, and RTL remain open. |

2026-10-08: motion blur adds adaptive FFT convolution for wide streaks while retaining the
row kernel merged in [#902](https://github.com/shep95/brahma/pull/902) for shorter
streaks and unsupported cases. native release medians of three paired runs on an intel
i7-9750h with 12 rayon workers, synthetic RGBA8 at 30° / distance 1000: 24 mp
22.594 → 9.785 s (2.31×), and 1.5 mp
2.455 → 0.592 s (4.15×).
full-image comparisons differ by at most one u8 code level; distance 64/256 retain the
byte-identical row output. these measure decode/halo/filter/write/prune, excluding ui proxy
creation, compositing and upload; the ui remains synchronous. FFT work is scheduled within
a conservative 512 mib working-set estimate, excluding stored surface tiles and row fallback
memory. the crossover is a heuristic; images dominated by near-cutoff alpha can need extra
scalar work. these measurements do not establish photoshop filter parity.

2026-10-10: proximity match search medians on an AWS c7i.4xlarge improved 1.54–3.82x
for four synthetic stroke regions (50–300 px holes), with identical source displacements.
bounding the mask table to the hole and pruning losing SSD candidates reduced table storage
by about 90–93 percent there. full 24–36 mp kernel inputs improved 4.60–6.24x, but normal
engine strokes already use a cropped region. see [method and limits](proximity-match-performance.md).

2026-10-09: the remove tool is in the j flyout (`paint.remove`, a background job). a stroke
around an object also removes what it encloses. the fill is non-local patch completion with
texture features (a. newson et al., IPOL 2017: `crates/algo/src/nonlocal.rs`), finished by a
best-patch copy and gradient-domain seam hiding. on 20 holes in public-domain photos
(`remove_quality`, `docs/development.md` › remove tool quality) the fill keeps 0.89 of the
original's texture (wexler/PatchMatch completion as content-aware fill uses it: 0.54). on a 24 mp
document a 100 px ring takes 120 ms and a 1000 px ring 2.5 s (`remove_bench`). it is not
generative: large objects over complex structure fill less convincingly than photoshop's ai mode
(#41). 2026-10-12: the options bar has photoshop's three options: sample all layers, create a new
layer (the fill lands on a new layer above the target, `paint.remove` `newLayer`) and remove after
each stroke; off, the strokes collect as a tint and ↩ (or ✓) applies them together as one job and
one undo step (`strokes`), esc (or ⊘) drops them (#1750).

2026-10-08: the tools update above is checked against the current toolbar groups in
[`panels.rs`](../crates/ui-egui/src/panels.rs), the pencil and tool-cycle tests in
[`pencil_tests.rs`](../crates/ui-egui/src/pencil_tests.rs), patch's live-preview test in
[`patch_preview.rs`](../crates/ui-egui/src/patch_preview.rs), and vertical type's point/paragraph
creation and undo test in [`type_tool_tests.rs`](../crates/ui-egui/src/type_tool_tests.rs).
these implementations remove the tools from the missing list; they do not establish photoshop
behavioural parity. the historical estimates and corpus measurements in this assessment are
unchanged; current measured floors remain in the scorecard.

2026-10-08: GPU lower-stack checkpoints reuse unchanged composites after property edits.
on a dense 24 mp / 12-layer stack, warm top-layer opacity edits at 8/16/32-bit depth measured
p50 229/228/186 → 31/25/26 ms (AMD radeon 860m, windows, vulkan; synchronized compositor only,
`bench_stack_reuse`). gains in the existing 20 mp p2–p5 canvas fixture were small and
run-order sensitive: a reversed-order repeat measured p50
13.2/13.2/12.0/8.2 → 12.5/12.2/12.0/8.0 ms; p5 p95 remains 13.7 ms, above its 10 ms budget.
one checkpoint is capped at 512 mib within the existing total budget;
the 24 mp run held 384 mb. planning and output/mipmap work remain. these local comparisons
do not replace the published baseline or enable budget enforcement on other hardware.

2026-10-07: camera raw PSD mapping covers relative custom white balance, light/presence,
parametric and four point curves, HSL, color grading, sharpening/noise detail, grain and numeric
post-crop vignette controls. two revisions of one supplied photoshop ACR 18.4 PSD preserve their
filter descriptors without edits. the updated revision stays opaque because of active manual
optics and unverified vignette style; a descriptor projection verifies its supported controls.
synthetic 8/16/32-bit PSD → edit → `.pcraft` → PSD tests preserve the filter stack, mask and
undo/redo. unmapped camera raw fields/versions remain opaque; photoshop
acceptance of generated exports and pixel parity are still unverified. corpus floors above are
unchanged.

2026-10-10: select and mask's shift edge now computes the exact octagonal footprint
with a square window extreme plus a dyadically built manhattan ball. on a c7i.4xlarge
(16 vcpus), full tiled CPU refinement at radius 64 / shift edge +100% takes
3.019 s -> 0.762 s at 24 mp (3.96x),
3.706 s -> 0.921 s at 36 mp (4.02x).
three alternating paired release runs include guide sampling, guided filtering,
boundary distances, quantization and assembly. every output mask matches exactly.
the dense fractional 24 mp radius-64 dilation kernel improves 36.00x;
circular soft-mask kernels improve 20.76-172.16x at 24-36 mp, radii 4-64.
bounded halo tiles reduce the large-mask workspace allocation bound from 192/288 mb
to about 115/163 mb, including output and 16 workers' scratch, excluding the source.
see [method and results](octagonal-mask-performance.md).

2026-10-10: five CPU imaging algorithms now reuse source work and interpolation.
on an m7i.4xlarge (16 vcpus), three alternating paired release runs measured
dense extrude blocks (8/128 px, 24 mp RGB8) 160.301 s -> 0.387 s (414.69x);
blast (24 mp CMYK8) 6.231 s -> 0.709 s (8.79x);
spin blur (36 mp RGB8) 9.439 s -> 5.869 s (1.61x);
lens correction (24 mp CMYK16) 0.808 s -> 0.565 s (1.43x);
trap (50 px, 36 mp CMYK16) 4.627 s -> 1.333 s (3.47x).
default 30/30 px extrude is 6.26x faster at 24 mp. all 72 full-operation output
comparisons match exactly, including alpha, across 24 cases and 8/16/32-bit depths.
timers include surface assembly but exclude ui, undo and trap document flattening.
see [method, raw measurements and reproduction](five-algorithm-performance.md).

### where we're going (priority order)

1. **ship the fixes:** cut 0.2.1 once the current batch lands, so users validate them.
2. **PSD fidelity to 170/170** with round trips, plus the photoshop-authored reference set for smart
   filters, the text engine and effect shapes (https://github.com/storytold/brahma-corpus, fetched by
   `cargo xtask corpus --all`); enforce floors in `corpus.rs`.
3. **workflow acceptance tests:** 30–50 real tasks (retouch a portrait, social post with text and
   effects, composite with masks and adjustment layers, CMYK print prep…) scripted end to end and
   checked against photoshop's output on every build. their pass rate becomes the headline parity
   number.
4. **tool coverage and workflow depth:** pen variants (direct selection landed, #790; add/delete anchor point, #1044);
   finish content-aware move's transform on drop and live result preview.
   pencil, mixer brush, patch, vertical type, pattern stamp and rotate view are already available on the toolbar.
5. **complex-document performance** (#125/#128) and GPU tiling beyond the texture limit (#49).
6. **recurring visual qa** (`cargo run -p brahma-engine --example designer_psd`) and fast
   turnaround on user reports (os, document size, layer count, screenshot).
7. later / needs decisions: generative ai backend (#41), scripting compatibility (ExtendScript /
   UXP / .atn), flathub (#173), wayland pen pressure (#79).

## revision history

| date | change | summary |
|---|---|---|
| 2026-10-13 | minor | vector shapes: on-canvas corner radius widgets on live rectangles (TOOL-214-12); estimates unchanged |
| 2026-10-13 | minor | stale "missing tool" rows corrected against main: color replacement (`paint.colorReplacement`), red eye (`paint.redEye`) and add/delete anchor point (the pen flyout) are live toolbar tools and frame landed, so art history brush is the only tool still missing; percentages unchanged |
| 2026-10-12 | minor | free transform options bar h/v skew fields; percentages unchanged |
| 2026-10-12 | minor | `.atn` import: 112 events mapped (TrueGrit texture supply sets play end to end); percentages unchanged |
| 2026-10-12 | minor | photoshop actions `.atn` load and save (90 events mapped both ways); percentages unchanged |
| 2026-10-12 | minor | rows checked against `main` and the regenerated scorecard: menu count 631, workspace checklist and preference count, transform context menu, puppet warp mesh, select and mask, anchor-point tools, smart object conversion, stamp visible, mask targeting, free transform commit/cancel, vietnamese catalog, windows ARM64; percentages, weights and hours not re-measured |
| 2026-10-10 | minor | added measured tilt-shift computation/filtered-proxy gains and inconclusive cases; parity percentages and budgets unchanged |
| 2026-10-10 | minor | readiness-by-audience table: percent and hours for full, mainstream and essentials; full number confirmed as the additive weighted sum over dimensions (46%, reported ~45%) |
| 2026-10-10 | minor | added mainstream-practitioner (~43%) and essentials-user (~61%) numbers with written weights and discounts; full ready-for-real-work rechecked against its written weights (46%, reported ~45%), unchanged |
| 2026-10-10 | minor | stage checked against the core-workflow alpha gate in `roadmap.md`: passes |
| 2026-10-10 | major | full re-measure against photoshop 2026 27.11.0 (installed bundle inspected); two numbers, weights, per-dimension and per-area hours calibrated from git history; merged `parity-estimate.md` and the roadmap's honest assessment into this file |
| 2026-10-05 | major | honest parity assessment in `docs/roadmap.md`: surface 60–70%, ready for real work 25–35% |
| 2026-10-03 | major | `docs/parity-estimate.md`: menu breadth 625/625, weighted ~82–86% |
