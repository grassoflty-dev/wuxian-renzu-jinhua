#!/usr/bin/env bash
# Compile unchanged native bundle-validation tests directly against the real
# source repository identity, not the headless adapter's altered manifest.
set -euo pipefail
source_root=$(realpath "$1")
deps=$(realpath "$2")
output=$(realpath -m "$3")
mkdir -p "$output"
version=$(python3 -c 'import tomllib,sys; print(tomllib.load(open(sys.argv[1],"rb"))["package"]["version"])' "$source_root/server-rs/Cargo.toml")
CARGO_MANIFEST_DIR="$source_root/server-rs" CARGO_PKG_VERSION="$version" rustc \
  --edition=2021 --test "$source_root/server-rs/tests/bundle_identity_build.rs" \
  -L "dependency=$deps" \
  --extern "serde=$(find "$deps" -maxdepth 1 -name 'libserde-*.rlib' | head -1)" \
  --extern "serde_json=$(find "$deps" -maxdepth 1 -name 'libserde_json-*.rlib' | head -1)" \
  --extern "sha2=$(find "$deps" -maxdepth 1 -name 'libsha2-*.rlib' | head -1)" \
  -o "$output/native-bundle-fixture-tests"
"$output/native-bundle-fixture-tests" --test-threads=1
