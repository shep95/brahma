# testing

tests are the gate for source changes. choose the narrowest relevant package tests during development, then run the repository gates appropriate to the touched layers.

```sh
cargo test -p <package>
cargo clippy -p <package> --all-targets -- -D warnings
cargo xtask layers
```

for changes below or at l6, also run `cargo xtask wasm`. command changes require the ignored adversarial test:

```sh
cargo test -p brahma-engine --test panic_hunt -- --ignored
```

format changes should include valid round trips, exact boundary values, truncation, inconsistent lengths, malformed structures, and allocation/decompression limit cases. pixel behavior should be exercised at applicable 8-bit, 16-bit, and 32-bit depths.

ui changes need rendered evidence from the offscreen snapshot example or a control-channel screenshot. a successful build or HTTP response is not visual verification.

documentation changes are verified with:

```sh
mdbook build book
```

internal markdown links and every `SUMMARY.md` destination should also be checked. the documentation workflow performs a clean mdBook build in ci.
