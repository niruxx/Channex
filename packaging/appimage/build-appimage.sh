#!/bin/sh
# Builds dist/Channex-<version>-<arch>.AppImage with linuxdeploy and its
# GTK plugin, which bundle GTK 4, libadwaita and their libraries.
#
# The AppImage runs on distros whose glibc is at least as new as the
# build machine's, so build it on the oldest system you want to support
# that still has GTK 4.18+ and libadwaita 1.8+ (e.g. Fedora 43).
#
# Video and GIF playback use the host's GStreamer plugins (install
# gst-plugins-good / gst-libav or your distro's equivalents).
#
# On first run this downloads linuxdeploy and linuxdeploy-plugin-gtk from
# their GitHub releases into build/tools/.
set -eu

ROOT=$(cd "$(dirname "$0")/../.." && pwd)
APPID=io.github.niruxx.Channex
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT/Cargo.toml" | head -n1)
ARCH=$(uname -m)
TOOLS=$ROOT/build/tools
mkdir -p "$TOOLS"

if [ ! -x "$TOOLS/linuxdeploy-$ARCH.AppImage" ]; then
  curl -fsSL -o "$TOOLS/linuxdeploy-$ARCH.AppImage" \
    "https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-$ARCH.AppImage"
  chmod +x "$TOOLS/linuxdeploy-$ARCH.AppImage"
fi
if [ ! -x "$TOOLS/linuxdeploy-plugin-gtk.sh" ]; then
  curl -fsSL -o "$TOOLS/linuxdeploy-plugin-gtk.sh" \
    "https://raw.githubusercontent.com/linuxdeploy/linuxdeploy-plugin-gtk/master/linuxdeploy-plugin-gtk.sh"
  chmod +x "$TOOLS/linuxdeploy-plugin-gtk.sh"
fi

BIN=${CHANNEX_BIN:-$ROOT/target/release/channex}
if [ ! -x "$BIN" ]; then
  (cd "$ROOT" && cargo build --release --locked)
fi

APPDIR=$(mktemp -d)/AppDir
trap 'rm -rf "$(dirname "$APPDIR")"' EXIT
install -Dm755 "$BIN" "$APPDIR/usr/bin/channex"
install -Dm644 "$ROOT/data/$APPID.metainfo.xml" "$APPDIR/usr/share/metainfo/$APPID.appdata.xml"

mkdir -p "$ROOT/dist"
cd "$ROOT/dist"
# Works without FUSE (containers, CI).
export APPIMAGE_EXTRACT_AND_RUN=1
export DEPLOY_GTK_VERSION=4
export PATH="$TOOLS:$PATH"
export OUTPUT="Channex-$VERSION-$ARCH.AppImage"
"$TOOLS/linuxdeploy-$ARCH.AppImage" \
  --appdir "$APPDIR" \
  --desktop-file "$ROOT/data/$APPID.desktop" \
  --icon-file "$ROOT/data/icons/hicolor/scalable/apps/$APPID.svg" \
  --plugin gtk \
  --output appimage
ls -la "$ROOT/dist/$OUTPUT"
