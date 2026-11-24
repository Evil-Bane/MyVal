#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

JNI_OUT="$ROOT_DIR/android/app/src/main/jniLibs"
KT_OUT="$ROOT_DIR/android/app/src/main/java"
mkdir -p "$JNI_OUT" "$KT_OUT"

echo "=== 1. Cross-compiling myval-ffi for arm64-v8a (release, LTO, stripped) ==="
cargo ndk -t arm64-v8a -o "$JNI_OUT" build -p myval-ffi --release

echo "=== 2. Building host myval_ffi library for UniFFI metadata extraction ==="
cargo build -p myval-ffi

if [[ -f "$ROOT_DIR/target/debug/libmyval_ffi.so" ]]; then
  HOST_LIB="$ROOT_DIR/target/debug/libmyval_ffi.so"
elif [[ -f "$ROOT_DIR/target/debug/libmyval_ffi.dylib" ]]; then
  HOST_LIB="$ROOT_DIR/target/debug/libmyval_ffi.dylib"
else
  echo "Expected host myval_ffi shared library not found in target/debug" >&2
  exit 1
fi

echo "=== 3. Generating Kotlin UniFFI bindings ==="
cargo run -p myval-ffi --features bindgen-cli --bin uniffi-bindgen -- \
  generate --library "$HOST_LIB" --language kotlin --out-dir "$KT_OUT"

SO_PATH="$JNI_OUT/arm64-v8a/libmyval_ffi.so"
KT_PATH="$KT_OUT/uniffi/myval_ffi/myval_ffi.kt"
ls -lh "$SO_PATH" "$KT_PATH"
