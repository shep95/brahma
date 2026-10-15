# ui parity: tools, handles, modifiers and feel

> **last reviewed:** 2026-10-10 · **last updated:** 2026-10-14 · **change:** minor (guide keys and clicks; menu bar from the keyboard, #4988; crop tool content-aware fill; on-canvas corner radius widgets, TOOL-214-12; curvature pen edits the active path; clip thumbnails to layer bounds; pen ⇧ 45°; last selected tool restored at launch; layers panel filter-type dropdown with kind, name and mode, #5048; free transform box and marching ants follow the selection, #5297; history: the trash deletes the selected snapshot, rows drag onto the trash, #5324; smart filter reordering by drag; smart object perspective row corrected to done, #426; free transform h/v skew fields; toolbar tool shortcuts can be reassigned; ⇧-dragged guides land on ruler ticks; layer and selection keys; tool keys; spring-loaded tool letters; type formatting keys; pen auto add/delete, #4144; smart filter rows; stale missing-tool rows corrected: color replacement, red eye and the add/delete anchor point pair are live toolbar tools, frame landed; no percentage changed) · **target:** adobe photoshop 2026

how brahma's direct manipulation compares with photoshop 2026 (27.11.0): the toolbox, the
handles and modifier keys of each gesture, nudging, snapping, numeric precision, navigation,
shortcuts and panels. painting feel has its own checklist in [`brush-parity.md`](brush-parity.md)
and right-click menus in [`context-menu-parity.md`](context-menu-parity.md). the design system
(themes, tokens, widgets) is [`ui-design.md`](ui-design.md).

status: **done** (behaves as in photoshop, with a test), **partial** (exists, differs or untested),
**missing**. evidence is a file under `crates/ui-egui/src/` unless another path is given, a
scorecard id, or an issue. modifier names are macos first (⌘ ⌥ ⇧); windows/linux use ctrl and alt.

**summary (estimated, weights = how often the gesture is used):** ~40% ready. tools 56/68
present (82%, measured). free transform modifiers and crop are close to photoshop; the gaps are
in the long tail of modifiers, in-transform ui, spring-loaded tools, panels and preferences.
remaining: 60–110 h for transforms and modifiers (gap g5), 60–100 h for tools (g6), 150–250 h
for panels and workspace (g9).

## tools

measured: the `Tool` enum (`state.rs:64`) and `TOOL_SECTIONS` (`panels.rs:17`, 22 flyout slots)
against the 67 `<tool>` entries of photoshop's `Default Keyboard Shortcuts.kys` plus remove,
triangle and curvature pen (minus legacy rounded rectangle and targeted adjustment) = 68.

| group | present | missing |
|---|---|---|
| move, artboard | move, artboard | |
| marquee | rectangular, elliptical, single row, single column | |
| lasso | lasso, polygonal, magnetic | |
| selection | object selection, quick selection, magic wand | object selection's object finder (on by default in its options bar): resting the pointer highlights the object a click selects (`select.object` with `point`; ⇧ adds, ⌥ subtracts, ⇧⌥ intersects); dragging keeps the rectangle mode (#4087). heuristic, not a learned segmenter (#2788). |
| crop and slice | crop, perspective crop, slice, slice select | |
| frame | frame (frames are groups clipped by a rectangular or elliptical mask) | |
| eyedropper | eyedropper, color sampler, ruler, note, count | |
| retouch | remove, spot healing, healing, patch, content-aware move, red eye | |
| brush | brush, pencil, mixer brush | |
| stamp | clone stamp, pattern stamp | |
| history | history brush | art history brush |
| eraser | eraser, background eraser, magic eraser | |
| fill | gradient, paint bucket | |
| focus, toning | blur, sharpen, smudge, dodge, burn, sponge | |
| pen | pen, freeform pen, curvature pen (#1054), convert point (#3204; also ⌥ with pen), add anchor point, delete anchor point (p cycles them; #4144 auto add/delete with the pen itself) | |
| type | horizontal, vertical, vertical type mask, horizontal type mask | |
| path selection | path selection, direct selection | |
| shapes | rectangle, ellipse, triangle, polygon, line, custom shape | (star is a polygon option in photoshop) |
| view | hand, rotate view, zoom | |
| toolbar controls | edit toolbar, colour chips, quick mask, screen mode | |

| toolbar behaviour | status | evidence |
|---|---|---|
| letter shortcuts, ⇧+letter cycles the group, "use shift key for tool switch" (on by default, as in photoshop) | done | `shortcuts.rs:413` |
| group remembers the last tool used | partial | l always resets to lasso (#2608) |
| press-and-hold or click-drag opens the flyout | missing | right-click only (#2301) |
| edit toolbar customization | done | `edit.toolbar` |
| last selected tool restored after quitting and relaunching; temporary tool switches excluded | done | `tool_memory.rs`; `prefs_ui::tests::selected_tool_survives_a_restart_without_option_changes`, `selected_tool_excludes_temporary_switches` |

## free transform (⌘t) and transform handles

`transform_tool.rs` (3,158 lines), behaviour summary in [`ui-design.md`](ui-design.md#interaction-models-match-photoshop-cc).

| behaviour | status | evidence |
|---|---|---|
| eight handles, drag inside moves, outside rotates with rotation cursor and live angle | done | #1948 |
| corners and edges proportional by default; ⇧ frees them (edge stretches one axis) | done | #1873 |
| ⌥ scales about the reference point; ⌥-click places it; draggable reference point | done | |
| ⌘-corner distort, ⌘-edge skew, ⌘⌥⇧ perspective, ⇧ rotate in 15° steps | done | |
| warp: presets, bend, split crosswise/vertical/horizontal, 3×3–5×5 and custom grids | done | |
| options bar x/y, w/h % with link, angle | done | |
| with a selection, the box is the selection's rectangle and the marching ants follow the box live | done | `transform_frame`, `transform_tool.rs` (`draw_selection`): move, scale, rotate, skew, distort, perspective and warp, before the transform is applied; also the move tool's show transform controls (#5297) |
| options bar h/v skew fields | done | `transform_tool.rs` (`Parts`, `skew_about_pivot`): w/h, angle and skew read as photoshop decomposes the box; `skew_fields_lean_the_box_about_the_reference_point` |
| interpolation in the options bar | partial | bicubic/bilinear/nearest; no bicubic smoother/sharper/automatic, preserve details |
| commit/cancel buttons in the options bar | missing | #2442 |
| right-click menu during transform (scale, rotate, skew, distort, perspective, warp, flips, rotates) | missing | TOOL-214-5 |
| arrow keys nudge the box 1 px, ⇧ 10 px | done | `move_mods.rs` |
| preview keeps layer order, blend mode and mask | partial | mask yes (BUG-205-2); order/blend not (#2439) |
| preview clipped to the canvas, follows edits while open | done | #2201, #2099 |
| hand (space) mid-transform doesn't interrupt it | partial | #2302 |
| blend mode / opacity edits while transforming | missing | #1384 |
| locked layer refused before the drag | missing | fails only at commit (#2585) |
| transform a copy (⌥⌘t), step and repeat (⌥⇧⌘t) | done | 2026-10-07 |
| transform a targeted alpha channel | missing | BUG-207-4 |
| smart object perspective (non-affine) | done | FILE-215-6; #579 fixed by #426: distort / perspective keep a projective placement, re-rendered from the source, read and written as the PSD `nonAffineTransform` |
| type-layer transform while editing (⌘ held), no ⌘-corner distort on type | done | `ui-design.md` |
| puppet warp mesh follows the outline | missing | block grid (TOOL-214-6) |

## move tool and nudging

| behaviour | status | evidence |
|---|---|---|
| arrow 1 px, ⇧-arrow 10 px for layers | done | `move_mods.rs` |
| arrow nudge of a selection outline (selection tools) | done | #1502 |
| arrow nudge of the crop box, x swaps orientation | done | TOOL-214-19 |
| arrow nudge of path points with direct selection | missing | no arrow handling in `direct_select.rs` |
| ⌘-arrow nudges the layer from any tool | missing | #2474 |
| ⌥-drag duplicates, ⇧ constrains to 45° | done | `move_mods.rs` |
| auto-select with hover outline, box-select layers, double-click fit | missing | TOOL-214-4 |
| auto-select bounding box and snapping follow the picked layer | partial | follow the previous layer (#2666) |
| ⌘-drag temporary move shows the box and snaps | partial | #2681 |
| ⌘-right-click layer list under the pointer | done | `layer_pick_ui.rs` |
| double-click type to edit it (caret at the pointer, type tool) | done | #1132, `type_tool::edit_at` |
| moving a selection on a mask moves the mask, not the pixels | partial | #2468 |

## selection modifiers

| behaviour | status | evidence |
|---|---|---|
| ⇧ square/circle, ⌥ from centre, live w × h readout (marquee) | done | `marquee_tests.rs` |
| ⇧ add, ⌥ subtract, ⇧⌥ intersect from modifiers at press | partial | works; only a badge test (BUG-208-4) |
| space repositions the marquee while dragging | done | `hold_keys.rs` |
| polygonal lasso ⇧ snaps to 45° | missing | #2352 |
| autoscroll at the window edge while selecting | missing | #2231 |
| single-pixel rows and columns kept on release | partial | #2391 |
| esc deselects / ⌘z undoes the last selection | partial | #2674, #1432 |
| select › modify live preview | missing | #1752 |

## pen, paths and shapes

| behaviour | status | evidence |
|---|---|---|
| click corner, drag smooth, ⌥ breaks a handle, ⌘ direct-selects, ⇧ 45° | done | TOOL-214-10; `vector_ui.rs` (#3751) |
| handle on the last point, close-drag handle | partial | #1482 |
| pen auto add/delete: click a segment of the active path to add, an anchor to delete; ⇧ suspends it | done | #4144 |
| curvature pen edits the active path with no modifier: drag an anchor (the curve reflows), click a segment to add an anchor, double-click smooth ↔ corner; cursor badge + on segments, four arrows on anchors | done: `direct_select.rs` (`curvature_press`, `curvature_hover`), `path.moveAnchors`/`path.convertPoint` with `curvature`, `direct_select_tests.rs` | |
| path operations in the options bar (combine, subtract, intersect, exclude) | partial | `layer.combineShapes.*`; pen options #1397 |
| on-canvas corner-radius handles on live shapes | done: every selected live rectangle shows a widget at each arc centre with path selection, the rectangle tool and move with show transform controls; a drag rounds every corner of every selected rectangle to the same radius, ⌥ that corner on each, with the corner radius cursor and an `R:` readout, previewed and committed as one history step (`corner_radius.rs`, `corner_radius_tests.rs`) | TOOL-214-12 |
| shape stroke controls (dash, caps, joins, align) | done | TOOL-1053-1, [`shape-strokes.md`](shape-strokes.md) |
| rotated shape selection hints correct | partial | #2512 |
| curved shape bounds exact | partial | one pixel too wide (#2537) |

## crop

| behaviour | status | evidence |
|---|---|---|
| drag outside rotates (⇧ 15°), angle readout; ↵ rotates and crops in one undo step | done | TOOL-214-18 |
| straighten button and ⌘-drag line | done | TOOL-214-21 |
| default-mode resize snaps handles without camera feedback; image and frame use the same camera in each rendered frame | done | `crop_mode.rs::a_live_resize_snaps_the_handle_without_feedback_from_the_camera`, `tests/crop_default_mode.rs::a_resize_renders_the_frame_and_image_with_the_same_camera` (2026-10-10) |
| ratio presets and w × h changes immediately fit the current frame around its centre; swap button and x share the fitted ratio, enter needs no resize | done | `crop_ui.rs::choosing_a_ratio_fits_and_commits_without_a_resize`, `crop_size.rs::choosing_a_size_with_the_canvas_ratio_commits_without_a_resize`, `tests/crop_default_mode.rs` (2026-10-10) |
| roomy crop options bar does not scroll; narrow tool bars retain horizontal scrolling | done | `panels.rs::a_wide_crop_options_bar_does_not_scroll`, `panels.rs::narrow_tool_options_can_scroll_to_the_last_control` (2026-10-10) |
| overlays (thirds, grid, diagonal, triangle, golden ratio/spiral), o / ⇧o | done | TOOL-214-20 |
| crop shield, show cropped area (h) | done | TOOL-214-22 |
| default mode: the image turns and moves behind an upright box; auto center preview; use classic mode | done | TOOL-214-23 |
| w × h × resolution with units, built-in size presets, front image | done | TOOL-214-9 (`crop_size.rs`) |
| saved custom crop presets | missing | TOOL-214-9 |
| fill: content-aware fill: a turned or extending crop fills the uncovered corners and strips from the image (each piece with its own sampling window), on the background or else the active pixel layer, in the crop's one undo step; generative expand not offered | done | `tests/crop_content_aware.rs`, `crop_ui.rs::commit_passes_the_fill_choice` (#3823, 2026-10-12) |
| delete cropped pixels off | partial | |

## snapping, guides, rulers, precision

| behaviour | status | evidence |
|---|---|---|
| snap to guides, grid, layers, document bounds and centre, selection, slices; ctrl disables other gestures, while distort corners snap to guides and intersections | done | `engine/src/snap.rs`, `snap_ui.rs` (actual-corner grab-offset regressions, #3632) |
| snap vector tools and transforms to the pixel grid | done | `snap_ui.rs:196` |
| smart guides (magenta alignment lines) | done | |
| smart guide distance and spacing labels | missing | |
| ruler units px/in/cm/mm/pt/pica/% | done | |
| ⇧-drag a guide out of a ruler: it lands on the nearest ruler tick | done | `rulers.rs` `snap_to_tick` (#3770) |
| status bar follows the ruler unit | partial | #2602 |
| ruler corner and gaps drawn as photoshop | partial | #2600, #2601, #2551 |
| guides stored as f64 | missing | `Vec<f32>` (ui-217-20) |
| geometry precision | partial | `geom::Point` f64, paths f64; paint internals mostly f32 |
| numeric fields: arithmetic, scrubby labels | partial | units in expressions and double-click reset missing (ui-217-13) |

## navigation

| behaviour | status | evidence |
|---|---|---|
| space = hand, ⌘space / ⌘⌥space = zoom in/out, spring-loaded while held | done | `hold_keys.rs` |
| hold any tool letter for a temporary tool | done | `spring_tools.rs`: held ≥ 300 ms or used on the canvas, release gives the previous tool back; ⇧ never springs (#2672) |
| scrubby zoom | done | `zoom_tool.rs` |
| flick panning | done | `flick_pan.rs` (preferences › tools › enable flick panning) |
| animated zoom | missing | unread pref; #566 |
| double-click zoom tool = 100%, double-click hand = fit | partial | zoom missing (#2508) |
| ⌥-wheel zoom in gentle steps | missing | ui-217-18 |
| wheel, smooth scrolling and pinch as in photoshop | partial | #2485, #2463, #1989 |
| trackpad rotate drives rotate view | done | `rotate_view.rs:136` |
| zoom levels to 3200% and beyond | partial | #1813 |
| physical 100% at any display scale | done | #1943 |
| ⌃tab / ⌃⇧tab switch document tabs | missing | #2340 |

## keyboard shortcuts

| measure | value | kind |
|---|---|---|
| photoshop default shortcuts (`.kys`) | 58 tool keys, 107 menu commands with 115 shortcuts, 25 tool-area commands | measured |
| brahma | ~199 shortcut assignments, 113 distinct modifier/f-key combos, 20 tool letters, 3 temporary-tool bindings | measured (source) |
| shortcut audit | 214 → 0 failures (2026-10-07) | measured then; not re-run here |
| `.kys` import | done (`engine/src/kys.rs`) | |
| shortcut glyphs per platform | done (ui-217-19) | |
| toolbar tool shortcut editing | done (#4086): each tool has a row under tools; assignments persist, removed tools leave the default key group, and unchanged tools keep shift cycling. toolbar and flyout hints follow the binding | `tool_shortcuts.rs`, `shortcut_capture_tests.rs` |
| brush size/hardness keys on non-us layouts | done (#3109): `[` `]` need AltGr on many layouts, so photoshop uses other keys there (german: ö / #, ⇧ for hardness). `layout_keys.rs` has a data table of such layouts (`LAYOUTS`, add rows to support one more) and a fallback for all others: the keys at the us `[` / `]` positions act as `[` / `]` when they don't type brackets. off once a real bracket is typed; edit › keyboard shortcuts overrides still apply | |
| layer and selection keys: ⌘, toggles visibility, ⌘\ targets the mask, / locks transparency, ⌘⌥2 loads luminosity, ⌘⌥⌫ fills from history, tab / ⇧tab rename the next layer, ⌥-click a thumbnail fits the layer | done (2026-10-11) | |
| tool keys: ← / → turn the brush tip (1°, ⇧ 15°), ⇧-click with type starts new text over old, space+f / ⇧space+f cycle the pasteboard colour | done (2026-10-11) | |
| type formatting keys while editing text | done: ⇧⌘k, u, b, i and / toggle all caps, underline, faux bold, faux italic, strikethrough (off when every selected character has it, else on); ⌥↑/↓ leading ∓2 pt, ⇧⌥↑/↓ baseline shift ±2 pt (10 with ⌘); ⌥←/→ tracking ±20 over a selection (100 with ⌘; a collapsed caret still kerns); ⌥⇧⌘h toggles hyphenate for the touched paragraphs. one `type.setStyle` step per press (new `leadingBy`/`baselineShiftBy`/`trackingBy`; auto leading starts from its laid-out value); the global ⇧⌘k/u/b/i never fire while typing. differs: a collapsed caret in non-empty text changes nothing, since the model can't hold a pending style for the next characters typed (empty text takes it as its typing style) (`type_tool.rs`, `input_tests.rs` `type_tool_format_keys_*`) | |
| guides: click selects, delete deletes, right-click menu, ⌥-click turns a guide, double-click opens guides, grid & slices | done (2026-10-12) | |

## panels and workspace

30 of 35 window-menu panels exist (measured). missing: libraries, comments, contextual task bar,
discover, generative variations. dock tabs: layers, channels, paths, history, actions, layer
comps, properties, adjustments, navigator, histogram, info, color, swatches, character,
paragraph; the rest open as in-app floating windows.

| behaviour | status | evidence |
|---|---|---|
| panel groups float and tear off into os windows | missing | ui-217-5, #1828, #1213 |
| drag panels to reorder and regroup | partial | #2272, #2305 |
| docked resize pushes the neighbour | partial | #2573 |
| color panel wheel and RGB/HSB/CMYK/lab sliders | missing | ui-217-17, #2569 |
| window size and dock width remembered | done | ui-217-2 |
| command palette with recents | done | ui-217-7 |
| menu bar from the keyboard: alt / F10 focus the bar, ← → ↓ ↩ drive it, alt+letter opens a menu, underlined mnemonics run items | done | `menu_keys.rs`, #4988; macos uses the native bar |
| every preference takes effect | partial | 46 of 149 do nothing (scorecard) |
| ui scale presets | missing | ui-217-9, #2492 |
| history snapshots: camera button, snapshot rows (click returns and selects, rename, delete; the trash deletes the selected snapshot, and a snapshot or state dragged onto it, #5324), `history.*Snapshot` commands | partial | `history_cmds.rs`, `snapshot_rows` in `panels.rs`; new snapshot dialog, first snapshot and history options to come (#2966) |
| themes: photoshop's four greys | partial | five themes; #2661 |
| document tabs: overflow, context menu, reorder | done | ui-217-6; middle-click close #2570 |
| layers panel: wheel steps blend modes, drop on new/trash | partial | ui-217-14 |
| layers panel filter-type dropdown (kind, name, effect, mode, attribute, color, smart object, selected, artboard) | partial | kind, name and mode; the other six missing. `layer_filter_ui.rs`, `ui.set {layerFilter}` (#5048) |
| smart filter rows: click selects, drag onto another smart object moves, ⌥-drag copies, drag onto delete removes | done | `smart_ui.rs`, `layer.smartFilter.transfer` (#4271) |
| layers panel: clip thumbnails to layer bounds (panel menu, `interface.clipThumbnailsToLayerBounds`) | done (2026-10-12) | `layer_thumb_bounds_tests.rs`; #4281. thumbnail size and the panel options dialog to come |
| smart filter rows: drag up or down the stack to reorder (a line marks the drop; the bottom filter is applied first) | done | `smart_ui.rs` `reorder_drop`, `layer.smartFilter.move` |

## revision history

| date | change | summary |
|---|---|---|
| 2026-10-13 | minor | menu bar from the keyboard: alt / F10 focus, alt+letter, underlined mnemonics (#4988) |
| 2026-10-13 | minor | layers panel filter row: kind is a real dropdown with kind, name and mode filters, drivable by `ui.set {layerFilter}` (#5048); row added as partial |
| 2026-10-13 | minor | stale "missing tool" rows corrected against main: color replacement (`paint.colorReplacement`), red eye (`paint.redEye`) and add/delete anchor point (the p flyout cycle) are live toolbar tools, so only art history brush is missing; no percentage or estimate changed |
| 2026-10-13 | minor | smart object perspective (non-affine) corrected to done: #426 landed four-corner placements with PSD round trip; scorecard FILE-215-6 flipped |
| 2026-10-10 | major | first version, from a source audit against photoshop 2026 27.11.0's shortcuts file and the open issues |
| 2026-10-12 | minor | closed #3632: distort and ctrl/cmd corner drags snap actual corners to guides and intersections; shared guide snapping respects extras and canvas guides visibility, with offset, visibility and constrained-alignment regressions |
| 2026-10-10 | minor | fixed crop live resize snapping feedback and outdated-camera rendering, applied ratios immediately to centred inscribed previews, and removed two-point options-bar overflow; added pointer, rendering, dropdown/enter, swap/x and scrolling regressions |
| 2026-10-11 | minor | flick panning done: releasing a moving hand drag (tool, space or middle button) glides and decays, gated by enable flick panning |
| 2026-10-11 | minor | history snapshots: the camera button takes one, snapshot rows return to it as a new state, rename and delete it (#2966) |
| 2026-10-14 | minor | history: a clicked snapshot stays selected and the trash deletes it; snapshot and state rows dragged onto the trash are deleted (#5324) |
| 2026-10-11 | minor | ⇧-drag a guide out of a ruler lands on the nearest ruler tick (#3770) |
| 2026-10-11 | minor | layer and selection keys from photoshop's shortcut set: ⌘, toggles, ⌘\, /, ⌘⌥2, ⌘⌥⌫, tab rename-next, ⌥-click thumbnail to fit |
| 2026-10-11 | minor | tool keys from photoshop's shortcut set: ← / → rotate the brush tip, ⇧-click for new type, space+f canvas colour |
| 2026-10-11 | minor | spring-loaded tool letters done (#2672): holding any tool letter (z included) past 300 ms, or using the canvas while it is down, makes the tool temporary until release |
| 2026-10-11 | minor | type formatting keys done: ⇧⌘k, u, b, i and /, ⌥↑/↓, ⇧⌥↑/↓, ⌥←/→ over a selection and ⌥⇧⌘h format the selected type while editing, and no longer reach color settings, desaturate, auto color or inverse |
| 2026-10-12 | minor | pen auto add/delete on the active path, with its options-bar checkbox, hover feedback and + / − cursor badge (#4144) |
| 2026-10-12 | minor | toolbar tool shortcuts can be reassigned, removed and reset in keyboard shortcuts (#4086); capture, restart, group cycling, conflict and text-focus regressions |
| 2026-10-12 | minor | smart filter rows can be selected, and dragged (⌥: copied) onto another smart object or onto delete (#4271) |
| 2026-10-12 | minor | free transform options bar: h and v skew fields; a sheared box reads as skew instead of a taller h |
| 2026-10-12 | minor | crop tool fill › content-aware fill done (#3823); delete cropped pixels off split into its own row, still partial |
| 2026-10-12 | minor | layers panel: clip thumbnails to layer bounds frames each layer's pixels instead of the canvas (#4281) |
| 2026-10-12 | minor | guides: ⌥-click turns a guide through the click; double-click opens preferences › guides, grid & slices |
| 2026-10-13 | minor | smart filter rows can be dragged up or down their stack to reorder the filters |
| 2026-10-13 | minor | on-canvas corner radius widgets on live rectangles (TOOL-214-12) |
| 2026-10-13 | minor | curvature pen edits the active path: drag anchors, click a segment to add, double-click smooth/corner, cursor badges |
| 2026-10-11 | minor | pen ⇧ 45° done: ⇧-click puts the next anchor on the nearest 45° ray, ⇧-drag turns its handles in 45° steps (TOOL-214-10 done, #3751) |
| 2026-10-14 | minor | last selected tool restored across launches; temporary overrides excluded, explicit eraser choices retained, and final selection flushed on normal exit |
| 2026-10-13 | minor | free transform with a selection: the box is the selection's rectangle, and the marching ants follow it while it is transformed (#5297) |
