#!/usr/bin/env bash
# Installs a Linux application-menu/taskbar entry for Chess Analyzer using the
# logo at assets/logo.svg (rasterized to assets/icon.png), so launchers/docks
# show the real app icon instead of a generic one. User-local install, no
# root needed.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

BIN_PATH="$(pwd)/target/release/chess-analyzer"
if [ ! -x "$BIN_PATH" ]; then
  BIN_PATH="$(pwd)/target/debug/chess-analyzer"
fi
if [ ! -x "$BIN_PATH" ]; then
  echo "No built binary found at target/release or target/debug — run 'cargo build --release' first." >&2
  exit 1
fi

ICON_DIR="$HOME/.local/share/icons/hicolor/256x256/apps"
APPS_DIR="$HOME/.local/share/applications"
mkdir -p "$ICON_DIR" "$APPS_DIR"

cp assets/icon.png "$ICON_DIR/chess-analyzer.png"
sed "s#__EXEC_PATH__#$BIN_PATH#" assets/chess-analyzer.desktop > "$APPS_DIR/chess-analyzer.desktop"
chmod +x "$APPS_DIR/chess-analyzer.desktop"

command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$APPS_DIR" || true
command -v gtk-update-icon-cache >/dev/null 2>&1 && gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true

echo "Installed desktop entry pointing at $BIN_PATH"
echo "Installed icon to $ICON_DIR/chess-analyzer.png"
echo "You may need to log out/in (or restart your taskbar/dock) to see it update."
