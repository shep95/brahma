# invert: an example brahma plug-in

a complete WebAssembly filter plug-in in about 70 lines of `no_std` rust: no allocator, no imports,
under 1 kib built. it inverts the colour channels and leaves alpha alone, matching
image › adjustments › invert. the ABI is documented in [`docs/plugins.md`](../../../docs/plugins.md).

## build

```sh
rustup target add wasm32-unknown-unknown
./build.sh
```

`build.sh` runs `cargo build --release --target wasm32-unknown-unknown` and copies the module to
`crates/plugins/tests/fixtures/invert.wasm`, the test fixture the host's tests use. this crate is not
a member of the brahma workspace.

## install

```sh
# Into a running app (control channel) or the CLI, as a command:
plugin.install {"path": "examples/plugins/invert-rs/target/wasm32-unknown-unknown/release/brahma_plugin_invert.wasm"}
plugin.run {"id": "org.brahma.example.invert"}
```

or copy the `.wasm` into the folder set in edit › preferences › plug-ins (with *use additional
plug-ins folder* on); it then appears under filter › plug-ins.
