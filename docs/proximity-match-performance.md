# proximity match search and memory

spot healing's proximity match chooses a source displacement by comparing the ring around
the healing hole. two changes reduce that search's work:

- source-validity counts use a summed-area table over the hole's nonzero bounding box.
  every excluded cell outside that box is zero, so translating and clipping queries gives
  the same counts as a full-image table. this also covers separated holes and canvas edges.
- each candidate's SSD is compared with the best score after every 32 ring samples. all
  SSD terms are nonnegative, so the partial sum divided by the final sample count and
  multiplied by the same distance penalty is a lower bound on the final score. a candidate
  that reaches the best score cannot win the existing strict comparison.

the sample order, accumulation order, distance penalty, candidate visit order, tie handling,
search radius and source-pixel copying stay the same. nonfinite best scores disable pruning.
the cropped table checks allocation, arithmetic and source slices and returns none on failure.
the other summed-area tables used by content-aware completion and texture synthesis are unchanged.

## measurements

recorded 2026-10-10 on an AWS c7i.4xlarge, intel xeon platinum 8488c, 16 vcpus, 32 gib RAM,
rust 1.99.0, normal workspace release profile. this kernel is serial. the input is a synthetic
periodic RGB float texture and a circular hole; values are generated before timing. each path
gets a warmup and five measured runs, alternating old/new order. every run checks identical
source displacement against the old implementation.

the four small regions follow the existing retouch benchmark's stroke-domain sizing:
`width = height = diameter + 2 * (diameter + 16)`, search radius `diameter + 16`.
ring width is `clamp(diameter / 8, 3, 16)`. large-image cases use search radius 512.
these are kernel calls, excluding brush rasterization, surface reads, poisson blending,
document history and compositing.

| input region | hole diameter | before p50 | after p50 | speedup | old table | new table |
|---|---:|---:|---:|---:|---:|---:|
| 182 x 182 | 50 px | 2.924 ms | 0.765 ms | 3.82x | 133,956 b | 10,000 b |
| 332 x 332 | 100 px | 0.828 ms | 0.409 ms | 2.02x | 443,556 b | 40,000 b |
| 632 x 632 | 200 px | 1.959 ms | 1.272 ms | 1.54x | 1,602,756 b | 160,000 b |
| 932 x 932 | 300 px | 4.816 ms | 1.372 ms | 3.51x | 3,481,956 b | 360,000 b |
| 6000 x 4000 | 32 px | 67.862 ms | 14.672 ms | 4.63x | 96,040,004 b | 4,096 b |
| 6000 x 4000 | 256 px | 96.509 ms | 15.469 ms | 6.24x | 96,040,004 b | 262,144 b |
| 7200 x 5000 | 32 px | 99.693 ms | 21.658 ms | 4.60x | 144,048,804 b | 4,096 b |
| 7200 x 5000 | 256 px | 132.300 ms | 22.397 ms | 5.91x | 144,048,804 b | 262,144 b |

table storage alone falls by roughly 90–93 percent in these stroke regions. on a full
24–36 mp input it falls from 96–144 mb to 4 kib for the 32 px hole, or 256 kib for the
256 px hole. those are table sizes, not total process memory: the input image, hole mask
and sampled ring still exist. the initial hole-bounds scan remains linear in image size.

the engine already crops a normal healing stroke to a local region. the large-image cases
show the public kernel's scaling, not an additional 4.6–6.2x ui speedup. source textures,
hole shapes, candidate validity and the quality of the current best match affect pruning;
these measurements do not establish a worst-case speedup or photoshop image-quality parity.
a hole whose bounds cover the image still needs a full-size table.

## correctness and reproduction

tests compare cropped counts with full-table counts for rectangles wholly inside, outside,
crossing or clipped against the hole's bounds. they include separated holes, canvas edges,
very large query coordinates and checked table allocation/bounds failures.

search tests compare exact source displacements with the previous implementation over
1, 3, 4 and 5 channels; 8-bit, 16-bit and float working values; several ring/search radii;
empty, one-pixel and separated holes; uniform ties and nonfinite samples. a completely
unknown image still has no source candidate.

```sh
cargo test --release -p brahma-algo proximity_integral_release_comparison -- --ignored --nocapture
```

the old search is retained only in the test module. the selected source displacement is
unchanged, so downstream source-patch copying and gradient-domain blending get the same input.
