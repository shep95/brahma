# roadmap

> **last reviewed:** 2026-10-11 · **last updated:** 2026-10-11 · **change:** minor (adaptive unsharp mask performance evidence linked; photomerge status correction retained; alpha gate, focus and estimates unchanged) · **target:** adobe photoshop 2026

- ui.5: panels can detach, join other tab groups, split, resize and redock through shared docking transactions. custom layouts persist with workspaces, respect workspace locking, and retain collapse flags for intact native groups.

- ui.3: shared icons, accessible icon buttons and panel tabs come from the pinned `craft-ui` library. tab overflow stays within narrow panels; document tiling uses bounded shared geometry. palettes and document commands remain app-owned. existing strict-clippy boolean/range warnings are corrected.

- ui.4: compact toolbars keep colour and mode controls visible while tool rows scroll. long tool-options rows scroll horizontally and reveal keyboard focus; offscreen geometry and interaction regressions cover all themes and 1×/1.5×/2× display scales.


forward-looking plan: milestones, the **current focus** list, and what's next with estimates.
where we stand today is in [`target-app-parity.md`](target-app-parity.md) (two numbers, by
dimension and area) and the ranked work list in [`gaps.md`](gaps.md); the one-page summary is
[`ROADMAP.md`](../ROADMAP.md). measured numbers: [`scorecard.md`](scorecard.md)
(`cargo xtask scorecard`: performance budgets, corpus floors, per-area checklists, dead
preferences) and [`parity-checklist.md`](parity-checklist.md) (`cargo xtask parity`: menu wiring,
628/628, which says nothing about behaviour).

status legend: ✅ done · 🟡 in progress · ⬜ not started. hours are opus 5.5 agent wall-clock hours
for one agent working sequentially (calibration in
[`target-app-parity.md`](target-app-parity.md#remaining-effort-and-calibration)).

**stage: alpha** (ready for real work ~45%). beta needs ~75% and reliable PSD: ~1,100–1,800 h.

## alpha gate

the core workflows a typical photoshop professional runs every day, checked end to end on the main
platform (macos, 0.6.0), including saving and reopening the work. any **no**, or a **partial**
that blocks the workflow, would make brahma pre-alpha.

| core workflow | works end to end? | evidence | hours to pass |
|---|---|---|---:|
| photo retouch: open JPEG/raw, heal/clone/remove, adjustment layers, export JPEG, save and reopen | yes | spot healing, healing, clone, patch, remove tools; 16 adjustment layers; raw for DNG/CR2/NEF/ARW/RAF; JPEG export; MCP workflow tests (`automation/tests/agent_tasks.rs`). quality complaints (#1092, #2104) and no CR3 don't stop the workflow | 0 |
| composite: layers, masks, selections, blend modes, smart objects; save as PSD and `.pcraft` and reopen | yes | PSD round trip ≥ 169/170, 308/309, 258/256 corpus files; `.pcraft` atomic save and crash recovery. multi-layer smart object conversion (#2604) and mask targeting (#2458) are partial but have workarounds | 0 |
| cut-out: select a subject, refine the edge, mask, export PNG with transparency | partial (not blocking) | object/quick selection, select subject (classical), refine edge dialog, layer masks, PNG export. select and mask is a dialog, not the workspace; no refine hair (#2671) | 0 (depth: 80–140 h, g7) |
| design: type, shapes, layer styles, artboards; export PNG/JPEG; save and reopen | partial (not blocking) | point/paragraph type, shapes with stroke controls, all 10 layer effects, artboards, export as. one macos report of a 60 s freeze on choosing the type tool (#2518, not reproduced); no type on a path | 0 (fix #2518: 2–6 h) |
| paint with a pen tablet: pressure brushes, layers, save and reopen | partial (not blocking) | pressure, tilt, rotation on macos (AppKit monitor); every brush settings section. lag with big brushes (#2619) and feel gaps ([brush-parity.md](brush-parity.md)) | 0 (feel: 100–180 h, g4) |
| exchange psds with photoshop users: open a client PSD, edit, send a PSD back | partial (not blocking for alpha; blocking for beta) | photoshop-authored oracle 133/256 render as photoshop does; most exports open in photoshop (smart objects verified), but some don't (#1281, #2469) | 0 for alpha; 150–250 h for beta (g1) |

**result: passes.** every core workflow completes end to end and the work saves and reopens in
brahma; the partial rows are depth and fidelity gaps with workarounds, not blockers. ready for
real work is ~45%, inside the alpha band (~40–75%), but close to its floor.

## milestones

| m | status | where we are |
|---|---|---|
| m0 skeleton | ✅ | workspace, xtask (layers / wasm / ci / stats / corpus / parity), ci workflow |
| m1 foundation | ✅ | geom, color (27 blend modes), raster (COW tiles, any depth), doc, ops, cms (ICC) |
| m2 PSD v1 | ✅ | brahma-psd: 134/135 real files byte-exact round trip |
| m3 viewer app | ✅ | egui shell (pro / studio / classic themes), 13+ codecs, native and web (trunk) builds |
| m4 native format + engine | ✅ | ~700 commands, `.pcraft` (incremental, autosave, crash recovery), CLI, persistent preferences |
| m5 GPU compositor | 🟡 | wgpu compositor drives the canvas, layer effects, vector masks, artboards, pattern fills and every clip case included (≤1/255 vs CPU); multichannel documents fall back to the CPU |
| m6 paint + select | 🟡 | brush engine, all selection tools, multi-layer selection, snapping + smart guides, free transform + warp; stylus pressure on windows (WM_POINTER via winit), the web (pointer events, with tilt/twist), macos (AppKit event monitor: pressure, tilt, rotation, eraser end) and linux X11 (xinput2 raw valuators); on linux wayland a pen opens the window through xwayland (native tablet-v2 still open, #79) |
| m7 adjust + filters | 🟡 | 16 adjustment layers + destructive-only adjustments, 70+ filters incl. blur gallery, actions record/replay, fade |
| m8 PSD v2 | 🟡 | adjustments (incl. selective color, color lookup), fills, effects, patterns, text, shapes, smart objects, alpha channels; oracle floors 146/170 (io), 236/309 (psd-tools), 133/256 (photoshop-authored) |
| m9 text, vector, styles | 🟡 | type engine + warp text, shapes / pen / paths, all 10 effects on CPU and GPU (parity ≤1/255, 30/31 corpus effect files on the GPU) |
| M10 smart features | 🟡 | classical select subject / object, content-aware fill and scale, healing, auto-align / auto-blend; ml backend not started |
| M11 automation + formats | 🟡 | MCP (headless + live bridge), batch, image processor, prefs over MCP; dod test passes (10 agent tasks over MCP, `automation/tests/agent_tasks.rs`); since 2026-10-07 the CLI rejects unknown flags and answers `<subcommand> --help`, batch runs report same-name outputs instead of overwriting them, MCP `command_batch` steps can start background jobs, and headless saves never flatten over the opened file; JP2 / DICOM / DPX / C2PA pending |
| M12 pro parity | 🟡 | CMYK / lab / indexed / bitmap / duotone, ICC + soft proofing, channels + quick mask, smart-object stack modes, artboards, layer comps; print, HDR, timeline pending; photomerge is implemented, with real-world quality validation still open |

releases: 0.1.0 (2026-10-02), 0.2.0 (2026-10-05, first signed and notarized), 0.3.0 (10-07),
0.5.0 (10-08), 0.6.0 (10-10).

## current focus

ranked; each item links to its gap entry. pick from the top unless an issue is assigned to you.

1. **PSD that photoshop trusts** ([g1](gaps.md#g1-psd-fidelity-and-photoshop-acceptance)):
   fix photoshop refusing our files (#1281, #2469; text-layer case #2374 closed); add a
   photoshop re-open check (FILE-216-4); photoshop-oracle floor 133/256 → 200+. 150–250 h.
2. **performance budgets** ([g2](gaps.md#g2-interactive-performance)): the 22 over-budget
   scenarios, the P12 GPU crash at 150 layers, brush and cursor lag (#2619, #2566). 200–350 h.
   local adaptive [unsharp mask optimisation](unsharp-mask-performance.md) improves native
   threshold-zero 24 mp RGBA CPU application by 37–102x at radii 300–1000, with measured
   rounding differences and an exact fallback; no published budget is closed by that measurement.
3. **user bug backlog** ([g3](gaps.md#g3-user-reported-bug-backlog)): 579 open issues, most from
   0.5/0.6 users; triage weekly, fix regressions first. 250–400 h.
4. **brush and transform feel** ([g4](gaps.md#g4-brush-feel), [g5](gaps.md#g5-transform-handles-and-modifiers)):
   [`brush-parity.md`](brush-parity.md) and [`ui-parity.md`](ui-parity.md) rows marked partial
   or missing. 150–250 h.
5. **missing tools** ([g6](gaps.md#g6-missing-tools)): 14 of photoshop's 68. 60–100 h.

## what's next, with estimates

| next milestone | scope | estimate |
|---|---|---:|
| 0.7 "files you can trust" | current focus 1 plus JPEG options, AVIF read, photoshop PDF read | 200–320 h |
| 0.8 "fast enough" | current focus 2; GPU tiling past the texture limit (#49); complex layout documents | 200–350 h |
| 0.9 "feels like photoshop" | current focus 4 and 5; transform context menu; floating panels; color panel | 300–500 h |
| beta | all of the above, type depth, select and mask / content-aware fill workspaces, hindi/arabic/vietnamese + RTL, bug backlog under 100 | 1,100–1,800 h total |
| later, needs owner decisions | ai backend (#41), plug-in hosting (.8bf / UXP), scripting compatibility | 550–1,050 h |

## milestone definitions

each milestone has a **definition of done (dod)** and must leave `main` green on all tier-1 platforms plus a wasm build.

| m | name | scope (key items) | dod / acceptance |
|---|---|---|---|
| **m0** | skeleton | workspace, all crate stubs, lints, `xtask` (layers check, ci), ci matrix, testkit | `cargo test --workspace` green; `cargo check --target wasm32-unknown-unknown -p brahma-engine -p brahma-ui-egui` green; layering check passes |
| **m1** | foundation | geom, color (formats, blend math for all 27 modes, srgb/linear), raster (sparse COW tiles, u8/U16/F32), doc (full type model incl. CMYK/lab/adjust/smart), ops (history) | property tests (proptest) on tile COW and history; blend-mode reference tests against published formulas |
| **m2** | PSD v1 | `brahma-psd`: header, resources, layer records, channel data (raw/RLE/ZIP/ZIP+pred), masks, groups (lsct), unicode names, unknown-block passthrough, merged image, PSB; writer | round-trip byte-stability tests; synthetic PSD generator tests; fuzz target; ≥150 unit tests |
| **m3** | viewer app | codecs (png/jpeg/tiff/webp/gif/bmp/tga/pnm/qoi/exr/hdr, all read+write), CPU compositor, egui shell (menu from registry, canvas pan/zoom, layers panel, history), open/save, native + web build | opens PNG/JPEG/PSD, shows layers, toggles visibility, undo/redo, saves PNG/PSD; web build loads a file from the browser |
| **m4** | native format + engine | `.pcraft` bundle, command registry with schemas, jobs/cancellation, snapshots via arc-swap, CLI (`convert`, `run`, `inspect`) | CLI parity tests: every GUI command also runs headless |
| **m5** | GPU compositor | wgpu planner backend, tile residency, parity tests vs CPU (all blend modes), viewport on GPU, mips | GPU vs CPU ≤1/255; pan/zoom 60 fps on a 100 mp / 20-layer document |
| **m6** | paint + select | platform input (pen: macos/windows/linux/web), brush engine v1, eraser, marquee/lasso/wand, move, free transform, crop | brush latency <1 frame; selection-restricted paint tests |
| **m7** | adjust + filters | 16 adjustment layers, first 30 filters, schema-generated dialogs + live preview, actions record/replay | golden tests at 8/16/32f; action replay determinism |
| **m8** | PSD v2 | adjustment layers, fill layers, lfx2 effects (descriptor parser), text (EngineData) preserve+render, smart objects, vector masks | composite-oracle pass rate ≥90% on corpus |
| **m9** | text, vector, styles | parley text layers, shapes/pen/paths, all 10 layer effects on GPU | visual goldens; PSD text round-trip |
| **M10** | smart features | `ml` (ort native / ort-web on the web), select subject/object/sky, remove bg, remove tool, content-aware fill, healing, ai denoise, RAW develop | quality benchmarks on a public dataset; timing budgets |
| **M11** | automation + formats | MCP server, batch, scripting, remaining formats (JP2, DICOM, DPX…), C2PA | an agent completes 10 scripted edit tasks via MCP |
| **M12** | pro parity | CMYK/lab ui, print, HDR display/merge, photomerge quality validation, timeline, layer comps, artboards, symmetry, neural filters | `xtask parity` ≥ 90% of photoshop menu checklist |

## revision history

| date | change | summary |
|---|---|---|
| 2026-10-11 | minor | linked adaptive unsharp mask performance and quality evidence; milestones, alpha gate and estimates unchanged |
| 2026-10-10 | minor | photomerge is implemented; real-world quality validation remains open |
| 2026-10-10 | minor | alpha gate table (core-workflow gate from the progress-docs standard): passes, stage stays alpha |
| 2026-10-10 | major | progress-docs standard: honest assessment and dated measurements moved to `target-app-parity.md`, affinity notes to `file-format-parity.md`; new current focus and estimates; `parity.md` renamed `parity-checklist.md` |
| 2026-10-08 | minor | tools, affinity import and performance measurements added |
| 2026-10-05 | major | honest parity assessment added |
