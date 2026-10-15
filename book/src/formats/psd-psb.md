# PSD and PSB

`brahma-psd` is a standalone clean-room reader and writer based on adobe's public PSD specification. it supports PSD version 1 and PSB version 2, preserves unknown blocks for round-trip fidelity, and decodes channel data lazily.

the parser accepts all four PSD compression methods handled by the crate and represents format-level structures before `brahma-io` maps them to `brahma-doc`. the current public API and fidelity behavior are documented in [`crates/psd/README.md`](https://github.com/shep95/brahma/blob/main/crates/psd/README.md).

## implemented parser protections

current source enforces several explicit limits:

- document width and height: at most 300,000 pixels each;
- document channels: at most 56;
- layer channels: at most 64;
- decoded channel data: at most 2 gib for one decode layout;
- ActionDescriptor nesting: at most 64 levels;
- pattern edge: at most 30,000 pixels;
- arithmetic for row counts and decoded lengths uses checked operations.

malformed-input tests, property tests, and `cargo-fuzz` targets cover the main parser and ActionDescriptor parser. these controls reduce risk but do not prove the parser safe for every hostile file or bound total work across every embedded object.

## remaining hardening

priorities include broader PSB and embedded-object fuzz corpora, measuring aggregate allocations and parse time, ensuring every nested length is budgeted, and retaining minimized crashing inputs as regression fixtures. see [fuzzing and regression testing](../security/fuzzing.md).
