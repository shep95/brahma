# facet and color halftone performance

2026-10-10. two changes remove repeated work from the existing pixelate filters:

- **facet:** calculate luma once per source pixel and calculate each 3x3 quadrant once. a three-row ring shares that quadrant with its four output pixels. the nine-sample addition order, variance calculation, quadrant order and strict tie comparison stay the same.
- **color halftone:** memoize the five-tap ink calculation and dot radius in the rotated screen grid, separately for each channel. pixels still use the same candidate dots, distances and anti-alias coverage. lab conversion, CMYK ink direction and alpha handling stay the same.

facet uses about 291 kib of additional scratch for a 256x256 output tile. the luma plane is capped at 1,048,576 samples. halftone tables are capped at 2 mib per channel. scratch allocations are fallible; an incomplete halo, oversized table or unsuitable screen coordinates uses the original calculation. the default tiled filter path stays parallel on native targets and single-threaded on wasm.

## full filter timings

EC2 m7i.4xlarge, 16 vcpus (intel xeon platinum 8488c), ubuntu 24.04, rustc 1.99.0, default release profile, `RAYON_NUM_THREADS=16`. base: `7722172585a01cbdb93c06f0f5ff2634fcb17999`. the changed algorithms are in `1bc3a9f1`. before and after were built in separate target directories; no other builds or benchmarks ran during measurement.

each entry is the median of three alternating pairs: before/after, after/before, before/after. the timer covers `apply_in`, including tile reads, filtering, assembly, depth conversion, writeback and pruning. fixture construction, the small warmup and output serialization are outside the timer. the fixture is a dense deterministic native-color image with varying alpha, negative document origin and transparent pixels. these are CPU filter application times; window presentation and undo bookkeeping are outside this measurement.

| filter | size | model + alpha | depth | before (ms) | after (ms) | speedup |
|---|---:|---|---|---:|---:|---:|
| facet | 24 mp | RGB | u8 | 684.4 | 311.5 | 2.20x |
| halftone r8 | 24 mp | RGB | u8 | 1137.3 | 603.8 | 1.88x |
| facet | 36 mp | RGB | u8 | 1027.5 | 454.5 | 2.26x |
| halftone r8 | 36 mp | RGB | u8 | 1707.8 | 898.0 | 1.90x |
| facet | 24 mp | RGB | u16 | 756.9 | 370.1 | 2.04x |
| halftone r8 | 24 mp | RGB | u16 | 1198.9 | 664.7 | 1.80x |
| facet | 24 mp | RGB | f32 | 806.9 | 435.7 | 1.85x |
| halftone r8 | 24 mp | RGB | f32 | 1272.0 | 725.6 | 1.75x |
| halftone r2 | 24 mp | RGB | u8 | 1417.8 | 716.4 | 1.98x |
| halftone r64 | 24 mp | RGB | u8 | 1138.9 | 587.5 | 1.94x |
| facet | 24 mp | CMYK | u8 | 7888.4 | 574.2 | 13.74x |
| halftone r8 | 24 mp | LAB | u8 | 3588.7 | 718.3 | 5.00x |

every pair was compared with `cmp` over the complete output, serialized as little-endian f32 sample bits in tile order. all 36 comparisons matched exactly, including the 16-bit and float cases. [raw three-run timings and output SHA-256 hashes](pixelate-performance.json) are committed alongside this table.

## reproduce

build the same example against the base and the changed tree. keep their cargo target directories separate:

```sh
git worktree add --detach ../brahma-pixelate-base 7722172585a01cbdb93c06f0f5ff2634fcb17999
cp crates/algo/examples/bench_pixelate.rs ../brahma-pixelate-base/crates/algo/examples/
(cd ../brahma-pixelate-base && CARGO_TARGET_DIR=target cargo build --release -p brahma-algo --example bench_pixelate)
CARGO_TARGET_DIR=target cargo build --release -p brahma-algo --example bench_pixelate

RAYON_NUM_THREADS=16 ../brahma-pixelate-base/target/release/examples/bench_pixelate facet 6000 4000 u8 1 before.bin rgb
RAYON_NUM_THREADS=16 target/release/examples/bench_pixelate facet 6000 4000 u8 1 after.bin rgb
cmp before.bin after.bin
sha256sum before.bin after.bin
```

repeat each pair three times, reversing the order for the middle pair. use `7200 5000` for 36 mp; `u16` or `f32` for the other depths; `halftone`, `halftone-small` or `halftone-large` for the screen cases; and `cmyk` or `lab` for the other color models. the example prints the timed operation separately from file writing.

## correctness and repository checks

the independent scalar oracles retain the original kernels. tests compare every sample bit across all eight color models, alpha on/off, constant colors and edges, random samples, HDR/negative values, non-finite samples, thin regions, partial halos, extreme screen offsets, extended channel counts and scratch rejection. tiled surface tests add 8/16/32-bit storage, soft selections, tile sizes 1/7/17/256 and both transparent and repeated edges.

passed: 401 algorithm tests; clippy with warnings denied; formatting; dependency layering; the full `cargo xtask wasm` set; `cargo xtask perf --quick` on llvmpipe; and scorecard generation. no command, ui or format contracts changed.
