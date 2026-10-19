# brahma-aesthetic

aesthetic recommendations computed from a design-research corpus: colour schemes in oklch, type
pairings and modular scales, placement and measurement (grid, target sizes, gestalt spacing,
weber-ratio steps), and motion timing. plus an image analyser that reads a picture's statistics
(lightness, contrast, edge density, chroma, hue spread) and names the emotional register they
predict, following the finding that the response to an image arrives before its content is
recognised.

layer 2: depends on nothing in the workspace. no ui, no i/o.

## public api

| item | what |
|---|---|
| `Request { register, base, surface, density, touch }` | what to recommend for |
| `recommend(&Request) -> Recommendation` | palette, typography, layout, motion, cited principles |
| `analyze(w, h, rgba) -> Option<Analysis>` | image statistics and the register they predict |
| `recommend_for_image(Request, &Analysis)` | the analysis fills the register and base the request left open |
| `search(query, limit)` | ranked corpus entries |
| `Oklch` | the colour space the palettes are built in, with gamut-safe `to_srgb8` |

## the corpus

`data/corpus.json` is generated from the research notes by `scripts/distill.py` (one entry per
pattern, finding, technique or convergence: title, short summary, domain, tags). regenerate
with a new or longer notes file:

```sh
python3 crates/aesthetic/scripts/distill.py notes.txt > crates/aesthetic/data/corpus.json
```

the rule base in `src/rules.rs` carries the numbers the recommender computes with; each rule's
comment names the corpus entry it comes from.

## engine commands

`aesthetic.recommend`, `aesthetic.analyze` (the active document) and `aesthetic.search` in
`brahma-engine`; the aesthetic advisor window (window › aesthetic advisor) draws them.
