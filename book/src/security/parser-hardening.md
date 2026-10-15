# parser hardening

parser safety is broader than “does not crash.” a parser must also bound memory, CPU time, recursion, output expansion, and the number of objects it creates.

## implemented controls

### PSD/PSB

`brahma-psd` enforces maximum document dimensions and channel counts, bounds decoded channel data, limits ActionDescriptor nesting and pattern edges, and uses checked arithmetic for key decoded-size calculations. tests cover malformed headers, truncation, property-generated inputs, and dedicated fuzz targets.

### raster codecs

`brahma-codecs` has configurable `Limits` for width, height, pixel count, and decoded allocation. the default policy is applied across enabled decoders and tested with oversized PNG, PNM, JPEG, and TIFF headers plus random/mutated inputs.

### native format

`brahma-format` bounds manifest, blob, and total decompressed bytes; admits the declared canvas (by default at most 300000 pixels per side, the largest canvas `file.new`, canvas size and PSB import create, so whatever saves reopens; `LoadOptions::max_canvas_side` / `max_canvas_pixels` let a caller lower it) before decoding anything for it; validates ZIP structure and decompressed size; verifies crcs and content hashes; rejects unsupported versions; and property-tests random bundles/manifests.

## review checklist

for every attacker-controlled count, offset, dimension, or length:

1. parse into a type that can represent the file field without truncation.
2. validate semantic limits before allocation.
3. use `checked_add`, `checked_mul`, or a deliberately safe saturating comparison.
4. verify a range lies inside the input before slicing.
5. charge decoded bytes, objects, and recursion against a budget.
6. return a format-specific error; never panic or loop indefinitely.
7. test zero, maximum, maximum-plus-one, truncation, and inconsistent declarations.

## future hardening

- define aggregate parse budgets shared across nested PSD resources and embedded objects;
- add time/work counters for highly fragmented but size-bounded files;
- exercise stack depth beyond ActionDescriptors, including recursive document relationships;
- add format-specific decompression ratio and object-count regression cases;
- benchmark worst-case accepted inputs on constrained machines;
- retain minimized fuzz artifacts with a non-sensitive provenance note.
