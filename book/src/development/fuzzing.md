# fuzzing

fuzzing procedures, current targets, security properties, and corpus rules are maintained in [fuzzing and regression testing](../security/fuzzing.md).

the existing fuzz workspaces are intentionally separate from the main cargo workspace because `cargo-fuzz` uses nightly rust and sanitizer instrumentation. example:

```sh
rustup toolchain install nightly
cargo install cargo-fuzz --locked
cd crates/codecs
cargo +nightly fuzz run decode_any
```

reproduce a crash, minimize it, add a deterministic regression test, and then fix it. a time-limited fuzz run with no findings is evidence of that run only; it is not proof that a parser is secure.

the main GitHub actions workflow currently runs fuzzing only when manually dispatched with the `fuzz` input. scheduled or hosted continuous fuzzing remains future work.
