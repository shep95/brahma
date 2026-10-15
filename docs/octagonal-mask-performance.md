# exact octagonal mask shifts

select and mask applies shift edge as flat grayscale dilation or erosion. previously,
`matting::morph` ran one full 3x3 pass for each radius step, alternating a square and a
cross. a 64 px shift needed 64 image passes. the filter supports fractional coverage.

## decomposition

for radius `r`, let `a = ceil(r / 2)` and `b = floor(r / 2)`. the original footprint is
exactly a square of radius `a` followed by a manhattan ball of radius `b`:

```text
max(abs(dx) - a, 0) + max(abs(dy) - a, 0) <= b
```

equivalently, `max(abs(dx), abs(dy)) <= r` and `abs(dx) + abs(dy) <= r + a`.
reordering the unit squares and crosses preserves the footprint on a rectangular
canvas: a path to any included pixel can stay between its endpoints, inside the canvas.
no outside pixel participates in the old clipped passes or in the new result.

the square uses the existing van herk / gil-werman prefix/suffix window extrema, with
constant comparisons per sample. native dense buffers of at least 262,144 pixels run
the horizontal windows in parallel, transpose in 32x32 cache blocks, filter contiguous columns, then transpose
back. the two full-size float work planes are reused. small inputs and wasm keep the
serial square pass. each axis caps the window radius at that axis's extent.

the manhattan ball is built from the radius's binary digits. a radius-`2d` ball is the
union of five radius-`d` balls, centered at the output and at its four axial offsets of
`d` pixels. for any point in the larger ball, choose its larger absolute coordinate.
moving the center `d` pixels along that axis leaves manhattan distance at most `d`.
clamping that center to the canvas retains this property and adds no out-of-footprint
pixels. an odd radius adds one unit cross after doubling.

for masks without NaNs or negative zero, this changes the work from **o(width * height * radius)** to
**o(width * height * log(radius + 1))**. large native cross passes also run in parallel.
small radii 0-2 retain the original path. NaNs and negative zero retain the original
comparison order, including nan payloads and signed-zero behavior. other values,
including infinities and HDR mask values, use exact max/min comparisons with no
quantization or floating-point sums.

## measurements

recorded 2026-10-10 on an AWS c7i.4xlarge, intel xeon platinum 8488c, 16 vcpus,
32 gib RAM, rust 1.99.0, standard workspace release profile. native rayon uses 16
workers. the 384x384 case is serial in both paths; it represents the size of a 256 px
core with a 64 px halo. large masks use 256x128 output cores with a radius halo,
scheduled across the native workers. constant halo windows fill their cores directly.

input is a circular grayscale mask with a 7 px fractional edge. input generation is
outside timing. each case warms both paths, then measures three runs each in alternating
old/new order. the table gives the median. every run compares every output pixel exactly
against the original repeated-pass implementation.

| mask | radius | operation | before p50 | after p50 | speedup |
|---|---:|---|---:|---:|---:|
| 384 x 384 | 4 px | dilate | 5.559 ms | 3.301 ms | 1.68x |
| 384 x 384 | 4 px | erode | 4.765 ms | 2.941 ms | 1.62x |
| 384 x 384 | 16 px | dilate | 21.254 ms | 4.666 ms | 4.55x |
| 384 x 384 | 16 px | erode | 18.393 ms | 4.080 ms | 4.51x |
| 384 x 384 | 32 px | dilate | 42.498 ms | 5.469 ms | 7.77x |
| 384 x 384 | 32 px | erode | 37.368 ms | 4.874 ms | 7.67x |
| 384 x 384 | 64 px | dilate | 84.287 ms | 6.601 ms | 12.77x |
| 384 x 384 | 64 px | erode | 72.744 ms | 5.603 ms | 12.98x |
| 6000 x 4000 | 4 px | dilate | 986.909 ms | 41.678 ms | 23.68x |
| 6000 x 4000 | 4 px | erode | 847.584 ms | 40.829 ms | 20.76x |
| 6000 x 4000 | 16 px | dilate | 3630.533 ms | 50.315 ms | 72.16x |
| 6000 x 4000 | 16 px | erode | 3069.906 ms | 47.808 ms | 64.21x |
| 6000 x 4000 | 32 px | dilate | 7153.449 ms | 60.465 ms | 118.31x |
| 6000 x 4000 | 32 px | erode | 6024.692 ms | 57.223 ms | 105.28x |
| 6000 x 4000 | 64 px | dilate | 14414.481 ms | 99.749 ms | 144.51x |
| 6000 x 4000 | 64 px | erode | 11942.596 ms | 83.131 ms | 143.66x |
| 7200 x 5000 | 4 px | dilate | 1483.109 ms | 59.551 ms | 24.91x |
| 7200 x 5000 | 4 px | erode | 1263.256 ms | 57.847 ms | 21.84x |
| 7200 x 5000 | 16 px | dilate | 5463.782 ms | 67.911 ms | 80.46x |
| 7200 x 5000 | 16 px | erode | 4593.086 ms | 65.145 ms | 70.51x |
| 7200 x 5000 | 32 px | dilate | 10766.328 ms | 81.414 ms | 132.24x |
| 7200 x 5000 | 32 px | erode | 9048.115 ms | 78.240 ms | 115.65x |
| 7200 x 5000 | 64 px | dilate | 21348.501 ms | 124.007 ms | 172.16x |
| 7200 x 5000 | 64 px | erode | 17968.390 ms | 106.289 ms | 169.05x |

## full CPU refinement

the second comparison runs the actual tiled `refine_mask` pipeline, swapping only the
morphology callback between the previous loop and the new implementation. it includes
guide sampling, guided filtering, boundary distances, quantization, tile assembly and
output trimming. both paths use the same synthetic RGB guide and a trimmed circular
selection, radius 64 and shift edge +100%. each result's bounding box and every mask
byte must match. warmup and three alternating paired runs use the same method as above.
document sizes are 24 and 36 mp; the selection is cropped by the normal workflow.

| document | before p50 | after p50 | speedup |
|---|---:|---:|---:|
| 6000 x 4000 | 3019.176 ms | 761.687 ms | 3.96x |
| 7200 x 5000 | 3705.934 ms | 920.891 ms | 4.02x |

a third kernel comparison uses a dense fractional gradient, `0.6*x/w + 0.4*y/h`,
so no tile can take the constant-window shortcut:

| document | before p50 | after p50 | speedup |
|---|---:|---:|---:|
| 6000 x 4000 | 14207.853 ms | 394.622 ms | 36.00x |

kernel tables include filter allocations. the CPU refinement table includes the rest
of the refinement function. document history and ui rendering are outside that timing.

## bounded workspace

for image-shaped mask buffers without NaNs or negative zero, at least 1,048,576 pixels, width at least 256,
height at least 128 and radii 3-64, the returned f32 output is the only full-image allocation. each active worker uses at most a 384x256 halo window, with three f32
planes and less than 6 kib of line scratch. at 16 workers the derived allocation bound,
including output, is about **115 mb at 24 mp** or **163 mb at 36 mp**, versus the old
**192 mb / 288 mb** two-plane workspace. this is about **40% / 43% less**. these are
allocation bounds, not measured process RSS; the source and caller-retained copies are
separate. for n pixels and p workers, the bound is `4*N + P*(12*384*256 + 6144)`
bytes. constant windows need no filter scratch. smaller calls and larger radii use
the dense decomposition, with two f32 work planes plus line scratch.

## correctness and reproduction

coverage includes all binary masks on small rectangular canvases up to 4x3; every radius
0-65 on fractional 8-bit, 16-bit and HDR float values; impulse footprints at corners,
edges and interior positions; thin images; partial transpose blocks; worker-count
identity; infinities, nan payloads and signed zero. shifted tiled masks are compared
with whole-buffer refinement across canvas and tile edges. checked dimensions reject
short input and multiplication overflow. radii larger than the canvas saturate to
reachable extents on the decomposition path, including very thin canvases.

```sh
cargo test --release -p brahma-algo octagonal_morphology_release_comparison -- --ignored --nocapture --test-threads=1
cargo test --release -p brahma-algo tiled_refinement_release_comparison -- --ignored --nocapture --test-threads=1
cargo test --release -p brahma-algo dense_matte_release_comparison -- --ignored --nocapture --test-threads=1
```

the benchmark's old path is also the production fallback for exceptional values and
small radii. it uses the previous loop without a changed footprint or tie order.
