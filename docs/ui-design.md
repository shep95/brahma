# ui design system

## themes

| theme | intent |
|---|---|
| pro | photoshop-style spectrum dark: flat charcoal panels (#323232), dark tab strips, spectrum blue accent (#378ef0), pill buttons, checkboxes, compact 12 px type |
| pro medium | photoshop-style medium-gray panels and a dark canvas |
| studio | dark studio style: near-black, rounded cards, pill tabs, violet accent, toggles |
| studio light | studio on light surfaces |
| classic | windows-2000 bevels, square corners, navy selection |
| adwaita | GNOME's libadwaita light palette on the studio layout: white header bar and cards on a grey sidebar, GNOME blue accent (#3584e4), 6 / 9 / 12 px radii, grey (not red) close button |
| adwaita dark | the libadwaita dark palette on the same layout |
| solarized dark | ethan schoonover's solarized palette: base03 canvas, base02 panels, base0 text, solarized blue accent |
| breeze light | an adaptation of KDE plasma's breeze light on the studio layout: cool light blue-grey surfaces a few levels from breeze's so the app blends into a plasma desktop, breeze-style blue accent, 5 px radii, red close button on hover; text and status colours are darker than breeze's for contrast |
| breeze dark | the same for breeze dark: cool dark blue-grey surfaces (darkest canvas, then dock, cards, chrome and controls) near breeze's, blue accent, 5 px radii; secondary text and status colours are lighter than breeze's for contrast |
| soft dark | low-contrast dark on the studio layout: muted violet-grey surfaces (never near-black), softened off-white text, periwinkle accent, generous 7/10/14 px radii; text still holds 4.5:1 on every surface (`theme.rs` tests) |
| soft light | the light counterpart: pale violet-grey surfaces, softened dark-violet text instead of black, periwinkle accent, the same radii |

both breeze themes are our own values close to plasma's (not a copy of its colour schemes) and put text readability first: every text colour holds 4.5:1 on every surface and on hovered, pressed and selected states (`theme.rs` tests), so their secondary text, accent and status colours are darker (light) or lighter (dark) than breeze's own, and the primary button is a deeper blue with white text.

edit → preferences → interface shows an appearance mode selector (sync with system, dark, light) above separate light and dark theme cards with brahma editor previews and radio choices. auto is opt-in and follows the operating system while the app is running. on linux it reads the desktop portal once and then listens for its `SettingChanged` signal (no polling, and the ui repaints only when the value changes); `gsettings` runs at most once at start-up, by absolute path, when the portal gives no answer. a missing system appearance falls back to dark. new installs keep photoshop's default: dark mode with pro medium (pro light is the saved light theme). existing saved single-theme preferences migrate to a fixed dark or light mode with their chosen theme. the header appearance button cycles auto → light → dark; its icon shows a monitor, moon or sun for the selected mode. window → theme and `ui.set {"theme":"classic"}` select a theme and fix the mode to its light or dark family.

window → theme and macos brahma → appearance also offer sync with system. its checkmark follows the saved auto mode rather than the displayed palette; choosing a manual theme selects its fixed appearance mode.

the toolbar's foreground and background colour chips follow the `Tokens::round_chips` flag. pro and classic leave it off and draw photoshop's overlapping squares with default colors and switch colors above them. studio, studio light, solarized dark, adwaita, adwaita dark, breeze light, breeze dark, soft dark and soft light (the studio-layout themes) turn it on and draw large round chips with a curved switch colors arrow centred right under them, its heads touching the chips: stacked vertically in one tool column, with default colors at their top-right, and side by side in two tool columns, with default colors at the toolbar's left edge. another non-pro theme can opt in by setting the flag.

## rules

- **colours and radii come from tokens.** read them with `Tokens::get(ctx)`; never hard-code a colour in a widget.
- **shared widgets live in `widgets.rs`:** `card` (panel group; pro renders a photoshop tab strip), `value_field`, `slider`/`slider_row`, `toggle`/`checkbox`, `primary_button`/`secondary_button`, `dropdown` (with a chevron icon), `hairline`/`vline`, `elided_label`.
- **a label never overflows its row.** wrap by default. beside a fixed-width control, or anywhere the row cannot reflow, use `widgets::elided_label(ui, text, max_w)`: a longer translation shrinks with `…` and shows the full string on hover instead of running over its neighbours (#3580). a *redundant* caption may be dropped on a narrow dock where its control already carries the meaning (e.g. the layers lock icons); an *essential* one elides. this mirrors photoshop: a fixed layout with elision and tooltips, not per-locale reflow.
- **icons:** lucide svgs in `assets/icons` (ISC licence), embedded via `icon_data.rs`. regenerate that file when you add icons.
- **fonts:** inter (ui) and JetBrains mono (numbers), both OFL. named families `medium` and `semibold` are available via `theme::medium()` and `theme::semibold()`.
- **photoshop layout grammar (pro):**
  - essentials dock order: color | swatches, then properties | adjustments, then layers | channels | paths (layers fills the remaining height).
  - options-bar labels end with a colon ("size:").
  - document tabs read "name @ 12.5% (RGB/8)".
  - toolbar tool groups carry a corner triangle.
- **verify every visual change** with `ui.screenshot`, at several window sizes and in every theme.

## adding icons

```sh
curl -sfL -o assets/icons/<name>.svg https://raw.githubusercontent.com/lucide-icons/lucide/main/icons/<name>.svg
# regenerate the embedded table
{ printf '%s\n' '//! Lucide icons (ISC licence), embedded and tinted at runtime.' '' 'pub static ICONS: &[(&str, &[u8])] = &['; \
  for f in assets/icons/*.svg; do n=$(basename $f .svg); echo "    (\"$n\", include_bytes!(\"../../../assets/icons/$n.svg\")),"; done; echo '];'; } \
  > crates/ui-egui/src/icon_data.rs
```

## interaction models (match photoshop cc)

| feature | module | behaviour |
|---|---|---|
| type tool | `type_tool.rs` | click: point text with the placeholder "lorem ipsum" selected. drag: paragraph box. inline caret and selection drawn from the text engine layout. ⌥/⌘ word and line navigation, ↩ newline, ⌘↩ or esc commits, a click outside commits. one history step per session (`coalesce`). a new layer is named after its text; an empty one is deleted. |
| free transform | `transform_tool.rs` | ⌘t. corners and edges scale proportionally (⇧ frees them: an edge then stretches one axis); ⌥ scales about the reference point; ⌘-corner distorts; dragging outside rotates (⇧ snaps to 15°); dragging inside moves. preview = document without the moving pixels + a textured 24×24 mesh. ↩ or a double-click commits via `edit.transform {rect, quad}`. |
| quick layer pick | `quick_pick.rs` | macos: ⌘⌥⌃-click with any tool selects the topmost visible layer with pixels under the pointer (`layer.pickAt`) without switching tools; the drag and release are swallowed. windows/linux have no third modifier distinct from the ⌃⌥ brush resize, so the gesture is a no-op there (use the move tool's auto-select). a failed pick is a status-bar error. |
| layers rows | `panels.rs` | double-click: on the name renames in place; on the background makes it a normal layer; on an adjustment or fill thumbnail opens its properties; on a smart object thumbnail opens its contents (edit contents); anywhere else on the row opens layer style. |
| layer masks | `panels.rs` | clicking the mask thumbnail targets the mask (corner-bracket frame; the tab reads "layer, layer mask/8"). brush, eraser (paints background colour), gradient, bucket and moving the selected pixels (⌘-drag, `select.float`; the hole takes the background colour) then send `"target": "mask"`. adjustment and fill layers target their mask automatically. |
| path thumbnails | `vector_ui.rs` | ctrl/cmd-click the thumbnail of a work path, saved path or layer path to load it as a selection; shift adds, alt subtracts, shift+alt intersects. clicking the row name only selects the path. |
| levels / curves | `tone.rs` | histogram of the image *below* the adjustment. curves: click to add a point, drag out to delete. every change is a coalesced `layer.setAdjustment`, so one drag = one undo step and the canvas updates at full resolution on the GPU. |
| shape stroke options | `shape_stroke_ui.rs`, `vector_ui.rs` | direct editing: valid changes apply immediately, with one numeric drag per undo step in properties. escape/outside click closes the popup and retains changes; a child dropdown keeps the parent open. the options bar sets new-shape defaults; properties edits the selected shape. |

shape stroke options (updated 2026-10-11) sizes to its content and scrolls only when necessary.
it has no apply/cancel footer. solid, dashed and dotted are full clickable rows with line
samples and visible selection; opening the editor alone never changes a shape. invalid pending
numeric input leaves the last valid style in effect. fill and stroke use clickable swatches
and the existing compact colour picker, preserving alpha and offering no color. the style
button remains available when no stroke is set; deliberately choosing a stroke colour or
style enables a 3 px stroke. colours, borders and selection treatments come from theme tokens.
these tool defaults are serde state, but are not restored across app restarts. see
[`shape-strokes.md`](shape-strokes.md) for geometry, colour-management and persistence limits.

while editing type, hold ctrl (windows/linux) or command (macos) for an oriented transform
frame. drag inside to move, a corner to scale (shift keeps proportions; alt/option scales about
the reference point), a middle handle to skew, or just outside the frame to rotate (shift snaps
to 15°). drag the reference point to move the rotation centre. these type gestures follow
adobe's type guide, independently
of the free transform preference: ctrl/cmd never distorts a type corner.

without ctrl/cmd, paragraph handles resize the container and reflow its text. temporary
transforms keep its logical dimensions and all character/paragraph styles. a drag begun with
ctrl/cmd owns the pointer until button release, even if the modifier is released first. its
preview does not change the document; release applies one coalesced `type.edit`, and typing
continues in the same layer. escape during a drag cancels that preview; focus loss or changes
to its source also discard it. holding the modifier, moving the reference point, or returning
the pointer to its start creates no history entry. ctrl/cmd+t while typing still toggles the
character panel.

where the font lacks a symbol (e.g. ∠ ↦ ▔), draw it with the painter or use a lucide icon; never ship
missing-glyph boxes. check every new panel with the offscreen snapshot tool (`docs/development.md`).

## title bar

the app's top bar (`panels::title_bar`) starts with the brand mark (the app icon, `brand.rs`), then the menus, the document title and the workspace controls, like photoshop's ps tile and menu row.

- **windows and linux:** the window has no os decorations, so there is one bar, not two stacked. the top bar is the title bar (`titlebar.rs`): minimize, maximize/restore and close (red on hover, `caption_close` tokens) sit flush in the window's top-right corner, the free gap between the menus and the controls drags the window and a double-click there maximizes it, and invisible 5 pt edges (12 pt corners) resize it. close runs file › exit, so unsaved documents are asked about first.
- **macos:** the traffic lights sit over the integrated title strip (`integrated_titlebar`).
- **narrow windows** drop controls that are also in a menu before anything overlaps: the theme toggle (preferences), then search (edit › search), then the workspace switcher narrows (window › workspace).

## compact tool controls

the toolbar scrolls its tool rows when two columns still exceed the available height. foreground
and background colours stay visible below those rows, together with pro's quick mask and screen
mode controls. long options rows scroll horizontally. both scrollers bring newly focused
controls into view and retain the existing tool actions and popups.

the offscreen regression fixture uses production panels, fonts and SVG loaders across every
theme, 800×480 and 1280×720 point viewports, and 1×, 1.5× and 2× display scales. to retain its
60 initial/scrolled pngs while running the compact geometry and interaction checks:

```sh
CRAFT_UI_VISUAL_FIXTURES=/tmp/brahma-compact cargo test -p brahma-ui-egui --lib compact_visual_regressions -- --nocapture
```

keyboard tests clear the configurable `window.togglePanels` canvas shortcut through preferences,
then send normal tab and space events through the app's shortcut dispatcher. they exercise
focus traversal and activation with that nondefault binding; default canvas tab still toggles
panels. these offscreen checks do not establish native window or platform input acceptance.

## menus

horizontal and vertical ruler ticks, labels and pointer markers are clipped to their respective
strips. their intersection is a clear origin square, including when the view is panned, flipped
or rotated; the origin square retains its ruler-unit context menu.

an embedding application's `panels.menu_bar` and `panels.rail` flags belong to its current
session. restoring a saved workspace or remembered panel layout preserves both flags, including
when the serialized layout contains their values. panel visibility, dock layout, tabs and timeline
visibility follow the saved layout.

the pro status bar's document dimensions readout follows preferences › units & rulers › rulers,
including changes made from a ruler's context menu. it uses the document resolution for physical
units, each side's own extent for percentages and the selected point-size convention for points
and picas. the readout retains the document's pixels-per-inch value.

`menu_catalog.rs` holds photoshop's menu tree (standard command names, order, separators, default shortcuts). items whose id matches an engine or ui command are live; others render disabled until implemented. give new commands the catalogue's id (for example `image.imageSize`) and they light up in the right place automatically.

menus never run off the window: the menu bar's menus and submenus scroll with arrows (`menu_nav::level`), and long right-click menus (layers, canvas tools, channels, paths, document tabs) wrap their rows in `widgets::menu_scroll`, so they move up to fit and scroll only when taller than the visible window.

## automation for visual checks

use `ui.click {x,y}`, `ui.move`, `ui.key` and `ui.type` (synthetic input in screen points) to open menus, popups and context menus, then `ui.screenshot`.

## font menus

the type options bar, character panel, character/paragraph style editors and glyphs panel
share a searchable family picker. each visible row shows an `AaBbCc` sample rendered by
brahma's text engine in that family (script/symbol fonts use characters they support). a family whose face also covers arabic (alef, lam, meem, ain and the sample letters) shows a short `أبجد هوز` sample left of it, shaped right-to-left by the same engine; its row reserves the extra width once that row has been painted, and latin-only families are unchanged.
samples are cached with a bounded cache and follow display scale and theme text colour.
builds without system fonts preview the bundled fonts.
while the menu is open, up/down applies the previous/next matching family and scrolls it into
view; enter accepts the selection and closes the menu. escape closes it, keeping already
applied changes (edit › undo restores text-layer font changes).

## preferences

preferences has **apply**, **ok** and **cancel**. apply saves the edited sections and keeps the
dialog open; it is disabled when the values match the saved preferences. ok saves and closes.
cancel discards only changes made since the last successful apply. a failed apply leaves the
draft open for correction. settings marked for the next launch still require a restart.
on windows and linux, system title bar displays “applies at next launch.” beneath its checkbox,
including before editing and after apply. saving this preference does not restart the app or
change the current window's decorations; the next launch reads the saved choice.

## high DPI and 4k displays

edit → preferences → interface → ui scale applies immediately. auto follows the operating
system's display scale, including fractional scales such as 125%, 150% and 175% (on linux,
wayland's fractional scale or X11's `Xft.dpi`), and updates when the window moves between
monitors. a 4k display the system runs at 100% stays at 100% (#1278). the fixed choices (75% to
300%) set an absolute ui scale, for example 200% on an unscaled 4k display. canvas zoom shortcuts
continue to control the document independently of ui scaling.

on a wayland session with a fractional display scale, the interface is sharp only in a native
wayland window, where the window is drawn at the display's physical resolution
(`wp_fractional_scale_v1`). through xwayland the compositor enlarges a lower-resolution window, so
everything is blurred and looks larger than the chosen scale (#1072). the window opens through
xwayland when a pen is attached (#639), with preferences › performance › linux display server =
X11, or with `WAYLAND_DISPLAY=` cleared; those are also the ways to get file drag and drop, which
native wayland lacks (#386). sharp text or drag and drop: on such a desktop it is one or the other
until winit supports wayland drops.

interface → ui font size changes interface text independently of ui scale and canvas zoom.
tiny, small (the default), medium and large use 10/12, 1, 14/12 and 16/12 of the theme's original
font sizes, preserving the relative sizes of headings, captions and numeric fields. apply or ok
updates text immediately; the choice persists across launches and theme changes. CJK fallback
fonts use the same size, including fonts loaded after the preference changes.

## localisation

strings in code stay english and are the default lookup keys. `crates/ui-egui/src/i18n` maps them
to display text at render time from one catalog per language (`i18n/<code>.tsv`; the format is
documented in the header of `ja.tsv`). command ids, menu paths used for logic, the control channel,
the CLI and MCP always use the english ids and labels.

languages shipped (`complete_menus` marks a catalog that covers every menu string and `tl!` literal;
the tests enforce it):

- english (`en`), the source language.
- japanese (`ja`), complete.
- simplified chinese (`zh-hans`; see [`localization-zh-hans.md`](localization-zh-hans.md)), complete.
- traditional chinese (`zh-hant`), complete, in the vocabulary used in taiwan; `zh-TW`, `zh-HK`,
  `zh-MO` and `zh-Hant-*` locales all resolve to it. the resolver distinguishes the two chinese
  scripts, so neither catalog is shown to the other script's locales.
- spanish (`es`), complete.
- russian (`ru`), complete, with three plural forms (`one|few|many`, see `plural_russian`).
- korean (`ko`), complete, with one plural form.
- french (`fr`), complete, with two plural forms (0 and 1 use the singular).
- czech (`cs`), complete, with three plural forms.
- indonesian (`id`), complete, with one plural form.

- `tr(lang, s)` plain strings; `tr_ctx` when one english word needs different translations;
  `tr_id(lang, command_id, label)` for menu items (keyed by command id, english label as the
  fallback); `trn(lang, n, one, other)` for plurals; `fmt` fills `{name}` placeholders, which
  translators may reorder.
- the language is preferences › interface › language (`interface.language`: `auto` or a language
  code; `auto` follows the system locale, an unknown code falls back to `auto`).
- language changes apply without a restart; preferences previews the chosen language until
  apply or ok commits it. cancel restores the committed language. regional preference tags
  resolve through the same locale matcher. see [`localization.md`](localization.md).
- to add a language: add `<code>.tsv` and one row in `i18n::LANGUAGES` (code, native name, catalog,
  plural rule). the dropdown, locale matching and the catalog tests (well-formed, no duplicates,
  placeholders and ellipses agree, command ids exist) pick it up. set `complete_menus` once every
  menu string is translated; a test then enforces it. a language with more plural forms lists them
  all in `@plural` entries (russian: `one|few|many`); the row's plural rule picks the form.
- translations are clean-room: written from the meaning of the english text in ordinary vocabulary,
  never from another product's localisation resources.

localised so far: menus, the command palette, dialogs and panels (literals wrapped in `tl!("…")`;
widgets such as `checkbox`, `slider_row`, `dropdown` and the buttons translate their labels
themselves). a test fails when a `tl!` literal, a menu string, a blend mode name, brush section name or a generated
preference label has no entry in a language marked `complete_menus`. not translated: status-bar
messages and errors (they stay english, also for agents), names that are user data (layers, styles,
documents), strings assembled with `format!` that were not converted to `fmt`/`trn`. not done yet:
right-to-left layout inside panels and dialogs (the shell mirrors, `direction.rs`),
locale-aware number and date formats, automatic language detection on the web build (native
builds read `LANG`/`LC_*`, the macos preferred languages and the windows user locale).

camera raw has explicit coverage for all ten available languages, including korean and
simplified chinese. its contextual `cameraRaw` entries distinguish tonal regions,
colour-band names and font-weight light. the title uses a `{layer}` placeholder; user layer
names and the universal RGB/lab channel symbols stay unchanged. collapse ids use english
source keys, so switching languages preserves open sections. mixer tabs wrap within the
panel width. catalog coverage and actual shell language-switch tests enforce these rules.

native CJK font fallback follows the selected ui script and resets its cache when switching
languages; font delivery on the web remains separate work.

## arithmetic in numeric fields

click a numeric value and type an expression, then press enter or move focus to finish.
numeric fields accept `+`, `-`, `*`, `/`, remainder `%`, powers `^` or `**`, parentheses,
scientific notation, and `pi` / `tau`. for example, `1920/2` gives `960`, and `(30+15)*2`
gives `90`. shared value fields apply plain numbers live and arithmetic when editing
finishes, preserving rounded/integer workflows. field limits and integer rounding
still apply; invalid expressions, division by zero, and non-finite results are rejected.
expressions are evaluated locally as arithmetic, never as scripts.
