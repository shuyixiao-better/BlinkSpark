#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INSTALL_TO_APPLICATIONS="${INSTALL_TO_APPLICATIONS:-0}"

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

bash "$ROOT_DIR/scripts/generate_macos_icon.sh"

if ! cargo bundle --version >/dev/null 2>&1; then
  echo "[INFO] Installing cargo-bundle..."
  cargo install cargo-bundle --locked
fi

ARCH="$(uname -m)"
case "$ARCH" in
  arm64)
    TARGET="aarch64-apple-darwin"
    ;;
  x86_64)
    TARGET="x86_64-apple-darwin"
    ;;
  *)
    echo "[ERROR] Unsupported macOS architecture: $ARCH" >&2
    exit 1
    ;;
esac

rustup target add "$TARGET"

pushd "$ROOT_DIR" >/dev/null
cargo bundle --release --target "$TARGET" --format app
popd >/dev/null

APP_PATH="$ROOT_DIR/target/$TARGET/release/bundle/osx/BlinkSpark.app"
if [ ! -d "$APP_PATH" ]; then
  echo "[ERROR] Bundle completed but app not found: $APP_PATH" >&2
  exit 1
fi

echo "[OK] App bundle created: $APP_PATH"

if [ "$INSTALL_TO_APPLICATIONS" = "1" ]; then
  DEST="$HOME/Applications/BlinkSpark.app"
  mkdir -p "$HOME/Applications"
  rm -rf "$DEST"
  cp -R "$APP_PATH" "$DEST"
  xattr -dr com.apple.quarantine "$DEST" || true
  echo "[OK] Installed to: $DEST"
fi
