# handoff: where the work stands

this file is the memory between sessions. a fresh session (or a new tab) reads it first, then
`AGENTS.md`. keep it current: update the task table and the "in flight" section on every push.

branch: `claude/compassionate-gates-3ec6rv` on github.com/shep95/brahma (the repo's only branch,
so also its default). every push goes there.

## live site

- vercel project `brahma-web` in team houseofasher (id `prj_wwHVxBBu5cStwerqT2oTBPOADCDl`).
- production: https://brahma-web.vercel.app (also brahma-web-houseofasher.vercel.app).
- the project was created through the api, not the github import, so pushes may not deploy
  by themselves. to deploy: vercel mcp `create_deployment` with `project: "brahma-web"`,
  `target: "production"`, `gitSource {type github, org shep95, repo brahma, ref <branch>, sha}`,
  and **no `teamId` argument** (passing one returns 403 through this connection).
- `get_deployment` works without `teamId` too; the build-log endpoint returns 404 through this
  connection, so failures are diagnosed from `errorCode`/`errorMessage` plus local repros.
- build: `packaging/web/vercel-build.sh` (installs rust >= rust-version, the static musl trunk,
  thin lto, 2 jobs). about 7 to 10 minutes. headers and csp live in `vercel.json`.
- vercel authentication is off on the project (public site).

## tasks

| # | task | state |
|---|---|---|
| 1 | rename photocraft to brahma everywhere | done |
| 2 | rebrand third parties to #houseofasher | done |
| 3 | lower-case all user-facing text | done (switch in `i18n::set_lowercase`) |
| 4 | reword "sin-pattern" phrases | done |
| 5 | bring-your-own ai api keys (preferences › ai integrations) | done |
| 7 | app icon from the user's silhouette image | **blocked: the image never arrived as a file.** when it does: `python3 packaging/icon-from-image.py <file>` then commit |
| 8 | warm near-black theme (#141310, gold accent) as default | done |
| 9 | remove third-party links | done |
| 10 | full test suite green | done (one pre-existing failure, `smart_object_curves_preview_keeps_its_size_on_a_reduced_proxy`, also fails upstream) |
| 11 | visual review | done |
| 12 | security audit + weekly workflow | done (`.github/workflows/security.yml`) |
| 13 | vercel layout: one static site, no services, no bindings | done, confirmed by the user |
| 14 | web ui bugs: sidebar clicks, panel scrolling, slow open, responsiveness | in progress (see below) |
| 15 | aesthetic recommender trained on the research file | todo |
| 16 | motion system (springs) matching the aesthetic | todo |
| 17 | quality and speed: startup, file open, frame pacing | todo |
| 18 | installable + offline web app (manifest, service worker, document.title) | in progress |
| 19 | this file | ongoing |

## in flight (task 14 and 18)

reproduced in headless chromium against the local build (`packaging/web` + playwright script
in the session scratchpad, re-creatable from `docs/development.md`):

- the browser tab title never changed: eframe's web backend ignores `ViewportCommand::Title`.
  fixed: `WebShell::sync_title` mirrors `panels::window_title` to `document.title`.
- at short window heights the dock squeezed the layers group under its own header rows
  (overlapping rows). fixed: `dock::show` scrolls the whole column when the groups' minimum
  heights don't fit (`DockLayout::min_column_height`); layers' minimum raised to 200.
- at narrow widths (< 980 pt) the dock (min 250 pt) plus toolbar leave no canvas and the colour
  panel's hue strip draws over the toolbar. planned: a compact layout where the dock becomes a
  drawer over the canvas opened from the icon rail (`panels::right_dock`), one group at a time,
  closing on a click outside; slide animation from the new motion module.
- pwa: `apps/brahma-web/manifest.webmanifest`, `sw.js` (cache-first for hashed wasm/js,
  network-first for the page), icons 192/512/maskable, registration inline in `index.html`.
  vercel headers for `sw.js` and the manifest added.
- file open speed: not yet measured properly (the title signal was broken). measure with a
  12 mp jpeg drop; the decode runs on the ui thread in wasm (`open_bytes`, no background jobs).

## research inputs

two files the user uploaded (not committed; ask for them again if needed): `shepherd —
aesthetic research loop` (3,068 lines: colour systems in oklch, typography, motion grammar,
gestalt, fechner-weber sizing, perception psychology) and `shepherd — quantum artificial
intelligence` (23 mb persona document; only the responsiveness, animation and security-posture
passages are relevant). the recommender (task 15) distils the first into a compact knowledge
table committed with the crate.
