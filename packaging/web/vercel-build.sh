#!/usr/bin/env bash
# Build the browser version on Vercel.  Used by /vercel.json as the build command.
#
# Vercel's build image has Node but no Rust, so this installs a stable Rust toolchain with the
# wasm32 target plus a pinned Trunk, then runs the same Trunk release build as
# packaging/web/package.sh.  Output lands in dist/web (vercel.json's outputDirectory).
#
# Memory: Vercel builders have limited RAM, so the wasm is linked with thin LTO (as in the
# Dockerfile) and a small number of parallel jobs. Override with CARGO_PROFILE_WASM_RELEASE_LTO
# and CARGO_BUILD_JOBS in the Vercel project's environment variables.
set -euxo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

# Must match TRUNK_VERSION in .github/workflows/ci.yml and release.yml.
TRUNK_VERSION="${TRUNK_VERSION:-0.21.14}"
export CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"
export RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}"
export PATH="$CARGO_HOME/bin:$PATH"

# The workspace needs rust >= rust-version in Cargo.toml (edition 2024). The build image may
# ship an older rustc, so install our own stable toolchain unless the one on PATH is new enough.
NEED="$(sed -n 's/^rust-version = "\(.*\)"/\1/p' "$ROOT/Cargo.toml" | head -n1)"
have_rust() {
  command -v rustc >/dev/null 2>&1 && command -v rustup >/dev/null 2>&1 || return 1
  local v; v="$(rustc --version | awk '{print $2}')"
  [ "$(printf '%s\n%s\n' "$NEED" "$v" | sort -V | head -n1)" = "$NEED" ]
}
if ! have_rust; then
  echo "==> installing Rust (stable, wasm32-unknown-unknown; need >= $NEED)"
  curl -fsSL https://sh.rustup.rs | sh -s -- -y --profile minimal \
    --default-toolchain stable --target wasm32-unknown-unknown --no-modify-path
  hash -r
fi
rustup target add wasm32-unknown-unknown
rustc --version
cargo --version

# The static musl build: the glibc build needs a newer glibc than Vercel's build image ships.
if ! command -v trunk >/dev/null 2>&1 || [ "$(trunk --version | awk '{print $2}')" != "$TRUNK_VERSION" ]; then
  echo "==> installing trunk $TRUNK_VERSION"
  arch="$(uname -m)"
  curl -fsSL "https://github.com/trunk-rs/trunk/releases/download/v${TRUNK_VERSION}/trunk-${arch}-unknown-linux-musl.tar.gz" \
    | tar -xz -C "$CARGO_HOME/bin" trunk
fi
trunk --version

export CARGO_PROFILE_WASM_RELEASE_LTO="${CARGO_PROFILE_WASM_RELEASE_LTO:-thin}"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}"
export BRAHMA_BUILD_SHA="${BRAHMA_BUILD_SHA:-${VERCEL_GIT_COMMIT_SHA:-}}"
export BRAHMA_BUILD_DATE="${BRAHMA_BUILD_DATE:-$(date -u +%Y-%m-%d)}"

echo "==> trunk build --release (lto=$CARGO_PROFILE_WASM_RELEASE_LTO, jobs=$CARGO_BUILD_JOBS)"
(cd "$ROOT/apps/brahma-web" && trunk build --release --locked)

SITE="$ROOT/dist/web"
[ -f "$SITE/index.html" ] || { echo "error: $SITE/index.html missing" >&2; exit 1; }
# The site must work from any path (public_url = "./" in Trunk.toml).
if grep -Eq '(src|href)="/[^/]' "$SITE/index.html"; then
  echo "error: $SITE/index.html has root-absolute URLs" >&2
  exit 1
fi
ls -l "$SITE"
