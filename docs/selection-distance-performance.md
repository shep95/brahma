# selection distance transforms

the distance transform used by select > modify > expand, contract and border now runs
as two native parallel passes. it also serves matting and mask feathering callers in the
engine. the compositor has a separate implementation and is outside this change.

the first pass finds the nearest selected pixel above and below each pixel. since its
input is binary, two sweeps replace the general parabola envelope for that axis. vertical
distances are stored as integers in column order. the second pass copies 32-row bands
into contiguous rows and evaluates the lower envelope of squared-distance parabolas.
both passes are linear in the number of pixels. WebAssembly uses the same scalar kernels.

the row envelope follows felzenszwalb and huttenlocher, 2012.
its intersections use `(fq - fp) / (2 * (q - p)) + (q + p) / 2` in f64. this avoids the old
subtraction of f32 coordinate squares, which lost half-pixel boundary precision on large
canvases. distances are rounded to f32 only at the output.

## correctness

the previous transform can return a positive distance at a selected pixel past coordinate
4096. a 6000 x 1 fully selected mask reproduces it. the regression tests also compare sparse,
competing sites in 65537 x 2 and 2 x 65537 masks against direct euclidean distances.

tests enumerate every mask up to 4 x 3 against a brute-force oracle, check rectangular and
partial row bands against the previous implementation, preserve fractional expand/contract/
border coverage on smaller inputs, and compare native worker counts 1, 2 and 4. empty or
inconsistent dimensions return an empty buffer. no-seed masks retain the finite 1e10 result.
empty and fully selected masks skip the transform passes.

large benchmark inputs check every selected pixel has zero distance and check independent
nearest-site distances at corners and the centre. the sparse-grid cases match the old
output exactly. ellipse and dense cases intentionally correct old distance errors, so this
is not a claim of identical output everywhere or of measured photoshop parity.

## measurements

recorded 2026-10-10 on an AWS c7i.4xlarge in ap-southeast-2: 16 vcpus, 32 gib RAM,
rust 1.99.0, the workspace's normal release profile, 16 rayon workers. synthetic masks are
an ellipse, a sparse regular grid, and a deterministic dense pattern. input generation,
reference validation and destruction of the output happen outside the timed interval.
each implementation gets a warmup, then three measured runs with alternating order.

| size | mask | before p50 | after p50 | speedup | changed distances | largest correction |
|---|---|---:|---:|---:|---:|---:|
| 24 mp | ellipse | 630.74 ms | 83.11 ms | 7.59x | 827,500 | 1 px |
| 24 mp | sparse | 621.81 ms | 71.03 ms | 8.75x | 0 | 0 px |
| 24 mp | dense | 1048.21 ms | 99.73 ms | 10.51x | 2,034,343 | 1.4142135 px |
| 36 mp | ellipse | 994.14 ms | 124.48 ms | 7.99x | 2,431,817 | 1 px |
| 36 mp | sparse | 984.52 ms | 109.94 ms | 8.96x | 0 | 0 px |
| 36 mp | dense | 1588.23 ms | 150.28 ms | 10.57x | 4,771,985 | 1.4142135 px |

these are kernel timings. mask conversion, selection history, document compositing and
canvas upload are excluded. they do not establish an end-to-end interaction budget.

## memory and limits

the old transform used one f32 plane. the new transform needs one u32 plane plus the f32
result, or eight bytes per pixel, excluding the caller's input mask. it also uses up to
`32 * width * 4 + width * (size_of::<usize>() + 8)` scratch bytes per active row task.
at 24 mp the two planes total 192 mb; at 36 mp they total 288 mb. fully empty and fully
selected masks need only the result plane. small mixed masks below 65536 pixels run serially.

the extra plane buys contiguous writes and independent parallel bands. this remains a dense
transform, so it does not solve streaming selections on very large PSB documents. dimension
multiplication, element-size limits and input length are checked before allocation.

## reproduce

```sh
cargo test --release -p brahma-algo distance_transform_release_comparison -- --ignored --nocapture
EDT_WORKERS=1 cargo test --release -p brahma-algo distance_transform_release_comparison -- --ignored --nocapture
```

the previous scalar implementation is retained only in the test module as the reference
and benchmark baseline.
