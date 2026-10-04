#!/usr/bin/env bash
# Workspace-only Windows linker proof. Does not execute the produced EXE.
set -euo pipefail
output=$(realpath -m "$1")
mkdir -p "$output/smoke/src"
printf '[package]\nname="recovery-windows-smoke"\nversion="0.1.0"\nedition="2021"\n' > "$output/smoke/Cargo.toml"
printf 'fn main() { println!("windows-linker-smoke"); }\n' > "$output/smoke/src/main.rs"
CARGO_TARGET_DIR="$output/target" RUSTFLAGS='-C target-feature=+crt-static' \
  cargo xwin build --manifest-path "$output/smoke/Cargo.toml" --target x86_64-pc-windows-msvc
file "$output/target/x86_64-pc-windows-msvc/debug/recovery-windows-smoke.exe"
llvm-readobj --coff-imports "$output/target/x86_64-pc-windows-msvc/debug/recovery-windows-smoke.exe"
