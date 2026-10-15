# context-menu inventory and parity contract

> **last reviewed:** 2026-10-10 · **last updated:** 2026-10-10 · **change:** trivial (status line added; the inventory is the 2026-10-06 branch-point snapshot, current status lives in context-menu-parity.md) · **target:** adobe photoshop 2026

this is the source-code inventory of brahma's right-click behavior at the branch point, captured 2026-10-06. it distinguishes a menu from another secondary-button gesture. the initial selection, paths row, and document-tab menus added by this pr are called out separately. photoshop comparison should be checked against an identified photoshop version and platform before claiming exact parity; the current layers menu explicitly targets photoshop 2026 order.

## canvas: every tool

the shared rule is **command/ctrl + right-click** on opaque document pixels → a topmost-first list of visible layers at that point; choosing a row dispatches `layer.select`. the move tool gets that menu on plain right-click too. the list comes from `layer.pickAt` and is omitted over fully transparent pixels. on the remaining tools, plain right-click depends on the tool group below. the canvas behavior lives in `canvas.rs`, `paint_mouse.rs`, `layer_pick_ui.rs`, and `brush_resize.rs`.

| tools | plain right-click today | other secondary gesture | parity work |
|---|---|---|---|
| move | layer-under-pointer list (`layer.pickAt`, `layer.select`) | command/ctrl gives the same list | verify photoshop modifier and auto-select behavior; test overlaps, masks, groups, transformed pixels, empty areas. |
| brush, pencil, eraser, background eraser | brush preset picker (size, hardness, presets) | alt/option + right-drag changes size/hardness via `tools.setBrush`; brush/eraser can right-drag erase via `tools.rightClickWithPaintingTools=erase` | verify whether context should expose brush presets, settings, or tool-specific eraser controls. keep erase preference and modifier priority deterministic. |
| spot healing, healing, clone stamp, history brush, blur, sharpen, smudge, dodge, burn, sponge | brush preset picker | alt/option + right-drag changes size/hardness | compare each tool's photoshop picker and source/retouch options; use existing brush command route. |
| quick selection | brush preset picker | alt/option + right-drag changes size/hardness | check selection-tool brush options and whether mode controls belong in picker. |
| rectangular/elliptical marquee, lasso, polygonal lasso, magnetic lasso, magic wand, object selection | selection context menu: the full photoshop list with a selection, select all / reselect / color range / load selection without (`canvas_tool_menu.rs`, rows in `docs/context-menu-parity.md`) | command/ctrl opens layer list | confirm the no-selection variant against a dated photoshop capture. |
| crop, slice, slice select | no plain secondary action | command/ctrl opens layer list | define crop/straighten commit and cancel, aspect/preset, slice selection/edit behavior separately. |
| eyedropper, ruler, note, count | no plain secondary action | command/ctrl opens layer list | check tool-specific sampling, measurement, annotation, and count menus. |
| gradient, paint bucket | no plain secondary action | command/ctrl opens layer list | expose gradient preset and fill settings through commands if photoshop's context ui provides them; avoid painting on right-click. |
| type | no plain secondary action | command/ctrl opens layer list | test text edit versus non-edit context, spelling, type options, and selection behavior. |
| pen, path selection | no plain secondary action | command/ctrl opens layer list | high-priority path-point context contract: add/delete/convert point, close path, make selection, fill/stroke path, path operations when applicable. |
| rectangle, ellipse shape, triangle, polygon, line, custom shape | no plain secondary action | command/ctrl opens layer list | define shape versus path context, point edit, fill/stroke, and transform entries. |
| hand, zoom | view menu: zoom in, zoom out, fit on screen, fill screen, 100%, 200%, print size (`canvas_tool_menu.rs`, #3242) | command/ctrl opens layer list | confirm the row order against a dated photoshop capture. |

canvas invariant: secondary click must not accidentally start a paint stroke, commit a crop, or edit the document; every mutating menu action dispatches a command. escape/outside click closes a popup. existing `ui.pointer` supports a secondary button; ui tests should exercise both native and control-driven gestures.

## other right-click surfaces

| surface | current entries or behavior | command route / gap |
|---|---|---|
| layers panel row | blending options; duplicate/delete; group from layers; quick export PNG / export as; artboard/frame from layers; convert to smart object; content-specific rasterize; add/disable/apply/delete layer mask; create/release clipping mask; link/select linked; copy/paste/clear layer style; merge layers/down, merge visible, flatten image; rename layer. multi-select changes duplicate/delete/merge labels and target. | `layer_menu_ui::entries` supplies ids for all except inline rename, then `menus::invoke` or engine dispatch. missing or disabled commands are filtered/greyed. check menu visibility and enabled state for groups, backgrounds, masks, type, adjustment/fill, smart objects, and multi-selection. |
| channels panel row | alpha: duplicate/delete/rename, spot conversion or merge, selection-display and overlay controls. quick mask: exit, display and overlay. composite/color: duplicate. layer mask: enable/disable, delete. common: new channel, new spot channel, split, merge. | `channel.*`, `layer.layerMask.*`, `select.editInQuickMaskMode`, `ui.renameChannel`. check selection loading/saving, channel options, and per-row enablement against photoshop. |
| brushes panel preset | rename brush, delete brush. | routed through `brush.presets.*` panel actions. check duplicate/export/import and library organization. |
| brushes panel group | rename group, delete group. | routed through preset panel actions; confirm group deletion consequences. |
| generic presets group (gradients, patterns, swatches, etc.) | rename group, delete group. | `preset_panels.rs`; verify each collection's appropriate create, export/import, and delete semantics. |
| generic preset item | rename, delete, move to another group. | `preset_panels.rs`; behavior is generic across preset collections. |
| tool presets item | rename tool preset, delete tool preset. | `tool.presets.edit`; compare current-tool filtering and preset application. |
| toolbar grouped tool slot | right-click or long-press opens its tool flyout. | sets active `Tool` ui state. single-tool slots have no flyout. |
| toolbar screen mode | standard, full screen with menu bar, full screen. | `view.screenMode.*` via `menus::invoke`. |
| color chip in color panel | right-click sets background color (left-click sets foreground). | direct ui state behavior; verify foreground/background target commands. |
| wide angle constraint | right-click deletes that constraint. | tool-local action; no menu. |
| camera raw point curve | right-click deletes a curve point. | tool-local action; no menu. |
| document tab / loading tab | no context menu at branch point. this pr adds close, close others, close all for document tabs. left click activates; close icon closes/cancels. | existing guarded file commands handle unsaved documents; reveal file and tab layout remain open. |
| paths panel row | make selection, fill/stroke path, save work path or duplicate path, delete path or vector mask as applicable (added in this pr). | `path.*` and `layer.vectorMask.delete`; direct action on the clicked path. |
| history, properties and empty panel areas | no `context_menu` handler found in ui source at this audit. | audit each panel's photoshop menu and prioritize its useful commands; do not invent dead entries. |

## contract for implementation

1. record a photoshop reference matrix by version, operating system, surface, tool and state (empty/selection/path/text edit/multi-layer). capture exact item labels, order, separators, submenus, disabled conditions, and modifier variants. mark observations versus inference.
2. encode menu descriptors as data with predicate tests. route document edits through existing or new engine command ids; view-only actions can stay in ui. a menu on the canvas must receive the document coordinate and relevant target from hit testing, so actions operate on what was clicked rather than silently on an unrelated active object.
3. prioritize shared canvas menus, then high-value tool-specific interactions (path, selection, type, crop/zoom), then panel and tab gaps. reuse layers/channels implementations where their command contract already exists. preserve the existing move/layer pick, brush picker, erase preference, alt/option brush resize, and toolbar flyout gestures.
4. test each descriptor's item order and enablement, dispatch with expected params, empty/invalid states, modifiers, and action result. add ui gesture tests for right-click opening, target selection, escape/outside dismissal, and the absence of accidental edits. validate native and web builds.
5. capture screenshots from the running app for representative tool groups and panel states, with test fixture documents rather than personal photos. a screenshot proves visible menu composition; command tests prove its behavior.

## revision history

| date | change | summary |
|---|---|---|
| 2026-10-10 | trivial | status line, revision history; marked as the branch-point snapshot |
| 2026-10-06 | major | source inventory of right-click behaviour |
