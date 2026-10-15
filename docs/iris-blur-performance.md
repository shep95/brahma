# iris blur computation (2026-10-10)

base: `7722172585a01cbdb93c06f0f5ff2634fcb17999` (freshly fetched
`origin/main` and `upstream/main`). branch: `codex/iris-blur-invariants-20261010`.

## upstream investigation

open, closed and merged issues/prs were searched for iris, blur gallery, blur,
sampling, portable and SIMD. no overlapping iris blur change was found.
gallery history includes [pr #1010](https://github.com/shep95/brahma/pull/1010),
which normalizes nonfinite pin angles and maps nonfinite distance to infinity
(part of #916). its behaviour is retained in prepared iris geometry and tested.
[pr #1524](https://github.com/shep95/brahma/pull/1524), merged as
`d651be1393b709d638dd9d0e4824a420f795d3ae`, is the merged SIMD-related work:
the [review](https://github.com/shep95/brahma/pull/1524#issuecomment-6072959394)
requested removal of the AVX2/pulp portion because it did not improve complete
filter timings. the author removed that portion before merge. main has portable
contiguous mosaic row reads, with exact-output tests, rather than a SIMD subsystem.

[pr #1602](https://github.com/shep95/brahma/pull/1602) carries fixed-channel
gaussian/resize accumulators from #1511, allowing compiler vectorization without
unsafe code or a new dependency. those box accumulators use f32; gallery uses f64
running sums for tile independence, so they are not an exact drop-in replacement.
no nightly `std::simd`, architecture-specific instructions, new dependency or
`target-cpu=native` is introduced here. transitive codec SIMD dependencies do not
provide a gallery kernel.

[pr #1899](https://github.com/shep95/brahma/pull/1899) already runs filter
previews on a cancellable worker, with a 150 ms settle delay for stale requests.
[pr #2207](https://github.com/shep95/brahma/pull/2207) addresses preview scale.
this change leaves the worker, proxy size, engine command and ui interactions intact.

## algorithm and numerical contract

iris blur interpolates seven gaussian levels (zero plus six geometrically spaced
sigmas). this is not a ray/disk sampling kernel. every level, box radius, weight,
premultiplication, unpremultiplication and pin combination remains the same.

- prepare each pin's centre, rotation, normalized radii, exponent, feather and
  blur once per tile. allocation failure uses on-demand preparation instead.
- a point outside a pin's normalized bounding square is beyond its superellipse.
  its falloff is saturated; skip the square root/powers there. inside the square,
  use the original expression, divisions and floating-point order.
- copy interleaved source samples together and premultiply in place, following
  the portable row-read principle in #1524.
- for each nonzero level, retain the source's top/left and trim the right/bottom
  to the output plus that level's actual three-box reach. removing leading rows
  or columns would change running-sum cancellation, especially for HDR. keeping
  the start preserves exact results; the trailing clamp cannot reach output.
  tilt-shift and field retain their full windows.

there is no additional image-sized cache. the premultiplied window, sigma map,
output accumulator and reused level buffer already existed. the level buffer
reserves the original full-window capacity once when needed, preventing row
appends from doubling capacity; only its populated length and blur scratch shrink.
prepared geometry uses 36 bytes per pin per active tile. source reads, tile sizes,
rayon dispatch, selection mixing and cancellation services remain unchanged.

`tests_iris_reference.rs` freezes the base kernel. differential tests require
exact finite f32 bits (including signed zero), with matching nan classifications.
they exercise 1–6 channels, RGB/gray/CMYK/lab, alpha/no alpha, u8/U16/F32, negative
and HDR samples, extreme HDR cancellation, rotated/translated/anisotropic pins,
roundness and blur thresholds, feather 0.998, image edges, cropped output, full
focus/full blur, empty/multiple/4096 pins, degenerate images, nonfinite/extreme pin
fields, selections and multiple tile sizes. cancellation before work and after
progress preserves the source. invalid optimized source lengths/zero channels
return an empty internal result. public API/command behaviour is unchanged.

## measurement method

windows x86_64, AMD ryzen 7 9700x (8 cores / 16 logical), 64 gib RAM, stable rust
1.99.0 / LLVM 23.1.1. normal workspace release profile (opt-level 3, thin LTO, one
codegen unit), eight rayon workers. no concurrent cargo builds or other task
benchmarks were scheduled during the measurements; ordinary system activity remains.

`bench_iris` constructs deterministic textured RGB layers with varied alpha and
HDR F32 values. fixture construction and output destruction are excluded; source
reads, pin preparation, filtering, selection-free surface writeback and pruning
are included. each case has one untimed warmup, then repeated complete calls.
the unchanged base executable is retained locally for repeat comparisons. use
the same harness on the base revision to reproduce a comparison.

```powershell
$env:CARGO_TARGET_DIR = 'target/agent-iris-blur'
$env:RAYON_NUM_THREADS = '8'
cargo run --release -p brahma-algo --example bench_iris -- 6000 4000 3
cargo run --release -p brahma-algo --example bench_iris -- 6000 6000 3
cargo run --release -p brahma-algo --example bench_iris -- 1500 1000 5 4
cargo run --release -p brahma-algo --example bench_iris -- 2000 1334 5
cargo run --release -p brahma-algo --example bench_iris -- 128 96 5
cargo test --release -p brahma-algo iris_profile_baseline_stages -- --ignored --nocapture
```

the 2000×1334 run is a preview-sized stress workload at fixed blur parameters.
brahma actually reduces 24 mp by factor four to 1500×1000 (its factor is the
ceiling of sqrt(pixels/2,500,000)). the separate 1500×1000 run models that existing
filtered-proxy workload directly: top-level document blur 15/80/300 becomes
3.75/20/75; the nested multiple-pin JSON amounts retain their existing values.
neither run measures proxy construction, engine history, preview-worker queueing,
compositing, GPU upload, frame presentation or native slider response. baseline and
candidate always use identical resolution, blur values and sample counts.

before-change stage profiling of a 512×512 output tile, one rotated rounded pin:
geometry 4.3–5.8 ms; premultiplication 1.5/3.5/11.7 ms at blur 15/80/300;
each active gaussian level about 3–9/18–19/87–100 ms. whole kernels were
30.8/52.1/208.8 ms (unused levels are skipped). these diagnostic single-tile
figures isolate the computation; end-to-end results below use repeated runs.

## repeated release results

values are median milliseconds (minimum–maximum), not frame times. run order:
base 24 mp / stress / small, candidate 24 mp / stress / small, base 24 mp again;
then base 36 mp, candidate 36 mp, base 36 mp again, base default proxy and candidate
default proxy. each full-size base column pools six samples from two three-repeat
rounds; each candidate has three. proxy, stress and small-image columns each have
five samples.
both base rounds have slower medians than the candidate in every 24 mp case.
variation is visible, particularly in the larger proxy runs; these are local
measurements, not cross-machine performance guarantees.

small = blur 15; medium = blur 80, rotation 27°; large = blur 300.
multiple = two pins at blur 80/40, one with roundness 65 and one rotated 73°.
focused = radii 5×5, blur 80; blurred = pin centre x = −5, blur 80.

### 6000×4000 (24 mp)

| depth / case | base ms (range) | candidate ms (range) | speedup |
|---|---:|---:|---:|
| u8 medium | 1395.7 (1191.4–1550.9) | 1092.7 (1070.0–1113.7) | 1.28× |
| u8 small | 626.8 (589.8–655.9) | 426.9 (425.8–437.3) | 1.47× |
| u8 large | 4760.6 (4369.7–5154.0) | 3148.4 (3112.9–3175.3) | 1.51× |
| u8 multiple | 1789.9 (1695.9–1885.7) | 1016.2 (1015.6–1018.6) | 1.76× |
| u8 focused | 489.8 (472.8–527.3) | 405.3 (400.6–423.1) | 1.21× |
| u8 blurred | 1161.4 (1102.5–1195.4) | 981.3 (924.7–1007.1) | 1.18× |
| U16 medium | 1522.5 (1410.5–1577.9) | 1115.9 (1108.9–1119.3) | 1.36× |
| F32 medium | 1570.8 (1453.0–1656.7) | 1131.1 (1117.1–1134.3) | 1.39× |

### 6000×6000 (36 mp)

| depth / case | base ms (range) | candidate ms (range) | median ratio |
|---|---:|---:|---:|
| u8 medium | 1976.4 (1842.1–2360.1) | 1923.9 (1745.4–1993.3) | 1.03× |
| u8 small | 854.1 (773.1–947.7) | 806.0 (784.6–818.5) | 1.06× |
| u8 large | 6162.4 (5767.0–6578.0) | 5423.8 (5245.8–5995.2) | 1.14× |
| u8 multiple | 2595.8 (2544.7–2731.7) | 1822.8 (1781.5–1826.7) | 1.42× |
| u8 focused | 708.7 (684.9–740.7) | 628.6 (618.0–630.7) | 1.13× |
| u8 blurred | 1705.5 (1585.5–1821.2) | 1609.4 (1561.4–1613.6) | 1.06× |
| U16 medium | 2217.2 (2034.6–2438.8) | 2065.5 (1961.2–2127.9) | 1.07× |
| F32 medium | 2346.8 (2318.4–2411.9) | 1702.1 (1647.2–1795.3) | 1.38× |

the second base round was faster than the candidate for u8 small and medium,
despite better pooled medians: neither demonstrates a consistent improvement at
36 mp. U16 medium and fully blurred ranges also overlap; their gains are
inconclusive. large blur improves both round medians but ranges overlap.
multiple pins, full focus and F32 medium have separated observed ranges.

### 1500×1000 default filtered proxy for a 24 mp document

single-pin blur values below are the document values divided by four; multiple
pins retain the supplied JSON blur values, matching the existing preview path.

| depth / case | base ms (range) | candidate ms (range) | speedup |
|---|---:|---:|---:|
| u8 medium | 51.8 (47.9–54.9) | 42.6 (39.3–45.5) | 1.22× |
| u8 small | 28.7 (27.4–31.0) | 24.9 (23.5–25.9) | 1.15× |
| u8 large | 144.0 (132.5–153.0) | 110.6 (104.0–111.9) | 1.30× |
| u8 multiple | 178.6 (174.8–205.9) | 111.6 (109.3–114.7) | 1.60× |
| u8 focused | 18.0 (17.1–19.0) | 15.0 (14.8–15.9) | 1.20× |
| u8 blurred | 33.1 (31.9–36.9) | 28.2 (27.8–29.9) | 1.17× |
| U16 medium | 52.1 (48.1–55.3) | 42.8 (40.7–49.4) | 1.22× |
| F32 medium | 60.4 (56.6–63.5) | 45.0 (43.9–50.0) | 1.34× |

every default-proxy median improves. U16 medium ranges overlap, so that case's
gain is less certain than the other seven separated-range cases.

### 2000×1334 preview-sized stress workload

| depth / case | base ms (range) | candidate ms (range) | speedup |
|---|---:|---:|---:|
| u8 medium | 306.0 (280.6–465.1) | 163.8 (150.1–203.0) | 1.87× |
| u8 small | 82.5 (75.0–87.4) | 59.0 (58.2–63.4) | 1.40× |
| u8 large | 2051.0 (1724.3–2087.2) | 1324.2 (1106.4–1533.7) | 1.55× |
| u8 multiple | 303.6 (289.2–330.7) | 209.1 (200.5–231.5) | 1.45× |
| u8 focused | 61.5 (60.6–73.2) | 55.4 (51.6–78.7) | 1.11× |
| u8 blurred | 151.0 (146.5–155.1) | 143.3 (136.3–145.8) | 1.05× |
| U16 medium | 269.7 (262.7–289.8) | 216.1 (207.1–249.0) | 1.25× |
| F32 medium | 252.5 (237.9–295.5) | 228.0 (207.6–254.8) | 1.11× |

the already fully blurred stress case has little distance/level work to eliminate;
its improvement is modest. focused and F32 stress ranges overlap, so their small
median gains are inconclusive. these cases do not justify additional machinery.

### 128×96 overhead

| depth / case | base ms (range) | candidate ms (range) | speedup |
|---|---:|---:|---:|
| u8 medium | 30.8 (30.3–31.3) | 17.7 (17.7–17.8) | 1.74× |
| u8 small | 4.1 (4.0–4.1) | 3.2 (3.1–3.3) | 1.26× |
| u8 large | 280.4 (276.1–294.6) | 149.1 (145.2–151.2) | 1.88× |
| u8 multiple | 26.0 (25.7–26.6) | 13.0 (13.0–13.1) | 1.99× |
| u8 focused | 1.7 (1.6–1.7) | 1.2 (1.2–1.3) | 1.36× |
| u8 blurred | 6.9 (6.9–7.0) | 6.3 (6.3–6.4) | 1.09× |
| U16 medium | 30.4 (30.2–31.0) | 18.3 (18.3–18.3) | 1.67× |
| F32 medium | 30.1 (30.0–30.5) | 18.3 (18.2–18.7) | 1.64× |

windows process peak working set for one entire 24 mp suite (including fixture
creation and all depths/cases): base repeat 2,068,135,936 bytes, candidate
1,990,852,608 bytes, about 3.7% lower. this is an allocator-dependent process
high-water measurement sampled via `PeakWorkingSet64`, not per-filter live memory
or a new allocation bound. existing full-window buffer capacities remain the same.

maximum observed finite pixel error in the differential tests is **zero**: exact
bits, rather than an average-error bound. matching every finite pixel covers
focus-transition structure, sampling aliasing, opacity and tile boundaries without
permitting a localized artifact to hide in a mean error. nan classifications match.

## validation and limits

algorithm tests: 399 passed, 15 existing/diagnostic tests ignored. the five iris
differential/cancellation tests also pass in release; their profiling test is
opt-in. all-target
algorithm clippy passes with warnings denied. layering passes (29 crates). wasm
checks pass for all required crates and HEIF; unrelated existing dead-code warnings
in CMS/engine remain. scorecard regeneration produces identical content; no budget,
corpus floor, menu parity or checklist status changed. `test-corpus --changed`
correctly skips this algorithm-only change. commands and ui code are unchanged,
so command panic-hunt and ui screenshot checks are not triggered.

`cargo xtask perf --quick` completed successfully, running its configured quick
native CPU/GPU examples. its regression comparison was skipped because the
checked-in baseline is for an apple m4 pro, while this machine is windows/ryzen
with a radeon rx 9060 xt. quick mode does not enforce the full-size budgets.
results are retained locally in `target/agent-iris-blur/perf/results.json`;
the iris base/candidate measurements above provide this change's comparison.
final complete wasm and algorithm formatting checks also pass.

final diagnostic single-tile reference/candidate calls (same profile workload,
one pair each, not a statistical benchmark): blur 15, 31.6 → 25.3 ms; blur 80,
52.9 → 43.1 ms; blur 300, 224.1 → 174.7 ms. the repeated complete-filter results
remain the performance evidence used to retain the change.

native iris GUI frame/slider latency, ARM/macos/linux performance and wasm runtime
speed were not measured. compilation confirms wasm compatibility only. photoshop
equivalence beyond the base implementation's semantics is not asserted. the private
shared craftrules checkout was unavailable at the supplied locations; repository
AGENTS.md and architecture/development/contributing/control/performance rules were
followed. existing source allocation and per-tile cancellation limits are retained.
