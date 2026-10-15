# tilt-shift computation (2026-10-10)

starting base: `a28d786638ffbb84c5fb7d5d30ea1c3d594cd3bc`, freshly fetched
`origin/main`. branch: `codex/tilt-shift-performance-20261010`. worktree:
`C:\Users\Colum\.codex\worktrees\6c32\brahma`. at investigation time,
`upstream/main` was `5963e5b7f0aefa0d94b5cf67ef3c1c9b6d3399dd`, two commits ahead:
[recent entries](https://github.com/shep95/brahma/commit/b773a39efa4a65888fcce4a5d8dffcf4fa45700a)
and [temporary move](https://github.com/shep95/brahma/commit/5963e5b7f0aefa0d94b5cf67ef3c1c9b6d3399dd).
neither changes filtering. the original checkout, local main and other worktrees
were not edited. cargo uses `target/agent-tilt-shift` throughout.

## upstream evidence and choice

GitHub open/closed/merged prs and open/closed issues were searched for blur,
tilt-shift, field blur, sampling, SIMD, portable, std::simd and rustfft. relevant
bodies, discussions, final diffs, merge commits, gallery history and current
source were inspected before the implementation was selected.

| work | status and evidence | consequence for tilt-shift |
|---|---|---|
| [#1524: portable mosaic reads](https://github.com/shep95/brahma/pull/1524), [merge d651be13](https://github.com/shep95/brahma/commit/d651be1393b709d638dd9d0e4824a420f795d3ae) | the SIMD-related proposal the human referred to. its AVX2/pulp portion was experimental and removed before merge: the [review](https://github.com/shep95/brahma/pull/1524#issuecomment-6072959394) found no complete-filter gain to justify dependencies/features/ci. portable contiguous row reads and exact-output tests landed. | reuse contiguous reads. main has no general portable-SIMD gallery backend or nightly std::simd path to reuse. |
| [#1471: adaptive rustfft motion blur](https://github.com/shep95/brahma/pull/1471), [merge c7dbb37b](https://github.com/shep95/brahma/commit/c7dbb37b3c61c18223e56c10d06efc71e48c7677) | merged. builds on [#902](https://github.com/shep95/brahma/pull/902)'s merged bilinear taps/row convolution. wide uniform streaks benefit from FFT; short/unsupported cases retain rows. current code bounds FFT allocations and rejects unsuitable coordinates/samples from that path. | tilt-shift is spatially varying, not one uniform convolution. a single FFT would change the band; one FFT per existing gaussian level would preserve spatial blending but replace linear-time box cascades with transforms, padding, complex scratch and numerical differences. no demonstrated crossover warrants that machinery here. |
| [#867: gaussian running sums and bands](https://github.com/shep95/brahma/pull/867), [#628: box blur](https://github.com/shep95/brahma/pull/628), [#1602: fixed-channel accumulators](https://github.com/shep95/brahma/pull/1602) carrying closed [#1511](https://github.com/shep95/brahma/pull/1511) | merged, with radius-independent separable passes, traversal/allocation improvements and compiler-friendly accumulators. | gallery already uses separable boxes, with f64 running sums. replacing them with the f32 gaussian accumulators would change HDR cancellation/rounding. avoid a broad kernel/traversal rewrite. |
| [#1499: radial sampling](https://github.com/shep95/brahma/pull/1499) | merged after [review](https://github.com/shep95/brahma/pull/1499#issuecomment-6071866387) removed a changed-output polar approximation and a separate preview scheduler. exact transform preparation and interleaved taps remained. | preserve the kernel and reuse current services. tilt-shift already prepares its band geometry once per tile and has no radial/disk samples to precompute. |
| [#2578: iris blur](https://github.com/shep95/brahma/pull/2578), [merge fa33712d](https://github.com/shep95/brahma/commit/fa33712dd3bd87c4502c01dbeae3d235383e1b46) | already merged in the starting base. prepares iris geometry and adds exact-output contiguous premultiplication and per-level trailing-halo trimming to the shared helper; only iris opts in. | opt tilt-shift into that established helper. no new algorithm, dependency, cache or scheduler is needed. field remains unchanged. |
| [#1899: preview worker/cancellation](https://github.com/shep95/brahma/pull/1899), [#1825: keep preview during commit](https://github.com/shep95/brahma/pull/1825), [#2207: preview scale](https://github.com/shep95/brahma/pull/2207) | merged. latest-request worker; cancel stale work after 150 ms without parameter changes, preserving updates during dragging. pixel parameters scale with the existing proxy. | leave the ui and worker intact; faster tile computation also shortens the existing worker's filtering work. |
| active [#2685](https://github.com/shep95/brahma/pull/2685), [#2580](https://github.com/shep95/brahma/pull/2580), [#2739](https://github.com/shep95/brahma/pull/2739) | respectively spin/lens/extrude/wind/trap sampling, general compression/io/compositing allocation work, and smart-filter stage caching/background jobs. #2685 touches gallery's spin path, not tilt-shift's gaussian field. | no existing tilt-shift fix or overlapping implementation found. keep this change confined to tilt-shift's opt-in and its tests. |

[issue #464](https://github.com/shep95/brahma/issues/464) records gaussian
preview lag and profiling with copying and waits prominent; [#211](https://github.com/shep95/brahma/issues/211)
tracks broader slow algorithms. they inform the need for complete-filter and
preview-workload measurements, rather than claiming a microbenchmark fixes GUI latency.

rustfft's primary documentation
describes o(n log n) transforms and automatic native AVX/SSE/NEON selection, with
different wasm SIMD requirements. that is useful existing infrastructure for
uniform expensive kernels, but does not remove the spatially varying blend or
improve the asymptotic cost of gallery's existing box passes. no FFT or explicit
SIMD candidate is benchmarked here; the comparison above is an algorithmic and
maintenance assessment, not a measured claim that they can never be faster.

## numerical and memory contract

tilt-shift has one band, described by `centerX`, `centerY`, `angle`, `focus`,
`transition` and `blur`; its engine/model has no pins parameter. six nonzero
gaussian levels plus the zero level retain their existing geometric spacing,
three-box radii, smoothstep sigma field and hat weights. sharp/blurred and
transition pixels still blend their original levels. pixel centres, image
coordinates, transparent premultiplication and edge policy are unchanged.

the optimized helper clones the already interleaved source and premultiplies it
with the same per-pixel operations. for each gaussian level it keeps the
original top/left, trimming only right/bottom to the output plus that level's
summed box radii. its artificial trailing edge cannot influence output through
the three finite-support passes. keeping the leading edge preserves the
running-sum initialization, update order and f64 cancellation, including HDR.
no samples or levels are dropped. cropped-output containment retains the
helper's existing fallback to a full window.

the existing source, premultiplied buffer, sigma map, accumulator and reusable
level buffer remain. full-window capacity is still reserved once; populated
length and blur scratch shrink. there is no retained cache, feature dispatch,
threshold or planning cost. source halo reads, auto tile sizes, worker count,
selection mixing, output writeback, undo/redo and cancellation services remain
the same. no change requires a native CPU feature or unsafe code.

one pre-existing crash found by the synthetic tests is fixed: a nan low-level
blur amount previously became f32::clamp's invalid upper bound. it now acts as
identity. the regression asserts the frozen starting implementation panics and
the corrected kernel returns the source. valid finite parameters keep exact bits.

`tests_tilt_shift_reference.rs` freezes the starting scalar gallery/tilt-shift
expressions, using the unchanged gaussian helpers. differential tests compare
every finite f32 bit (including signed zero), with matching nan classifications:
1–6 channels, RGB/gray/CMYK/lab and the expanded bitmap/indexed/duotone/multichannel
storage, u8/U16/F32, alpha/no alpha, narrow and soft band
boundaries, translated/rotated bands, tiny/empty/one-row images, blur thresholds,
full focus/full blur, extreme and nonfinite parameters, HDR/negative/high-contrast
samples, nonfinite pixels, selections, transparent/repeated edges and tile sizes
16/47/128. cancellation before work and after progress preserves the source.
malformed internal source lengths and zero channels are also checked.

## measurement method

windows x86_64, ryzen 7 9700x (8 cores/16 logical), 64 gib installed RAM;
stable rust 1.99.0 / LLVM 23.1.1. normal workspace release profile: opt-level 3,
thin LTO, one codegen unit. eight rayon workers. builds/tests are stopped during
measurements, and process inspection checks for concurrent cargo/rustc/benchmark
work. ordinary system activity and thermal/allocator variation remain.

`bench_tilt_shift` constructs deterministic textured RGBA surfaces with varying
alpha and HDR F32 values. two untimed warmups precede each case. timing covers
complete `apply_in`: source/halo reads, tiled field and level computation, rayon,
writeback and pruning. fixture construction and output destruction are excluded.
it uses the unchanged pipeline, not an isolated hot loop. baseline executables
are built before editing the algorithm and retained in local `plan/`.
the `blurred` case puts the band outside the image through the low-level rust
API; it is a saturated-field stress case, outside the engine dialog's centre
range. all other cases use the supported dialog parameters. no pin list is
invented for tilt-shift. model performance is measured for RGB; other models are
covered by differential correctness tests.

the preview workload models the existing 24 mp document's 1500×1000 filtered
proxy (factor 4), with blur 15/80/300 becoming 3.75/20/75. position, angle, focus
and transition are relative and unchanged. this times filtering a proxy after a
parameter update. it excludes proxy construction, session/history, queue/settle
delay, compositing, GPU upload and presentation; it is not actual GUI input/frame
latency. the implementation does not alter the preview factor or kernel.

```powershell
$env:CARGO_TARGET_DIR = 'target/agent-tilt-shift'
$env:RAYON_NUM_THREADS = '8'
cargo run --release -p brahma-algo --example bench_tilt_shift -- 6000 4000 5 1
cargo run --release -p brahma-algo --example bench_tilt_shift -- 6000 6000 5 1
cargo run --release -p brahma-algo --example bench_tilt_shift -- 1500 1000 7 4
cargo run --release -p brahma-algo --example bench_tilt_shift -- 128 96 7 1
```

## results

order: base a, candidate, base b, with all four sizes completed per implementation
round. full-size cases have five timed samples per round; proxy and small cases
have seven. base p50 pools the two unchanged rounds (10/14 samples); candidate
p50 uses 5/7 samples. ranges include every timed sample, not only best runs.
local raw logs are `plan/tilt-{baseline,candidate,baseline-repeat}-*.jsonl`.

the parameters are fixed by the example: `small` is blur 15, centred, angle 0,
focus 0.1, transition 0.15; `medium` is blur 80 at angle 27; `large` is blur 300,
angle -71, focus 0.03, transition 0.4. `wide_focus` is blur 80, angle 0, focus 0.4,
transition 0.05; `sharp_transition` is centred at (0.2, 0.8), blur 80, angle 63,
focus 0.05, transition 0.01. `focused` uses focus 1; `blurred` uses the low-level
outside-image centre (-5, -5). the latter two use blur 80. proxy blur alone is
divided by four. U16/F32 time the rotated medium case.

### 24 mp (6000×4000)

| depth / case | base ms (range) | candidate ms (range) | speedup |
|---|---:|---:|---:|
| u8 medium | 1729.3 (1530.2–1996.6) | 1241.4 (1227.0–1256.5) | 1.39× |
| u8 small | 609.5 (594.2–616.1) | 451.7 (443.1–479.9) | 1.35× |
| u8 large | 5340.1 (5042.7–5861.9) | 3698.9 (3498.1–3722.3) | 1.44× |
| u8 wide_focus | 1204.9 (1177.8–1273.9) | 828.7 (792.5–874.5) | 1.45× |
| u8 sharp_transition | 1615.6 (1576.2–1636.4) | 1148.4 (1135.1–1153.2) | 1.41× |
| u8 focused | 433.5 (426.1–438.2) | 406.1 (399.6–410.5) | 1.07× |
| u8 blurred | 1105.7 (1051.6–1146.0) | 963.6 (938.0–1045.8) | 1.15× |
| U16 medium | 1935.8 (1877.7–1978.8) | 1306.8 (1268.9–1403.0) | 1.48× |
| F32 medium | 1961.7 (1858.5–2010.5) | 1316.8 (1270.2–1396.5) | 1.49× |

### 36 mp (6000×6000)

| depth / case | base ms (range) | candidate ms (range) | speedup |
|---|---:|---:|---:|
| u8 medium | 2458.7 (2066.6–2541.9) | 1668.0 (1656.9–1682.4) | 1.47× |
| u8 small | 806.6 (733.7–864.2) | 655.1 (627.0–678.9) | 1.23× |
| u8 large | 8856.0 (6766.5–9301.8) | 6176.2 (5780.6–6622.0) | 1.43× |
| u8 wide_focus | 1612.8 (1562.4–1672.7) | 1066.4 (1057.8–1084.3) | 1.51× |
| u8 sharp_transition | 2529.7 (2448.6–2590.1) | 1714.6 (1705.0–1890.4) | 1.48× |
| u8 focused | 646.3 (627.5–680.0) | 614.1 (613.8–618.5) | 1.05× |
| u8 blurred | 1633.2 (1595.3–1687.9) | 1577.6 (1566.4–1623.2) | 1.04× |
| U16 medium | 2618.4 (2498.4–2687.4) | 2020.7 (1994.9–2027.0) | 1.30× |
| F32 medium | 2669.3 (2561.2–2741.6) | 2020.5 (1734.9–2173.3) | 1.32× |

### 1500×1000 filtered preview proxy, factor 4

| depth / case | base ms (range) | candidate ms (range) | speedup |
|---|---:|---:|---:|
| u8 medium | 66.9 (63.1–82.0) | 55.9 (54.9–59.0) | 1.20× |
| u8 small | 29.4 (27.2–32.9) | 28.0 (26.4–29.9) | 1.05× |
| u8 large | 179.4 (157.0–205.6) | 132.8 (116.7–149.6) | 1.35× |
| u8 wide_focus | 52.1 (46.7–72.0) | 40.4 (38.9–44.0) | 1.29× |
| u8 sharp_transition | 64.3 (54.6–75.4) | 44.2 (41.2–45.7) | 1.45× |
| u8 focused | 17.9 (16.7–18.7) | 15.0 (14.6–15.1) | 1.20× |
| u8 blurred | 37.4 (36.1–48.0) | 29.5 (28.2–33.0) | 1.27× |
| U16 medium | 89.2 (81.3–95.8) | 58.1 (56.0–59.0) | 1.54× |
| F32 medium | 93.7 (83.1–102.4) | 60.3 (57.9–65.8) | 1.55× |

### 128×96 small-image overhead

| depth / case | base ms (range) | candidate ms (range) | speedup |
|---|---:|---:|---:|
| u8 medium | 30.25 (30.09–31.83) | 18.14 (17.69–19.34) | 1.67× |
| u8 small | 3.93 (3.85–3.99) | 3.15 (3.13–3.22) | 1.25× |
| u8 large | 278.90 (273.62–313.48) | 145.97 (142.65–148.51) | 1.91× |
| u8 wide_focus | 26.08 (25.29–27.79) | 15.48 (15.38–15.69) | 1.69× |
| u8 sharp_transition | 31.12 (30.48–46.89) | 17.40 (17.35–17.65) | 1.79× |
| u8 focused | 1.56 (1.50–1.64) | 1.07 (1.06–1.09) | 1.46× |
| u8 blurred | 7.57 (6.82–11.06) | 6.19 (6.05–7.02) | 1.22× |
| U16 medium | 30.99 (30.58–32.48) | 17.94 (17.61–18.07) | 1.73× |
| F32 medium | 31.53 (30.28–48.73) | 17.90 (17.82–18.74) | 1.76× |

all nine 24 mp cases have separated observed ranges. gains are modest for the
focused and saturated stress fields. at 36 mp the saturated field's ranges
overlap; no convincing gain is established there. default blur 15 on the preview
proxy also has overlapping ranges: its 1.05× pooled median change is inconclusive.
medium/large proxy cases improve 1.20/1.35×, and U16/F32 medium proxy cases improve
1.54/1.55×. the small-image saturated case is also inconclusive; the remaining
small-image cases improve 1.25–1.91× with separated ranges.

there is meaningful drift, so the pooled p50 must not imply precise repeatability:
24 mp u8 medium base a/b medians are 1909.1/1564.3 ms (candidate 1241.4), giving
1.54/1.26× against the individual rounds. the 36 mp large case's base a/b medians
are 7234.7/9170.0 ms (candidate 6176.2), giving 1.17/1.48×. process CPU sampling
during the repeat found the benchmark using about 7.6 cores and no other heavy
CPU consumer. the report retains the full variation rather than attributing it
to a proven cause.

for one complete 24 mp suite (fixture construction, warmups and all depths/cases
included), sampled windows `PeakWorkingSet64` is 2,162,429,952 bytes for the base
and 2,044,227,584 for the candidate, about 5.5% lower (2.01 → 1.90 gib).
this is an allocator-dependent process high-water measurement, not a per-filter
live-memory bound. allocation counts are not instrumented; the source-copy,
sigma-map, level-metadata, output and level-buffer allocations remain, and full-window buffer
capacity is retained. smaller populated levels and scratch account for the
structural reduction in work. neither SIMD nor FFT crossover performance is
claimed; no adaptive threshold is introduced.

maximum observed finite pixel error is **zero** in the differential tests. every
pixel must match, so localized band-edge, seam, opacity or aliasing errors cannot
hide in an average-error metric. nan classifications match. the nan blur crash
regression intentionally differs from the starting kernel by returning identity.

## validation and remaining limits

the full `brahma-algo` suite passes: 425 tests, 18 ignored, no failures.
all six tilt-shift differential/regression tests also pass in release. strict
all-targets clippy passes for the algorithm crate. the existing engine filter
integration group passes all 16 tests, including selection, repeat, undo and
16/32-bit filtering. `cargo xtask layers` reports 29 crates with no violations;
`cargo xtask wasm` passes the required workspace packages and the HEIF-enabled
codec check. regenerating the scorecard produces identical content.

`cargo xtask perf --quick` exits successfully; all five configured quick benches
(`perf_scenarios`, `type_bench`, `layout_bench`, `camera_raw_bench`, `bevel_bench`)
complete. reports are retained locally in
`target/agent-tilt-shift/perf/{results.json,summary.md}`. the regression comparison
is skipped because the checked-in baseline is from an apple m4 pro/macos machine
class. quick mode does not enforce full-size budgets. the layout harness still
reports unrelated frame overruns; completing this gate does not establish
broader responsiveness or budget compliance. formatting and git whitespace
checks pass.

algorithm checks also pass for aarch64/x86_64 macos, aarch64 windows and
aarch64/x86_64 linux. these are compilation checks from windows, not executions
or performance measurements on those platforms. native timing evidence is
limited to this windows x86_64 machine. `cargo xtask test-corpus --changed`
correctly skips its suites because no format, io, compositor, GPU or text crate
changed. command registrations and ui code are unchanged.

the measurements establish equivalence to the starting brahma kernel, not a
new photoshop oracle or broader parity increase. cold startup, actual GUI
input/frame latency, allocation counts and other platforms' native performance
remain unmeasured. no performance budget, scorecard checklist or parity floor is
raised from these local results. the shared craftrules checkout was unavailable
at the documented paths; the repository's own contributor and validation rules
were followed.
