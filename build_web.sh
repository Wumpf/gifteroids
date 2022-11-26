#!/usr/bin/env bash

# Web build script put together by looking at
# https://github.com/emilk/egui/blob/041f2e64bac778c9095fbf4316dc1f7c7bceb670/sh/build_demo_web.sh
# https://rustwasm.github.io/docs/wasm-bindgen/examples/without-a-bundler.html#using-the-older---target-no-modules
# https://bevy-cheatbook.github.io/platforms/wasm.html
# and friends

set -eu
script_path=$( cd "$(dirname "${BASH_SOURCE[0]}")" ; pwd -P )
cd "$script_path"

RELEASE=false

CRATE_NAME="gifteroids"
BUILD_DIR="out"

while test $# -gt 0; do
  case "$1" in
    -h|--help)
      echo "build_demo_web.sh [--release] [--open]"
      echo ""
      echo "  --release: Compile for release, and run wasm-opt. Removes all debug symbols as well."
      exit 0
      ;;

    --release)
      shift
      RELEASE=true
      ;;

    *)
      break
      ;;
  esac
done

# rust build setup
echo "Ensuring build setup…"
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.83

# Clear output from old stuff:
rm -rf ${BUILD_DIR}

echo "Cargo build…"
if [[ "${RELEASE}" = true ]]; then
  BUILD=release
  cargo build --release --target wasm32-unknown-unknown
else
  BUILD=debug
  cargo build --target wasm32-unknown-unknown
fi

# Get the output directory (in the workspace it is in another location)
TARGET=`cargo metadata --format-version=1 | jq --raw-output .target_directory`
echo "Target path was ${TARGET}"

echo "Generating JS bindings for wasm…"
TARGET_NAME="${CRATE_NAME}.wasm"
WASM_PATH="${TARGET}/wasm32-unknown-unknown/$BUILD/$TARGET_NAME"
wasm-bindgen "${WASM_PATH}" --out-dir ${BUILD_DIR} --target web --no-typescript

if [[ "${RELEASE}" = true ]]; then
  echo "Optimizing wasm…"
  wasm-opt "${BUILD_DIR}/${CRATE_NAME}_bg.wasm" -Oz --fast-math -o "${BUILD_DIR}/${CRATE_NAME}_bg.wasm"
fi

echo "Finished ${BUILD_DIR}/${CRATE_NAME}_bg.wasm"
wc -c "${BUILD_DIR}/${CRATE_NAME}_bg.wasm"

echo "Copying assets"
cp index.html ${BUILD_DIR}
cp -r assets ${BUILD_DIR}/assets