#!/usr/bin/env bash
set -euo pipefail

# cross uses Docker/Podman so the host does not need separate cross-linkers.
# A target matching the current host is built directly with cargo.
CROSS_BIN="${CROSS:-cross}"
HOST_TARGET="$(rustc -vV | awk '/^host:/ { print $2 }')"
mkdir -p dist

build_and_copy() {
    local target="$1"
    local source_name="$2"
    local output_name="$3"

    echo "==> Building $output_name ($target)"
    if [[ "$target" == "$HOST_TARGET" ]]; then
        cargo build --locked --release --target "$target"
    else
        if ! command -v "$CROSS_BIN" >/dev/null 2>&1; then
            echo "error: cross is required to build $target on $HOST_TARGET" >&2
            echo "run: make setup-cross" >&2
            exit 1
        fi
        "$CROSS_BIN" build --locked --release --target "$target"
    fi
    cp "target/$target/release/$source_name" "dist/$output_name"
}

build_named_target() {
    case "$1" in
        linux-arm64)
            # Works on 64-bit ARM Linux, including 64-bit Raspberry Pi OS.
            build_and_copy \
                "aarch64-unknown-linux-gnu" \
                "sendpushover" \
                "sendpushover-linux-arm64"
            chmod +x dist/sendpushover-linux-arm64
            ;;
        linux-x86_64)
            build_and_copy \
                "x86_64-unknown-linux-gnu" \
                "sendpushover" \
                "sendpushover-linux-x86_64"
            chmod +x dist/sendpushover-linux-x86_64
            ;;
        windows-x86_64)
            build_and_copy \
                "x86_64-pc-windows-gnu" \
                "sendpushover.exe" \
                "sendpushover-windows-x86_64.exe"
            ;;
        *)
            echo "error: unknown build '$1'" >&2
            echo "choices: linux-arm64, linux-x86_64, windows-x86_64" >&2
            exit 2
            ;;
    esac
}

if (( $# == 0 )); then
    set -- linux-arm64 linux-x86_64 windows-x86_64
fi

for name in "$@"; do
    build_named_target "$name"
done

printf '\nBuilds written to dist/:\n'
ls -lh dist/
