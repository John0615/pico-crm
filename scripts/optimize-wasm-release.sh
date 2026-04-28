#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PKG_DIR="${1:-$ROOT_DIR/target/site/pkg}"

if ! command -v wasm-opt >/dev/null 2>&1; then
    echo "wasm-opt not found in PATH" >&2
    exit 1
fi

if [[ ! -d "$PKG_DIR" ]]; then
    echo "WASM package directory not found: $PKG_DIR" >&2
    exit 1
fi

mapfile -t WASM_FILES < <(find "$PKG_DIR" -maxdepth 1 -type f -name '*.wasm' | sort)

if [[ "${#WASM_FILES[@]}" -eq 0 ]]; then
    echo "No wasm files found in: $PKG_DIR" >&2
    exit 1
fi

total_before=0
total_after=0

for wasm_file in "${WASM_FILES[@]}"; do
    tmp_file="${wasm_file}.tmp"
    before_size="$(stat -c '%s' "$wasm_file")"

    wasm-opt -Oz \
        --enable-bulk-memory \
        --enable-nontrapping-float-to-int \
        --enable-sign-ext \
        --enable-mutable-globals \
        "$wasm_file" \
        -o "$tmp_file"

    mv "$tmp_file" "$wasm_file"

    after_size="$(stat -c '%s' "$wasm_file")"
    total_before=$((total_before + before_size))
    total_after=$((total_after + after_size))

    printf '%-60s %10s -> %10s bytes\n' "$(basename "$wasm_file")" "$before_size" "$after_size"
done

printf '\nTotal: %s -> %s bytes\n' "$total_before" "$total_after"
