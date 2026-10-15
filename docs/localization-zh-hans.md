# simplified chinese catalog

the `zh-hans` catalog is registered as **简体中文** in the shared i18n foundation
introduced by [brahma pr #169](https://github.com/shep95/brahma/pull/169).
simplified chinese support landed in
[pr #282](https://github.com/shep95/brahma/pull/282), after the v0.2.0 release.
that release does not contain the language selector or this catalog; use a build
that includes the localisation changes.

the chinese wording is original and uses ordinary image-editing terminology;
no proprietary translation resources were extracted or copied. contributions
use the repository's MIT or apache-2.0 license.

## selecting simplified chinese

select **preferences > interface > language > 简体中文**, or set
`interface.language` to `zh-hans` through the existing `prefs.set` command.
the native build's **auto** setting reads `LANG`/`LC_*`, the macos preferred
languages and the windows user locale. `zh`, `zh-CN`, `zh-SG` and `zh-Hans`
variants resolve to this catalog. traditional chinese locales (`zh-TW`, `zh-HK`,
`zh-MO`, `zh-Hant`) resolve to the separate `zh-hant` catalog.

the web build does not detect the browser language automatically; choose the
language manually there.

## files and integration

- `crates/ui-egui/src/i18n/zh-hans.tsv` contains the translations, independent of
  the lookup implementation. its UTF-8 columns are `context<TAB>source<TAB>translation`.
- `crates/ui-egui/src/i18n/mod.rs` registers `zh-hans` with one plural form and
  `complete_menus: true`. shared tests enforce coverage of every menu string,
  `tl!` literal, blend mode, brush section and generated preference label.
- shortcut templates retain `{key}` so the shell supplies the correct key for
  the current platform. layer-count messages use one chinese plural form.

keep english source keys, contexts, command ids, placeholders and escapes intact.
retain the trailing `…` on commands that open a dialog. missing translations use
the framework's english fallback. user-supplied names and document data are not
translated. product names and technology names such as brahma, #houseofasher,
OpenType, RGB, CMYK and lab retain their spelling. status messages, errors and
automation output remain english as described in [ui design](ui-design.md#localisation);
complete catalog coverage does not claim those strings are translated.

## terminology

| english | simplified chinese |
| --- | --- |
| layer / layer comp | 图层 / 图层复合 |
| mask / clipping mask | 蒙版 / 剪贴蒙版 |
| selection / feather | 选区 / 羽化 |
| blend mode / opacity | 混合模式 / 不透明度 |
| adjustment layer | 调整图层 |
| smart object / smart filter | 智能对象 / 智能滤镜 |
| canvas / artboard | 画布 / 画板 |
| brush / stroke | 画笔 / 描边 |
| fill / gradient | 填充 / 渐变 |
| path / rasterize | 路径 / 栅格化 |
| preset / swatch | 预设 / 色板 |
| export / preferences | 导出 / 首选项 |

## validation and maintenance

run `cargo test -p brahma-ui-egui`, the touched-crate all-target clippy check,
`cargo xtask layers` and `cargo xtask wasm`. the shared catalog tests validate
duplicate keys, placeholders, ellipses, menu coverage, `tl!` literals, brush
sections and blend modes. chinese-specific tests cover locale selection, fallback,
plural messages, formatted labels and the painted brush section rows and heading. render and inspect the menus, preferences and representative
dialogs with the existing offscreen `snapshot` example.

font delivery is separate from the translation data. the catalog adds no font
assets; the shell's CJK fallback loads fonts separately. verify missing glyphs,
label layout and platform-specific shortcuts visually on each target platform,
and verify web glyph coverage separately.

when english labels change, update their source keys and translations together,
preserving contexts and placeholders. the complete-catalog tests catch missing
keys, and the simplified chinese regression test checks dynamic shortcut labels,
layer counts and english fallback. review changed source meanings against the
current english ui.
