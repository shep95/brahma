# context-menu parity contract

> **last reviewed:** 2026-10-13 · **last updated:** 2026-10-13 · **change:** minor (layer-mask thumbnail menu, #4575) · **target:** adobe photoshop 2026

status: implementation contract, updated 2026-10-09. scope: canvas, document tabs, and panels. “right-click” means a secondary pointer click; macos control-click should also open the same menu where the platform sends a context-menu gesture. photoshop changes a context menu with the active tool, selection, and panel item. adobe documents that principle but does not publish a complete current menu catalog. **documented** means an adobe source explicitly names the gesture or action. **inferred** means a brahma design target or an item known from a main menu whose presence in a photoshop context menu still needs a current-product capture. do not present inferred rows as exact photoshop menus. adobe: panels and menus, adobe: shortcuts and context menus.

## dispatch contract

1. hit-test the target before opening: modal operation/handle, text selection, canvas content, panel row or icon, tab, then empty workspace. a generic canvas menu must not swallow a more specific target.
2. snapshot the target and selection when the menu opens. a row in an existing multiselection acts on that set; a row outside it becomes the target before its command runs. apply the same rule to layers, channels, paths, and presets.
3. resolve items from a pure menu model keyed by target kind, active tool, document state, platform, and active operation. the model supplies label, command id, params, enabled state, and reason. every mutating item dispatches an engine command; view-only items may update shell state. an unavailable feature is disabled with a reason or omitted, never a clickable no-op.
4. preserve primary-button tool behavior. secondary click only opens a menu unless a right-button drag gesture or the user's explicit right-erase preference owns the gesture. a drag must not briefly open a menu, and opening a menu must never paint, transform, sample, or commit.
5. anchor at the pointer and clamp to the viewport; support keyboard context-menu key/shift+F10 where available, escape/outside click, separators only between nonempty groups, checked/radio state, keyboard navigation, and screen-reader labels.
6. use the same command id and enablement as the menu bar/options bar. provide ui inspection data and control-channel operations to select a menu item without pixel coordinates, while retaining real pointer tests and screenshots.
7. test each menu model independently for conditional rows and wrong-state safety; test actual secondary-click routing, menu selection, undo/redo, and no accidental primary action.

## canvas tool matrix

the “brahma at branch point” column records behavior before this pr. picker means the brush preset quick picker in paint_mouse.rs; layers means the hit-tested layer list in layer_pick_ui.rs. command/ctrl + secondary-click opens the layer list with every tool, and move opens it without a modifier. exact photoshop secondary-click items beyond adobe-documented rows are **proposed** until captured from current photoshop.

| tool(s) in brahma | photoshop / desired contextual action | brahma at branch point | priority, evidence |
|---|---|---|---|
| move | choose a layer at the point; an auto-select choice also exists in the options bar | layers | p0; adobe layer movement; exact pop-up layout requires capture |
| rectangular/elliptical marquee, lasso, polygonal lasso, magnetic lasso, magic wand, quick selection, object selection | active selection: the 2026 selection menu (see *selection canvas reference* below); no selection: making a selection | quick selection: picker (a brush tool, as in photoshop); the other six: the selection menu below (#614, #738) | p1; no-selection variant still needs a dated capture |
| crop | during crop: ratio/preset, overlay, clear/reset, commit/cancel **(inferred context placement)** | no tool menu | p1; pending crop target |
| eyedropper | sample-size/source choices **(inferred)** | no tool menu | p2; adobe illustration uses eyedropper, but does not publish item text |
| ruler, note, count | unit / measurement, note edit/delete, count-group operations **(inferred)** | no tool menu | p2 |
| brush, pencil, eraser, background eraser, spot healing, healing, clone stamp, history brush, blur, sharpen, smudge, dodge, burn, sponge | brush preset, size and hardness where applicable | picker | p0; adobe brush settings. alt+secondary-drag on windows and control+option-drag on macos adjust size/hardness (adobe cursor shortcuts) |
| gradient, paint bucket | gradient preset/type/mode or fill source/tolerance **(inferred context placement)** | no tool menu | p1 |
| type | in text: cut/copy/paste, spelling suggestions, text formatting; outside text: type-layer action **(inferred context placement)** | while editing text (canvas_tool_menu.rs `TYPE_EDIT_MENU`): undo, cut, copy, paste, select all (on the text), check spelling…, find and replace text…, faux bold/italic (checked), rasterize type, create work path, convert to shape, horizontal/vertical and the anti-alias modes (checked), convert to paragraph/point text, warp text…; esc closes only the menu. over a misspelled word the menu starts with up to 3 spelling suggestions (replace the word) and add to dictionary (#4370); ignore / ignore all and the non-editing type-layer menu remain open | p1; separate editing and nonediting states; adobe type editing |
| hand, zoom | fit on screen, 100%, 200%, print size; zoom step choices **(inferred context placement)** | view menu (#3242): zoom in, zoom out, fit on screen, fill screen, 100%, 200%, print size. zoom in/out step about the right-clicked point (preferences › tools › zoom clicked point to center centres it). outside the image the pasteboard menu still wins. | p1; view actions stay out of document history; row order still needs a dated photoshop capture |
| pen, path selection | path/point actions, make selection, fill/stroke path, free transform path **(inferred context placement)** | pen canvas right-click follows the user-provided 20-row photoshop capture below. paths rows offer make selection, fill and stroke. path selection's own canvas menu remains open. | p1; distinguish node, segment, path, and blank canvas |
| rectangle, ellipse shape, triangle, polygon, line, custom shape | shape/path operations, fill/stroke settings, transform **(inferred context placement)** | no tool menu | p2 |
| slice, slice select | slice properties, duplicate/delete, divide, guides **(inferred context placement)** | no tool menu | p2 |
| any tool while free transform is active | scale, rotate, skew, distort, perspective, warp, rotate/flip variants, commit/cancel **(inferred exact context placement)** | missing per docs/scorecard.md TOOL-214-5 | p0; transform operations documented by adobe; type restrictions documented here |
| any tool with command/ctrl + secondary click | list visible layers with pixels under pointer | layers | p0; preserve existing route |

the layer-list menu on an empty pixel or all-hidden hit should be empty/closed; it must not select a transparent or hidden layer. the right-erase preference is an alternate brahma behavior, and its selected state takes precedence over the brush picker only for brush/eraser.

## pen canvas reference capture (2026-10-07)

the user supplied a photoshop screenshot of a closed path in path mode. the table preserves every row in its order; separator boundaries appear between the groups. gray rows in that capture remain visible but disabled for a plain work path. enablement changes with the active shape layer, saved path, copied style, and symmetry state. ellipsis rows open a parameter dialog before changing the document.

| group | rows in screenshot order | brahma action and applicability |
|---|---|---|
| path | create vector mask; delete path | `layer.vectorMask.fromPath` on an eligible layer; `path.delete` on a work/saved path |
| preset | define custom shape… | `edit.defineCustomShape` with a name dialog |
| selection/paint | make selection…; new guides from shape; fill path…; stroke path… | `path.toSelection` dialog; `view.newGuidesFromShape` for shape layers; `path.fill` and `path.stroke` dialogs on pixel layers |
| export | clipping path… | `path.clippingPath.set` on a saved path, with flatness parameter |
| geometry | free transform path | `path.transform` with translation, scale and rotation parameters |
| shape operation | unite shapes; subtract front shape; unite shapes at overlap; subtract shapes at overlap | the four `layer.combineShapes.*` operations on shape layers with multiple components |
| copy style | copy fill; copy complete stroke | `path.style.copyFill` / `copyStroke` when the shape has that style |
| paste style | paste fill; paste complete stroke | `path.style.pasteFill` / `pasteStroke` when a matching style has been copied |
| layer view | isolate layers | `select.isolateLayers` for an active layer |
| symmetry | make symmetry path; disable symmetry path | `paint.symmetryFromPath` on a path; `paint.symmetryDisable` when symmetry is active |

the reference does not establish photoshop's behavior for right-clicks on a single anchor, a segment, or empty canvas; these still need separate captures. brahma's disabled rows do not call a command.

## selection canvas reference (2026-10-08)

right-click with a marquee, lasso, polygonal lasso, magic wand or object selection tool. rows keep the command id, dialog and enablement of their menu-bar twins (`canvas_tool_menu::SELECTION_MENU`); a greyed row never runs.

| group | rows with an active selection (photoshop 2026 order) | brahma command |
|---|---|---|
| selection | deselect; select inverse; feather…; select and mask… | `select.deselect`, `select.inverse`, `select.modify.feather`, `select.selectAndMask` |
| store | save selection…; make work path… | `select.saveSelection`; `select.toWorkPath` after a tolerance dialog |
| layer | layer via copy; layer via cut; new layer… | `layer.new.layerViaCopy`, `layer.new.layerViaCut`, `layer.new.layer` |
| transform | free transform; transform selection; distort; perspective | `edit.freeTransform`, `select.transformSelection`, `edit.transform.distort`, `edit.transform.perspective` |
| fill | fill…; stroke…; content-aware fill… | `edit.fill`, `edit.stroke`, `edit.contentAwareFill` |
| filter | last filter | `filter.lastFilter` |
| fade | fade… | `edit.fade` |

photoshop's generative fill and delete and fill selection rows are left out: brahma has no such commands. with nothing selected the menu offers select all, reselect, color range… and load selection… (`NO_SELECTION_MENU`). adobe-adjacent sources document color range there; the other rows are **inferred** until a dated capture confirms them.

## panel and workspace matrix

| surface / target | desired options and state | brahma now | source and confidence |
|---|---|---|---|
| layers panel row | blending options, duplicate/delete, group, export, convert/rasterize, mask, clipping, link, style copy/paste/clear, merge/flatten, rename; filter by layer kind and selection | implemented in layer_menu_ui.rs; item enablement uses menu commands | brahma inventory. adobe documents layer rename/delete, color tagging, rasterization, and flattening here; exact combined ordering **inferred** without a live capture |
| layers panel color label / thumbnail / mask thumbnail / effect row | color label; context actions specific to hit icon | layer-mask thumbnail: disable/delete/apply layer mask, add/subtract/intersect mask with selection, select and mask… (layer_menu_ui.rs `show_mask`, #4575; mask options… missing). generic row on the other targets; inspect before extending | adobe explicitly documents right-click on layer/group to assign a color (layers); other icon menus need capture |
| channels panel row | duplicate/delete/rename, channel display and overlay color, spot conversion; new/split/merge | implemented in channels_panel.rs | brahma inventory; exact photoshop order **inferred** |
| brushes and preset panels | rename/delete brush or group, export selected brushes, view/reset preset choices | rename/delete, and preset group/item menus exist | adobe explicitly documents export selected brushes from brush/group right-click (brushes); photoshop pop-up-panel rename/delete/reset/view options are documented, though some are panel-button menus |
| paths, history, actions, swatches, gradients, patterns, styles, libraries, properties | target-specific duplicate/delete/rename and panel operations where a panel exists | paths row menu added in this pr; other panels need audit | **inferred** catalog; record a photoshop capture before claiming parity |
| document tab | close, close others, close all, reveal in file manager, duplicate/move to window where supported | close, close others, close all, reveal in finder, and drag to reorder implemented; duplicate/move to window where supported remains open | **inferred** exact list; p1 |
| empty pasteboard / workspace | gray, black, custom canvas color choices; verify any additional entries in a current photoshop capture | no menu | adobe documents right-clicking outside the image for canvas color; exact current layout still needs a capture |
| text field / selected text | native edit menu and spelling suggestions without routing to active image tool | edit menu implemented in text_field_menu.rs for every text field (undo, cut, copy, paste, delete, select all; a right-click inside the selection keeps it; never reaches the canvas or tool menu). spelling suggestions live in the type tool's edit menu (canvas_tool_menu.rs), not in generic fields, which hold no type layer text (ui-217-8) | p1; **inferred** exact spelling list |
| tool icon in options bar | reset tool, reset all tools | audit | explicitly documented in adobe photoshop reference; historical source, verify modern label |
| adaptive wide angle constraint line | auto, horizontal, vertical orientation | current right-click deletes constraint | p1 behavior mismatch: adobe documents orientation choice |

## delivery slices and acceptance

1. core routing: target hit-test, menu model, keyboard accessibility, control inspection, conflict rules. existing move layer list and painting picker remain functional.
2. canvas high frequency: selection, free transform, type, hand/zoom, gradient/paint bucket. add rows only where the underlying command works. record exact photoshop label/order/disabled-state captures before declaring 1:1 parity.
3. panels: paths row actions, document tabs and distinct layer eye/thumbnail/mask/effect targets; finish brushes export and observed channels/preset gaps. adobe documents the eye-icon visibility gesture.
4. long tail: shape/path, crop, metadata and specialist tools. keep a per-tool fixture with no selection, active selection, masked/locked layer, and active operation where applicable.

each slice needs pure model tests (including every item dispatching a known command), ui pointer tests for secondary click and modifier precedence, command/undo tests, a disabled-state test, and screenshots from brahma with the menu actually open. compare labels, order, and state against dated photoshop screenshots; a screenshot of a different tool or menu bar is insufficient evidence. maintain a checklist beside each captured reference: photoshop version/platform, target description, tool, selection/document state, modifiers, and exact items observed.

## revision history

| date | change | summary |
|---|---|---|
| 2026-10-13 | minor | layer-mask thumbnail menu (#4575) |
| 2026-10-10 | trivial | status line, revision history |
| 2026-10-09 | minor | selection canvas reference and tool matrix updates |
| 2026-10-07 | minor | pen canvas reference capture |
