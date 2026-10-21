#!/bin/sh
# Builds every example plug-in and copies the modules into assets/plugins (the gallery's
# bundled samples) and the test fixture. Needs `rustup target add wasm32-unknown-unknown`.
set -eu
cd "$(dirname "$0")"
mkdir -p ../../assets/plugins
for dir in invert-rs posterize-rs vignette-rs; do
  (cd "$dir" && cargo build --release --target wasm32-unknown-unknown --quiet)
  name=$(basename "$dir" -rs)
  out="$dir/target/wasm32-unknown-unknown/release/brahma_plugin_$name.wasm"
  cp "$out" "../../assets/plugins/$name.wasm"
  echo "built $name ($(wc -c < "$out") bytes)"
done
cp ../../assets/plugins/invert.wasm ../../crates/plugins/tests/fixtures/invert.wasm
