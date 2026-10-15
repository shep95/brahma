# korean terminology and dynamic ui labels

this supplements the korean catalog merged in #582. the existing language
registry, live language switching and korean font loading are retained.
translations use the meaning of brahma's english labels and terminology
observed in the korean photoshop ui in october 2026. no proprietary translation
resources, fonts, screenshots or user documents are included.

## terminology

| english source | korean ui |
|---|---|
| gaussian blur | 가우시안 흐림 효과 |
| liquify | 픽셀 유동화 |
| adaptive wide angle | 응용 광각 |
| brush tip shape | 브러시 끝 모양 |
| smudge tool | 손가락 도구 |
| puppet warp | 퍼펫 뒤틀기 |
| render (menu) | 렌더 |
| sharpen (menu) | 선명 효과 |
| sharpen (command) | 선명하게 |

command-id translations disambiguate menu names from actions. photoshop terms
are used when the operation matches; brahma-specific features retain their
own meaning. technology names, units and user-provided names are not translated.
the terminology test protects representative tool names and this distinction.

## dynamic coverage

generated filter parameter names, symbolic choices, gallery categories/effects,
job titles, layer-style menus and shortcut labels now go through the catalog.
the filter test enumerates engine filter/adjustment parameter specs and gallery
names to catch new korean omissions that literal-only scans cannot find.
two newly introduced literals, `Enable {name}` and `No patterns`, are present in
all nine non-english catalogs. existing fallback behavior is preserved for
untranslated dynamic labels in other languages.

this is not a claim that all application output is localized. engine error
messages, arbitrary plug-in labels and user document/layer/preset names remain
outside this translation change. the source coverage report measures its
scanned key set, not every possible runtime string.

## validation

on main 7055f0c plus this change:

- ui tests: 728 passed, 3 ignored, including live language-switch tests.
- clippy for the ui crate and all targets: passed with warnings denied.
- i18n coverage: 1,406/1,406 scanned keys for all ten registered languages;
  all seven coverage-tool tests passed.
- dependency-layer and WebAssembly checks passed. existing CMS dead-code
  warnings appear during WebAssembly checks.
- offscreen french-to-korean language switching and the korean brush settings
  panel were rendered and visually inspected for missing glyphs/clipping.

re-run with `cargo test -p brahma-ui-egui --locked`,
`cargo clippy -p brahma-ui-egui --all-targets --locked -- -D warnings`,
`cargo test -p xtask i18n_coverage`, `cargo xtask i18n-coverage`,
`cargo xtask layers`, and `cargo xtask wasm`.

IME composition and PSD font matching are separate contributions.
