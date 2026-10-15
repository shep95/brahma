# unsharp mask performance

large-radius native unsharp mask uses separable FFT convolution with the original
discrete gaussian coefficients and the existing `rustfft` dependency. a streamed
separable path preserves the original tap order for exact processing.

FFT is eligible for radii 64–1000 with at least 65,536 output pixels, a nonzero
amount and a zero or negative threshold. positive thresholds, unstable samples,
unsupported geometry and insufficient scratch use the exact path. WebAssembly
retains exact processing. alpha, selections, edges and cancellation are preserved.
smart sharpen, high pass and shared blur math are unchanged.

FFT changes summation order and permits small rounding differences. the measured
maximum was one 8-bit storage level, approximately one 16-bit level, and 8.762e-6
for the HDR fixture. exact-path comparisons were byte-identical. transform axes
are capped at 32,768; FFT scratch grouping has a 1 gib budget, excluding document
tiles. this is not a whole-process memory limit.

before/after numbers, benchmark methodology, numerical tolerances and preview
measurement limits are recorded in [pr #2987](https://github.com/shep95/brahma/pull/2987).
the comparisons use synthetic images and measure CPU computation, not slider
event-to-screen latency. no published performance budget or rendering-parity
floor is raised by these measurements.
