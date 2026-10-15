# shape stroke options

updated 2026-10-11: direct editing, compact layout, and clickable colour/style controls.

brahma's shape tool options bar has a stroke options button beside the existing width
and colour controls. it sets defaults for new rectangle, ellipse, triangle, polygon, line
and custom shape layers. pen in shape mode uses the same style defaults; open pen shapes retain
their existing foreground colour and minimum 1 px width. to edit an existing shape,
use properties → appearance → stroke options.

fill and stroke swatches are clickable and open the existing compact colour picker. choosing
a colour preserves its alpha; no color removes the corresponding paint. tool-bar fill uses
the foreground colour and stroke uses the background colour, as before. choosing a stroke
colour or style deliberately enables a 3 px stroke when no stroke is present. the style button
remains available with no stroke, but opening or dismissing it alone does not enable one.
properties edits the selected shape; the options bar continues to set defaults for new shapes.

the editor offers solid, dashed (4 widths on, 2 off), dotted (zero-length dashes with round
caps, spaced every 2 widths), and custom repeating dash/gap sequences. dash lengths and
offsets are multiples of the stroke width; width itself is in document pixels. odd-length
imported sequences repeat twice, following the existing renderer convention. up to 16
dash/gap pairs can be edited, with add pair / remove pair. zero-length dashes draw dots with
round caps and squares with square caps; butt caps draw nothing at a zero-length dash.
each preset's full row, including its line sample, is one clickable target with a visible
selected state and an accessible label.

caps are butt / flat, round, and square. they affect open endpoints and individual dashed
segments. closed solid shapes have no endpoints, so their cap control is disabled. corners
are miter, round, or bevel; miter limit is a ratio, 1–500 (the existing default remains 100).
sharp miters exceeding the limit become bevels. ellipse has no sharp corners, so its corner
controls are disabled. the line tool retains its existing filled-bar geometry: strokes follow
the bar's closed outline. use an open pen shape for a centerline with endpoint caps.

inside / center / outside alignment applies to paths whose subpaths are all closed. inside
and outside clip a double-width stroke against the shape's fill area and respect the existing
fill rule and component operations. open and mixed open/closed paths use center; their
alignment control is disabled. dash lengths continue to use the specified width, including
for inside and outside. opacity is independent of width and paint.

stroke options applies valid changes immediately and has no apply or cancel buttons.
properties sends changes through `shape.edit`; one continuous numeric drag is one undo step.
escape or a click outside closes the popup and retains already applied changes. opening or
closing it without changing a value creates no history entry. invalid pending numeric values
do not replace the last valid style. child dropdowns can open and select an option without
dismissing stroke options; escape closes the child before the parent. the popup fits its
content and scrolls only when the available height requires it.

the compact line and rectangle previews use the real vector renderer. tool-default changes
do not edit the document. shape drag previews use the vector renderer, with a cached screen
proxy capped at 2048 pixels on the longest axis; like the prior drag overlay, their colours
bypass monitor-profile conversion. committed shapes, including live properties edits, use
the normal colour-managed compositor.

finite values are required: width 0–1,000,000 px, opacity 0–100%, dash/gap lengths
0–10,000 widths, offset ±1,000,000 widths, at most 32 lengths, and a nonempty pattern total
of at least 0.01 widths. interactive commands reject geometry beyond ±1,000,000 px,
over 4096 knots/subpaths, more than 100,000 tessellation points, or a dash workload over
100,000 steps. file-supplied invalid/dense patterns are bounded during rendering. there are
no custom saved preset libraries in this change. style defaults use existing serde ui state,
but the app does not save and restore these defaults across restarts.

all fields already exist in `ShapeStroke` and the native manifest/conversion. old `.pcraft`
files retain their serde defaults; saves preserve all style fields. PSD import/export maps
the same fields through `vstk` (line width, dash set/offset, cap, join, alignment, miter limit,
opacity, and paint); existing unknown layer blocks and original compatible vector records
remain handled by the existing i/o path. stroke overprint, non-normal PSD vector-stroke
blend modes, corner-fitting dash optimization, and photoshop's saved stroke preset libraries
are not implemented. no photoshop-authored oracle establishes exact stroke ui parity here.

automation: `shape.create`, `shape.edit`, and `shape.info` expose document strokes. the
control channel also supports `ui.set {shapeStroke: {width, align, cap, join, miterLimit,
dashes, dashOffset, opacity}}` for tool defaults, merging partial fields; null sets width to
zero. set tool colours with `tools.setColors`. `ui.inspect` reports `toolOptions.shape_stroke`,
`toolOptions.stroke_width`, and the transient `strokeEditor`. keyboard focus, standard number
fields, colour pickers, and ComboBoxes remain native egui controls.

## upstream and public references

- [pr #2218](https://github.com/shep95/brahma/pull/2218) introduced these controls with
  staged previews. the direct-edit interaction above replaces that apply/cancel workflow.
- [issue #2496](https://github.com/shep95/brahma/issues/2496) tracks broader shape tool
  parity, including selected-shape editing from the options bar and additional fill types;
  those remain separate work.
- [issue #1053](https://github.com/shep95/brahma/issues/1053) documents the missing
  stroke controls and existing engine/model support. gradient fill ui remains separate work.
- [issue #1931](https://github.com/shep95/brahma/issues/1931) requests sharp stroke options.
- [pr #233](https://github.com/shep95/brahma/pull/233),
  [#397](https://github.com/shep95/brahma/pull/397), and
  [#1495](https://github.com/shep95/brahma/pull/1495) already address stroke compositing,
  hairlines, and depth preservation; their implementations are reused.
- [pr #2031](https://github.com/shep95/brahma/pull/2031) supplies the click-to-create
  dialogs. both dialogs and drag creation use its shared `create_shape` helper, which receives
  the complete stroke defaults. a gesture regression checks both paths and their undo steps.
- adobe's photoshop shape fill/stroke guide
  describes the tool options and properties entry points, and its
  cap/join reference
  defines endpoint and corner behavior and the 1–500 miter range. dash units reuse
  brahma's existing documented `ShapeStroke`/`vstk` convention.
