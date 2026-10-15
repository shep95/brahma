# brahma roadmap

**stage: alpha** · next: beta, ~30 points (ready for real work ~45% → ~75%, with reliable PSD) and ~1,100–1,800 h away

> **last reviewed:** 2026-10-11 · **last updated:** 2026-10-13 · **change:** minor (oracle floor 145 of 258 and 953 open issues, as in gaps.md and on GitHub; toolbox count 56 of 68, 12 missing) · **target:** adobe photoshop 2026

brahma's one-page summary of how close it is to adobe photoshop 2026 (27.11.0, measured from
the installed app on 2026-10-10) at version 0.6.0. detail: [target-app parity](docs/target-app-parity.md)
(methodology, weights, evidence), [gaps](docs/gaps.md) (the ranked work list),
[roadmap](docs/roadmap.md) (milestones, current focus), [scorecard](docs/scorecard.md) and
[menu checklist](docs/parity-checklist.md) (both generated and measured),
[architecture](docs/architecture.md).

hours are opus 5.5 agent wall-clock hours for one agent working sequentially, calibrated from this
repository's 1,004 merged prs (see [calibration](docs/target-app-parity.md#remaining-effort-and-calibration)).
about 80% of the work parallelizes.

## headline numbers

| | value | kind |
|---|---|---|
| **feature breadth** | **~76%** | partly measured: menus 628/628, tools 56/68, panels 30/35, formats 13+4 of 31 |
| **ready for real work** | **~45%** (40–50%) | estimated: weighted by dimension below |
| mainstream practitioner | ~43% (38–48%) | estimated: weekly areas of a typical pro, with discounts ([method](docs/target-app-parity.md#mainstream-practitioner-43)) |
| essentials user | ~61% (55–67%) | estimated: core features only ([method](docs/target-app-parity.md#essentials-user-61)) |
| remaining to **beta** | **~1,100–1,800 h** | estimated |
| remaining to **full parity** | **~2,300–4,100 h** | estimated; ai and plug-ins need owner decisions |

### readiness by audience

| audience | ready % | opus 5.5 agent-hours to ~95% | work that dominates |
|---|---:|---:|---|
| full target (ready for real work) | ~45% (additive weighted sum: 46%) | 2,300–4,100 | everything below, plus ai tier, plug-in/scripting ecosystem, video, localization, hardware |
| mainstream practitioner | ~43% | 1,700–2,800 | depth in the 11 weekly areas (1,180–2,000), performance budgets, stability on real machines, PSD exchange with photoshop |
| essentials user | ~61% | 400–700 | startup and GPU robustness, brush and transform responsiveness, type tool, discoverability (flyouts, commit buttons), AVIF/PDF open |

hours calibrated as in [remaining effort](docs/target-app-parity.md#remaining-effort-and-calibration) (2–9 h per checklist
item, 8–20 h per format or subsystem, from the repo's 1,004 merged prs); each tier is a subset of
the one above. about 80% parallelizes (independent crates and command modules); performance
architecture and the PSD writer are the serial parts.

**why alpha.** core workflows (open, layer, select, paint, adjust, filter, type, save) work end to
end, and breadth is high, but 953 issues are open (2026-10-13), 22 of 25 performance budgets
are missed, and psds we write are not yet reliably accepted by photoshop (#1281, #2469). beta
needs ~75% ready and no blocking PSD gap. the [alpha gate](docs/roadmap.md#alpha-gate) passes: all six core workflows
(retouch, composite, cut-out, design, paint, PSD exchange) complete end to end and save and reopen;
four are partial in depth, none blocked.

## by dimension

| dimension | % ready | hours to full | doc |
|---|---:|---:|---|
| features (depth) | 55% (breadth 76%) | 600–1,000 | [target-app-parity](docs/target-app-parity.md#feature-areas) |
| ui/ux fidelity (tools, handles, modifiers, brush feel, panels) | 40% | 350–600 | [ui-parity](docs/ui-parity.md), [brush-parity](docs/brush-parity.md), [context-menu-parity](docs/context-menu-parity.md) |
| file formats | 60% (presence ~80%) | 300–500 | [file-format-parity](docs/file-format-parity.md) |
| hardware (GPU, pen, displays) | 50% | 80–150 | [hardware-parity](docs/hardware-parity.md) |
| localization | 55% | 80–140 | [localization-parity](docs/localization-parity.md) |
| performance | 25% (3/25 budgets met, measured) | 200–350 | [scorecard](docs/scorecard.md#performance) |
| stability | 45% | 150–250 | [gaps](docs/gaps.md#g3-user-reported-bug-backlog) |
| platforms | 80% | 30–60 | [scorecard](docs/scorecard.md#distribution) |
| ecosystem / plug-ins / scripting | 15% | 250–450 | [gaps](docs/gaps.md#g16-plug-in-compatibility) |
| ai features | 10% | 300–600 | [gaps](docs/gaps.md#g15-ai-and-generative-features) |

## features

| area | % ready | hours |
|---|---:|---:|
| layers, masks, blend modes, smart objects, styles | 65% | 150–250 |
| selections, select and mask, channels | 55% | 120–200 |
| painting and brush engine | 55% | 150–250 |
| retouching (healing, clone, patch, remove, content-aware) | 50% | 100–180 |
| transform, warp, puppet, liquify | 55% | 100–180 |
| adjustments and filters | 70% | 120–200 |
| type | 40% | 120–200 |
| vector shapes and paths | 55% | 80–140 |
| colour management and modes | 70% | 60–100 |
| camera raw and computational photo | 45% | 120–200 |
| automation, actions, scripting | 45% | 120–200 |
| file i/o | 60% | 300–500 |
| print, export, save for web | 50% | 60–100 |
| video / timeline, data-driven graphics | 30% | 100–200 |
| workspace, panels, preferences | 40% | 150–250 |
| ai / generative | 10% | 300–600 |

rows and evidence: [target-app-parity › feature areas](docs/target-app-parity.md#feature-areas).

## languages

measured with the `cargo xtask i18n-coverage` key universe (1,608 keys). `partial` everywhere
because engine messages are still english. detail: [localization-parity](docs/localization-parity.md).

| language | code | ui strings | status |
|---|---|---:|---|
| english | `en` | 100% | full |
| simplified chinese | `zh-hans` | 100% | partial |
| spanish | `es` | 100% | partial |
| hindi | `hi` | 0% | none |
| arabic | `ar` | 0% (RTL shell only) | none |
| french | `fr` | 100% | partial |
| portuguese (brazil) | `pt-br` | 100% | partial |
| indonesian | `id` | 100% | partial |
| japanese | `ja` | 100% | partial |
| german | `de` | 100% | partial |
| korean | `ko` | 100% | partial |
| vietnamese | `vi` | 0% | none |

also shipped: traditional chinese, russian, ukrainian, czech, polish, greek, dutch, italian (8).

## upcoming

| rank | milestone | estimate |
|---|---|---:|
| 1 | PSD that photoshop trusts: save as corruption, photoshop re-open check, photoshop-oracle floor 145 → 200+ of 258 | 150–250 h |
| 2 | performance budgets: the 22 over-budget scenarios and the 150-layer GPU crash | 200–350 h |
| 3 | user bug backlog to under 100 open | 250–400 h |
| 4 | brush and transform feel (brush-parity, ui-parity rows) | 160–290 h |
| 5 | the 12 missing tools | 60–100 h |

detail and the next releases: [docs/roadmap.md](docs/roadmap.md).

## progress log

| date | entry |
|---|---|
| 2026-10-10 | full re-measure against photoshop 2026 27.11.0 (installed bundle inspected): breadth ~76%, ready ~45%, alpha. progress docs reorganized to the craftrules standard (`parity.md` → `parity-checklist.md`, `parity-estimate.md` merged into `target-app-parity.md`). 0.6.0 released. |
| 2026-10-09 | remove tool, rotate view, crop rotation; OpenRaster and paint.NET; 39 affinity documents in the corpus |
| 2026-10-08 | 0.5.0. affinity import; magnetic lasso, pattern stamp, mixer brush, pencil, patch, vertical type on the toolbar |
| 2026-10-07 | 0.3.0. 10 ui languages; shortcut audit 214 → 0 failures; TIFF decode 10× faster |
| 2026-10-05 | 0.2.0, first signed and notarized release. honest assessment: ready for real work 25–35% |
| 2026-10-03 | menu breadth 625/625 |
| 2026-10-01 | menu breadth 224 → 532 of 625 in a day |
| 2026-09-30 | first commit |

## revision history

| date | change | summary |
|---|---|---|
| 2026-10-13 | minor | focus item 1 quoted the oracle floor as 133 of 256; it is 145 of 258 (`crates/io/tests/corpus.rs`, gaps.md g1). open issues 579 → 953 (2026-10-13) |
| 2026-10-11 | minor | toolbox count 56 of 68 (12 missing) with the convert point tool (#3204) |
| 2026-10-10 | minor | readiness by audience with hours per tier |
| 2026-10-10 | minor | mainstream-practitioner and essentials-user numbers added |
| 2026-10-10 | minor | alpha gate result (passes; stays alpha) in the stage explanation |
| 2026-10-10 | major | first version, per craftrules `standards/progress-docs.md` |
