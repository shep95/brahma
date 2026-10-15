# building

brahma uses the rust workspace at the repository root. install the stable rust toolchain, clone the repository, and run commands from that root.

on linux, install the windowing, gl and GTK development packages first. the development guide's
[prerequisites](https://github.com/shep95/brahma/blob/main/docs/development.md#prerequisites)
list them for debian/ubuntu, fedora, arch and opensuse.

```sh
cargo build --workspace
cargo run --release -p brahma -- image.psd
cargo run -p brahma-cli -- --help
```

the standard repository gates are:

```sh
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo xtask layers
cargo xtask wasm
```

`cargo xtask wasm` requires the `wasm32-unknown-unknown` target. ui work should also be checked using the offscreen snapshot example described in the [development guide](https://github.com/shep95/brahma/blob/main/docs/development.md).

## build the documentation

install mdBook, then build from the repository root:

```sh
cargo install mdbook --locked
mdbook build book
```

the generated site is written to `book/output/` and is not source documentation.
