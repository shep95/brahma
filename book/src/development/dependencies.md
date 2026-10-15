# dependencies

cargo dependencies are pinned through `Cargo.lock` and exercised by the normal workspace build, test, clippy, and WebAssembly jobs. parser and GPU dependencies deserve additional review because they process attacker-controlled data or interface with native drivers.

## current repository state

no `cargo-deny` policy file, `cargo-audit` job, or equivalent dependency-advisory workflow is present in the current tree. do not describe dependency auditing as implemented until configuration and ci evidence land.

## proposed checks

```sh
cargo audit
cargo deny check
```

a future `deny.toml` should be reviewed by maintainers and encode accepted licenses, advisory policy, source rules, and duplicate-version policy. advisory exceptions need an owner, rationale, compensating control, and expiration/review date. ci tools and GitHub actions should also be version-pinned and reviewed as dependencies.

dependency changes should document why a crate is needed, whether it processes untrusted input, unsafe/native code exposure, supported targets, maintenance status, and license compatibility. avoid replacing source review with a scanner result.
