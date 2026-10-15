# brush and painting parity

> **last reviewed:** 2026-10-10 · **last updated:** 2026-10-14 · **change:** minor (dragged preset tip preview; hue jitter amount track; brush settings minimum fields follow their control; checkbox-only rows and non-destructive smoothing switch; shaped-tip cursor follows the view; color replacement is a live toolbar tool; no brush-feel estimate changed) · **target:** adobe photoshop 2026

painting is where photoshop users notice differences first, so it gets its own checklist: the
brush engine (`crates/paint/src/brush.rs`), the brush settings ui
(`crates/ui-egui/src/brush_sections.rs`), the live stroke renderer, pen input
([`hardware-parity.md`](hardware-parity.md)), and the brush-based tools. status words as in
[`ui-parity.md`](ui-parity.md).

**summary (estimated):** breadth ~90% (every brush settings section and option exists), ready for
real work ~55%. the engine models what photoshop models; the gaps are feel (latency, pressure
response, edge quality, smoothing), a few missing options, and live previews for retouch tools.
remaining: 100–180 h (gap g4 in [`gaps.md`](gaps.md)). human: feel must be checked on real
tablets against photoshop side by side.

## brush settings sections

| section | status | evidence |
|---|---|---|
| brush tip shape: size, angle, roundness, hardness, spacing, flip x/y | done | `brush.rs` |
| shape dynamics (size/angle/roundness jitter, pen pressure/tilt/rotation/wheel controls, minimums, flip jitter, brush projection) | done | #174 |
| scattering (both axes, count, count jitter) | done | |
| texture (pattern, scale, brightness, contrast, mode, depth, each tip) | done | |
| dual brush | done | |
| color dynamics (fg/bg, hue/sat/brightness jitter, purity, per tip) | done | |
| hue jitter uses a standard percentage-amount track instead of a hue spectrum | done | #3825 (hue jitter appearance only); rendered track checked in all 14 themes; numeric and pointer edits retain percentage units and engine command routing. |
| transfer (opacity/flow/wetness/mix jitter and controls) | done | |
| shape dynamics and transfer minimum fields disabled when their control is off, retaining their values | done | #3825 (minimum-field interaction only); `brush_sections.rs` ui regressions cover all six fields, reactivation and disabled input |
| brush pose (tilt, rotation, pressure overrides) | done | |
| noise, wet edges, build-up, protect texture | done | |
| smoothing: amount, pulled string, stroke catch-up, catch-up on end, adjust for zoom | done | `brush.rs:428-438` |
| section locks | done | #174 |
| noise, wet edges, build-up, smoothing and protect texture: checkbox-only rows keep the selected controls open | done | `brush_panel_tests.rs`, #4283; smoothing enable/disable preserves its amount and options, including imported ABR tool settings |
| brush tip from an image file | missing | TOOL-213-12 |
| erodible, airbrush and bristle tips (photoshop's special tips) | partial | not audited in detail; "3d-based brushes don't work" (#1660) |
| anti-aliasing off for brushes | missing | #1666 (pencil is the aliased path) |

## pressure and pen

| behaviour | status | evidence |
|---|---|---|
| pressure drives size/opacity/flow per sample (no shared pressure within a frame) | done | #2112 |
| global pressure curve in preferences | done | #2195 |
| per-control pressure curves | missing | TOOL-213-11 |
| quick taps keep their pressure | partial | #1967 fixed one case; tap paints a full blob (#1798), tap sensitivity (#1933) |
| smooth pressure on windows | partial | "jumpy" (#2046) |
| tilt and rotation drive angle/roundness | partial | macos, X11, web; not windows; rotation not applied (#1301) |
| eraser end of the pen switches to eraser | partial | macos; flipped wacom on windows not (#2285) |

## stroke quality and feel

| behaviour | status | evidence |
|---|---|---|
| no seams, hard edges stay hard, zero-scatter defaults, ABR tips never black | done | TOOL-213-13 |
| smooth line weight along a pressure ramp | partial | stair-steps (#1776) |
| interpolation and smoothing match photoshop | partial | #2138, #2057 |
| soft-edge falloff matches photoshop's curve | partial | #2149, #1619 |
| brush preview and cursor crisp, not pixelated | partial | #1433 |
| cursor outline equals the painted footprint at every zoom | partial | #1761, #2364; shaped tips trace their outline (#3467), turned and flipped with a rotated or flipped view |
| latency: dab to screen within a frame | partial | budgets P13/P16/P33 over (scorecard); big-brush lag #2619, #1994; cursor lag #2566, #1343 |
| no hitch at mouse-up on a long stroke | not measurable | P32 needs a frame harness |
| live stroke for brush and eraser | done | |
| live stroke for every brush-based retouch tool | missing | TOOL-213-7 |
| no os arrow cursor over the canvas with a pen | partial | TOOL-213-14 |

## gestures and cursors

| behaviour | status | evidence |
|---|---|---|
| ⌃⌥-drag (macos) / alt+right-drag (windows) resizes and changes hardness with a readout | done | TOOL-213-10 |
| `[` `]` size, `⇧[` `⇧]` hardness, number keys opacity, ⇧+numbers flow | done | 2026-10-07 |
| ⌥-click samples a colour with painting tools; eyedropper comparison ring | done | TOOL-213-9 |
| ⇧-click draws a straight line from the last point | done | |
| hold `~` with the brush or pencil to erase with its brush: clear mode while held (rebindable under tools › temporary) | done | `hold_keys.rs`, #3439 |
| clear painting mode in the brush and pencil mode menus | done | `paint_mode_dropdown` in `panels.rs`, `"mode": "clear"`, #3439 |
| right-click opens the brush preset picker | done | [`context-menu-parity.md`](context-menu-parity.md); xp-pen right-click not (#1980), macos issue #2360 |
| precise / normal / full-size tip cursors, crosshair in the tip | partial | caps lock precise cursor not audited; round pencil cursor requested (#2662, #1057) |
| brush choice kept per tool (brush, eraser, blur each their own) | partial | #2542 |

## symmetry

| mode | status |
|---|---|
| vertical, horizontal, dual axis, diagonal | done |
| from a path (make symmetry path) | done |
| wavy, circle, spiral, parallel lines, radial, mandala | missing (#2659) |

## brush-based tools

| tool | status | notes |
|---|---|---|
| brush, pencil (aliased, auto erase), eraser (brush/pencil/block) | done | pencil 1 px quality #2139 |
| mixer brush (wet, load, mix, flow, sample all layers) | partial | "incomplete" (#2247) |
| clone stamp (opacity, flow, aligned, sample, overlay) | partial | clone source w/h link and alt target cursor (TOOL-213-8) |
| pattern stamp (aligned, impressionist) | done | |
| history brush | done | art history brush missing |
| spot healing, healing | partial | grain not matched on flat colour (#2104), "horrible in general" (#1092), stops responding after switching (#1751) |
| blur, sharpen, smudge (premultiplied, sample all layers, spacing) | done | BUG-207-2/3, #2171 |
| dodge, burn, sponge | done | |
| background eraser, magic eraser | done | |
| color replacement | done | |
| quick selection (brush-driven) | done | |

## presets

| behaviour | status | evidence |
|---|---|---|
| .abr import (v1/v2/v6) and export, persistent presets, folders | done | `psd/src/abr.rs` |
| brushes panel: drag a folder into a folder | missing | #2014 |
| dragged brush preset preview | done | cached, translucent tip thumbnail plus name in panel list/grid and the preset picker; `ui-egui/src/brushes_tab.rs`, real pointer-drag test in `brushes_tab_tests.rs` (#4225, partial) |
| tool presets (.tpl) | partial | tool presets panel exists; .tpl import not found |
| import 500 brushes without blocking the ui | not run | P24 |

## revision history

| date | change | summary |
|---|---|---|
| 2026-10-13 | minor | stale "missing tool" row corrected against main: color replacement (`paint.colorReplacement`) is a live toolbar tool; no brush-feel estimate changed |
| 2026-10-10 | major | first version, from a source audit and the open issues |
| 2026-10-11 | minor | `~` held with the brush or pencil erases with that tool's brush, and clear joins their mode menus (#3439) |
| 2026-10-12 | minor | five brush settings switch rows toggle without selecting a properties pane; smoothing has a default-on switch and retains its settings while disabled (#4283). no brush-feel estimate changed. |
| 2026-10-12 | minor | a shaped tip's cursor outline turns and flips with a rotated or flipped view, and follows pixel aspect correction |
| 2026-10-13 | minor | shape dynamics and transfer minimum inputs follow their control and retain their values while disabled (#3825, partial). no brush-feel estimate changed. |
| 2026-10-13 | minor | hue jitter uses the same themed amount slider as other jitter controls (#3825, partial). no brush-feel estimate changed. |
| 2026-10-13 | minor | dragging a brush preset shows its cached tip thumbnail and name in the brushes panel and preset picker (#4225, partial). no brush-feel estimate changed. |
