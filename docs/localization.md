# ui localisation and live language switching

brahma uses the same rust catalog implementation in the native egui shell and the
WebAssembly build. english source strings are lookup keys. UTF-8 TSV catalogs supply plain,
contextual, command-id and plural translations; missing entries fall back to english.

## supported languages

| language | preference code | integer plural forms |
|---|---|---|
| english | `en` | one / other |
| 简体中文 | `zh-hans` | one form |
| 日本語 | `ja` | one form |
| 한국어 | `ko` | one form |
| русский | `ru` | one / few / many |
| українська | `uk` | one / few / many (teens use many) |
| français | `fr` | singular for 0 and 1 / plural for 2+ |
| 繁體中文 | `zh-hant` | one form |
| español | `es` | one / other |
| čeština | `cs` | one / few / other |
| deutsch | `de` | one / other |
| bahasa indonesia | `id` | one form |
| português (brasil) | `pt-br` | singular for 0 and 1 / plural for 2+ |
| italiano | `it` | one / other |
| polski | `pl` | one / few (2–4, not 12–14) / many |
| ελληνικά | `el` | one / other |
| nederlands | `nl` | one / other |
| magyar | `hu` | one form |
| tiếng việt | `vi` | one form |
| srpski | `sr` | one / few / many (teens use many) |

languages registered with `complete_menus` cover the current menu labels, `tl!` literals, blend
mode names, and generated preference labels; tests enforce that coverage. vietnamese (`vi`) is
selectable, resolves regional tags such as `vi-VN`, and covers all current stable localization
keys. `complete_menus` remains false while generated/dynamic long-tail labels are still under
review. this does not include every engine error or status message: those remain english, as do
automation command ids and user-provided document, layer and preset names.

## switch without restarting

open **edit > preferences > interface > language** and choose a language. the preferences
dialog previews its title, controls and buttons in the chosen language. **cancel** discards
the preview, **apply** commits it without closing the dialog, and **ok** commits and closes.
the rest of the interface follows the committed preference on its next draw. existing
documents, undo history and tools remain available throughout a language change.

the existing preference store saves the selection for future launches. `auto` follows the
native system locale; an unsupported code follows the same fallback. regional tags such as
`fr-CA`, `ko-KR`, `vi-VN` and `zh-CN` resolve to the corresponding registered catalog. traditional and
simplified chinese remain distinct. every portuguese locale (`pt`, `pt-BR`, `pt-PT`) uses the
brazilian portuguese catalog. the serbian catalog is written in latin script (ekavian); every
serbian locale (`sr-RS`, `sr-Latn-RS`, `sr_RS@latin`, and cyrillic `sr-Cyrl-RS`) resolves to it.

### first launch and system language

with no saved preference, `interface.language` defaults to `auto`, so the first rendered
window uses the first supported system ui language. an older preferences file without this
field also uses auto. if detection fails or none of the preferred languages is registered,
the ui uses english (`en`). a saved manual language choice takes precedence on later launches;
startup never overwrites it or writes the detected language into the preference file.

the native-only, pinned `sys-locale` dependency provides safe rust access to the platform apis:

- **windows:** `GetUserPreferredUILanguages`, in the user's preferred order. this uses ui
  languages rather than the region setting for dates and numbers; they can differ.
- **macos:** `CFLocaleCopyPreferredLanguages`, in preferred order, without starting `defaults`
  or depending on a particular `.plist` representation.
- **linux/BSD:** the platform's standard locale environment, including `LANGUAGE` lists.

`BRAHMA_LOCALE` is an optional single-tag override for reproducible launches; an unsupported
override falls back to english. empty overrides use the system. unix locale variables do not
override windows/macos ui-language preferences. tags are read once per process (at most 64,
128 bytes each) and matched against the supported language registry. changes to the
os language list itself take effect at the next launch. the web build keeps its existing
english auto fallback until browser-locale detection is implemented.

API implementation and licensing: sys-locale.

agents use the existing `prefs.set` command through CLI, MCP or the desktop control channel:

```json
{"method":"engine.execute","params":{"command":"prefs.set","params":{"path":"interface.language","value":"ko"}}}
```

control requests are processed before synchronising the drawing language, so a request
updates the labels in the same rendered frame. drawing language is local to the ui thread;
a scoped preview restores the previous language even if drawing unwinds. menu and dialog
identity continues to use command ids, independent of translated labels.

this is hot switching among the bundled catalogs. catalog files are embedded with
`include_str!`; editing a TSV file requires a rebuild. external catalog watching and
file-resource hot reload are not implemented.

## right-to-left languages

arabic, hebrew, persian, urdu and the other right-to-left languages listed in
`i18n::dir_for_code` mirror the window's chrome when chosen: the menu bar starts at the right
with file, the toolbox docks on the right and the panel dock and its rail on the left, and the
title, options and status bars read from the right (`crates/ui-egui/src/direction.rs`). the
direction follows the language code, so a catalog needs no extra setting.

not mirrored yet: panel interiors, dialogs, menu rows (shortcuts stay on the right), submenus
(they open to the right), document tabs and the canvas. before an arabic catalog reads
correctly, the ui also needs an arabic fallback font (the bundled ui fonts have no arabic
glyphs) and bidirectional word order: egui 0.36 shapes each right-to-left word with HarfRust,
so letters join, but lays a line's words out left to right, and its caret indexing is off in
right-to-left text (emilk/egui#8576, #8577). document text is laid out by the text engine.

no right-to-left catalog ships yet. to check the mirrored layout with any language, launch with
`BRAHMA_UI_DIRECTION=rtl` (or `ltr` to force left-to-right); it is read once per process
and has no effect on the web build:

```sh
BRAHMA_UI_DIRECTION=rtl cargo run -p brahma-ui-egui --example snapshot -- --out rtl.png
```

## fonts

the native on-demand CJK font loader prioritises the selected japanese, korean or chinese script.
switching languages rebuilds the font definitions and resets the existing loader, so fonts
removed from the definitions can be registered again. idle frames keep their font cache.
other languages retain the system's CJK fallback preference for document names and text.

font files are not added to this repository. native rendering uses installed system fonts
and the optional craft-fonts build input; see [fonts](development.md#fonts-craft-fonts).
the web build can select every catalog, but CJK glyph delivery and automatic browser-locale
detection remain separate outstanding work. switching catalogs does not supply missing fonts.

## add or maintain a catalog

1. add `<code>.tsv` under `crates/ui-egui/src/i18n`, following the header in `ja.tsv`.
2. register its code, native name and plural rule in `i18n::LANGUAGES`.
3. translate the meaning of the english source keys. preserve `{name}` placeholders, escaped
   characters and the trailing `…` on dialog commands. use `@id` to disambiguate a command
   label and `@plural` for count-sensitive text. keep product names such as brahma.
4. set `complete_menus` when the catalog covers all enforced keys. add source and license
   attribution for translation assets, then run the catalog and shell tests.

```sh
cargo test -p brahma-ui-egui --locked
cargo clippy -p brahma-ui-egui --all-targets --locked -- -D warnings
cargo xtask i18n-coverage
cargo xtask layers
cargo xtask wasm
```

render representative dialogs with the offscreen `snapshot` example, including a sequence
of language changes in one process. inspect the pngs for missing glyphs and truncated labels.
translations are original work or reuse the open brahma contributions credited in
[ATTRIBUTION.md](../ATTRIBUTION.md); no proprietary localisation resources are used.
