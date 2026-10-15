# localization parity

> **last reviewed:** 2026-10-12 · **last updated:** 2026-10-13 · **change:** minor (spanish dynamic labels; previously minor: serbian catalog) · **target:** adobe photoshop 2026

per-language status of brahma's ui. how catalogs work and how to add one:
[`localization.md`](localization.md). language notes: [korean](localization-ko.md), [spanish](localization-es.md),
[simplified chinese](localization-zh-hans.md), [italian](localization-it.md).

**how measured.** `cargo xtask i18n-coverage` (`xtask/src/i18n_coverage.rs`) defines the key
universe: `tl!("…")` literals and `trn(` plurals in `ui-egui` (tests excluded), menu-catalog
labels and path segments, `menus.rs` ui commands, and engine `CommandSpec` labels and menu paths:
**1,608 english keys** on 2026-10-10. each catalog (`crates/ui-egui/src/i18n/<code>.tsv`) was
checked against that universe by a source script replicating the xtask (no build). catalogs hold
more rows than the universe because they also translate dynamic `tl!(expr)` strings (~159 call
sites the tool can't enumerate). **not covered anywhere:** engine error and status messages and
filter parameter help, which are english in every language; so no language is `full`.

**photoshop 2026** ships ui language packs for cs, da, de, en, es, fi, fr (and fr-ca), hu, it,
ja, ko, nb, nl, pl, pt-br, ru, sk, sv, tr, uk, zh-hans, zh-hant (24 `.lproj` markers; one pack is
installed per creative cloud install, here `en_US`). it has no hindi, arabic, indonesian or
vietnamese ui; arabic and hebrew users get english "middle east" builds with the world-ready
composer, and its text engine handles RTL, devanagari and indic scripts.

**summary.** 7 of the 12 key languages at 100% of measured keys (status `partial` because of the
engine messages), 3 absent (hindi, arabic, vietnamese), plus 8 other languages. weighted by
speakers, ~55% ready. remaining: 80–140 h, plus native-speaker review (recorded only for italian,
2026-10-12).

## the twelve key languages

| language | code | ui strings (measured keys) | catalog rows | dialogs / tooltips / help | script support | native review | status | to `full` |
|---|---|---|---:|---|---|---|---|---:|
| english | `en` | 1,608 / 1,608 (100%) | source | all | latin | – | full | 0 h |
| simplified chinese | `zh-hans` | 1,608 / 1,608 (100%) | 2,366 | dialogs, tooltips yes; engine messages english | CJK fonts (on-demand loader), IME preedit/commit, vertical type | no | partial | 8–12 h |
| spanish | `es` | 1,844 / 1,844 (100%) | 4,010 | as above; complete dynamic ui key set | latin | no | partial | 6–10 h |
| hindi | `hi` | 0 (0%) | – | – | devanagari shaping in the text engine (parley/HarfRust); no devanagari fallback font (TYPE-218-3) | no | none | 20–30 h |
| arabic | `ar` | 0 (0%) | – | – | arabic shaping and bidi in the text engine, arabic fallback font; RTL shell layout but not inside panels, and ui labels in LTR word order (DIST-220-4), paragraph direction has no ui (TYPE-218-2), layer names as tofu (#1408, #1693) | no | none | 40–60 h |
| french | `fr` | 1,608 / 1,608 (100%) | 2,766 | as above | latin | no | partial | 6–10 h |
| portuguese (brazil) | `pt-br` | 1,608 / 1,608 (100%) | 2,788 | as above | latin; every `pt` locale uses it | no | partial | 6–10 h |
| indonesian | `id` | 1,608 / 1,608 (100%) | 2,764 | as above | latin | no | partial | 6–10 h |
| japanese | `ja` | 1,608 / 1,608 (100%) | 3,356 | as above | CJK fonts, IME (windows IME broken, #590), vertical type (tategaki); installed japanese fonts missing from the list (#2330) | no | partial | 10–16 h |
| german | `de` | 1,608 / 1,608 (100%) | 2,400 | as above | latin | no | partial | 6–10 h |
| korean | `ko` | 1,608 / 1,608 (100%) | 2,936 | as above; terminology from photoshop's korean ui ([notes](localization-ko.md)) | hangul fonts, IME | no | partial | 6–10 h |
| vietnamese | `vi` | 0 (0%) | – | – | latin with stacked diacritics (shaping supported) | no | none | 12–18 h |

## other shipped languages

8 more, each at 1,608 / 1,608 measured keys, status `partial` for the same reason: traditional
chinese `zh-hant` (2,329 rows), russian `ru` (2,544), ukrainian `uk` (3,375), czech `cs` (3,522),
polish `pl` (3,366), greek `el` (3,481), dutch `nl` (3,568), italian `it` (4,008; also every dynamic label the
polish `pl` (3,366), greek `el` (3,481), dutch `nl` (3,568), italian `it` (4,164; also every dynamic label the
other catalogs translate; reviewed by a native speaker, [notes](localization-it.md)). turkish `tr`
(3,731 rows) also covers every measured key. serbian `sr` (4,022 rows, latin script, ekavian, added 2026-10-12) covers every
measured key and the union of the other catalogs' dynamic labels; its one/few/many plurals reuse
the russian rule.

czech ui follow-up (2026-10-11): status and notice wrappers, hardware labels, brush and
transform accessibility names, rasterization prompts, profile mismatch messages and preset
counts now translate at their render/assignment sites. navigator's zoom level uses a context
(`Zvětšení`), while the tool remains `Lupa`. engine errors, profile/file/document names stay
verbatim; copied system info and control API diagnostics remain english while their on-screen
labels translate. import warning counts use parentheses in czech. new dynamic templates fall
back to english in catalogs without entries. this is additional ui coverage, not a claim that
engine localization or native review is complete.

## cross-language gaps

- engine error, status and parameter-help messages are english in every language.
- right-to-left ui layout is partial (2026-10-10): the shell mirrors for arabic, hebrew, persian,
  urdu and other right-to-left codes (`direction.rs`), but panel interiors, dialogs, menu rows,
  submenus and document tabs stay left to right. ui labels also lack an arabic fallback font, and
  egui 0.36 joins letters within a word but orders a line's words left to right (no bidi).
- web build: no browser-locale detection, no CJK web fonts.
- WebAssembly plug-ins disappear from filter › plug-ins when the ui isn't english (#2540).
- a user couldn't find the language option (#2532): it is in preferences › interface.

## revision history

| date | change | summary |
|---|---|---|
| 2026-10-10 | major | first version, measured from the catalogs with the `i18n-coverage` rules |
| 2026-10-10 | minor | right-to-left shell layout (menu bar, title, options and status bars, toolbox and dock sides) |
| 2026-10-11 | minor | czech dynamic status/notice and hardware text, render labels and navigator zoom context; engine messages remain english |
| 2026-10-11 | minor | czech review follow-ups: english system info diagnostics, id-aware shortcut labels, display-only auto translation, warning punctuation and wording; catalog row count refreshed |
| 2026-10-11 | minor | italian: 1,287 dynamic labels added (filter parameters, camera raw, lighting, panels, dialogs) and terminology made consistent |
| 2026-10-12 | minor | added turkish (`tr`) catalog with 100% measured ui coverage |
| 2026-10-12 | minor | spanish: 1,276 runtime-only labels added (filter and adjustment parameters, camera raw, panels, dialogs and status wrappers); generated-label coverage test added |
| 2026-10-12 | minor | italian native-speaker review: mistranslations (trap, HDR toning, display text) and phrasing fixed |
| 2026-10-12 | minor | serbian (`sr`, latin, ekavian) catalog: every measured key plus the dynamic labels of the other catalogs |
