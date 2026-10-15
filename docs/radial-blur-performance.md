# radial blur sampler optimization

the tiled filter prepares spin rotations or zoom scales once per operation and
reuses them across output tiles. bilinear taps load all channels together, with
specialized loops for one through five channels and a generic fallback. rayon
continues to parallelize output tiles on native builds.

draft, good and best keep upstream's 64, 256 and 4096 interval ceilings, sample
positions, transparent edges and alpha arithmetic. the positive-alpha
unpremultiply/premultiply sequence is retained to preserve floating-point rounding.
tests compare against the unchanged upstream sampler at large radii across
gray, RGB, CMYK and lab, with and without alpha, at u8, U16 and F32 depths.
selection blending and unusual float alpha are covered as well.

only interval counts reachable within the output area are prepared; small best
images avoid allocating/preparing the full roughly 64 mib transform table.
preparation and output allocations use checked sizes and fallible reservation.
if the optional optimization cannot prepare its tables, the existing sampler
runs instead. cancellation is checked during preparation and within output tiles.

this revision adds no source-size rejection: it retains upstream's shared
normalized source read and therefore still requires memory proportional to source
size. it does not claim bounded source memory or unlimited physical capacity.
the polar approximation and background preview scheduler are separate work.

## local comparison (2026-10-09)

| method | upstream | optimized | speedup | output |
|---|---:|---:|---:|---|
| spin | 66.10 s | 16.43 s | 4.02× | exact match |
| zoom | 19.19 s | 5.35 s | 3.59× | exact match |


baseline: current upstream sampler at main `578bc905`, through `kernel`, using
the same shared source read, 256px output tiles, 256 mib result batches, surface
writeback and pruning as the tiled filter. optimized: `apply_in`.
the benchmark's baseline wrapper covers full-canvas, unselected filtering only;
selection behavior is checked separately in tests.

machine: intel core i7-9750h (6 cores / 12 logical workers), linux, rust 1.98.1,
release profile with opt-level 3, thin LTO and one codegen unit. source: deterministic
textured RGBA8 with transparent pixels, 6000×4000 (24 mp). good quality, amount 1,
center (0.5, 0.5), 12 rayon workers. one paired run per method, upstream first,
after a small untimed warmup. entire filtering calls are timed; fixture generation
and exact comparison of every output tile are outside the timer.

these are single local measurements, not a cross-machine baseline or an
amount-100 performance claim. the lower amount keeps this review benchmark
bounded on the local laptop. quality ceilings are verified separately at large
radii. the example defaults to three alternating-order runs and amount 100 for
longer measurements. historical handoff polar/default timings do not describe
this output-preserving implementation.

```sh
cargo run --release -j 12 -p brahma-algo --example bench_radial -- 6000 4000 1 1
```
