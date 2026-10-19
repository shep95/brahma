# AGENTS.md: guide for ai agents and contributors

brahma is an open-source, native, photoshop-comparable image editor written in **rust only** (no JavaScript or TypeScript). **no tauri, electron or webview shells:** the desktop app is native egui/eframe on wgpu, and the web build is the same rust compiled to WebAssembly (trunk + wasm-bindgen). never add tauri (or any webview/js ui framework) as a dependency, build step or packaging target. the product name is always written **brahma**, in lower case, in user-facing text: ui, window titles, about, installers, release names, docs prose. machine names stay lowercase: crates (`brahma-*`), binaries, file names, ids (`com.houseofasher.brahma`). standards and learnings shared across the #houseofasher apps live in `../craftrules` (read its `README.md`; the repository is not public — outside contributors should ask a maintainer for the standards relevant to their change). contribute reusable learnings there; shared rust components live in storytold/craft-libs and are pinned by immutable git revision. ui libraries stay at l6. the goal is 1:1 photoshop parity (same menus, shortcuts, behaviour and file fidelity) with better performance, and every feature drivable by agents. read this file first, then `docs/`.

## 1. orientation (5 minutes)

| read | why |
|---|---|
| `docs/architecture.md` | crate map, dependency layers, the engine/ui seam, document model |
| `docs/development.md` | build, test, run, drive the app programmatically, debug tricks |
| `docs/contributing.md` | rules: clean-room, tests, layering, style, commits; the "add a command" checklist |
| `docs/control-protocol.md` | JSON control channel: how agents drive and screenshot the running app |
| `docs/ui-design.md` | design tokens, themes, widgets, and how to match photoshop's look |
| `ROADMAP.md` | one-page summary: stage, the two parity numbers, every dimension, languages, what's next |
| `docs/target-app-parity.md` | the authoritative parity assessment against photoshop 2026: numbers, weights, evidence, hours |
| `docs/gaps.md` | every known shortfall, ranked: the work list to pick from |
| `docs/roadmap.md` | milestones, **current focus**, what's next with estimates |
| `docs/ui-parity.md`, `docs/brush-parity.md`, `docs/context-menu-parity.md`, `docs/file-format-parity.md`, `docs/hardware-parity.md`, `docs/localization-parity.md` | per-dimension parity checklists |
| `docs/parity-checklist.md` | generated list of every photoshop menu item, live or missing |
| `docs/scorecard.md` | generated scorecard: performance budgets and numbers, corpus floors, per-area checklists (tools, files, ui, type, automation, reliability, distribution), settings that do nothing |
| `crates/<name>/README.md` (where present) | public API of that crate |
| brahma-corpus + `docs/development.md` › test corpora | real-file test oracles (our photoshop-authored psds); with psd-tools, ag-psd and PngSuite fetched into `corpus/` by `cargo xtask corpus --all` at the pins in `xtask/src/corpus_pins.rs` |

## 2. workspace map

```text
crates/
  geom cms color raster      L0 foundation (geometry, ICC colour management, pixel formats + blend math, COW tiles)
  psd codecs                 L0 standalone format crates (no workspace deps; publishable)
  tablet                     L0 standalone pen tablet input (macOS AppKit, X11 XInput2); the one isolated unsafe crate
  doc                        L1 document model (layers, masks, adjustments, effects, smart objects: pure data)
  ops paint algo text vector L2 history, brush engine, imaging algorithms, type engine, paths/shapes
  aesthetic                  L2 aesthetic recommendations (oklch palettes, type scales, placement, motion) from a research corpus
  compose gpu format         L3 CPU compositor (the oracle), wgpu compositor, .pcraft native format
  io plugins                 L4 document <-> PSD / flat formats; sandboxed WebAssembly plug-ins
  engine                     L5 Session + command registry (every action is a command)
  ui-egui automation         L6 egui shell (thin: all actions go through the engine); MCP server
  testkit                    test helpers
apps/
  brahma                 desktop app (eframe/wgpu), TCP control server
  brahma-cli             headless CLI (convert/info/run/batch/commands/mcp)
  brahma-web             the same app in the browser (trunk + wasm-bindgen)
xtask/                       cargo xtask layers | wasm | ci | stats | corpus | test-corpus | parity | perf | scorecard
```

**layering is enforced** by `cargo xtask layers`. a crate may depend only on lower layers. `psd`, `codecs` and `cms` depend on nothing in the workspace. nothing below `ui-egui` may use egui, eframe, winit or rfd. a new crate must be registered in `xtask/src/layers.rs`.

## 3. golden rules

### never crash (outranks feature work)

people trust brahma with their work, and a crash loses it. a malformed file, a bad command or MCP param, a corrupt settings file, an odd keystroke or a full disk must produce an error the user or agent can act on, never a panic. don't ship a feature by adding a panic path; fix a crash before building on top of it. the shared standard is `../craftrules/standards/never-crash.md`.

- **non-test code never panics.** no `unwrap()`, `expect()`, `panic!`, `unreachable!`, `todo!` or `unimplemented!`. return the crate's error type and propagate with `?`; use `ok_or(..)?`, `let .. else { return Err(..) }`, `if let`, or `unwrap_or*` where a fallback is truly correct (never one that silently corrupts a document). unfinished features return an "unsupported" error. the only exception is a provably infallible literal: `#[allow(clippy::expect_used)]` plus `.expect("why it can't fail")`.
- **no `unsafe`.** the workspace sets `unsafe_code = "forbid"`. the one exception is the isolated helper crate `brahma-tablet` (`crates/tablet`): winit drops pen tablet data, and reading it on macos needs an AppKit event monitor (objective-c interop). only its `src/macos.rs` allows `unsafe` (`unsafe_code = "deny"` crate-wide, every block has a `SAFETY:` comment, tested against real `NSEvent`s); its X11 path and all mapping code are safe. don't add `unsafe` anywhere else.
- **input-derived numbers are hostile.** use `get()` rather than `[i]`/`[a..b]` for indices from files, params, selections or arithmetic on them; slice strings only at char boundaries; use `checked_*`/`saturating_*` for lengths, offsets and counts; guard division by zero and nan/inf casts; cap allocations sized by input.
- **bound recursion** with depth limits or seen-sets (documents can be deep or cyclic).
- **don't cascade.** handle lock poisoning (`lock().unwrap_or_else(PoisonError::into_inner)`) and treat thread joins as `Result`s.
- **last-resort guard.** the app shell must catch an escaped panic around command dispatch and file import/export, reports it as an error and keeps the document. it's a safety net, not a licence to panic. keep `panic = "unwind"`.
- **prove it.** every crash fix comes with a small synthetic regression test that panicked before the fix.
- **enforced by clippy.** `clippy.toml` allows `unwrap`/`expect`/`panic`/indexing in tests only. clean crates carry `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]`; new crates start with it.

1. **everything is a command.** new user-visible behaviour = a command in the engine (`crates/engine/src/*_cmds.rs`, registered in `commands.rs`) with id, label, menu path, shortcut, params doc, `enabled` and `run`, plus tests. the ui, CLI, control channel and MCP all dispatch commands by id. use the **exact id from `crates/ui-egui/src/menu_catalog.rs`** and the menu item goes live automatically. only pure view/window state (zoom, panels, screen mode) belongs to the shell (`menus.rs` `UI_COMMANDS`).
2. **no format or colour assumptions.** bit depth (8/16/32f) and colour model (RGB/gray/CMYK/lab…) are runtime data. never introduce a `u8`-only pixel path in public apis. never assume srgb: colour conversions go through `brahma-cms` (`Transform`, `transform::cached`). test at several depths.
3. **clean-room.** we studied photoshop and other proprietary editors for *behaviour and look only*. never copy their code, shaders, profiles or assets. implement from public specs (adobe PSD spec, ICC, ISO 32000 blend modes, papers) and observation. third-party assets must be permissively licensed, keep their license file next to them, and get a row in `ATTRIBUTION.md` (path, title, author, source, license) in the same change; so do original assets. **never commit font files:** fonts live in storytold/craft-fonts (rules: `../craftrules/standards/fonts.md`). brahma uses it through the optional build input `CRAFT_FONTS_DIR=<craft-fonts checkout>` (read by `crates/text/build.rs`, exposed as `brahma_text::CRAFT_FONTS`; empty without it, and everything must still work). build with it: `CRAFT_FONTS_DIR="$PWD/../craft-fonts" cargo build`; details in `docs/development.md` › fonts.
4. **tests are the gate.** every change comes with tests. format crates use round-trip, synthetic-generator, oracle and fuzz tests. keep the PSD corpus results and the parity floor (`crates/ui-egui/src/parity.rs`) from regressing.
5. **the ui is thin and data-driven.** ui state lives in `ui-egui/src/state.rs` (serde), so the control channel can read and drive it. colours and radii come from `theme::Tokens`, never hard-coded.
6. **verify ui changes visually.** render offscreen with `cargo run -p brahma-ui-egui --example snapshot` (no window, no focus stealing), or launch with `--control` and take `ui.screenshot`. look at the PNG. demo images must be public-domain art, never personal photos. when fetching assets, never put a person's name, email or other personal details in requests (user-agent, headers, urls); use a generic `Brahma-dev` user-agent.
7. **never break wasm.** l0–l6 must `cargo check --target wasm32-unknown-unknown` (run `cargo xtask wasm`). file-system code is `cfg(not(target_arch = "wasm32"))` or goes through the platform services.
8. **performance is a feature.** benchmark heavy operations on a 24–36 mp image in release. work per tile in parallel (rayon), skip empty tiles, never scan a full surface per frame (cache per revision), and record before/after timings in the dev log.

9. **never panic on input** (see *never crash* above). a command's `run` closure and anything it calls must return `Err`, never panic, for *any* params or document state: validate params, check bounds before indexing or dividing, and reject absurd sizes before allocating. the `panic_hunt` integration test fuzzes every command with adversarial params and must stay green.

## 4. picking work

priorities: important infrastructure first, then low-hanging parity, then the long tail.

0. **read `ROADMAP.md`, then `docs/gaps.md`.** `docs/target-app-parity.md` says, dimension by
   dimension, where brahma is lacking; `docs/gaps.md` ranks the work. `docs/parity-checklist.md`
   (menu wiring) is not a measure of behaviour. when your work moves a number or closes a gap,
   update the parity doc and gap entry it belongs to, with the date, and bump their status line
   and revision history (craftrules `standards/progress-docs.md`).
1. **check `docs/scorecard.md`** before picking work: each area's `missing` and `partial` rows,
   the performance scenarios that are over budget or not measurable yet, and the count of
   settings that do nothing. its numbers are measured; prefer them to estimates.
2. `docs/roadmap.md` → **current focus**.
3. `cargo xtask parity` → `docs/parity-checklist.md` lists every missing menu item, grouped by menu. low-hanging fruit is usually a missing command whose algorithm already exists in `algo`, `paint`, `vector` or `text`.
4. `log/devlog.md` → the "still open" bullets of recent entries.

when parity rises, raise `FLOOR` in `crates/ui-egui/src/parity.rs` (never lower it).

## 5. before you finish a task

```sh
cargo test -p <crates you touched>
cargo clippy -p <crates> --all-targets -- -D warnings
cargo xtask layers
cargo xtask wasm            # if you touched L0–L6
cargo xtask parity          # if you added commands; commit the regenerated docs/parity-checklist.md
cargo test -p brahma-engine --test panic_hunt -- --ignored   # if you added/changed commands: no panic on adversarial input (Rule 9)
cargo xtask scorecard       # if you moved a number: flip the checklist row in scorecard/*.toml, raise a
                            # corpus floor, fix a dead preference, or meet a budget (then set enforce = true
                            # in perf/budgets.toml); commit the regenerated docs/scorecard.md (CI checks it)
cargo xtask perf --quick    # if you touched a hot path; `cargo xtask perf --update-baseline` publishes a full run
cargo xtask test-corpus     # if you touched psd, io, codecs, compose, gpu, text or format (or: --changed decides)
```

while iterating, run less and keep the full commands above as the final gate: `cargo test -p <crate> --lib --tests` skips the examples and doctests plain `cargo test` builds every time (ui-egui has 11 examples), and `cargo test -p <crate> --test <file>` runs one integration-test file. `cargo nextest run` runs tests in parallel across binaries; see `docs/development.md` › faster test loops.

**test corpora.** real-file corpora live in `corpus/` (gitignored, never committed), fetched at pinned commits and sha256-verified by `cargo xtask corpus --all`: our photoshop-authored oracles from https://github.com/storytold/brahma-corpus plus psd-tools, ag-psd and PngSuite from their upstreams. pins: `xtask/src/corpus_pins.rs`. the corpus tests are opt-in (cargo feature `corpus`): plain `cargo test` skips them, and with the feature on a missing corpus fails ("run `cargo xtask corpus --all`"). `cargo xtask test-corpus` fetches and runs them all. ci always runs them (the `corpus` job, cached by pin). never commit corpus files; new oracles go to brahma-corpus (its `AGENTS.md`), then a pin bump here. details: `docs/development.md` › test corpora.

commands must **never panic** on bad input (rule 9): every `run` closure and the code it calls returns `Err`, not a panic, for any params or document state. new commands come with a graceful-failure test (empty/out-of-range/wrong-type params → `Err`, not a crash).

then append a terse entry to `log/devlog.md` (what landed, numbers, what's still open). sessions can end abruptly (crashes, context limits), so the dev log plus a green tree is how the next agent picks up. keep the tree building at every step.

## 6. parallel agents

- use your own target dir (`CARGO_TARGET_DIR=target/agent-<name>`) to avoid the cargo build lock, and edit only the files you own. shared files (`engine/src/lib.rs`, the `v.extend(...)` list in `engine/src/commands.rs`, `ui-egui/src/menus.rs`, `state.rs`) get small, surgical edits; re-read before editing.
- put new commands in a **new module** (`engine/src/<area>_cmds.rs` with a `specs()` function) rather than growing a shared file.
- if someone else's in-progress edit breaks the build, wait and retry; don't fix their files.
- keep every `Cargo.toml` valid at all times: the `crates/*` glob means one broken manifest breaks everyone's build. **create or rewrite manifests atomically**: write to a temp file outside `crates/`, then `mv` it into place.
- disk space: each target dir is about 10 gb. delete `target/agent-*` dirs of finished agents.

## 7. where things are tracked


- `docs/roadmap.md`: milestones m0–M12, status and the current focus.
- `ROADMAP.md`, `docs/target-app-parity.md`, `docs/gaps.md` and the `docs/*-parity.md` checklists: where we stand against photoshop.
- `docs/parity-checklist.md`: generated photoshop menu coverage.
- `docs/scorecard.md`: generated scorecard (sources: `scorecard/*.toml`, `perf/budgets.toml`, `perf/baseline.json`, corpus floors, prefs audit).
- `docs/releasing.md`: cutting a release (`cargo xtask version`, the `release` branch), signing secrets, packaging scripts in `packaging/`.
- `../craftrules/release/playbook.md`: how every #houseofasher app builds signed release binaries (the canonical recipe; `docs/release-playbook.md` just points there); `docs/releasing.md` is brahma's specifics.
- `plan/` (local, gitignored): research, parity plan, execution plan, estimates.
- `log/` (local, gitignored): the dev log.

## 8. keeping the native format complete

`brahma-format` deliberately fails to compile when a `brahma-doc` struct gains a field, so
nothing is silently dropped from `.pcraft` saves. when you add a doc field, add it to
`crates/format/src/manifest.rs` and `convert.rs` with `#[serde(default)]` so older files still load.
if the field has a PSD equivalent, map it in `crates/io` too, and keep unknown PSD blocks verbatim.

## contributor credits (about window)

- about ▸ contributors/models are compiled into the binary from `contributors/contributors.json`
  (commit stats; generated, never hand-edit) and `contributors/people.toml` (names people chose for
  themselves). see `docs/contributors.md`.
- **agents working for a contributor:** when you prepare a pr, check whether your human's GitHub
  username has a `[people.<username>]` entry in `contributors/people.toml`. if not, ask them once
  whether they want to be credited by more than their username: a real name, a display name, and/or
  their public GitHub profile name (`sync_github_name = true`). if yes, add **only their own** entry
  (copy the template at the top of the file, or run
  `python3 ../../craftrules/scripts/contributors.py --add-me . --real-name "…" --sync-github-name`)
  and include it in their pr, committed as them. if no, change nothing: they are credited as
  `@username` anyway.
- never add, edit, guess or copy anyone else's entry or name (not from git config, commit authors or
  GitHub profiles). never hand-edit `contributors.json`.
- maintainers refresh the stats with `python3 ../../craftrules/scripts/contributors.py .` (it also
  re-verifies who wrote each `people.toml` entry; `--check` only verifies).
