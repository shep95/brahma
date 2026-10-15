# contributing

- **language:** rust only. no JavaScript or TypeScript. on the web, `wasm-bindgen` generates a small loader; never hand-write js.
- **licence:** contributions are MIT or apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`, `NOTICE`).
- **clean-room:** do not copy code, shaders, icons, ICC profiles or other assets from proprietary software (such as photoshop). match behaviour and look by observation and public specs. only use assets with permissive licences, keep their license next to the asset (e.g. `assets/fonts/OFL-*.txt`, `assets/icons/LICENSE-lucide.txt`), and list every asset in `ATTRIBUTION.md`. **fonts** live in storytold/craft-fonts, never in this repo: add new fonts there, and build with them through the optional `CRAFT_FONTS_DIR` build input (`docs/development.md` › fonts; rules in `../craftrules/standards/fonts.md`, a sibling checkout — `storytold/craftrules` is not public, so ask a maintainer for the rules relevant to your change).
- **never crash:** non-test code must not panic: no `unwrap`/`expect`/`panic!`/`unreachable!`/`todo!`/`unimplemented!` and no `unsafe`. errors go through `Result` and `?`, input-derived indices and sizes are checked, and every crash fix ships with a regression test. see *never crash* in `AGENTS.md`.
- **commands, not handlers:** new features are engine commands with tests, and the ui calls them (checklist below).
- **layering:** `cargo xtask layers` must pass. register new crates in `xtask/src/layers.rs`.
- **tests:** required for every change. format code needs round-trip and malformed-input tests. pixel code is tested at 8, 16 and 32-bit. if you touch psd, io, codecs, compose, gpu, text or format, also run the real-file corpus tests: `cargo xtask test-corpus` (fetches the pinned corpora, including our photoshop oracles from https://github.com/storytold/brahma-corpus, then runs the `corpus`-feature tests; ci always runs them). never commit corpus files; see `docs/development.md` › test corpora.
- **style:** `cargo fmt`, and `cargo clippy -- -D warnings`. match surrounding code. comments explain *why*.
- **ui:** use `theme::Tokens` and `widgets::*`. verify visually (offscreen `snapshot` example or the control channel) before submitting, and attach before/after screenshots to prs.
- **commits:** small, focused, with a clear subject line.

## adding a command

1. **find the id.** search `crates/ui-egui/src/menu_catalog.rs` for the photoshop menu item. using its id makes the menu item live with no ui work. commands without a photoshop menu entry use a descriptive id in the same style (`layer.smartFilter.delete`) and an empty menu path.
2. **algorithm** goes in the lowest crate that fits (`algo` for imaging, `paint`, `vector`, `text`, `cms`), with unit tests. it takes depth-agnostic surfaces (`brahma-raster`), works per tile, and is deterministic (seeded randomness).
3. **command** goes in an engine module (`crates/engine/src/<area>_cmds.rs`) exposing `specs()`, registered with `v.extend(...)` in `commands.rs`. fill in:
   - `id`, `label`, `menu` path and photoshop's default `shortcut`,
   - a params doc string such as `{"radius":px=4,"mode":"a|b"="a"}` (this is what agents read through `commands` / `command_list`),
   - an `enabled` predicate (greys the menu item out),
   - `run`, which returns a JSON result and records exactly **one** history step (use `coalesce` for drags and typing sessions).
4. **respect the context:** active selection (feathered), layer vs mask target, locks, the colour model and depth.
5. **tests** in the module: behaviour, undo/redo, disabled states, bad params, several depths.
6. **dialog** (if it has parameters): filters and adjustments get schema-driven dialogs with live preview (`ui-egui/src/filter_dialog.rs`, `dialogs.rs`).
7. run `cargo xtask parity` and commit the updated `docs/parity-checklist.md`.
