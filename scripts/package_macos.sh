#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INSTALL_TO_APPLICATIONS="${INSTALL_TO_APPLICATIONS:-0}"
MACOS_BUILD_TARGET="${MACOS_BUILD_TARGET:-universal}"
APP_NAME="BlinkSpark"

usage() {
  cat <<EOF
Usage:
  ./scripts/package_macos.sh

Environment variables:
  MACOS_BUILD_TARGET   Build target: universal|arm64|x86_64 (default: universal)
  INSTALL_TO_APPLICATIONS
                       1 to install into \$HOME/Applications/${APP_NAME}.app

Examples:
  ./scripts/package_macos.sh
  MACOS_BUILD_TARGET=x86_64 ./scripts/package_macos.sh
  INSTALL_TO_APPLICATIONS=1 ./scripts/package_macos.sh
EOF
}

if ! command -v cargo >/dev/null 2>&1; then
  echo "[ERROR] Missing cargo. Install Rust first: https://rustup.rs" >&2
  exit 1
fi

if ! command -v rustup >/dev/null 2>&1; then
  echo "[ERROR] Missing rustup. Install Rust first: https://rustup.rs" >&2
  exit 1
fi

if ! command -v clang >/dev/null 2>&1; then
  echo "[ERROR] Missing Apple clang (Xcode Command Line Tools). Run: xcode-select --install" >&2
  exit 1
fi

if ! command -v iconutil >/dev/null 2>&1; then
  echo "[ERROR] Missing iconutil (Xcode Command Line Tools). Run: xcode-select --install" >&2
  exit 1
fi

if ! command -v lipo >/dev/null 2>&1; then
  echo "[ERROR] Missing lipo (Xcode Command Line Tools). Run: xcode-select --install" >&2
  exit 1
fi

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

bash "$ROOT_DIR/scripts/generate_macos_icon.sh"

if ! cargo bundle --version >/dev/null 2>&1; then
  echo "[INFO] Installing cargo-bundle..."
  cargo install cargo-bundle --locked
fi

bundle_target() {
  local target="$1"
  rustup target add "$target"
  cargo bundle --release --target "$target" --format osx
}

verify_arch_contains() {
  local bin_path="$1"
  local expected="$2"
  local archs
  archs="$(lipo -archs "$bin_path")"
  if [[ "$archs" != *"$expected"* ]]; then
    echo "[ERROR] Binary architecture mismatch. Expected '$expected', got '$archs'" >&2
    exit 1
  fi
}

pushd "$ROOT_DIR" >/dev/null
case "$MACOS_BUILD_TARGET" in
  universal)
    ARM_TARGET="aarch64-apple-darwin"
    X86_TARGET="x86_64-apple-darwin"
    bundle_target "$ARM_TARGET"
    bundle_target "$X86_TARGET"

    ARM_APP_PATH="$ROOT_DIR/target/$ARM_TARGET/release/bundle/osx/${APP_NAME}.app"
    X86_APP_PATH="$ROOT_DIR/target/$X86_TARGET/release/bundle/osx/${APP_NAME}.app"
    UNIVERSAL_TARGET="universal-apple-darwin"
    APP_PATH="$ROOT_DIR/target/$UNIVERSAL_TARGET/release/bundle/osx/${APP_NAME}.app"

    if [ ! -d "$ARM_APP_PATH" ] || [ ! -d "$X86_APP_PATH" ]; then
      echo "[ERROR] Missing per-arch app bundle. arm64: $ARM_APP_PATH x86_64: $X86_APP_PATH" >&2
      exit 1
    fi

    rm -rf "$APP_PATH"
    mkdir -p "$(dirname "$APP_PATH")"
    cp -R "$ARM_APP_PATH" "$APP_PATH"

    ARM_BIN="$ARM_APP_PATH/Contents/MacOS/$APP_NAME"
    X86_BIN="$X86_APP_PATH/Contents/MacOS/$APP_NAME"
    UNIVERSAL_BIN="$APP_PATH/Contents/MacOS/$APP_NAME"
    lipo -create "$ARM_BIN" "$X86_BIN" -output "$UNIVERSAL_BIN"

    verify_arch_contains "$UNIVERSAL_BIN" "x86_64"
    verify_arch_contains "$UNIVERSAL_BIN" "arm64"
    ;;
  arm64)
    TARGET="aarch64-apple-darwin"
    bundle_target "$TARGET"
    APP_PATH="$ROOT_DIR/target/$TARGET/release/bundle/osx/${APP_NAME}.app"
    verify_arch_contains "$APP_PATH/Contents/MacOS/$APP_NAME" "arm64"
    ;;
  x86_64)
    TARGET="x86_64-apple-darwin"
    bundle_target "$TARGET"
    APP_PATH="$ROOT_DIR/target/$TARGET/release/bundle/osx/${APP_NAME}.app"
    verify_arch_contains "$APP_PATH/Contents/MacOS/$APP_NAME" "x86_64"
    ;;
  *)
    echo "[ERROR] Invalid MACOS_BUILD_TARGET: $MACOS_BUILD_TARGET" >&2
    usage
    exit 1
    ;;
esac
popd >/dev/null

if [ ! -d "$APP_PATH" ]; then
  echo "[ERROR] Bundle completed but app not found: $APP_PATH" >&2
  exit 1
fi

echo "[OK] App bundle created: $APP_PATH"
echo "[OK] Binary architectures: $(lipo -archs "$APP_PATH/Contents/MacOS/$APP_NAME")"

if [ "$INSTALL_TO_APPLICATIONS" = "1" ]; then
  DEST="$HOME/Applications/${APP_NAME}.app"
  mkdir -p "$HOME/Applications"
  rm -rf "$DEST"
  cp -R "$APP_PATH" "$DEST"
  xattr -dr com.apple.quarantine "$DEST" || true
  echo "[OK] Installed to: $DEST"
fi
