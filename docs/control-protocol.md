# control protocol

shape stroke defaults are drivable with `ui.set {shapeStroke: {width, align, cap, join,
miterLimit, dashes, dashOffset, opacity}}`; partial fields merge, null disables the stroke.
values use the existing `shape.edit` names and units. colour uses `tools.setColors`.
`ui.inspect` exposes `toolOptions.shape_stroke`, `stroke_width`, and `strokeEditor`.
stroke options applies valid ui edits immediately; `strokeEditor` is the open editor state,
not an uncommitted document preview. escape/outside clicks close it without reverting edits.
see [shape stroke options](shape-strokes.md) for limits, preview and persistence behavior.

the desktop app listens on `127.0.0.1:<port>` (loopback only). start it with a token file so the credential is not exposed in the process command line:

```sh
brahma --control 7878 --control-token-file /private/path/brahma-control.token \
  --automation-read-root /work/project --automation-write-root /work/project
```

if the file does not exist, brahma creates it with a fresh 256-bit token. on unix the new file is mode `0600`; on windows, protect it with an appropriate user-only ACL. an existing file is reused. if neither a token nor token file is configured, brahma generates a token for that launch and writes it to standard error. `BRAHMA_CONTROL_TOKEN` and `BRAHMA_CONTROL_TOKEN_FILE` are the environment-variable equivalents.

the first request on every TCP connection must authenticate. no other method is dispatched before this succeeds:

```json
{"id": "auth", "method": "auth", "params": {"token": "<64 hexadecimal characters>"}}
```

authenticate once per connection. a later `auth` frame on the same connection gets `{"ok": false, "error": "already authenticated"}` and the connection keeps serving.

after the successful authentication reply, each request is one JSON line:

```json
{"id": 1, "method": "ui.inspect", "params": {}}
```

each reply is one JSON line with the same `id`:

```json
{"id": 1, "ok": true, "result": { ... }}
{"id": 2, "ok": false, "error": "unknown tool `foo`"}
```

the desktop server waits up to 60 seconds for a reply. a request still queued at that deadline is rejected before dispatch; a timeout does not cancel work that has already started. the two timeouts read differently, both starting with `timeout`: a request that was never dispatched (still queued at the deadline) says `timeout: not run` and is safe to re-send, while `timeout: no reply in 60s` means the command was dispatched and may still be running and still apply, so check the document before re-sending it. the timeout error names the likely causes: the command may still be running, or the request was queued while the window was not drawing frames — on wayland the frame update (and with it the control drain) only runs on compositor frame callbacks, so a display that is asleep or a fully occluded window makes every request time out while the app itself is healthy.

the transport is `apps/brahma/src/control_server.rs`, and the handlers are in `crates/ui-egui/src/control.rs`. the MCP server (`brahma-cli mcp --bridge 127.0.0.1:<port>`, crate `brahma-automation`) wraps this same protocol. see [MCP bridge](#mcp-bridge) below.

## methods

- `engine.execute {command, params}`: run any engine or ui command by id. engine commands run directly with their default params and never open a dialog. use `ui.menu.invoke` for menu-click behaviour, which opens a command's dialog when no params are given. `params` must be a JSON object (omit it or pass `null` for none); an array, string, number or boolean is an error naming the command, in every transport (control channel, `serve`, MCP and the CLI)
- `engine.commands`: list commands with enablement; the `result` is a JSON array of command objects (see [command list replies](#command-list-replies))
- `ui.inspect`: full ui state (tool, panels, views, dialogs, windows, window size). the menu tree is not included; use `ui.menu.list`. `brushPicker` is `null` while closed and `{pos: [x, y], size: [w, h]?, list: {collapsed, filter, renaming, showName, showStroke, showTip, scale}}` while the brush preset picker is open (coordinates in screen points; `size` is the picker's content size once its corner grip was dragged, `null` before that, and the drag remembers both dimensions; the three `show*` flags pick the parts of each preset's card — at least one is always on; `scale` is the footer slider, 0.15–2 with 1 the standard, multiplying the cards' width and height — the heights stop at half size and the name and tip-size text hold their 1.0 size — and dropping the tips' size numbers at 0.40 and below; the parts and the scale are remembered across restarts, while the search, collapsed groups and a rename in progress are not). `view` holds the view/window/type preferences: `screen_mode`, `extras`, `show` and `snap_to` flags, `flip_horizontal`, `arrange` (window › arrange layout), pixel aspect, font preview size, language options. `perf.timings.gpuInfo` holds the graphics adapter, backend, driver, the backend chosen at launch and why, the canvas renderer (`gpu`/`cpu`) and, after a device loss, `lost` (`help.systemInfo` returns the same as `info`)
- `ui.set {tool?, panels?, dockTabs?, dock?, dockWidth?, colorPanel?, maskTarget?, vectorMaskTarget?, selectionMode?, zoom?, center?, rotation?, fit?, theme?, brushSize?, brushSection?, brushTab?, brushesView?, brushPicker?, brushPickerName?, brushPickerStroke?, brushPickerTip?, brushPickerScale?, gradientBlendMode?, gradientClassic?, shapeStroke?}`: change ui state; any other field is an error, checked before anything changes (`zoom` is the canvas zoom where 1 = 100%, one document pixel per physical display pixel, so a scaled display doesn't magnify the image; `theme` is `pro`, `proMedium`, `studio`, `studioLight` or `classic`; `selectionMode` is the selection tools' options-bar mode, 0 new, 1 add, 2 subtract, 3 intersect; `brushSection` indexes the brush settings sections, `brushTab` 0 = brush settings, 1 = brushes; `brushesView` is `list` or `grid` for the brushes panel; `brushPickerName`, `brushPickerStroke` and `brushPickerTip` are the brush preset picker's card parts (booleans, at least one on), and `brushPickerScale` its footer card-scale slider (0.15 to 2, 1 the standard); `brushPicker` is `[x, y]` in screen points to open the brush preset picker there (as a right-click with a painting tool does) or `null` to close it, and picking a preset in it is `tools.setBrush {"preset": name}`; `dock` is `{order: ["layers", …], heights: {"properties": 180}, collapsed: ["color"], tabOrder: {"layers": ["Paths", "Layers", "Channels"]}}`, the right-dock groups top to bottom, their heights in points, the groups collapsed to their tab strip and each group's tab order left to right by panel name (a group left out keeps its default order, unknown names are ignored and tabs not listed follow in their default order; `dockTabs` keeps indexing tabs in their default order); `dockWidth` sets the right dock width in points, clamped to 250..520; `colorPanel` is `{background}`, whether the color panel edits the background colour; `maskTarget` targets the active layer's mask, which, like photoshop, swaps in the foreground/background pair kept for masks (white/black until one is picked) and swaps the pixels' pair back when the pixels are targeted again)
  - `typeTransform` is `null` outside a temporary ctrl/cmd type-tool drag. during a drag it reports its source document/revision, original affine, oriented `frame` (layer, local bounds, document-space corners and pivot), and captured `gesture`. drive it with `ui.pointer` and `command: true` at `down`; `move`/`up` may release the modifier. the document/history remain unchanged during preview, and `up` applies one `type.edit` within `textEdit`'s session. pressing escape during the drag cancels only the preview. this pointer state is never restored from saved ui state.
- `ui.menu.invoke {id, params?, wait?}` / `ui.menu.list`: activate a menu item by id; list the menu tree. a menu item that starts a background job (a filter without a dialog, such as blur more) replies with the job's result once it has been applied; with `"wait": false` the reply is `{job, pending: true}` at once. window ends with the open documents as `window.document.<index>` (tab order, the active one checked, #2991); invoking one brings that document to the front and replies `{document: index}`
- `ui.set` also accepts `gradientBlendMode` (a layer blend mode name, such as `Difference`) and `gradientClassic` (boolean) for the gradient tool options bar.
- `ui.set` also accepts `camTransformOnDrop` (boolean), the content-aware move tool's transform on drop checkbox: a drop then puts a scale/rotate box on the patch (`ui.inspect` › `camTransform`: its document/revision, drag `offset` and `frame`), driven with `ui.pointer` (handles as free transform's; outside the box rotates), committed with `ui.key Enter` (one `paint.contentAwareMove` with `scaleX`, `scaleY`, `rotate`) or dropped with `Escape`.
- `ui.set` also accepts `tileGrid` (boolean), a developer overlay of the active layer's storage tiles: the tile grid over the document, every stored tile filled and labelled `tx,ty`, tiles with identical pixels in a shared colour and unique ones in blue. it has no menu item and view › extras doesn't hide it. `ui.inspect` reports it as `view.tile_grid`.
- `ui.set` also accepts `layerFilter` (#5048), the layers panel filter row: `{by?, kinds?, name?, mode?}`, any subset. `by` is `kind`, `name` or `mode` (the filter-type dropdown); `kinds` lists the kind toggles that are on (`pixel`, `adjustment`, `type`, `shape`, `smart`); `name` matches part of a layer name, ignoring case; `mode` is a blend mode name (`Multiply`, `Pass Through`). kind and name filter only once a kind is on or text is typed; mode always filters, by normal at first, as in photoshop. fields left out keep their value; a bad one is an error and changes nothing.
- each painting tool keeps its own brush, as in photoshop: `tools.setBrush` and `ui.set {brushSize}` change the active tool's brush, and switching tools swaps in that tool's own (size, hardness, mode, opacity, flow, dynamics, smoothing). a tool used for the first time starts from the brush in hand. the colours, the erase flag and the stroke seed stay shared. so `ui.set {tool: "eraser", brushSize: 12}` sets the eraser's size, and `ui.set {tool: "brush"}` afterwards brings the brush's own size back.
- `ui.set` also accepts the eyedropper options bar (#1649): `eyedropperSampleSize` (`"point"` or 1, 3, 5, 11, 31, 51, 101: the side of the averaged square, shared by the eyedropper, a painting tool's alt-click eyedropper, the curves/color picker eyedroppers and the info panel), `eyedropperSample` (`current`, `currentAndBelow`, `all`, `allNoAdjustments`, `currentAndBelowNoAdjustments`) and `eyedropperRing` (boolean, show sampling ring). `ui.inspect` reports them under `toolOptions` (`eyedropper_size`, `eyedropper_sample`, `eyedropper_ring`). the engine command `document.sampleColor {x, y, size, sampleLayer}` returns the colour such a sample picks.
- `ui.set` also accepts the crop tool's overlay menu (#1919): `cropOverlay` (`thirds`, `grid`, `diagonal`, `triangle`, `goldenRatio`, `goldenSpiral`), `cropOverlayShow` (`auto`: only while the crop box is dragged, photoshop's default; `always`; `never`) and `cropOverlayOrientation` (0..3; triangle has two orientations and golden spiral four, the others ignore it). `ui.inspect` reports them under `toolOptions` (`crop_overlay`, `crop_overlay_show`, `crop_overlay_orientation`). in the app, o cycles the overlay and ⇧o its orientation while the crop tool shows a box.
- `ui.set` also accepts `cropFill` (`background`, photoshop's background (default), or `contentAware`), the crop tool's fill dropdown (#3823): with `contentAware`, committing a turned or extending frame passes `contentAware: true` to `image.crop`, which fills the area the image doesn't cover from its surroundings. `ui.inspect` reports it as `toolOptions.crop_content_aware`.
  both gradient modes use the current preset, including its opacity stops. select `Foreground to Transparent` through `gradient.presets.select`, or pass custom `stops` and `transparency: [[0,100],[1,0]]` (location 0..1, opacity 0..100). use `applyToLayer: false` to edit only the tool preset. classic gradient drags dispatch `paint.gradient` without overriding them.
- `ui.set` also accepts the crop tool's gear menu (#1919) as `cropShield`, an object patching any of `show_cropped_area` (show cropped area; off hides everything outside the box behind the canvas colour, h in the app), `enabled` (enable crop shield), `color` (`matchCanvas`, the pasteboard colour, or `custom`), `custom_color` (`[r, g, b]`, 0..255), `opacity` (0..100, default 75), `auto_adjust` (auto adjust opacity: half the opacity while the box is dragged), `classic_mode` (use classic mode, default off: the box turns and moves over the image; off is photoshop's default mode, where the box stays upright and the view turns and pans the image behind it while the crop tool shows a frame, p in the app; in both modes picking the tool turns a rotated view upright about its centre, classic mode keeping the camera upright with the frame drawn turned, and the view stays upright after a commit or cancel; classic mode greys out show cropped area, which then acts as on, and auto center preview) and `auto_center_preview` (default on: in default mode a box being resized keeps its centre where it is on screen, also after a pan, a drawn one moves to the middle of the canvas, and x re-centres it; a move never re-centres the box). `ui.inspect` reports it as `toolOptions.crop_shield`. the pending frame (`crop_rect`, `crop_angle`: its turn on the image) and the `image.crop` a commit runs are the same in both modes.
- `ui.dialog.open {kind, fields?}` (kinds `newDocument`, `about`, `layerStyle {effect?}`, `colorPicker {target: foreground|background}`, `command {command}`, `newCropPreset` / `deleteCropPreset`: the crop tool's new / delete crop preset… dialogs, fields `name` and `selected: [names]`; crop presets are `crop` tool presets, so `tool.presets.new/list/edit {"tool":"crop"}` drive them too) / `ui.dialog.set {dialog, field, value}` / `ui.dialog.confirm {dialog, wait?}` / `ui.dialog.cancel {dialog}`. like `ui.menu.invoke`, `ui.dialog.confirm` waits for a background job its command starts (a filter dialog's ok) and replies with the result; `"wait": false` replies `{job, pending: true}` at once
  - a layer style color field opens the shared picker with `ui.dialog.open {"kind":"colorPicker","parent":12,"effect":"fx2","field":"color"}`. `parent` is the layer style dialog id; `effect` is its effect instance id. `field` must be one of that instance's color parameters (`color`, or `from`/`to` for gradient overlay). the picker previews over the pending style without changing the document. picker ok changes only the parent parameter; cancel discards the preview. confirm the parent afterwards to apply the style. parent confirmation is refused while its picker is open; parent cancellation closes both. the binding is exposed in `ui.inspect` as `__layerStyleColor`; changed/closed targets or a document switch invalidate it.
  - `ui.dialog.open {kind:"colorPicker", textStyle:{kind:"character"|"paragraph", id:u32}}` opens the shared color picker for an explicitly enabled color attribute in style options. it opens the matching style panel/editor after validation; inherited colors must first be enabled through the existing style command. the picker reports its bound document/revision/style under `__textStyleColor` in `ui.inspect`. changes preview in an isolated document; ok runs the existing style `set` command once, cancel changes nothing. document edits/switches or closed/changed style options discard the picker. `textStyle` cannot be combined with `parent`, `effect`, `field` or `target`.
  - the shell's own dialogs (window › workspace › new workspace / delete workspace, view › pixel aspect ratio › custom pixel aspect ratio…, view › show › show extras options…, view › 32-bit preview options…) are listed in `ui.inspect`'s `dialogs` with the id `"shell"` (at most one is open). `ui.dialog.set`, `ui.dialog.confirm` and `ui.dialog.cancel` take `{"dialog": "shell"}`; ok runs the dialog's command and keeps the dialog open if it fails
  - the unsaved-changes prompt (closing a document, revert, quitting with unsaved work) is listed in `ui.inspect`'s `dialogs` with the id `"discard"` while it is up: `fields` gives the waiting `command` (`file.close`, `file.exit`…), the `document` it asks about, how many are `remaining`, and its `choices`. `ui.dialog.confirm` takes `{"dialog": "discard", "choice": "save" | "dontSave" | "cancel"}` (default `save`; revert takes `dontSave` or `cancel`) and answers it as the window's buttons do, one document at a time; `ui.dialog.cancel` with `{"dialog": "discard"}` cancels it. save of an untitled document opens the save dialog first, so the reply's `fileDialog` is true and the prompt stays open until that is answered.
  - layer style (`layerStyle {effect?}` opens on that effect kind): the fields hold the dialog's whole state, so agents read and drive it like a user. `effects` lists every effect instance in the layer's order as `{id, kind, on, params, fx}` (`fx` is the effect snapshot the dialog loaded; ok edits that snapshot with `params`, so anything the dialog doesn't model survives). `selected` picks the page (`blendingOptions` or an instance id), `preview` (default on) gates the live canvas preview, `p:blendingOptions` edits blend mode/opacity/fill opacity, and `patternList` names the usable patterns. ok replaces the layer's effects in one step (`layer.layerStyle.replace`); cancel discards. "new style…" saves the pending state via `style.presets.new {effects}`, a swatch click applies a style preset, and make/reset default use `layer.layerStyle.makeDefault` / `layer.layerStyle.defaultFor`
  - preferences (`ui.menu.invoke {id: "edit.preferences.interface"}`) exposes the sections in its `values` field. apply (`ui.dialog.apply`) and ok (`ui.dialog.confirm`) on preferences are refused over control because automation denies whole-preferences updates and filesystem-bearing preference sections. a refused `ui.dialog.apply` returns an error and keeps the dialog open; a refused `ui.dialog.confirm` returns an error but closes and removes the dialog. `ui.dialog.cancel` closes the dialog without saving. individual safe preferences can instead be changed with `engine.execute` using `prefs.set` (e.g. `{"command": "prefs.set", "params": {"path": "interface.showTooltips", "value": false}}`). settings marked for the next launch still require a restart.
  - edit › fill… (`ui.menu.invoke {id: "edit.fill"}`, shift+f5, shift+backspace) opens the fill dialog; its fields are `edit.fill`'s params (`contents`, `color`, `pattern`, `colorAdaptation`, `mode`, `opacity`, `preserveTransparency`), and ok remembers them in the preferences (`dialogs["edit.fill"]`).
  - save for web (`ui.menu.invoke {id: "file.export.saveForWebLegacy"}`) is resizable with independently scrolling settings. dialog fields `__view` (`original|optimized|2up|4up`), `__zoom` (`null` = fit, otherwise a factor; 1 = 100% in physical display pixels), and `__center: [u,v]` control the shared preview navigation. `__previewRects` reports pane rectangles in screen points. drag a preview to pan, scroll/pinch to zoom at the pointer, or double-click to fit. GIF/PNG-8 expose `colorTable: ["#rrggbb", …]` (opaque source entries) and `colorTableEdits: {"0":"#rrggbb", …}` (zero-based replacements after quantization). a swatch opens a color picker; edits update the preview and saved file without remapping pixel indices or changing the document. reset clears replacements. changing palette/size/matte/transparency settings regenerates the table and clears index-based edits. previews and generated tables use a bounded 2048-pixel proxy; full-size export uses that explicit table unchanged. `__paletteApproximate` reports proxy-derived palette generation, and enlarged proxy previews are labelled. the engine export command accepts the same table/edit parameters; its no-path estimate returns the source `colorTable` and `transparentIndex` as well as size statistics. opening a new dialog does not carry a previous image's table across.
  - a pixel tool pressed on a type, shape, smart object or fill layer (e.g. through `ui.pointer`) opens the "rasterize?" prompt instead of painting: a dialog with `__rasterize` (`type|shape|smartObject|fill`), `message`, `layer`, `tool` and `at`. `ui.dialog.confirm` runs the `layer.rasterize.*` command and then paints at `at` (two history states, `{"rasterized", "painted"}`); `ui.dialog.cancel` does nothing.
  - edit › transform › warp on editable type (`ui.menu.invoke {"id":"edit.transform.warp"}`, or the warp toggle of an open transform box) opens warp mode with no prompt, as in photoshop: the mesh can't be dragged or split, only the options bar's warp style, bend, h and v apply, and the commit sets the layer's warp text in one step, so the type stays editable. directly: `engine.execute {"command":"edit.transform.warp","params":{"style":"flag","bend":40}}` on a type layer. distort and perspective stay greyed for type.
- `ui.window.open {document?}` / `ui.window.close {window}`: extra document windows
- `ui.pointer {events: [{kind: down|move|up, x, y, pressure?, tiltX?, tiltY?, rotation?}], tool?, modifiers?, button?}`: drive the active tool in document coordinates (pressure 0..1, tilt in degrees -90..90, barrel rotation 0..360: a simulated pen). modifier flags (`shift`, `alt`, `command`, `ctrl`, `space`) go either at the top level or grouped under `modifiers`; when `modifiers` is present, top-level flags are ignored. `space: true` holds space (the crop frame, marquee, lasso or shape being drawn then moves instead of growing). with the crop tool in its default mode (use classic mode off) a drag inside the box moves the image and one outside turns it, the view panning and turning with it; a drag's `move`/`up` points are read as the view showed the document at its `down`, so an `up` where the last `move` was changes nothing more. `command: true` with a painting, retouching, eraser, gradient, eyedropper or other non-selection tool makes the gesture the move tool's, as holding ⌘ (ctrl off the mac) does in the app: a drag moves the active layer (the selected pixels when there is a selection), with `alt: true` as well it duplicates the layer first, and a click picks the layer under the point; the tool is unchanged afterwards. the hand, rotate view, zoom, crop, slice, path selection, pen, shape, type and selection tools keep `command` for themselves. `ctrl: true` with `alt: true` and a brush tool is the on-canvas brush resize instead (in the app, ctrl+alt-drag off the mac). `button: "right"` (or `"secondary"`) is the right button: with the move tool (or `command: true` with any tool) it opens the canvas layer menu (`layerMenu` in `ui.inspect` lists the layers there, topmost first; select one with `layer.select`); with marquee, lasso, magic wand, object selection, or pen it opens a tool context menu (`canvasToolMenu` in `ui.inspect`, with ordered command ids, separators, and enabled states); with zoom or hand it opens the view menu there (zoom in and zoom out step about the pressed point; `view.fillScreen` is its fill screen row); with a painting tool it opens the brush preset picker, or erases with preferences › tools › right-click with painting tools = erase. the menus open at the screen position of the pressed document point. use `ui.context.choose {id}` for enabled tool-context actions. the pen menu is documented in `docs/context-menu-parity.md`. `button` is `left` (default), `right`, `secondary` or `middle`, and an event's `kind` is `down`, `move` (default) or `up`. with a color picker as the top dialog, `down` and `move` sample the image into its new colour instead (its eyedropper, like a click on the canvas) and the tool is not driven. a `down`/`up` at the same point with the rectangle, ellipse, triangle or polygon tool opens create rectangle / ellipse / triangle / polygon (photoshop's click-to-size dialog); set its `width` and `height` (document pixels), `fromCenter`, `radii` [tl, tr, br, bl] (rectangle) or `sides` and `starRatio` (percent, polygon) with `ui.dialog.set`, then `ui.dialog.confirm` runs `shape.create` at the clicked point.
  - magnetic lasso (`tool: "magneticLasso"`): a `down`/`up` pair is a click. the first sets the first fastening point (its modifiers set the selection mode); later clicks fasten a point, or close the border on the first point. `move` events trace the border with or without a button held, fastening points by themselves; with `alt: true` a click draws a straight segment and a drag a freehand one. `ui.inspect` shows the border in progress under `magnetic` (`path`, `anchors`, `live`, `mode`). `ui.key` enter closes it along the edges, backspace removes the last fastening point, escape cancels. to select along edges in one call instead, use the `select.magneticLasso` command with rough points around the shape.
  - levels or curves adjustment layer, properties eyedropper armed (`layerPicker` in `ui.inspect`: `{layer, point, tool}`, set by the panel's set black/neutral gray/white point buttons): a `down` sets that point from the image below the layer as one `layer.setAdjustment {"layer", "eyedropper": {"point", "at": [x, y], "size"}}` (send that command directly to skip the ui) and the tool is not driven; esc, another tool or another layer disarms it.
- `ui.click {x, y, button?, count?}` / `ui.move {x, y}`: synthetic pointer input in screen points (`button`: `left` (default), `right` (or `secondary`) or `middle`). `count` (default 1, `2` for a double-click) is at most 256; a larger one is an error and queues nothing
- `ui.key {key, command?, shift?, alt?, ctrl?}` (flags may also be grouped under `modifiers`): press and release a key, e.g. `{"key": "ArrowLeft", "shift": true}`. `{"key": "AltLeft"}` is a tap on alt, which puts the keyboard on the in-window menu bar (then `ArrowRight`, `ArrowDown`, `Enter`, `Escape` or a menu's letter drive it); `{"key": "F", "alt": true}` opens the menu with that mnemonic (#4988)
- `ui.pointer`, `ui.click`, `ui.move`, `ui.key` and `ui.screenshot` validate every parameter before acting, as `ui.set` does: an unknown field (top-level, under `modifiers` or in an event), a value of the wrong JSON type (`"x": "abc"`, `"ctrl": "yes"`), a `button`, `kind`, `tool` or `key` outside the accepted set, or a number outside its range (`pressure`, tilt, `rotation`, `count`) is an error naming the parameter, and nothing is queued or applied. a `null` field counts as omitted and takes its default
- engine commands (`engine.execute`, `brahma-cli run`, MCP `command_run`) reject a param key their `params` doc does not name for the commands listed in `STRICT` (`crates/engine/src/param_check.rs`, today `layer.hideLayers` and `layer.showLayers`): `{"ids":[4]}` is the error `unknown parameter \`ids\` (expected: layer)`, and nothing runs. other commands still ignore unknown keys until their `params` doc is complete and they are added to that list (`coalesce` and `target` are always accepted).
- `ui.type {text}`: type text (goes to the focused widget, or to the canvas while the type tool is editing)
- `ui.resize {width, height}`: resize the main window
- `ui.gpu.simulateLoss {error?}`: act as if the wgpu device was lost (or reported an error with `error: true`). the app switches to the CPU image compositor for the rest of the session and shows the same recovery warning as on a real loss. `gpuFallbackNotice` in `ui.inspect` contains the reason while the warning is open. keep using CPU saves CPU compatibility for the next launch; retry GPU saves GPU mode and asks the user to save and restart. the window renderer still requires a working graphics or software adapter. returns `wasActive` and `gpuInfo`. for testing the fallback
- `ui.screenshot {path?, focus?}`: capture the main window (PNG). with no path the reply contains
  base64 PNG data; a path is relative to the automation write root. raises the window first
  (default) because occluded macos windows stop rendering
- `ui.focus`: bring the main window to the front
- `app.open {path}` / `app.save {path?, tgaBits?}`: relative file i/o through the configured automation roots (`app.open` reads under the read root, `app.save` writes under the write root; absolute paths, `..` and paths escaping the root are refused, and both fail closed when no root was granted). both reply with `warnings` (import/export notes such as "adjustment layer flattened"; `[]` when none), also shown to the user in the status bar and as a notice (`notices` in `ui.inspect`); `app.open` also returns the `path` and document `name` (directory `.pcraft` — the native on-disk form — loads through the read root's capability; the single-file (zip) form takes the bytes path), `app.save` the `path` written. `app.save` without `path` writes back only to the document's own PSD, PSB or `.pcraft` file, like file › save. `tgaBits` (24 or 32) sets a TGA's bits per pixel: 32 writes the alpha channel (or else the transparency) as its alpha, 24 none; without it, 32 with an alpha channel or transparency, else 24. automation saves never show TGA or TIFF options. automation opens and saves never fire script events. use these two rather than `file.open`, `file.save`, `file.saveAs` or `file.saveACopy`, which the control channel refuses (see [engine commands](#engine-commands))
- `app.quit`

## engine commands

change the ui language without restarting with `prefs.set`:
`{"path":"interface.language","value":"fr"}`. supported codes and preview/apply behaviour
are documented in [ui localisation](localization.md). `prefs.get` and the existing preference
store expose and persist the same setting; scripts keep using canonical command ids.

`engine.execute` runs any command by id. `engine.commands` (or the engine command `command.list`) lists them all, with labels, menu paths, shortcuts, a parameter description, and whether each is currently enabled (reply shape: [command list replies](#command-list-replies)). each row carries `enabled` (bool), `disabledReason` (`null` when enabled, otherwise the precondition that failed, such as `"no document open"`, `"no selection"` or `"the clipboard is empty"`) and `needsDocument` (true when the command is off because no document is open, so the first step is to create or open one; a command may still have its own precondition after that, such as an active text layer. false when enabled or off for another reason). with no document open, nearly every document command reports `enabled: false, needsDocument: true`; plan around `needsDocument` rather than reading `enabled` alone (#3664). examples:

| command | params |
|---|---|
| `file.new` | `{"width":1920,"height":1080,"mode":"rgb","depth":8,"background":"white"}` |
| `layer.new.layer` | `{"name":"Ink"}` |
| `layer.select` | `{"layer":id\|str,"mode":"replace\|toggle\|range\|add\|remove"}` or `{"name":"Background","occurrence":0?}` (occurrence is zero-based document walk order; required for duplicate names): ⌘-click = toggle, ⇧-click = range; `document.inspect` reports `selectedLayers` and a per-layer `selected` flag |
| `channel.target` / `channel.setVisible` | channels panel target and eyes (`"composite"`, colour name, alpha index or name, `"quickMask"`). `document.inspect` (and `channel.list`) report `channels`: composite/colour/alpha rows with visibility, channel options, `quickMask`, `target`. pixel commands without a `target` param follow the targeted channel |
| `layer.setProps` | `{"layer":id?,"name":…,"visible":…,"opacity":0..1,"blend":"Multiply"}` |
| `layer.fill.solidColor.set` | `{"layer":id?,"document":id?,"color":"#rrggbb"}`: edit a solid color fill in one undo step; RGB without alpha preserves the fill's alpha. float RGB arrays and a stored `Color` are also accepted. `document` rejects stale dialog targets. |
| `layer.fill.pattern.set` | `{"layer":id?,"document":id?,"pattern":id\|name?,"scale":1..1000?,"angle":-180..180?,"link":bool?,"phase":[x,y]?}`: edit a pattern fill (the pattern fill dialog's ok) in one undo step; omitted values keep the layer's, `phase` `[0,0]` is snap to origin. `layer.newFillLayer.pattern` from a menu (no params) and `layer.layerContentOptions` on a pattern fill layer open the pattern fill dialog. |
| `layer.layerMask.edit` | `{"layer":id?,"document":id?,"density":0..100?,"feather":0..1000?}`: edit pixel-mask density (%) and feather (px) without changing mask pixels, including adjustment masks. at least one setting is required; `document` rejects stale targets. a shared `coalesce` key groups slider updates into one undo step. |
| `layer.setExpanded` | `{"layer":id?,"expanded":bool?,"all":bool?}`: open/close a group in the layers panel (no `expanded`: toggle; `all`: every group). a view change saved with the document (PSD open/closed folder), not an undo step; `document.inspect` reports `expanded` on groups |
| `layer.newAdjustmentLayer.hueSaturation` | `{"hue":30,"saturation":10}` |
| `paint.stroke` | `{"points":[[x,y,pressure],…],"size":20,"color":"#ff0000"}` |
| `select.rect` | `{"x":0,"y":0,"width":100,"height":50,"mode":"add","ellipse":false}` |
| `document.inspect` | `{}`: layer tree, history, snapshots (`[{id, name}]`, for `history.selectSnapshot`), selection bounds |
| `layer.list` | `{"document":index?,"flat":bool?}`: just the layers, top-down: `id`, `name`, `kind`, `visible`, `bounds` (`[x,y,w,h]` or null), `selected`, `active`; groups list their `children`, or with `flat` every layer is one row with its `depth` and `parent` group id |
| `document.pixel` | `{"x":10,"y":10}`: composite RGBA |
| `document.presets.list` | `{}` → `{presets:[{name,settings}]}`: saved new document configurations |
| `document.presets.save` | `{"name":"Product square","settings":{"width":1600,"height":1600,"background":"transparent"}}`: save a snapshot; names are trimmed, 1–255 characters, and ASCII case-insensitive duplicates are rejected |
| `document.presets.get` | `{"name":"Product square"}` → `{preset,params}`: fetch settings; pass `params` to `file.new` to create a document, optionally adding a document `name` |
| `document.presets.delete` | `{"name":"Product square"}`: delete the named saved configuration |
| `document.recent.list` | `{}` → `{recent:[settings]}`: file › new › recent, the settings of the documents last created with the new document dialog, newest first (at most 20) |
| `document.recent.add` | `{"settings":{…as document.presets.save}}`: put settings first in recent (the dialog does this after create; equal entries collapse); not recorded into actions |
| `type.hitTest` | `{"layer":id?,"x":px,"y":px}`: character under a document point. no `layer` picks the topmost visible type layer there. result `{"layer","index","line","inside"}` |
| `type.caret` | `{"layer":id?,"index":char}`: caret segment in document pixels, `{"index","line","segment":[[x,y],[x,y]]}` (rotated and vertical type included) |
| `type.navigate` | `{"layer":id?,"index":char,"move":"wordPrev\|wordNext\|linePrev\|lineNext\|lineStart\|lineEnd\|start\|end","x":px?}`: neighbouring caret. `x` keeps the column across line moves. result `{"index"}` |
| `actions.list` | `{}` → `{actions:[{name, steps}], recording}` (the name being recorded, or null) |
| `actions.get` | `{"action": name or index}` → `{name, steps:[[id, params], …], disabled:[step, …]}`. `steps` is the shape `file.automate.batch` and droplets take; `disabled` lists the 0-based steps playback skips |
| `actions.record` | `{"name":"Red"}` starts a new action (default name `Action N`). `{"action": name or index}` appends to one that exists |
| `actions.stop` | `{}` → `{action, steps}`. copies replayable journal entries since `actions.record` (queries and action-editing commands omitted) |
| `actions.play` | `{"action": name or index, "from": step?}`. `from` and `failed.step` are 0-based. returns `{action, ran, failed?:{step, id, error}}` and still returns ok when a step fails, so a partial run is reported. leaves one history step per step that ran. supports nested action calls up to 16 levels, rejecting cycles. while recording, playing another action appends one named `actions.play` step instead of its expanded commands. on an untrusted session each step is authorized the same way as a top-level command |
| `actions.move` | `{"action": name or index, "step": index?, "to": index}` moves a step within its action, or moves the whole action when `step` is omitted. `to` is the final zero-based index. pending steps can be reordered while recording; invalid moves leave the list unchanged. |
| `actions.delete` | `{"action": name or index, "step": index?}`. with a zero-based `step`: `{action, deletedStep, steps}`, also allowed while recording. without `step`: `{deleted}`, refused while recording |
| `actions.toggle` | `{"action": name or index, "step": index?, "enabled": bool?}` → `{action, disabled}`. switches one 0-based step (pending recorded steps included) on or off; omitting `enabled` flips it. without `step`, switches every step: on unless all already are. playback, batch and droplets skip steps that are off. in the desktop panel, click a step's check box |
| `actions.rename` | `{"action": name or index, "name": str}` → `{action, index}`. empty, whitespace-only or duplicate names are rejected and keep the old name. nested `actions.play` steps that call the action by name and its function key follow the new name. in the desktop panel, double-click an action's name to rename it in place (enter commits, esc cancels) |

the actions list fills the resizable dock group.

the actions panel shows replayable steps immediately while recording. click an individual
step to select it, then use the trash button to remove that step; selecting the action
heading instead targets the whole action. step deletion also works during recording
and does not undo the document edit. the control equivalent is
`actions.delete {"action": "Name", "step": 0}` (zero-based). omitting `step` deletes
the whole action, which still requires recording to be stopped.

desktop actions also record and replay view › fit on screen (`view.fitOnScreen`),
100% (`view.actualPixels`), zoom in and zoom out through their menu commands or shortcuts.
select an action and press record to append steps; the plus button starts a new action.
a final `["view.fitOnScreen", {}]` step fits the resized document when the canvas is laid out.
headless playback reports unsupported view commands as failed steps.

in the desktop actions panel, right-click an action to assign a function key (f1–F12),
or choose none to remove it. the binding follows the action name, regardless of the selected
row, and is saved in preferences. for example, the existing `edit.keyboardShortcuts` command
accepts `{"set":{"actions.play:My action":"F6"},"allowUnknown":true,"removeConflicts":false}`.
a named action takes precedence over the ordinary menu shortcut while it exists and is bound;
removing its binding or deleting it restores that menu shortcut. the panel moves a key away
from another action when assigning it. action playback still checks each nested command's
automation permissions.

ui-level commands (`view.zoomIn`, `window.theme.pro`, `edit.search`, …) are also accepted by `engine.execute` and `ui.menu.invoke`.

commands that read or write files by path, instead of through the automation roots, are refused with "automation command `…` uses ambient filesystem paths and is disabled; use capability-scoped document methods". that covers every `file.*` command except `file.new`, the `file.close*` commands and a few path-free ones such as `file.fileInfo`, so `file.open`, `file.save`, `file.saveAs` and `file.saveACopy` always fail here: open and save with `app.open` / `app.save`. path parameters of other commands (`layer.exportAs {path}`, `filter.distort.displace {mapPath}`, preset imports, and plug-in install/reload) are refused the same way. `prefs.set` rejects whole-preference updates and file-backed sections (`colorSettings`, `scriptEvents`, `historyLog`, `plugIns` and `scratchDisks`) so automation cannot configure ambient file access indirectly. the desktop app also refuses `image.mode.*`, which can load the colour profiles set in its preferences. the rules are in `crates/automation/src/workspace.rs`.

embedded smart objects can be opened with `layer.smartObjects.editContents {layer?}` or
unpacked with `layer.smartObjects.convertToLayers {layer?}`. this includes photoshop sources
stored by UUID in the document's embedded linked-layer blocks. sources requiring a disk read
remain refused, including nested linked objects that need re-rendering during unpacking.
in the opened contents document, use `layer.smartObjects.saveContents {}` to update the parent
in memory, or close the dirty contents document. for nested objects, save from the deepest
contents outward. `app.save` exports the active document to a scoped file; it is not a substitute
for save contents. save the parent document to disk when the edits are complete. these rules
also apply to recorded actions and synthetic ui input; interactive desktop commands retain
their usual file access. source replacement, relinking and export commands remain restricted.

`type.editText` starts inline editing of the active type layer and selects all its text, like
double-clicking its thumbnail in layers. text input, commit and cancel use the existing type tool
editing session; non-type layers return an error without changing the tool or document.

### command list replies

`engine.commands` (control channel and `serve`), the engine command `command.list` and the MCP
tool `command_list` all reply with a bare JSON array, not an object: there is no
`{"commands": [...]}` wrapper. on the control channel and `serve` the array is the reply's
`result` (`{"id":1,"ok":true,"result":[…]}`); the MCP tool returns it as its JSON text.
each element describes one command, in registry order:

```json
{"id":"file.new","label":"New…","menu":["File"],"shortcut":"Cmd+N","params":"{\"width\":1..300000=1920,…}","enabled":true,"disabledReason":null,"needsDocument":false}
```

| key | type | meaning |
|---|---|---|
| `id` | string | the id to pass to `engine.execute` / `command_run` |
| `label` | string | menu label |
| `menu` | array of strings | menu path above the item (`[]` for commands without a menu item) |
| `shortcut` | string or `null` | default shortcut, e.g. `"Cmd+Shift+X"` |
| `params` | string | human-readable parameter doc (not a JSON schema) |
| `enabled` | boolean | whether the command can run in the current state |
| `disabledReason` | string or `null` | why the command can't run now (`null` when `enabled`) |
| `needsDocument` | boolean | `true` when it is off because no document is open (#3664) |
| `keywords` | string, optional | other names the command is found by (#5060), e.g. `select.rect`'s elliptical marquee; only present on commands that have them |

filtering narrows the same array without changing its elements: `serve`'s `engine.commands
{filter}` and the MCP tool's `filter` keep commands whose id, label or keywords contain the text
(case-insensitive), and the MCP tool's `enabled_only` keeps those with `enabled: true`. the
desktop control channel's `engine.commands` takes no filter and always returns every command.
the shape is pinned by a test in `crates/automation/src/rpc.rs` (#3684).

### background jobs (#210)

long commands (every `filter.*`, `edit.contentAwareFill`, `edit.contentAwareScale`, `file.automate.photomerge`, `brush.presets.importAbr`), file opens and interactive saves (file › save, save as and the TIFF options continuation, #2017) run as background jobs in the desktop app: the window keeps drawing, the status bar shows progress with a cancel button, and jobs that lock the active document show a modal progress dialog (esc cancels). `engine.execute`, `ui.menu.invoke`, `ui.dialog.confirm` and `app.save` still wait for the result by default; pass `"wait": false` to get `{job, pending: true}` at once, then poll `jobs.list` (`state`: running, done, failed, cancelled; `progress` 0–1; the result or error) and stop it with `jobs.cancel {job}`. a cancelled or failed job leaves the document unchanged. while a job runs, commands that would edit its document fail with "… is still running on this document". `ui.inspect` reports `jobs` (running jobs, opening files, saving files). set `BRAHMA_INLINE_JOBS=1` to run everything inline.

## preferences

preferences (edit › preferences, grouped like photoshop's dialog sections) live in the engine, so
the same commands work in the app, the CLI and headless MCP. paths are dotted camelCase keys:
`<section>.<key>`, with sections `general`, `interface`, `workspace`, `tools`, `historyLog`,
`fileHandling`, `export`, `performance`, `scratchDisks`, `cursors`, `transparencyAndGamut`,
`unitsAndRulers`, `guidesGridAndSlices`, `plugIns`, `type`, `enhancedControls`, `rawDefaults`,
`integrations`, plus `shortcuts.<command id>`, `menus` (`hidden`, `colors.<id>`), `toolbar`
(`hidden`, `order`) and `colorSettings` (edit › color settings).

| command | params |
|---|---|
| `prefs.get` | `{"path":"performance.historyStates"}`; no path returns everything |
| `prefs.set` | `{"path":"cursors.painting","value":"precise"}` or `{"values":{"unitsAndRulers.rulers":"cm","guidesGridAndSlices.gridColor":"#ff8800"}}`. values are validated (choices, ranges, `#rrggbb` colours, shortcut syntax); a batch applies all or nothing. a section path takes an object and merges it key by key |
| `prefs.reset` | `{"path":"performance"}` (a section or key); no path resets everything. an individual `shortcuts.<id>` or `menus.colors.<id>` reset removes the override and returns `null`, including when it is already absent |
| `edit.preferences.<section>` | the section's values; in the app (no params) it opens the preferences dialog on that section |
| `edit.keyboardShortcuts` | `{"set":{"edit.fill":"Cmd+Shift+F"},"reset":true\|["id",…],"removeConflicts":true,"filter":"blur","list":false,"importKys":"<.kys XML>"}`: returns overrides, matching commands and conflicts. a shortcut moved to another command is removed from its old owner unless `removeConflicts` is false. `""` removes a shortcut, `null` restores the default. the held temporary tools are bindable too: `tools.temporary.hand` (space; also repositions a selection being drawn), `tools.temporary.zoomIn` (cmd+space), `tools.temporary.zoomOut` (alt+space; alt added to a held zoom in, cmd+alt+space, zooms out too), `tools.temporary.erase` (`` ` ``; with the Brush or Pencil, a stroke started while it is held is in Clear mode, `paint.stroke`/`paint.pencil` `"mode": "clear"`, erasing with that tool's brush); they list with `"hold": true`. ⌘ (Ctrl off the Mac) held alone is the Move tool with the painting, retouching and other non-selection tools, as in Photoshop; it is a modifier, not a binding. `importKys` applies a Photoshop `.kys` set (the file's XML text, at most 4 MiB): rows are matched to commands by menu label, a key equal to the default restores it, keys also given in `set` win, tool keys are not imported, and the result gains `import` (`name`, `imported`, `set`, `unknown`, `unreadable`, `alternates`, `toolKeys`). In the app it opens Keyboard Shortcuts and Menus, whose Import Shortcuts… button loads a `.kys` set into the dialog (opening or dropping a `.kys`, or Photoshop's live `keyboard shortcuts.psp`, does the same). On a desktop with Photoshop installed, the first launch that has no shortcut overrides yet asks whether to import the newest install's live set (Import / Don't Import; `ui.inspect` shows it as `kysOffer`); the answer is recorded in the preferences as `dialogs.photoshopShortcuts`, so it is asked once |
| `edit.menus` / `edit.toolbar` | `{"hide":["edit.fade"],"show":[…],"color":{"edit.fill":"red"},"reset":false}` / `{"hidden":["Sponge"],"order":[…]}` |
| `edit.colorSettings` | `{"workingRgb":"display-p3","workingCmyk":"coated-cmyk","workingGray":"sgray","policyRgb":"preserve\|convert\|off",…,"askOnMismatch":true,"intent":"perceptual","bpc":true}`; honoured when opening files (`file.openAs`, the app's file › open) and by image › mode and "working" profile specs. `color.profileMismatch {"action":"preserve\|convert\|discard\|assignWorking"}` answers the mismatch prompt |

values the app honours live: history states (every open document), effect-cache budget, interface
theme, canvas colour and border, checkerboard size and colours (CPU and GPU canvas), gamut warning
colour and opacity, guide/grid/smart-guide colours and styles, grid spacing and subdivisions, ruler
units (rulers, info panel, image size default unit), image interpolation (image size default),
painting and other cursors, zoom with scroll wheel, use shift key for tool switch, bundle arrow-key
nudges and nudge bundle pause (arrow keys sent with `ui.key` that follow each other within the pause
nudge the layers, selection outline or free transform box as one history step; an agent calling
`layer.translate` or `select.transformSelection` itself gets the same with a shared `coalesce` key),
keyboard shortcuts, hidden and coloured menu items, autosave interval and crash recovery, and the history
log text file. GPU on/off and the GPU tile size apply at the next launch.

the desktop app stores them in `preferences.json` in the platform config directory (macos
`~/Library/Application Support/Brahma`, windows `%APPDATA%\Brahma`, linux
`$XDG_CONFIG_HOME/brahma`; override with `BRAHMA_CONFIG_DIR`); autosaves go to its
`Recovery` folder, and the window size and position, the dock width and floating-panel geometry
to `ui.ron` beside it (a file holding values that would crash startup is discarded). in portable mode (a `portable.txt` or `brahma.portable` file beside the
executable, as in the windows portable zip) that directory is `BrahmaData` next to the
executable instead. the web build keeps them in `localStorage`. a save writes only the values
this instance changed since it last loaded or saved them over what storage holds now, so a second
browser tab (or app window) never reverts the other's changes; for a value changed in both, the
latest save wins. a failed write stays pending and is retried (after 2 s, doubling up to 30 s, and
sooner after a further change); the status bar reports the first failure and a notice a
persistent one.

user and imported (`.abr`) brush presets live in the config directory's `Presets` folder: one
`.pcbrushes` JSON file per preset group, content-addressed tip bitmaps under `tips/`, and an
`index.json` with the group order and deleted built-ins (see `brahma_engine::preset_store`).
the store loads in the background at launch and syncs after every brush preset change; built-ins
are never written. the actions list is `actions.json` in that same folder and survives a restart.
headless CLI/MCP sessions and the web build keep brush presets and the action list for the session
only, unless a store is attached. gradient presets (including imported `.grd` groups) persist with
the preferences.

named new document presets also persist with preferences (`presets.documents`) on desktop and
web. file › new › save preset… saves the current form; the saved tab selects a configuration or
deletes it. selection only fills the form: create still runs `file.new`, and the document name
is independent of the preset name. settings include pixel width/height, resolution in ppi,
mode, depth, background, and display `unit` (`px`, `in`, `cm`, `mm`, `pt`, `pica`) and
`resolutionUnit` (`in`, `cm`). the defaults match file › new. background color captures the
toolbox colour as `backgroundColor: [r,g,b]` (0..1), so reusing it does not follow later toolbox
changes. up to 256 presets are kept. invalid saved entries are skipped independently; other
presets and preferences still load. automatic recent configurations are separate follow-up work.

## snapping

with view › snap on, tool gestures snap to the view › snap to targets (guides and grid while they
are shown, layer edges and centres, document bounds and centre, selection edges) within 8 screen
pixels: move tool drags (the moved layers' bounds), marquee, crop, shape, type-box and pen points,
free transform handles and drags inside the box, and guides. distort corners snap to enabled
guides and their intersections, using the actual corner even when its handle is grabbed off-centre.
guide targets are excluded when guides, canvas guides or extras is hidden, for all snapping gestures.
ctrl/cmd corner distortion in free transform does the same with its enabled snap targets. holding
ctrl disables snapping for other gestures; ctrl used for an allowed corner distortion keeps snapping.
with view › show › smart guides on, the move tool also snaps to other layers and draws
magenta alignment lines. `ui.pointer` drives the same code, so agents get identical results.

## MCP bridge

`brahma-automation` provides an MCP server built on the official rust SDK (`rmcp`). it runs in one of two modes:

- **headless** (`brahma-cli mcp`): an in-process `brahma_engine::Session`. there is no window.
- **bridge** (`brahma-cli mcp --bridge 127.0.0.1:7878 --control-token-file <path>`): every tool is forwarded to a running `brahma --control 7878 --control-token-file <path>` over this protocol, so agents see and drive the live app.

the bridge keeps one authenticated TCP connection open, and drops and reconnects before a call once the connection has been idle long enough that the app (which closes idle connections after 30s) may have dropped it. a transport failure while sending a request or waiting for its reply, including a reply timeout, drops the connection and reports that the operation may have completed; inspect the document before retrying an edit. edits are never automatically resent (#1007); a connection that could not be established is reported as "the request was not sent" instead. the MCP server opts its pure-read tools (`ui_inspect`, `doc_inspect`, `command_list`, `jobs_list`, `session_list`, `ui_screenshot`) into one automatic retry on a fresh connection, since a read applies nothing; clients driving `BridgeClient` directly opt in per call with `call_read`. the next separate call reconnects and authenticates, and reply lines whose `id` does not match the request are skipped. the client's call timeout is 55s, strictly below the server's 60s reply deadline, so the bridge's error is the one callers see; commands that can run longer should pass `"wait": false` and poll `jobs.list`. it only accepts loopback addresses, because the app only listens on loopback. supply its bearer token with `--control-token-file`, `--control-token`, `BRAHMA_CONTROL_TOKEN_FILE`, or `BRAHMA_CONTROL_TOKEN`:

```sh
brahma-cli mcp --bridge 127.0.0.1:7878 \
  --control-token-file /private/path/brahma-control.token
```

how each MCP tool maps onto control methods in bridge mode:

| MCP tool | control method |
|---|---|
| `command_run {id, params, wait?}` | `engine.execute {command: id, params, wait}` |
| `jobs_list` / `jobs_cancel {job?}` | `jobs.list` / `jobs.cancel {job?}` |
| `command_list {filter?, enabled_only?}` | `engine.commands` (filtered by the MCP server; replies with the same bare array, rows keep `disabledReason` and `needsDocument`, see [command list replies](#command-list-replies)) |
| `doc_new {…}` | `engine.execute {command: "file.new", params}` |
| `doc_inspect` | `engine.execute {command: "document.inspect"}` |
| `doc_open {path}` | `app.open {path}` |
| `doc_place {path, …}` | `engine.execute {command: "file.placeEmbedded", params}` |
| `doc_export_layers {dir, …}` | `engine.execute {command: "file.export.layersToFiles", params}` |
| `doc_export_data_sets {dir, …}` | `engine.execute {command: "file.export.dataSetsAsFiles", params}` |
| `doc_import_data_sets {path, …}` | `engine.execute {command: "file.import.variableDataSets", params}` |
| `doc_save {path?, tgaBits?}` / `doc_export {path, tgaBits?}` | `app.save {path?, tgaBits?}` |
| `doc_render_preview {max_side?}` | `ui.screenshot`, returned directly as PNG image content |
| `session_list`, `ui_inspect` | `ui.inspect` |
| `ui_screenshot {max_side?}` | `ui.screenshot`, returned as PNG image content |
| `ui_pointer {events, modifiers?, button?}` | `ui.pointer` (other arguments are an error) |
| `ui_menu_invoke {id}` | `ui.menu.invoke` |
| `ui_set {fields}` | `ui.set` |
| `control_call {method, params}` | any method, passed through unchanged |

bridge previews capture the app window; passing `index` is an error. headless `doc_render_preview` supports `index` without changing the active document.

`doc_select` and `doc_close` work only in headless mode. the `ui_*` tools and `control_call` work only in bridge mode; in headless mode they return a tool error that explains how to start bridge mode.

**security note:** TCP control uses a bearer token, not client identity or general per-method
authorization. a client that possesses the token receives the non-filesystem control surface,
including ui input, command execution, and application control. keep token files private, do not
commit or log tokens, and do not pass a token directly on a shared system where process command
lines are visible. the protocol is unencrypted and must remain on loopback; do not tunnel or proxy
it to an untrusted host.

filesystem access fails closed unless launch-time read and/or write roots are granted with
`--automation-read-root` and `--automation-write-root`. the environment-variable equivalents
(`BRAHMA_AUTOMATION_READ_ROOT` / `BRAHMA_AUTOMATION_WRITE_ROOT`) are supported by
the desktop `brahma` process, but not by headless `brahma-cli mcp` or `serve`;
for those CLI modes, supply the flags explicitly. in MCP bridge mode, configure roots on the
desktop process rather than on the bridging CLI. request paths must be
non-empty, forward-slash relative paths beneath the applicable root, or absolute paths that lie
beneath it. an absolute path is compared with the root as text and the rest is checked like a
relative path; nothing is resolved on the filesystem. absolute paths outside the root, parent
traversal, alternate separators in relative paths, drive/device/stream prefixes, malformed
components, and link escapes are rejected before file effects. read and write authority are
independent; the parent of a new output file must already exist. engine commands that still use
ambient filesystem paths are disabled for automation until they are migrated to the same
capability interface. interactive desktop file pickers retain normal user-selected access.

## headless server

`brahma-cli serve` keeps one headless engine session (no window, no GPU) and answers the same
JSON-lines envelope on stdio, or on `127.0.0.1:<port>` with `--port <port>` (loopback only; each
authenticated connection shares the session). TCP uses the same first-frame `auth` exchange and
token options as desktop control. stdio does not require this TCP handshake because access is
inherited from the process pipe. it is the fastest way for a script or agent to make many edits:
no MCP framing, no app start-up per command. configure its file access with the same
`--automation-read-root` and `--automation-write-root` flags. implementation:
`crates/automation/src/rpc.rs`.

| method | params |
|---|---|
| `engine.execute` | `{command, params?, wait?}`: any engine command (`wait: false` starts a long one as a background job: `{job, pending}`) |
| `jobs.list` / `jobs.cancel` | `{}` / `{job?}`: background jobs; cancel one or all. every request (and MCP tool call) first applies the jobs that finished, so `doc.save`, `doc.inspect`, `doc.render` and `session.list` include a finished job's result without polling `jobs.list` first |
| `engine.commands` | `{filter?}`: registry with params docs and enablement (`enabled`, `disabledReason`, `needsDocument`); `result` is a bare array of command objects ([command list replies](#command-list-replies)) |
| `session.list` | open documents and the active index |
| `doc.open` / `doc.new` | `{path}` / `file.new` params |
| `doc.place` | `{path, scale? (percent), fit? (true), center? ([x, y]), index?}`: places a file as a smart object layer above the active one and returns `{layer, scale, bounds}`. `path` resolves beneath the read root, so this is the capability-scoped `file.placeEmbedded` — that command itself still refuses an ambient path |
| `doc.exportLayers` | `{dir, format? (png), prefix?, visibleOnly? (true), quality?, index?}` → `{files, count}`: one file per exported layer, named like `file.export.layersToFiles` names them. `dir` resolves beneath the write root, so the ambient command stays refused. the bytes come from the command's own core, so both paths write the same files; the write capability refuses to overwrite an existing file |
| `doc.exportDataSets` | `{dir, format? (png), dataSets? (all), naming? ("{name}\|{index}\|{document}"), index?}` → `{files, count}`: the capability-scoped `file.export.dataSetsAsFiles`. the batch is planned first and colliding names are refused before any file is written, as in the command |
| `doc.importDataSets` | `{path, delimiter? (,), index?}` → `{imported}`: the capability-scoped `file.import.variableDataSets`; the CSV is read beneath the read root |
| `doc.save` | `{path?, format?, quality?, index?, tiffLayers?, tgaBits?}` (`.pcraft` native, else export by extension; set `tiffLayers: true` to preserve TIFF layers rather than exporting a flattened TIFF; `tgaBits` 24 or 32 sets a TGA's bits per pixel, as `app.save` does). without `path` only a PSD, PSB or `.pcraft` document is written back to its own file, in its own format; anything else is an error and the file is left unchanged. a successful layered save (PSD, PSB or `.pcraft`) records the current revision as saved and makes the file the document's path, so `session.list` reports `dirty: false`; a flat export is a copy and leaves the document dirty |
| `doc.inspect` | `{index?}`: same JSON as `document.inspect` |
| `doc.render` | `{index?, maxSide? (1024; 0 = full), path?}`: PNG to `path`, else `{mime, base64}` |
| `doc.select` / `doc.close` | `{index}` / `{index?}` |
| `batch` | `{steps: [{command, params?, wait?} \| {method, params?}], stopOnError? (true)}` → `{completed, failed, results}` (a step with `wait: false` starts a long command as a background job, like `engine.execute`) |
| `methods` | the list above |

```sh
printf '%s\n' \
  '{"id":1,"method":"doc.open","params":{"path":"in.jpg"}}' \
  '{"id":2,"method":"batch","params":{"steps":[{"command":"image.adjustments.invert"},{"command":"filter.blur.gaussianBlur","params":{"radius":3}}]}}' \
  '{"id":3,"method":"doc.save","params":{"path":"out.png"}}' | \
  brahma-cli serve --automation-read-root /work/project --automation-write-root /work/project
```

the MCP server has the same batching as the `command_batch` tool (`{steps:[{id, params, wait?}], stop_on_error}`),
in headless and bridge mode. a step with `wait: false` returns `{job, pending}` at once, as `command_run`
does; later steps that edit the same document fail with a message naming the job until it ends.

## transport limits

the desktop and headless TCP listeners currently enforce:

- a 1 mib maximum encoded request line;
- an 8 mib maximum encoded JSON reply, including the newline;
- at most 16 simultaneously serviced connections per listener;
- a 30-second socket read/write timeout;
- at most 256 steps in a headless `batch` or MCP `command_batch` request.

an oversized line, excess connection, unauthenticated request, or unauthorized filesystem path is
rejected before command dispatch or file effects. the headless JSON-lines stdio server also
enforces the request and reply byte ceilings; MCP over stdio (`brahma-cli mcp`) does not
cap request bytes, because the MCP SDK reads its own request lines (only MCP tool results are
checked). a TCP connection is closed after an oversized
line; on stdio an oversized or non-UTF-8 line gets one error reply (`id: null`), the rest of that
line is skipped without being dispatched, and the session and its open documents keep serving. MCP tool results are checked as encoded JSON,
including the text/image content envelope, and the MCP bridge bounds incoming desktop replies.
a `batch` or `command_batch` stops at the first step whose result no longer fits the reply budget
(that step may have run; later ones do not). MCP charges each result at its size escaped inside
the text content, so the reply still lists `completed`, `failed` and every result so far, ending
with the budget error.

headless automation previews (`doc.render` / MCP `doc_render_preview`) allow a maximum requested
edge of 2048 pixels and a source document of at most 67,108,864 pixels. `maxSide: 0` (MCP
`max_side: 0`) still means full size, but fails if the document's longest edge exceeds 2048.
PNG results are capped at 5 mib before base64 encoding or writing a rendered file. preview
dimension/source checks run before compositing; the encoded-PNG check runs after encoding.
explicit trusted-local CLI rendering keeps its existing behavior.

batch replies have an aggregate byte budget with space reserved for the outer reply and id.
exhausting it stops later steps even when `stopOnError` is false. a result can exceed the reply
budget after an edit has run: the error says the operation may have completed. earlier steps
are not rolled back; inspect state before retrying. the MCP bridge drops an oversized incoming
reply without automatically retrying the operation. bridged screenshots are decoded with
8192-pixel edge, 16,777,216-pixel, and 64 mib allocation ceilings, even without downscaling.

these ceilings do not implement explicit JSON-depth policy, total session/document-memory
accounting, compositor scratch-space accounting, command cancellation/duration limits, or
general per-method capabilities. desktop screenshot capture/encoding and document import/export
still need their own operation budgets; the desktop reply ceiling applies after the ui creates
its response. a bounded output does not imply bounded command cost.

### rendering modes

`performance.renderingMode` accepts `auto`, `gpu`, or `cpu`. automatic is the default for new
settings; older `useGpu: false` or `gpuBackend: cpu` preferences continue to select CPU mode
until an explicit mode is saved. changes apply at the next launch. automatic and GPU both
fall back on graphics errors rather than risk documents. CPU compatibility composites images
on the CPU and prefers software window adapters, when available. macos still uses metal for
the window. a failed window renderer initialization retries once in CPU compatibility mode;
a driver process crash is detected by the startup marker on the next launch.

### camera raw dialog

on the desktop the dialog is a window of its own (title bar, moved and resized like any window), as adobe camera raw is; the main window is dimmed and takes no input while it is open, and closing the window is cancel. where the platform has no extra windows (the web build) it is drawn over the main window.

**screenshots:** `ui.screenshot` captures the main window only, and eframe cannot capture an
immediate viewport, so while camera raw has its own window the PNG shows the dimmed main window,
not the dialog. read the dialog's state through `ui.inspect` (`cameraRaw`: params, histogram,
zoom, control rectangles) and drive it with the `ui` params below. to *see* it, render offscreen
(`cargo run -p brahma-ui-egui --example snapshot -- --script '[["ui.menu.invoke", {"id": "filter.cameraRaw"}]]'`
or an `egui_kittest` harness): those embed
viewports, so the dialog is drawn over the main window and appears in the image.

`ui.menu.invoke {"id":"filter.cameraRaw","params":{}}` opens camera raw on the active
RGB/grayscale layer. `params: {"smartFilter": {"layer": id, "index": i}}` opens it on an existing
camera raw smart filter instead (as double-clicking the filter in the layers panel does): the
stored settings over the pixels below that filter, previewed through the filter mask; commit
then runs `layer.smartFilter.setParams` with every setting. `params.ui` accepts `set` (filter settings), `before`, `scope`, `view`, `commit` and
`cancel`. all parts of one request are validated before any is applied; a rejected request
leaves the settings, view state, preferences and document unchanged (and closes a dialog it
opened). unknown `ui` or settings properties, non-boolean `before` / `commit` / `cancel`, and
`commit` together with `cancel` are errors. point curves (`pointCurve`, `pointCurveRed`,
`pointCurveGreen`, `pointCurveBlue`) are empty or 2–16 finite points in 0–255 with inputs at
least one level apart. commit dispatches one `filter.cameraRaw` engine command; a failed commit
keeps the dialog open for correction. nothing else writes document history.

**opening a raw file.** an interactive open (file › open, open recent, drag and drop, the
command line) of a camera raw developed from its sensor data shows this dialog first, titled
"camera raw (name)", with **open** and **cancel**, as photoshop opens raws in adobe camera raw.
`ui.inspect.cameraRaw.openingRaw` is then `{name, path}`. commit (open) re-develops the raw when
temperature, tint, exposure, shadows, highlights, clarity, texture or dehaze changed (as-shot
white-balance gains, develop exposure, and the other five in linear light before the camera's
tone curve, as camera raw does for raw files; without an as-shot white balance, or without the file's bytes, they stay RGB
adjustments), applies the remaining settings as one `filter.cameraRaw` step and leaves the
document unmodified; its result is `{document, redeveloped, filter?}`. cancel closes the
document. `app.open` and other automation opens never show the dialog. the preference
`rawDefaults.openInCameraRaw` (default `true`) turns it off.

imported PSD camera raw filters whose processing settings are all mapped or neutral use this
same editor. `params.__cameraRawPsd` is reserved import/export metadata: preserve it when editing
stored parameters. other processing fields or versions remain opaque photoshop filters. see
[PSD smart filters](camera-raw-histogram.md#psd-smart-filters) for the supported settings and export
limits.

the response and `ui.inspect.cameraRaw` contain:

- `histogram`: `source` (`before` | `after`), `size`, `approximate`, 256-bin `red` / `green` /
  `blue`, `samples`, `transparent`, `invalid`, per-channel `underflow` / `overflow`, and exact
  endpoint counters `shadows` (≤0) / `highlights` (≥1). counts describe the bounded preview
  proxy in its RGB sample domain, not full-resolution or ICC display-gamut statistics.
- `previewRevision` (advances only when the proxy is re-developed), `renderMs`, `histogramMs`
  (CPU only; zero on WebAssembly).
- `curveState` (`selected`, `drag`) and `curveRect: [left, top, right, bottom] | null` (null while
  the curve section is closed). changing `set.pointCurve` cancels a stale gesture.
- `scope` (the view state below), `hasScopeSelection`, `vectorscope` (`bins`, `samples`,
  `before`, `selectedRegion`) or null, `scopeMs`, `scopeRevision`, `overlayRevision`.
- `pointerReadout` and `samplerReadouts`: `position`, `space` (`rgb` 0–255 in the preview sample
  space, or `lab` through the document ICC profile → D50), `values`, `alpha`; null on invisible
  pixels.
- `hoverSample`: `position`, `rgb` (histogram domain) and `hueSaturation` (turns, 0–1; only
  while the vectorscope is shown), or null.
- `hoveredZone`, `previewRect`, `scopeRect`, `vectorscopeRect`.
- `view` (`zoom: null` for fit, or a numeric factor where 1 = 100%, `center: [u,v]`,
  `hand`), `zoom` (effective factor, null before layout), `sourceSize`, `viewportRect`.
  at 100%, one source pixel maps to one physical display pixel, including on hidpi displays.
- `previewApproximate`, `detailPending`, `detailError`: whether the visible image still uses
  the proxy, whether full-resolution refinement is running, and any refinement error.

`params.ui.view` changes navigation only: `zoom: "fit" | 0.0001..=16`, `center: [u,v]`
(finite coordinates in 0..=1, clamped to keep the image within view), `hand: bool`. fit recentres
and follows window resizing; navigation resets when opening a different camera raw dialog.
invalid mixed requests are atomic, just like scope and filter settings. view requests do not
re-develop the proxy or change filter parameters/history.

with the zoom tool (z), click toggles fit / 100%; click-drag right/left scrubs in/out.
ctrl-drag (command on macos) fits a rectangular region. h selects hand; space temporarily pans,
including while placing colour samplers. alt/option + wheel zooms at the pointer; an unmodified
wheel pans. pinch zooms. ctrl/command +/- zoom, 0 fits, and alt/option + ctrl/command + 0 shows
100%; ctrl/command+shift-click also selects 100%. right-click opens fit / 100% / preset
percentages. tool double-clicks fit the image.
text fields retain their keys and wheel gestures over the settings panel still scroll it.

the proxy remains the fast path while changing settings. where more source pixels are needed,
camera raw later refines with the **same engine camera raw pipeline** at pixelScale=1 and the
same selection/filter-mask coverage. native refinement runs off the ui thread (one revision at
a time); stale results are discarded. only a bounded visible crop is uploaded to the GPU, and
pan/zoom reuse developed pixels. before and neutral settings read the original pixels directly.
absurd proxy domains (over 512 mp, including distant sparse off-canvas pixels) are rejected
before reading pixels. full-resolution refinement is capped at 64 mp to bound the existing full-image engine pipeline;
above that it explicitly reports unavailability and keeps the proxy. oversized viewport crops
also retain the proxy. without a browser worker, wasm explicitly retains the filtered proxy
rather than block its ui; before and neutral settings still crop the original source pixels.
the histogram and vectorscope continue to analyse the bounded proxy; readouts and clipping
warnings use the full-resolution pixels once refinement is visible.

`params.ui.scope` changes presentation only: `shadows` / `highlights` (clipping warnings, u / o),
`lab`, `samplerTool` (s), `vectorscope`, `selectedRegion` (requires a document selection),
`redRight`, `hideSkinLine`, `floating`, `floatingRect: [left, top, right, bottom]` (200×150 to
4096×4096 points), `sample: [u,v] | null`, and the probe list: `samplers: [[u,v], …]` replaces it,
or `clearSamplers: true`, `removeSampler: index` and `addSampler: [u,v]` edit it, applied in that
order (the two forms can't be combined). coordinates are normalized to the displayed proxy; at
most nine probes. `tone: {zone: "blacks|shadows|exposure|highlights|whites", delta}` adjusts the
same parameter as a histogram drag (`reset: true` sets it to 0); exposure clamps to ±5 stops, the
others to ±100.

hover, probe movement, theme changes and panel resizing never re-develop the image; before/after
and region changes rebuild only the dependent analysis. display options and the floating panel
geometry persist in preferences `dialogs["filter.cameraRaw.scope"]`; probes and vectorscope
visibility reset when the dialog opens. HDR scopes are not implemented. see
[camera-raw-histogram.md](camera-raw-histogram.md).

### editing and composing actions

newly recorded select layer steps store the layer name instead of a document-specific id,
and the actions panel shows `Select Layer (Background)`. duplicate names include a zero-based
`occurrence` in document walk order (including layers inside groups); keep that order consistent
between source and target documents. a missing name or occurrence stops playback; a name without
an occurrence is rejected when ambiguous. names are captured at selection time, so a later
rename in the action does not change the earlier target. numeric `layer` ids remain supported
for direct commands and older recordings, which the panel labels `Select Layer (ID 26)`.
string-valued `layer` targets from imported photoshop actions keep main's topmost-name matching
and show the same named label; `remove` selection mode is supported for either target form.
re-record old select layer steps: their original names cannot be recovered from an id alone.

creating an action scrolls its row into view; newly recorded steps are revealed immediately.
drag a step above or below another step in the same action to reorder it. drag an action header
to reorder the action list. the insertion line marks the drop position; dragging near the rows'
top or bottom scrolls the list. reordering is saved with the actions and does not undo image edits.

while recording, press a function key assigned to another action (or play it from the panel).
the recorder adds **play action: <name>** as one step, and the called action still executes.
playback resolves the name each time, so editing or moving the called action does not freeze or
retarget the caller. reassigning f6 later does not change an already recorded named call.
missing actions, recursive calls and nesting beyond 16 levels fail visibly; a child failure stops
the parent before its next step. every nested command retains the normal automation permission check.
existing recordings containing expanded commands remain unchanged; remove those steps and record
the named call again if desired.

actions sit in **set folders** (`set` on `actions.list` / `actions.get`, filled by load actions,
written by save actions, changed with `actions.assignSet`): the panel shows loose actions first,
then each set as a collapsible folder (`ui.actions.collapsedSets`); dropping an action among
another set's rows moves it into that set. each step has photoshop's two toggles: the check
(`actions.toggle`, skipped when off) and the **dialog** toggle (`actions.dialog`). playing a
step with its dialog on opens that command's dialog seeded with the recorded values and pauses:
the play result carries `paused: {step, id, dialog}`, ok runs the dialog's values and continues
from the next step, cancel stops the action. agents (`automation_input`) play headlessly with the
recorded values. the panel menu's **insert stop…** adds an `actions.stopMessage` step after the
selected step (`actions.insert` takes any command id and params). `actions.conditional {if, then,
else}` is photoshop's insert conditional as a step: it tests the document or active layer
(`always`, `landscape`, `modeIsRGB`, `hasLayers`, `isBackground`, `hasPixelMask`… see its params)
and plays the matching action by name, failing the caller when that action fails.

layer-panel **quick export as PNG** exports every selected layer as a separate PNG. a single
save dialog chooses the first filename; remaining files are numbered siblings with sanitized
layer names. selection is captured before opening the dialog; an unselected layer is excluded.

## shared panel docking

panel arrangements support tab grouping and reordering, horizontal and vertical splits,
and movable, resizable floating groups inside the application window. drag a panel tab,
or use its context menu. floating groups remain within the application viewport.

the shared layout begins with the existing workspace on the first docking operation.
workspace saves retain its groups, active tabs, split sizes, floating geometry, and the
positions of hidden panels. resetting a built-in workspace restores its default dock.

- `window.panel.move {panel, anchor, zone?, before?}` moves a panel into the target group.
  `zone` is `tab` (default), `center`, `left`, `right`, `top`, or `bottom`; `before` reorders
  tabs by panel id. the destination must already be visible.
- `window.panel.float {panel, x?, y?, width?, height?}` detaches a panel.
- `window.panel.dock {panel, anchor?, zone?, before?}` returns a panel to its previous
  dock placement when possible, or to the supplied destination.
- `window.panel.activate {panel}` reveals and selects a panel, restoring its saved
  position when possible. `window.panel.close {panel}` hides it.

panel ids are stable across themes. invalid ids, non-finite or invalid geometry, and
invalid destinations return an error without changing the arrangement.

lock workspace prevents moving, detaching, and closing panels; activating a panel remains available.

`window.panel.layout {action}` also accepts the shared, externally tagged action schema.
for example, `{"action":{"MoveFloating":{"panel":"properties","rect":[44,66,360,450]}}}`
updates a floating group's position and size; `{"action":{"ResizeSplit":{"path":[],"size":{"Ratio":0.35}}}}`
resizes the root divider (`false`/`true` path entries select first/second children).
`Move` with `placement: {"Tab":{"before":"swatches"}}` reorders tabs, and
`SetStackOpen {panel,open}` / `ResizeStack {panel,height}` control accordion entries.
the ui dispatches these same commands. unknown panel ids, invalid geometry, and stale
split paths return an error without changing the workspace.

the existing `ui.set` panel visibility, `dockTabs` selection and `dockWidth` controls
remain available after customization. width keeps its 250–520 point clamp. named
`dock.hidden_tabs` and intact native-group collapse settings are also preserved. a
legacy `dock` replacement that changes flat group order or heights is rejected after
customization: use `window.panel.layout` to change a custom split tree. a replacement
that changes legacy collapse settings is also rejected when a custom root group mixes
panels from different native groups; use `window.panel.group` to collapse or expand
that group. validation is atomic across mixed `ui.set` requests.

`window.panel.group {panel, operation}` applies `close`, `collapse`, `expand`, `up`,
or `down` to the panel's entire current group. with `operation:"move"`, provide
`anchor` and optional `zone`/`before` (as for panel.move), or a serialized `placement`,
to transfer all members atomically while preserving their order and selecting the
requested `panel`.
the floating group's header grip also transfers the whole group when dropped on a
dock target. the panel menu and icon rail remain
available in custom workspaces. `window.panel.<id> {show:true}` always reveals, selects
and expands the panel, including when another tab is selected. workspace snapshots
retain custom collapse settings and timeline visibility.
