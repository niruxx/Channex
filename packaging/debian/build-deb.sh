#!/bin/sh
# Builds dist/channex_<version>-1_<arch>.deb with only ar and tar, so it
# works on any Linux (no dpkg-deb needed). The binary needs glibc 2.38+,
# GTK 4.18+ and libadwaita 1.8+: Debian testing/unstable, Ubuntu 25.10+.
#
#   packaging/debian/build-deb.sh
#
# Uses target/release/channex if it exists (or $CHANNEX_BIN), else runs
# `cargo build --release --locked`.
set -eu

ROOT=$(cd "$(dirname "$0")/../.." && pwd)
APPID=io.github.niruxx.Channex
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT/Cargo.toml" | head -n1)
REVISION=1
case "$(uname -m)" in
  x86_64) ARCH=amd64 ;;
  aarch64) ARCH=arm64 ;;
  *) echo "unsupported architecture: $(uname -m)" >&2; exit 1 ;;
esac

BIN=${CHANNEX_BIN:-$ROOT/target/release/channex}
if [ ! -x "$BIN" ]; then
  (cd "$ROOT" && cargo build --release --locked)
fi

STAGE=$(mktemp -d)
trap 'rm -rf "$STAGE"' EXIT
DATA=$STAGE/data
CONTROL=$STAGE/control
mkdir -p "$CONTROL"

install -Dm755 "$BIN" "$DATA/usr/bin/channex"
strip --strip-unneeded "$DATA/usr/bin/channex" 2>/dev/null || true
install -Dm644 "$ROOT/data/$APPID.desktop" "$DATA/usr/share/applications/$APPID.desktop"
install -Dm644 "$ROOT/data/$APPID.metainfo.xml" "$DATA/usr/share/metainfo/$APPID.metainfo.xml"
install -Dm644 "$ROOT/data/icons/hicolor/scalable/apps/$APPID.svg" \
  "$DATA/usr/share/icons/hicolor/scalable/apps/$APPID.svg"

cat > "$CONTROL/control" <<CONTROL
Package: channex
Version: $VERSION-$REVISION
Architecture: $ARCH
Maintainer: niruxxdaboi <perezkyle6@gmail.com>
Installed-Size: $(du -sk "$DATA" | cut -f1)
Depends: libc6 (>= 2.38), libgcc-s1, libgtk-4-1 (>= 4.18), libadwaita-1-0 (>= 1.8), libglib2.0-0t64 (>= 2.66) | libglib2.0-0 (>= 2.66), libcairo2, libpango-1.0-0, libgdk-pixbuf-2.0-0, libgraphene-1.0-0
Recommends: gstreamer1.0-plugins-good, gstreamer1.0-libav
Section: net
Priority: optional
Homepage: https://github.com/niruxx/Channex
Description: Native GTK/libadwaita viewer for chan-style imageboards
 Browse 4chan, 8kun, Lainchan and other 4chan-API-compatible or LynxChan
 imageboards in a native GNOME-style app: catalogs, threads with quote
 previews, bookmarks, downloads, a media viewer, and classic skins.
CONTROL
(cd "$DATA" && find . -type f | LC_ALL=C sort | sed 's|^\./||' | xargs md5sum) > "$CONTROL/md5sums"

TAR="tar --owner=0 --group=0 --numeric-owner --sort=name --mtime=@${SOURCE_DATE_EPOCH:-0}"
$TAR -cJf "$STAGE/control.tar.xz" -C "$CONTROL" .
$TAR -cJf "$STAGE/data.tar.xz" -C "$DATA" .
printf '2.0\n' > "$STAGE/debian-binary"

mkdir -p "$ROOT/dist"
OUT=$ROOT/dist/channex_${VERSION}-${REVISION}_${ARCH}.deb
rm -f "$OUT"
# Member order matters to dpkg: debian-binary, control, data.
(cd "$STAGE" && ar rcD "$OUT" debian-binary control.tar.xz data.tar.xz)
echo "$OUT"
