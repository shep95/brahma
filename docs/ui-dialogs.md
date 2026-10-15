# dialogs

modal and modeless parameter windows in the egui shell. they mirror photoshop: no dimmed backdrop,
canvas pan/zoom under an open dialog, tab cycles text and number fields only, esc cancels, enter
confirms when ok is enabled. automation drives the same `fields` map as the ui (`ui.dialog.*` in
[`control-protocol.md`](control-protocol.md)).

## architecture

| piece | location | role |
|---|---|---|
| frame, routing, ok/cancel/esc, stacking | `crates/ui-egui/src/dialogs.rs` | single hub for every `UiState::dialogs` entry |
| alt+cancel → reset (opt-in) | `crates/ui-egui/src/dialog_alt_reset.rs` | relabels cancel while alt is held; dispatches per-marker `reset` |
| tab-through-fields | `crates/ui-egui/src/field_tab.rs` | photoshop-style tab / ⇧tab (tests in same file) |
| state | `crates/ui-egui/src/state.rs` | `Dialog`, `DialogKind`, `open_dialog` |
| button row | `crates/ui-egui/src/widgets.rs` | `dialog_buttons`, `focus_first_field`; `number_edit` registers for tab |
| menu / command entry | `dialogs::open_command_dialog`, `menus.rs` | picks a dedicated module, schema dialog, or bare confirm |
| filters | `filter_dialog.rs` | schema from engine params, live preview |
| adjustments | `adjust_dialog.rs` | shared editors with properties; preview via `adjust_preview` |
| generic forms | `view_cmds.rs` | `__form` grids for simple command params |
| dedicated bodies | see **command routing** below | `body` + usually `open` / `confirm` |

**routing.** non-command kinds use `DialogKind` alone. `DialogKind::Command` bodies are chosen by
marker keys on `fields` (`owns()` or `contains_key("__export")`, etc.); **first match in
`dialogs::show` wins**, so new markers must not collide with an earlier arm. `show`, `confirm`, and
`cancel` share the same dispatch table—wire all three when adding a dialog.

**field contract.** `Dialog.fields` is a JSON map: command parameters agents read and write, plus
private keys prefixed with `__`. never use `__` for user-facing or engine params.

### `DialogKind` (top-level stack)

| kind | body module | typical entry |
|---|---|---|
| `NewDocument` | `new_doc_ui.rs` | file › new, `file.new` with no size params |
| `About` | `dialogs.rs` (about / contributors / models tabs) | help › about brahma; `systemInfo: true` → system info variant |
| `Command` | routed—see below | most menu commands with parameters |
| `LayerStyle` | `layer_style.rs` | layer › layer style… |
| `Error` | inline in `dialogs.rs` | `message` field only (e.g. move lock, locked transform) |

### command routing (`DialogKind::Command`)

dispatch order matches `dialogs::show` in `dialogs.rs` (check earlier rows before adding a marker).

| match | module | role |
|---|---|---|
| `__guideLayout` | `guide_layout_ui.rs` | view › new guide layout… |
| `__fill` | `fill_ui.rs` | edit › fill… |
| `__patternFill` | `pattern_fill_ui.rs` | pattern fill layer / new fill layer › pattern… |
| `__stroke` | `stroke_ui.rs` | edit › stroke… |
| `__createShape` | `shape_dialog.rs` | click-with-shape-tool creation (rectangle, ellipse, …) |
| `__deleteLayer` | `delete_layer_prompt.rs` | layers panel trash confirmation |
| `__newLayer` | `new_layer_ui.rs` | layer › new › layer… (⇧⌘n; ⌥⇧⌘n makes the layer without it) |
| `__rasterize` | `rasterize_prompt.rs` | rasterize-before-paint prompt |
| `__resetAllTools` | `tool_reset.rs` | reset all tools confirmation |
| `__cropPresetNew` / `__cropPresetDelete` | `crop_presets.rs` | new / delete crop preset |
| `__variables` | `variables_ui.rs` | image › variables (define, data sets, apply) |
| `__web` / `__print` | `file_ui.rs` | save for web, print |
| `__colorPicker` | `color_picker_ui.rs` | foreground/background, nested field/command pickers (see nested keys below) |
| `__colorRange` | `color_range_ui.rs` | select › color range… |
| `__prefsui` | `prefs_ui.rs` | edit › preferences… and related preset/import subdialogs |
| `__export` | `export_dialog.rs` | export as (document or layer) |
| `__sizing` | `sizing.rs` | image › image size… / canvas size… |
| `__adjust` | `adjust_dialog.rs` | image › adjustments… (also sets `__filter` + `__preview` for live preview) |
| `__filter` | `filter_dialog.rs` (+ `select_mask_view.rs` when `__command` is `select.refineEdge`) | filter gallery commands, edit › color settings…, select and mask, and other schema dialogs |
| `__form` | `view_cmds.rs` | simple param grids (e.g. slice options, many image › apply image-style commands) |
| (none of the above) | — | **bare confirm**: title + ok/cancel only; ok runs `__command` with fields via generic `confirm` |

**menu shortcut.** `dialogs::open_command_dialog` opens guide layout, stroke, and color range
directly; then tries `adjust_dialog::open`, then `filter_dialog::open`; otherwise a bare confirm
with `__command` + `__label`. other dedicated modules are opened from `menus.rs`, tools, or panel
actions with their marker already set.

### shared private keys

| key | meaning |
|---|---|
| `__command` | engine command id run on ok (bare confirm, filters, sizing, …) |
| `__label` | menu/catalog label for the title |
| `__document` | target document id (validate staleness) |
| `__dialog` / `__field` | nested color picker: parent dialog id and field name to write on ok |
| `__params` | extra command params for command-targeted color pickers |
| `__preview` | live canvas preview enabled (filters and adjustments) |
| `__choices` | dropdown options for `__form` dialogs |
| `__filter` | filter-schema body (`filter_dialog`); also set on adjustments for preview plumbing |
| `__adjust` | image › adjustments editor kind (`adjust_dialog`) |
| `__form` | generic `view_cmds::form_body` |
| `__formOrig` | snapshot of non-`__` params at open; alt+reset restores it for `__form` dialogs |
| `__variablesOrig` | snapshot of `__variables` at open; alt+reset restores it |
| `__altReset` | set to `false` to opt a dialog out of alt+cancel → reset |
| `__export` | export as |
| `__sizing` | image size / canvas size |

### routing markers (dedicated `Command` bodies)

| key | module |
|---|---|
| `__guideLayout` | `guide_layout_ui` |
| `__fill` | `fill_ui` |
| `__patternFill` | `pattern_fill_ui` |
| `__stroke` | `stroke_ui` |
| `__createShape` | `shape_dialog` |
| `__deleteLayer` | `delete_layer_prompt` |
| `__newLayer` | `new_layer_ui` |
| `__rasterize` | `rasterize_prompt` |
| `__resetAllTools` | `tool_reset` |
| `__cropPresetNew`, `__cropPresetDelete` | `crop_presets` |
| `__variables` | `variables_ui` |
| `__web`, `__print` | `file_ui` |
| `__colorPicker` | `color_picker_ui` |
| `__colorRange` | `color_range_ui` |
| `__prefsui` | `prefs_ui` (string sub-kind: `prefs`, `presetsIO`, …) |

### nested color pickers (`__colorPicker` on the stack)

same hub entry as standalone pickers; `color_picker_ui::confirm` branches on extra keys:

| extra key | behaviour |
|---|---|
| `__layerStyleColor` | layer style child; ok writes effect colour on parent `LayerStyle` dialog |
| `__textStyleColor` | character/paragraph style colour (type panels) |
| `__shapeFill` | shape layer solid fill via `shape.edit` |
| `__command` = `layer.fill.solidColor.set` | solid color fill layer (no `__shapeFill`) |
| `__dialog` + `__field` | ok updates one field on a parent command dialog (e.g. fill › color…) |

## parallel modal systems (not the main stack)

| system | location | when |
|---|---|---|
| unsaved changes | `discard_ui.rs` | close, revert, close others/all, exit—save / don't save / cancel |
| shell dialog | `control.rs`, `ui.inspect` id `"shell"` | workspace, custom pixel aspect, show-extras options |
| camera raw | `camera_raw_ui.rs` | separate os window on desktop; see control-protocol § camera raw |
| error message | `DialogKind::Error` | e.g. `move_lock`—message + ok |
| nested pickers | `color_picker_ui` (+ markers in **nested color pickers**) | stack on parent; parent confirm refused while child open |

## rules

### architecture and commands

1. **thin ui, engine command on ok.** run `app.run` or a dedicated `*_ui::confirm`; one history step unless the command itself batches.
2. **register in the hub.** add `owns()` (or a stable marker), `body()`, `open()` if needed, and matching arms in `dialogs::show`, `confirm`, and `cancel` (and `open_command_dialog` when menu-driven).
3. **extend existing paths first.** filters → `filter_dialog`; adjustments → `adjust_dialog`; simple param grids → `__form`; avoid a second modal stack unless product needs camera raw–level ux.
4. **`fields` is the automation contract.** anything agents must drive lives in named fields, not only in local egui state.
5. **remember last-used values** after successful ok where photoshop does (`filter_dialog::remember`, prefs `dialogs["…"]`, module-specific helpers).

### chrome and layout

6. **use `dialog_buttons`** for ok / cancel / apply—platform order via `ButtonRole` (macos default action in the corner).
7. **respect width rules** in `dialogs::show` unless `file_ui::dialog_width`, color picker, or kind-specific mins apply (typical 380–440; layer style / export / picker up to 600; new document 800).
8. **scroll long bodies** inside the content (`DIALOG_CHROME` for filters); title and button row stay visible.
9. **transparent backdrop**—no dimming; previews are judged at true contrast.
10. **position:** first open centered; then persist offset (`place_id` per command or kind); title bar drag; clamp on screen.
11. **stacking:** only the **top** dialog is interactive; non-top instances render disabled; raise the live dialog if an older modal steals focus.

### keyboard and focus

12. **tab / ⇧tab:** text and number fields only. frame calls `field_tab::begin` / `end`; each field calls `field_tab::register` (`number_edit` does this; custom `TextEdit` must register manually).
13. **do not rely on egui default tab** through sliders, dropdowns, checkboxes, or preset cards.
14. **open on first value:** `first_field_focused` + `widgets::focus_first_field` for value-first dialogs (most filters; image/canvas size; not select and mask view keys).
15. **select-all on focus** when reached via tab or field arrows (`field_tab::select_all`; see `value_field`, `new_doc_ui`).
16. **enter** confirms when ok is enabled (`can_ok`); exceptions must be explicit (`guide_layout_ui`, color picker).
17. **esc** on topmost dialog with no popup open → cancel. **click outside does not dismiss** (canvas may still pan/zoom via `free_pointer_over`, `pan_delta`, `free_press`). esc ignores alt (it does not become reset).
18. **alt** turns cancel into **reset** when the dialog opts in via `dialog_alt_reset::supported` (dialog stays open). opt-in markers: `__guideLayout`, `__fill`, `__variables`, `__colorPicker`, `__sizing`, `__adjust` (all kinds), `__filter`, `__form`. each module implements its own `reset`; set `"__altReset": false` to disable.

### canvas and preview

19. **live preview** for filters/adjustments uses `filter_preview` / `adjust_preview`; `cancel()` clears preview and related shell state.
20. **under-dialog canvas** respects dialog rects and open popups; hand / space / middle-button pan on free canvas only.

### labels and i18n

21. **titles:** command dialogs use `__label` (catalogued; trailing `…` stripped in chrome); fixed kinds use `dialogs::title` + `tl!`.
22. **user-visible strings:** `tl!("…")`; command labels from the menu catalog where possible.

### confirm / cancel safety

23. **child dialogs first**—refuse parent confirm if a nested color picker is open; cancel parent closes children where implemented.
24. **validate before close** when params can be inconsistent (`guide_layout_ui::validate` in `confirm`).
25. **disable ok** when required input is missing (`crop_presets::can_confirm`, preferences apply when unchanged).
26. **preferences over control:** whole-dialog ok/apply may be refused; safe single prefs via `prefs.set` (see control-protocol).

### testing

27. **egui_kittest:** open dialog, `run_steps`, key presses, assert `fields` or accessibility roles (`field_tab` tests are the reference harness).
28. **tab-order regressions:** add tests when a dialog gains multiple text/number fields.
29. **filter preview:** `dialogs.rs` tests for preview persistence across background jobs.

### anti-patterns

30. no panics in `body` / `confirm` on bad input—errors to status or validation.
31. no hard-coded colours or radii—`theme::Tokens` and `widgets.rs`.
32. do not skip `confirm` dispatch—enter and `ui.dialog.confirm` use it.
33. do not expect click-outside-to-close on `egui::Modal`.
34. do not register sliders or dropdowns for `field_tab` unless product explicitly changes photoshop parity.

## adding a command dialog (checklist)

1. choose path: see **command routing** (`adjust_dialog`, `filter_dialog`, `__form`, `__sizing`, …, or new `*_ui` with a unique marker).
2. `open`: build `fields` (`__command`, `__label`, params, marker); `app.ui.open_dialog(DialogKind::Command, f)`.
3. `body(app, ui, &mut fields)`—mutate `fields` only; use shared widgets.
4. `confirm` / `cancel` arms in `dialogs.rs` (or delegate to module `confirm`).
5. tab: register every text/number field; use `value_field` / `number_edit` where possible.
6. **alt+reset (optional):** implement `reset` on the module and add an arm in `dialog_alt_reset` (same dispatch order as `dialogs::show`). snapshot open values under a `__*Orig` key when reset should restore “as opened” rather than factory defaults.
7. tests: at least open + confirm or tab sequence if non-trivial; if alt+reset is supported, assert reset restores fields and leaves the dialog open.
8. if menu-visible: command already in engine; menu id opens dialog via existing menu wiring.

## known gaps

- **tab registration** for plain `TextEdit` is manual; generic `__form` string fields may not join `field_tab` unless wired.
- **“dialog module”** is any ui routed from `dialogs::show`, not only files named `*dialog*`.
- **numeric field parity** (units in expressions, scrubby labels) is tracked in [`ui-parity.md`](ui-parity.md) (navigation / numeric fields), not duplicated here.

## revision history

| date | change |
|---|---|
| 2026-10-11 | initial doc from dialog codebase audit |
| 2026-10-11 | document all `DialogKind` values and `Command` routing markers |
| 2026-10-11 | alt+cancel → reset hub (`dialog_alt_reset`); opt-in markers |
